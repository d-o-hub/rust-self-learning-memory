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
#[path = "wrapper_helpers.rs"]
mod helpers;
use async_trait::async_trait;
use helpers::CompressionResult;
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
                original_size, self.min_compress_size
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
#[path = "wrapper_tests.rs"]
mod tests;
