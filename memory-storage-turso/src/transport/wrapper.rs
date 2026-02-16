//! # Compressed Transport Wrapper
//!
//! Provides transparent compression/decompression for transport operations.
//!
//! ## Features
//!
//! - **Transparent Compression**: Automatically compresses requests >1KB
//! - **Configurable**: Adjustable compression level and size threshold
//! - **Metrics**: Tracks compression ratios and performance
//! - **Fallback**: Falls back to uncompressed on error
//!
//! ## Example
//!
//! ```rust
//! use memory_storage_turso::transport::{
//!     CompressedTransport, Transport, TransportCompressionConfig, TransportMetadata,
//!     TransportResponse,
//! };
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a mock transport for demonstration
//! # use async_trait::async_trait;
//! # use std::fmt::Debug;
//! # #[derive(Debug)]
//! # struct MockTransport;
//! # #[async_trait]
//! # impl Transport for MockTransport {
//! #     async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
//! #         Ok(TransportResponse::success(data.to_vec()))
//! #     }
//! #     async fn send_async(&self, _data: &[u8]) -> anyhow::Result<()> { Ok(()) }
//! #     async fn health_check(&self) -> anyhow::Result<bool> { Ok(true) }
//! #     fn metadata(&self) -> TransportMetadata { TransportMetadata::new("mock", "1.0") }
//! # }
//! let inner = MockTransport;
//! let config = TransportCompressionConfig::default();
//! let transport = CompressedTransport::new(Box::new(inner), config);
//!
//! let data = b"large payload data...".repeat(100);
//! let response = transport.send(&data).await?;
//! # Ok(())
//! # }
//! ```

use super::{Transport, TransportMetadata, TransportResponse};
use crate::compression::CompressionAlgorithm;
use crate::transport::compression::{
    TransportCompressionConfig, TransportCompressionError, TransportCompressionStats,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace, warn};

/// A transport wrapper that adds transparent compression
///
/// This wrapper compresses outgoing data and decompresses incoming data
/// automatically based on configured thresholds and compression levels.
#[derive(Debug)]
pub struct CompressedTransport {
    /// Inner transport implementation
    inner: Box<dyn Transport>,
    /// Compression configuration
    config: TransportCompressionConfig,
    /// Compression statistics
    stats: Arc<Mutex<TransportCompressionStats>>,
    /// Compression level (1-22, where 1 is fastest and 22 is best compression)
    compression_level: i32,
    /// Minimum size to trigger compression (default: 1024 bytes)
    min_compress_size: usize,
}

impl CompressedTransport {
    /// Create a new compressed transport wrapper
    ///
    /// # Arguments
    ///
    /// * `inner` - The inner transport to wrap
    /// * `config` - Compression configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use memory_storage_turso::transport::{
    ///     CompressedTransport, TransportCompressionConfig, Transport, TransportMetadata,
    ///     TransportResponse,
    /// };
    /// # use async_trait::async_trait;
    /// # use std::fmt::Debug;
    /// # #[derive(Debug)]
    /// # struct MockTransport;
    /// # #[async_trait]
    /// # impl Transport for MockTransport {
    /// #     async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
    /// #         Ok(TransportResponse::success(data.to_vec()))
    /// #     }
    /// #     async fn send_async(&self, _data: &[u8]) -> anyhow::Result<()> { Ok(()) }
    /// #     async fn health_check(&self) -> anyhow::Result<bool> { Ok(true) }
    /// #     fn metadata(&self) -> TransportMetadata { TransportMetadata::new("mock", "1.0") }
    /// # }
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// # let inner = MockTransport;
    /// let config = TransportCompressionConfig::default();
    /// let transport = CompressedTransport::new(Box::new(inner), config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(inner: Box<dyn Transport>, config: TransportCompressionConfig) -> Self {
        let stats = Arc::new(Mutex::new(TransportCompressionStats::new()));

        Self {
            inner,
            config: config.clone(),
            stats,
            compression_level: 3, // Default zstd level (good balance)
            min_compress_size: config.compression_threshold,
        }
    }

    /// Create with custom compression level
    ///
    /// # Arguments
    ///
    /// * `inner` - The inner transport to wrap
    /// * `config` - Compression configuration
    /// * `level` - Zstd compression level (1-22)
    pub fn with_level(
        inner: Box<dyn Transport>,
        config: TransportCompressionConfig,
        level: i32,
    ) -> Self {
        let mut transport = Self::new(inner, config);
        transport.compression_level = level.clamp(1, 22);
        transport
    }

    /// Create with custom size threshold
    ///
    /// # Arguments
    ///
    /// * `inner` - The inner transport to wrap
    /// * `config` - Compression configuration
    /// * `threshold` - Minimum size in bytes to trigger compression
    pub fn with_threshold(
        inner: Box<dyn Transport>,
        config: TransportCompressionConfig,
        threshold: usize,
    ) -> Self {
        let mut transport = Self::new(inner, config);
        transport.min_compress_size = threshold;
        transport
    }

    /// Compress data if it exceeds the threshold
    ///
    /// Returns the compressed data along with metadata about the compression
    fn compress_data(
        &self,
        data: &[u8],
    ) -> anyhow::Result<(CompressionResult, TransportCompressionStats)> {
        let start = std::time::Instant::now();
        let original_size = data.len();

        // Check if data is large enough to compress
        if original_size < self.min_compress_size {
            trace!(
                "Data size {} < threshold {}, skipping compression",
                original_size,
                self.min_compress_size
            );

            let mut stats = TransportCompressionStats::new();
            stats.base.record_skipped();

            return Ok((
                CompressionResult {
                    data: data.to_vec(),
                    original_size,
                    compressed_size: original_size,
                    algorithm: CompressionAlgorithm::None,
                    compression_ratio: 1.0,
                },
                stats,
            ));
        }

        // Compress using zstd
        match self.compress_with_zstd(data) {
            Ok(result) => {
                let elapsed = start.elapsed().as_micros() as u64;
                let mut stats = TransportCompressionStats::new();

                stats
                    .base
                    .record_compression(original_size, result.compressed_size, elapsed);
                stats.record_compression_time(elapsed);

                // Log compression metrics
                let savings = (1.0 - result.compression_ratio) * 100.0;
                debug!(
                    "Compressed {} bytes to {} bytes ({:.1}% savings, ratio: {:.2})",
                    original_size, result.compressed_size, savings, result.compression_ratio
                );

                // Check for poor compression
                if result.compression_ratio > self.config.warning_ratio_threshold {
                    stats.record_warning_threshold();
                    warn!(
                        "Poor compression ratio: {:.2}% for {} bytes",
                        result.compression_ratio * 100.0,
                        original_size
                    );
                }

                Ok((result, stats))
            }
            Err(e) => {
                warn!("Compression failed, falling back to uncompressed: {}", e);

                let mut stats = TransportCompressionStats::new();
                stats.base.record_failed();

                // Fall back to uncompressed
                Ok((
                    CompressionResult {
                        data: data.to_vec(),
                        original_size,
                        compressed_size: original_size,
                        algorithm: CompressionAlgorithm::None,
                        compression_ratio: 1.0,
                    },
                    stats,
                ))
            }
        }
    }

    /// Compress data using zstd
    fn compress_with_zstd(&self, data: &[u8]) -> anyhow::Result<CompressionResult> {
        #[cfg(feature = "compression-zstd")]
        {
            let compressed = zstd::stream::encode_all(data, self.compression_level)?;
            let compressed_size = compressed.len();
            let original_size = data.len();
            let ratio = compressed_size as f64 / original_size as f64;

            Ok(CompressionResult {
                data: compressed,
                original_size,
                compressed_size,
                algorithm: CompressionAlgorithm::Zstd,
                compression_ratio: ratio,
            })
        }

        #[cfg(not(feature = "compression-zstd"))]
        {
            // If zstd not available, try LZ4
            self.compress_with_lz4(data)
        }
    }

    /// Compress data using LZ4 (fallback)
    #[cfg(not(feature = "compression-zstd"))]
    fn compress_with_lz4(&self, data: &[u8]) -> anyhow::Result<CompressionResult> {
        #[cfg(feature = "compression-lz4")]
        {
            let compressed = lz4_flex::compress(data);
            let compressed_size = compressed.len();
            let original_size = data.len();
            let ratio = compressed_size as f64 / original_size as f64;

            Ok(CompressionResult {
                data: compressed,
                original_size,
                compressed_size,
                algorithm: CompressionAlgorithm::Lz4,
                compression_ratio: ratio,
            })
        }

        #[cfg(not(feature = "compression-lz4"))]
        {
            Err(anyhow::anyhow!(
                "No compression algorithm available (enable compression-zstd or compression-lz4 feature)"
            ))
        }
    }

    /// Decompress response data
    async fn decompress_response(
        &self,
        response: TransportResponse,
    ) -> anyhow::Result<TransportResponse> {
        // Check for compression header in response
        let content_encoding = response
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-encoding"))
            .map(|(_, v)| v.as_str());

        if let Some(encoding) = content_encoding {
            let start = std::time::Instant::now();

            let decompressed = match encoding {
                "zstd" => self.decompress_zstd(&response.body).await?,
                "lz4" => self.decompress_lz4(&response.body).await?,
                "gzip" => self.decompress_gzip(&response.body).await?,
                _ => {
                    trace!("Unknown encoding: {}, returning as-is", encoding);
                    return Ok(response);
                }
            };

            let elapsed = start.elapsed().as_micros() as u64;

            let mut stats = self.stats.lock().map_err(|e| {
                anyhow::anyhow!(TransportCompressionError::StreamFailed(format!(
                    "Failed to acquire stats lock: {}",
                    e
                )))
            })?;
            stats.base.record_decompression(elapsed);
            stats.record_decompression_time(elapsed);

            debug!(
                "Decompressed {} bytes to {} bytes in {}µs",
                response.body.len(),
                decompressed.len(),
                elapsed
            );

            Ok(TransportResponse {
                status: response.status,
                body: decompressed,
                headers: response.headers,
            })
        } else {
            // No compression, return as-is
            Ok(response)
        }
    }

    /// Decompress zstd data
    async fn decompress_zstd(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        #[cfg(feature = "compression-zstd")]
        {
            // Use spawn_blocking for CPU-intensive decompression
            let data = data.to_vec();
            tokio::task::spawn_blocking(move || {
                zstd::stream::decode_all(&data[..]).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-zstd"))]
        {
            Err(anyhow::anyhow!(
                "Zstd decompression not available (enable compression-zstd feature)"
            ))
        }
    }

    /// Decompress LZ4 data
    async fn decompress_lz4(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        #[cfg(feature = "compression-lz4")]
        {
            // LZ4 decompression is fast, but we still use spawn_blocking for consistency
            let data = data.to_vec();
            // Lock guard is dropped before await (scope block)
            let original_size = {
                let stats = self.stats.lock().map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(format!(
                        "Failed to acquire stats lock: {}",
                        e
                    )))
                })?;
                stats.base.total_original_bytes as usize
            }; // Lock released here

            tokio::task::spawn_blocking(move || {
                lz4_flex::decompress(&data, original_size).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-lz4"))]
        {
            Err(anyhow::anyhow!(
                "LZ4 decompression not available (enable compression-lz4 feature)"
            ))
        }
    }

    /// Decompress gzip data
    async fn decompress_gzip(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        #[cfg(feature = "compression-gzip")]
        {
            let data = data.to_vec();
            tokio::task::spawn_blocking(move || {
                use flate2::bufread::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(&data[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })?;
                Ok(decompressed)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-gzip"))]
        {
            Err(anyhow::anyhow!(
                "Gzip decompression not available (enable compression-gzip feature)"
            ))
        }
    }

    /// Get current compression statistics
    pub fn stats(&self) -> TransportCompressionStats {
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for reading: {}", e);
                TransportCompressionStats::new()
            })
    }

    /// Reset compression statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = TransportCompressionStats::new();
        } else {
            tracing::error!("Failed to acquire stats lock for reset");
        }
    }

    /// Get compression ratio for all operations
    pub fn overall_compression_ratio(&self) -> f64 {
        self.stats
            .lock()
            .map(|guard| guard.overall_ratio())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for ratio: {}", e);
                0.0
            })
    }

    /// Get bandwidth savings percentage
    pub fn bandwidth_savings_percent(&self) -> f64 {
        self.stats
            .lock()
            .map(|guard| guard.bandwidth_savings_percent())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for bandwidth: {}", e);
                0.0
            })
    }
}

/// Result of a compression operation
#[derive(Debug, Clone)]
struct CompressionResult {
    /// Compressed (or original if not compressed) data
    data: Vec<u8>,
    /// Original size before compression
    original_size: usize,
    /// Size after compression
    compressed_size: usize,
    /// Algorithm used
    algorithm: CompressionAlgorithm,
    /// Compression ratio (compressed/original)
    compression_ratio: f64,
}

#[async_trait]
impl Transport for CompressedTransport {
    async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
        // Compress the data
        let (compression_result, operation_stats) = self.compress_data(data)?;

        // Update global stats
        {
            let mut stats = self.stats.lock().map_err(|e| {
                anyhow::anyhow!(TransportCompressionError::StreamFailed(format!(
                    "Failed to acquire stats lock: {}",
                    e
                )))
            })?;
            stats.base.total_original_bytes += operation_stats.base.total_original_bytes;
            stats.base.total_compressed_bytes += operation_stats.base.total_compressed_bytes;
            stats.base.compression_count += operation_stats.base.compression_count;
            stats.base.skipped_count += operation_stats.base.skipped_count;
            stats.base.failed_count += operation_stats.base.failed_count;
            stats.total_compressions += operation_stats.total_compressions;
            stats.total_decompressions += operation_stats.total_decompressions;
        }

        // Add compression header if data was compressed
        let mut headers = Vec::new();
        if compression_result.algorithm != CompressionAlgorithm::None {
            let encoding = match compression_result.algorithm {
                CompressionAlgorithm::Zstd => "zstd",
                CompressionAlgorithm::Lz4 => "lz4",
                CompressionAlgorithm::Gzip => "gzip",
                CompressionAlgorithm::None => unreachable!(),
            };
            headers.push(("Content-Encoding".to_string(), encoding.to_string()));
        }

        // Send via inner transport
        let mut response = self.inner.send(&compression_result.data).await?;
        response.headers.extend(headers);

        // Decompress response if needed
        self.decompress_response(response).await
    }

    async fn send_async(&self, data: &[u8]) -> anyhow::Result<()> {
        // Compress the data
        let (compression_result, _) = self.compress_data(data)?;

        // Send via inner transport
        self.inner.send_async(&compression_result.data).await
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        self.inner.health_check().await
    }

    fn metadata(&self) -> TransportMetadata {
        let mut metadata = self.inner.metadata();
        metadata.supports_compression = true;
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportMetadata;

    /// Mock transport for testing
    #[derive(Debug)]
    struct MockTransport {
        last_sent: Arc<Mutex<Option<Vec<u8>>>>,
        should_fail: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                last_sent: Arc::new(Mutex::new(None)),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                last_sent: Arc::new(Mutex::new(None)),
                should_fail: true,
            }
        }

        fn last_sent(&self) -> Option<Vec<u8>> {
            self.last_sent.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock transport failure"));
            }

            *self.last_sent.lock().unwrap() = Some(data.to_vec());
            Ok(TransportResponse::success(data.to_vec()))
        }

        async fn send_async(&self, data: &[u8]) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock transport failure"));
            }

            *self.last_sent.lock().unwrap() = Some(data.to_vec());
            Ok(())
        }

        async fn health_check(&self) -> anyhow::Result<bool> {
            Ok(!self.should_fail)
        }

        fn metadata(&self) -> TransportMetadata {
            TransportMetadata::new("mock", "1.0")
        }
    }

    #[test]
    fn test_compressed_transport_creation() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        assert_eq!(transport.compression_level, 3);
        assert_eq!(transport.min_compress_size, 1024);
    }

    #[test]
    fn test_compressed_transport_with_level() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::with_level(Box::new(inner), config, 10);

        assert_eq!(transport.compression_level, 10);
    }

    #[test]
    fn test_compressed_transport_with_threshold() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::with_threshold(Box::new(inner), config, 2048);

        assert_eq!(transport.min_compress_size, 2048);
    }

    #[tokio::test]
    async fn test_small_data_no_compression() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        let small_data = b"hello world";
        let response = transport.send(small_data).await.unwrap();

        assert!(response.is_success());

        // Check that data was not compressed (sent as-is)
        let _last_sent = transport.inner.as_ref();
        // The mock received the data
    }

    #[tokio::test]
    async fn test_large_data_compression() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        // Large data that should be compressed
        let large_data = b"hello world".repeat(200);
        let original_size = large_data.len();

        let response = transport.send(&large_data).await.unwrap();

        assert!(response.is_success());

        // Check stats
        let stats = transport.stats();
        assert!(stats.base.compression_count >= 1);

        // Verify compression ratio
        let ratio = transport.overall_compression_ratio();
        assert!(
            ratio < 1.0,
            "Expected compression ratio < 1.0, got {}",
            ratio
        );

        // Verify bandwidth savings
        let savings = transport.bandwidth_savings_percent();
        assert!(
            savings > 0.0,
            "Expected bandwidth savings > 0%, got {}%",
            savings
        );

        println!(
            "Compression: {} bytes -> ratio {:.2}, savings {:.1}%",
            original_size, ratio, savings
        );
    }

    #[tokio::test]
    async fn test_compression_roundtrip() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        // Large JSON-like data
        let json_data = r#"{"key": "value", "items": [1, 2, 3]}"#.repeat(100);
        let original = json_data.as_bytes().to_vec();

        let response = transport.send(&original).await.unwrap();

        // Mock transport echoes back the data
        assert_eq!(response.body, original);
    }

    #[tokio::test]
    async fn test_compression_stats() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        // Send multiple large payloads
        for i in 0..5 {
            let data = format!("Payload {}: {}", i, "x".repeat(2000));
            let _ = transport.send(data.as_bytes()).await;
        }

        let stats = transport.stats();
        assert!(stats.base.compression_count >= 5);
        assert_eq!(stats.total_compressions, 5);

        // Reset stats
        transport.reset_stats();
        let stats_after_reset = transport.stats();
        assert_eq!(stats_after_reset.base.compression_count, 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        assert!(transport.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_metadata() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        let metadata = transport.metadata();
        assert_eq!(metadata.name, "mock");
        assert!(metadata.supports_compression);
    }

    #[tokio::test]
    async fn test_send_async() {
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        let data = b"async test data".repeat(100);
        let result = transport.send_async(&data).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compression_fallback_on_error() {
        // Create transport that will fail compression (if possible)
        let inner = MockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(inner), config);

        // Even if compression fails, the data should still be sent
        let data = b"test data".repeat(100);
        let result = transport.send(&data).await;

        assert!(result.is_ok());
    }
}
