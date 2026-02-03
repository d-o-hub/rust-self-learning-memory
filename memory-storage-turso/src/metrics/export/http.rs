//! HTTP server for metrics export

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use super::{ExportConfig, PrometheusExporter};

/// HTTP server for serving Prometheus metrics
///
/// Starts an HTTP server that responds to GET /metrics requests
/// with Prometheus-formatted metrics.
pub struct MetricsHttpServer {
    config: ExportConfig,
    exporter: Arc<PrometheusExporter>,
    handle: Option<JoinHandle<()>>,
}

impl MetricsHttpServer {
    /// Create a new HTTP server (does not start it)
    pub fn new(config: ExportConfig, exporter: PrometheusExporter) -> Self {
        Self {
            config,
            exporter: Arc::new(exporter),
            handle: None,
        }
    }

    /// Start the HTTP server
    ///
    /// Returns an error if the server cannot bind to the configured address.
    pub async fn start(&mut self) -> crate::Result<()> {
        let (bind_address, port, path) = match &self.config.target {
            super::ExportTarget::Http {
                bind_address,
                port,
                path,
            } => (bind_address.clone(), *port, path.clone()),
            _ => {
                return Err(crate::Error::Storage(
                    "HTTP server requires Http target".to_string(),
                ));
            }
        };

        let addr: SocketAddr = format!("{}:{}", bind_address, port)
            .parse()
            .map_err(|e| crate::Error::Storage(format!("Invalid address: {}", e)))?;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| crate::Error::Storage(format!("Failed to bind: {}", e)))?;

        info!("Metrics HTTP server listening on http://{}/metrics", addr);

        let exporter = Arc::clone(&self.exporter);
        let path_clone = path.clone();

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, peer_addr)) = listener.accept().await else {
                    error!("Failed to accept connection");
                    continue;
                };
                let exporter = Arc::clone(&exporter);
                let path = path_clone.clone();
                tokio::spawn(spawn_connection_handler(stream, peer_addr, exporter, path));
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

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|h| !h.is_finished())
    }
}

impl Drop for MetricsHttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Spawn a connection handler task with error logging
async fn spawn_connection_handler(
    stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    exporter: Arc<PrometheusExporter>,
    path: String,
) {
    if let Err(e) = handle_connection(stream, peer_addr, exporter, path).await {
        warn!("Error handling connection from {}: {}", peer_addr, e);
    }
}

/// Handle a single HTTP connection
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    exporter: Arc<PrometheusExporter>,
    metrics_path: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse request line
    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // Only handle GET requests
    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    // Handle metrics endpoint
    if path == metrics_path {
        let metrics = exporter.export_metrics();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            metrics.len(),
            metrics
        );
        stream.write_all(response.as_bytes()).await?;
        info!("Served metrics to {}", peer_addr);
    } else if path == "/health" {
        let body = "OK";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await?;
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_server_creation() {
        let config = ExportConfig::http("127.0.0.1", 0); // Port 0 for auto-assign
        let exporter = PrometheusExporter::default();
        let server = MetricsHttpServer::new(config, exporter);

        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_http_server_requires_http_target() {
        let config = ExportConfig::stdout();
        let exporter = PrometheusExporter::default();
        let mut server = MetricsHttpServer::new(config, exporter);

        let result = server.start().await;
        assert!(result.is_err());
    }
}
