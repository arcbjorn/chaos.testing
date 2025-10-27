use crate::models::{CapturedRequest, Protocol, ResponseData};
use crate::parsers::HttpParser;
use crate::storage::Storage;
use anyhow::Result;
use chrono::Utc;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct HttpInterceptor {
    port: u16,
    storage_path: String,
    target_url: Option<String>,
}

impl HttpInterceptor {
    pub fn new(port: u16, storage_path: String) -> Self {
        Self {
            port,
            storage_path,
            target_url: None,
        }
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target_url = Some(target);
        self
    }

    pub async fn start(&self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        let storage = Arc::new(Storage::new(&self.storage_path)?);
        let target_url = Arc::new(self.target_url.clone());

        info!("HTTP interceptor listening on {}", addr);
        info!("Storing captures in: {}", self.storage_path);
        if let Some(target) = &self.target_url {
            info!("Forwarding requests to: {}", target);
        } else {
            warn!("No target URL - responses will be mocked");
        }

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let storage = Arc::clone(&storage);
            let target_url = Arc::clone(&target_url);

            debug!("Connection from {}", client_addr);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let storage = Arc::clone(&storage);
                            let target_url = Arc::clone(&target_url);
                            handle_request(req, storage, target_url)
                        }),
                    )
                    .await
                {
                    error!("Error serving connection: {}", err);
                }
            });
        }
    }
}

async fn handle_request(
    req: Request<Incoming>,
    storage: Arc<Storage>,
    target_url: Arc<Option<String>>,
) -> Result<Response<String>, hyper::Error> {
    let start = std::time::Instant::now();

    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    debug!("Request: {} {} {:?}", method, uri, req.version());

    let request_data = HttpParser::parse_request(&method, &uri, &headers, None);
    let request_id = Uuid::new_v4().to_string();

    let (response_data, response_body) = if let Some(target) = target_url.as_ref() {
        match forward_request(&method, &uri, &headers, target).await {
            Ok((resp_data, body)) => (Some(resp_data), body),
            Err(e) => {
                error!("Failed to forward request: {}", e);
                (
                    Some(ResponseData {
                        status_code: 502,
                        headers: Default::default(),
                        body: None,
                    }),
                    "Bad Gateway: Failed to reach target".to_string(),
                )
            }
        }
    } else {
        (
            Some(ResponseData {
                status_code: 200,
                headers: Default::default(),
                body: None,
            }),
            format!(
                "Intercepted: {} {}\nStored with ID: {}",
                method, uri, request_id
            ),
        )
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    let captured = CapturedRequest {
        id: request_id.clone(),
        timestamp: Utc::now(),
        protocol: Protocol::Http,
        request: request_data,
        response: response_data.clone(),
        duration_ms: Some(duration_ms),
    };

    if let Err(e) = storage.store_request(&captured) {
        error!("Failed to store request: {}", e);
    } else {
        info!("Captured: {} {} ({}ms)", method, uri, duration_ms);
    }

    let status = response_data
        .as_ref()
        .map(|r| StatusCode::from_u16(r.status_code).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);

    Ok(Response::builder()
        .status(status)
        .body(response_body)
        .unwrap())
}

async fn forward_request(
    method: &hyper::Method,
    uri: &hyper::Uri,
    headers: &hyper::HeaderMap,
    target: &str,
) -> Result<(ResponseData, String)> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}{}",
        target,
        uri.path_and_query().map(|p| p.as_str()).unwrap_or("/")
    );

    let mut req_builder = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => client.get(&url),
    };

    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            req_builder = req_builder.header(key.as_str(), value_str);
        }
    }

    let response = req_builder.send().await?;
    let status = response.status().as_u16();
    let resp_headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await?;

    Ok((
        ResponseData {
            status_code: status,
            headers: resp_headers,
            body: Some(body.as_bytes().to_vec()),
        },
        body,
    ))
}
