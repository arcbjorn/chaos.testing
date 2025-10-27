use crate::models::{CapturedRequest, Protocol};
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
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct HttpInterceptor {
    port: u16,
    storage_path: String,
}

impl HttpInterceptor {
    pub fn new(port: u16, storage_path: String) -> Self {
        Self { port, storage_path }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        let storage = Arc::new(Storage::new(&self.storage_path)?);

        info!("HTTP interceptor listening on {}", addr);
        info!("Storing captures in: {}", self.storage_path);

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let storage = Arc::clone(&storage);

            debug!("Connection from {}", client_addr);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let storage = Arc::clone(&storage);
                            handle_request(req, storage)
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
) -> Result<Response<String>, hyper::Error> {
    let start = std::time::Instant::now();

    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    debug!("Request: {} {} {:?}", method, uri, req.version());

    let request_data = HttpParser::parse_request(&method, &uri, &headers, None);

    let captured = CapturedRequest {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        protocol: Protocol::Http,
        request: request_data,
        response: None,
        duration_ms: Some(start.elapsed().as_millis() as u64),
    };

    if let Err(e) = storage.store_request(&captured) {
        error!("Failed to store request: {}", e);
    } else {
        info!("Captured: {} {}", method, uri);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(format!(
            "Intercepted: {} {}\nStored with ID: {}",
            method, uri, captured.id
        ))
        .unwrap())
}
