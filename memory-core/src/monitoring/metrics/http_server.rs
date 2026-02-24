use super::MetricsRegistry;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// HTTP server for serving Prometheus metrics at /metrics
pub struct MetricsHttpServer {
    registry: MetricsRegistry,
    handle: Option<JoinHandle<()>>,
}

impl MetricsHttpServer {
    /// Create new HTTP server
    pub fn new(registry: MetricsRegistry) -> Self {
        Self {
            registry,
            handle: None,
        }
    }

    /// Start the HTTP server on the given address
    pub async fn start(&mut self, addr: &str) -> std::io::Result<()> {
        let addr: SocketAddr = addr
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let listener = TcpListener::bind(addr).await?;
        info!("Metrics HTTP server listening on http://{}/metrics", addr);

        let registry = self.registry.clone();

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, peer_addr)) = listener.accept().await else {
                    error!("Failed to accept connection");
                    continue;
                };
                let registry = registry.clone();
                tokio::spawn(handle_connection(stream, peer_addr, registry));
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the HTTP server
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            info!("Metrics HTTP server stopped");
        }
    }
}

impl Drop for MetricsHttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Handle a single HTTP connection
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    registry: MetricsRegistry,
) {
    if let Err(e) = handle_connection_impl(&mut stream, peer_addr, &registry).await {
        tracing::warn!("Error handling connection from {}: {}", peer_addr, e);
    }
}

async fn handle_connection_impl(
    stream: &mut tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    registry: &MetricsRegistry,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    match path {
        "/metrics" => {
            let metrics = registry.export_metrics();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                metrics.len(),
                metrics
            );
            stream.write_all(response.as_bytes()).await?;
            info!("Served metrics to {}", peer_addr);
        }
        "/health" => {
            let body = "OK";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await?;
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }
    }

    Ok(())
}
