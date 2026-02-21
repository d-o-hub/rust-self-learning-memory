//! # Transport Layer
//!
//! Network transport utilities for Turso storage including:
//! - Compression for network bandwidth optimization
//! - Async I/O utilities
//! - Connection handling
//!
//! ## Components
//!
//! - `compression`: Async compression for transport operations
//! - `wrapper`: Transport wrapper with compression support

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "compression")]
pub mod wrapper;

#[cfg(feature = "compression")]
pub use compression::{
    AsyncCompressor, CompressionStreamResult, TransportCompressionConfig,
    TransportCompressionError, TransportCompressionStats, compress_transport, decompress_transport,
};

#[cfg(feature = "compression")]
pub use wrapper::CompressedTransport;

use async_trait::async_trait;
use std::fmt::Debug;

/// Response from a transport operation
#[derive(Debug, Clone)]
pub struct TransportResponse {
    /// Response status code
    pub status: u16,
    /// Response body
    pub body: Vec<u8>,
    /// Response headers
    pub headers: Vec<(String, String)>,
}

impl TransportResponse {
    /// Create a new transport response
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            body,
            headers: Vec::new(),
        }
    }

    /// Create a successful response (200 OK)
    pub fn success(body: Vec<u8>) -> Self {
        Self::new(200, body)
    }

    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        Self::new(status, message.as_bytes().to_vec())
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Transport trait for network operations
///
/// This trait abstracts over different transport implementations
/// (HTTP, gRPC, etc.) and provides a common interface for
/// sending and receiving data.
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    /// Send data and return a response
    ///
    /// # Arguments
    ///
    /// * `data` - The data to send
    ///
    /// # Returns
    ///
    /// A `TransportResponse` containing the response data
    async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse>;

    /// Send data without expecting a response (fire-and-forget)
    ///
    /// # Arguments
    ///
    /// * `data` - The data to send
    async fn send_async(&self, data: &[u8]) -> anyhow::Result<()>;

    /// Check if the transport is healthy
    async fn health_check(&self) -> anyhow::Result<bool>;

    /// Get transport metadata
    fn metadata(&self) -> TransportMetadata;
}

/// Metadata for a transport implementation
#[derive(Debug, Clone)]
pub struct TransportMetadata {
    /// Transport name
    pub name: String,
    /// Transport version
    pub version: String,
    /// Whether compression is supported
    pub supports_compression: bool,
    /// Maximum payload size
    pub max_payload_size: usize,
}

impl TransportMetadata {
    /// Create new transport metadata
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            supports_compression: false,
            max_payload_size: 10 * 1024 * 1024, // 10MB default
        }
    }

    /// Set compression support
    pub fn with_compression(mut self, supported: bool) -> Self {
        self.supports_compression = supported;
        self
    }

    /// Set max payload size
    pub fn with_max_payload(mut self, size: usize) -> Self {
        self.max_payload_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_response() {
        let response = TransportResponse::success(b"hello".to_vec());
        assert!(response.is_success());
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"hello");
    }

    #[test]
    fn test_transport_response_error() {
        let response = TransportResponse::error(404, "Not found");
        assert!(!response.is_success());
        assert_eq!(response.status, 404);
    }

    #[test]
    fn test_transport_metadata() {
        let metadata = TransportMetadata::new("test", "1.0")
            .with_compression(true)
            .with_max_payload(1024);

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0");
        assert!(metadata.supports_compression);
        assert_eq!(metadata.max_payload_size, 1024);
    }
}
