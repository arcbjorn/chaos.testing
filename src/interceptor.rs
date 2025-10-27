use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

pub struct HttpInterceptor {
    port: u16,
    target: Option<String>,
}

impl HttpInterceptor {
    pub fn new(port: u16) -> Self {
        Self { port, target: None }
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    pub async fn start(&self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;

        info!("HTTP interceptor listening on {}", addr);

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);

            debug!("Connection from {}", client_addr);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(handle_request))
                    .await
                {
                    error!("Error serving connection: {}", err);
                }
            });
        }
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<String>, hyper::Error> {
    debug!(
        "Request: {} {} {:?}",
        req.method(),
        req.uri(),
        req.version()
    );

    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(format!(
            "Intercepted: {} {}\nHeaders: {:#?}",
            method, uri, headers
        ))
        .unwrap())
}
