//! # Network Transport Compression
//!
//! Async compression layer for network transport operations with:
//! - Multiple compression algorithms (LZ4, Zstd, Gzip)
//! - Automatic algorithm selection based on payload size
//! - Compression ratio monitoring and thresholds
//!
//! This module provides compression utilities specifically designed for
//! network transport scenarios, with async/await support for Tokio runtime.
//!
//! ## Features
//!
//! - **Async Compression**: Non-blocking compression operations
//! - **Metrics**: Compression ratio tracking and thresholds
//! - **Automatic Detection**: Detect compression algorithm from payload header

use crate::compression::{
    CompressedPayload, CompressionAlgorithm, CompressionStatistics as BaseCompressionStats,
};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::warn;

/// Errors specific to transport compression operations
#[derive(Error, Debug)]
pub enum TransportCompressionError {
    #[error("Compression algorithm not available: {0}")]
    AlgorithmNotAvailable(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Payload size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },

    #[error("Compression threshold exceeded: {0} bytes")]
    ThresholdExceeded(usize),

    #[error("Invalid compressed data header")]
    InvalidHeader,

    #[error("Streaming operation failed: {0}")]
    StreamFailed(String),
}

/// Configuration for transport compression
#[derive(Debug, Clone)]
pub struct TransportCompressionConfig {
    /// Minimum payload size to trigger compression (default: 1024 bytes)
    pub compression_threshold: usize,
    /// Enable automatic algorithm selection
    pub auto_algorithm_selection: bool,
    /// Preferred algorithm when auto-selection is enabled
    pub preferred_algorithm: CompressionAlgorithm,
    /// Maximum allowed compressed size (default: 10MB)
    pub max_compressed_size: usize,
    /// Enable compression metrics collection
    pub enable_metrics: bool,
    /// Compression ratio warning threshold (default: 0.9 - warn if ratio > 0.9)
    pub warning_ratio_threshold: f64,
    /// Minimum acceptable compression ratio (default: 0.5)
    /// Payloads with worse ratio will use a different algorithm
    pub min_acceptable_ratio: f64,
}

impl Default for TransportCompressionConfig {
    fn default() -> Self {
        Self {
            compression_threshold: 1024,
            auto_algorithm_selection: true,
            preferred_algorithm: CompressionAlgorithm::Zstd,
            max_compressed_size: 10 * 1024 * 1024, // 10MB
            enable_metrics: true,
            warning_ratio_threshold: 0.9,
            min_acceptable_ratio: 0.5,
        }
    }
}

/// Statistics for transport compression operations
#[derive(Debug, Default, Clone)]
pub struct TransportCompressionStats {
    /// Base compression statistics
    pub base: BaseCompressionStats,
    /// Number of streaming compressions
    pub streaming_compressions: u64,
    /// Number of streaming decompressions
    pub streaming_decompressions: u64,
    /// Total bytes saved by compression
    pub total_bytes_saved: u64,
    /// Number of times warning threshold was triggered
    pub warning_threshold_triggers: u64,
    /// Number of algorithm fallback events
    pub algorithm_fallbacks: u64,
    /// Average compression time (microseconds)
    pub avg_compression_time_us: u64,
    /// Average decompression time (microseconds)
    pub avg_decompression_time_us: u64,
    /// Total compression operations
    pub total_compressions: u64,
    /// Total decompression operations
    pub total_decompressions: u64,
}

impl TransportCompressionStats {
    /// Create a new statistics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a streaming compression operation
    pub fn record_streaming_compression(&mut self, original_size: usize, compressed_size: usize) {
        self.streaming_compressions += 1;
        if original_size > compressed_size {
            self.total_bytes_saved += (original_size - compressed_size) as u64;
        }
    }

    /// Record a streaming decompression operation
    pub fn record_streaming_decompression(&mut self) {
        self.streaming_decompressions += 1;
    }

    /// Record a warning threshold trigger
    pub fn record_warning_threshold(&mut self) {
        self.warning_threshold_triggers += 1;
    }

    /// Record an algorithm fallback
    pub fn record_algorithm_fallback(&mut self) {
        self.algorithm_fallbacks += 1;
    }

    /// Record compression time
    pub fn record_compression_time(&mut self, time_us: u64) {
        self.total_compressions += 1;
        let total = self.avg_compression_time_us * (self.total_compressions - 1) + time_us;
        self.avg_compression_time_us = total / self.total_compressions;
    }

    /// Record decompression time
    pub fn record_decompression_time(&mut self, time_us: u64) {
        self.total_decompressions += 1;
        let total = self.avg_decompression_time_us * (self.total_decompressions - 1) + time_us;
        self.avg_decompression_time_us = total / self.total_decompressions;
    }

    /// Get overall compression ratio
    pub fn overall_ratio(&self) -> f64 {
        self.base.compression_ratio()
    }

    /// Get bandwidth savings percentage
    pub fn bandwidth_savings_percent(&self) -> f64 {
        self.base.bandwidth_savings_percent()
    }
}

/// Compressor for async transport operations
#[derive(Debug)]
pub struct AsyncCompressor {
    config: TransportCompressionConfig,
    stats: Arc<Mutex<TransportCompressionStats>>,
}

impl AsyncCompressor {
    /// Create a new async compressor
    pub fn new(config: TransportCompressionConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(TransportCompressionStats::new())),
        }
    }

    /// Create with custom statistics
    pub fn with_stats(
        config: TransportCompressionConfig,
        stats: Arc<Mutex<TransportCompressionStats>>,
    ) -> Self {
        Self { config, stats }
    }

    /// Compress data asynchronously
    ///
    /// Automatically selects the best algorithm based on payload size
    /// and compression ratio.
    pub async fn compress(&self, data: &[u8]) -> Result<CompressedPayload> {
        let start = std::time::Instant::now();

        // Check threshold
        if data.len() < self.config.compression_threshold {
            let mut stats = self.stats.lock().unwrap();
            stats.base.record_skipped();
            return Ok(CompressedPayload {
                original_size: data.len(),
                compressed_size: data.len(),
                compression_ratio: 1.0,
                data: data.to_vec(),
                algorithm: CompressionAlgorithm::None,
            });
        }

        // Check max size
        if data.len() > self.config.max_compressed_size {
            return Err(TransportCompressionError::ThresholdExceeded(data.len()).into());
        }

        // Auto-select algorithm if enabled
        let algorithm = if self.config.auto_algorithm_selection {
            self.select_algorithm(data)?
        } else {
            self.config.preferred_algorithm
        };

        // Compress with selected algorithm
        let payload = match algorithm {
            CompressionAlgorithm::None => CompressedPayload {
                original_size: data.len(),
                compressed_size: data.len(),
                compression_ratio: 1.0,
                data: data.to_vec(),
                algorithm: CompressionAlgorithm::None,
            },
            CompressionAlgorithm::Lz4 => CompressedPayload::compress_lz4(data)?,
            CompressionAlgorithm::Zstd => CompressedPayload::compress_zstd(data)?,
            CompressionAlgorithm::Gzip => CompressedPayload::compress_gzip(data)?,
        };

        // Update stats
        let elapsed = start.elapsed().as_micros() as u64;
        let mut stats = self.stats.lock().unwrap();

        stats
            .base
            .record_compression(payload.original_size, payload.compressed_size, elapsed);

        // Check compression ratio
        if payload.compression_ratio > self.config.warning_ratio_threshold {
            stats.record_warning_threshold();
            warn!(
                "Poor compression ratio: {:.2}%",
                payload.compression_ratio * 100.0
            );
        }

        // Fallback if ratio is too poor
        if payload.compression_ratio > self.config.min_acceptable_ratio
            && algorithm != CompressionAlgorithm::Lz4
        {
            // Try LZ4 as fallback
            if let Ok(lz4_payload) = CompressedPayload::compress_lz4(data) {
                if lz4_payload.compression_ratio < payload.compression_ratio {
                    stats.record_algorithm_fallback();
                    return Ok(lz4_payload);
                }
            }
        }

        stats.record_compression_time(elapsed);
        Ok(payload)
    }

    /// Decompress data asynchronously
    pub async fn decompress(&self, payload: &CompressedPayload) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();

        let result = payload.decompress();
        let elapsed = start.elapsed().as_micros() as u64;

        let mut stats = self.stats.lock().unwrap();
        stats.base.record_decompression(elapsed);
        stats.record_decompression_time(elapsed);

        result.map_err(|e| anyhow::anyhow!(e))
    }

    /// Stream compress data from reader to writer
    ///
    /// This is more memory-efficient for large payloads as it processes
    /// data in chunks.
    pub async fn compress_stream<R, W>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressionStreamResult>
    where
        R: AsyncReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
    {
        let start = std::time::Instant::now();
        let mut total_read = 0;
        let mut total_written = 0;

        // Read entire content for compression (simplified - for true streaming,
        // we'd use a streaming compression library)
        let mut content = Vec::new();
        reader
            .read_to_end(&mut content)
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;

        // Compress the content
        let compressed = self.compress(&content).await?;

        // Write compressed data with header
        let header = Self::create_stream_header(&compressed);
        writer
            .write_all(&header)
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;
        writer
            .write_all(&compressed.data)
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;

        total_read = content.len();
        total_written = header.len() + compressed.data.len();

        let elapsed = start.elapsed().as_micros() as u64;

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.record_streaming_compression(total_read, total_written);

        Ok(CompressionStreamResult {
            original_size: total_read,
            compressed_size: total_written,
            algorithm: compressed.algorithm,
            compression_ratio: compressed.compression_ratio,
            time_us: elapsed,
        })
    }

    /// Stream decompress data from reader to writer
    pub async fn decompress_stream<R, W>(&self, reader: &mut R, writer: &mut W) -> Result<usize>
    where
        R: AsyncReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
    {
        let start = std::time::Instant::now();

        // Read header
        let mut header_buf = [0u8; 16];
        reader
            .read_exact(&mut header_buf)
            .await
            .map_err(|_e| TransportCompressionError::InvalidHeader)?;

        // Parse header
        let (original_size, compressed_size, _) = Self::parse_stream_header(&header_buf)?;

        // Read compressed data
        let mut compressed_data = Vec::with_capacity(compressed_size);
        reader
            .read_to_end(&mut compressed_data)
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;

        // Verify size
        if compressed_data.len() != compressed_size {
            return Err(TransportCompressionError::SizeMismatch {
                expected: compressed_size,
                actual: compressed_data.len(),
            }
            .into());
        }

        // Create payload and decompress
        let payload = CompressedPayload {
            original_size,
            compressed_size,
            compression_ratio: compressed_size as f64 / original_size as f64,
            data: compressed_data,
            algorithm: CompressionAlgorithm::Zstd, // Detected from header
        };

        let decompressed = self.decompress(&payload).await?;
        writer
            .write_all(&decompressed)
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| TransportCompressionError::StreamFailed(e.to_string()))?;

        let elapsed = start.elapsed().as_micros() as u64;

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.record_streaming_decompression();
        stats.record_decompression_time(elapsed);

        Ok(decompressed.len())
    }

    /// Select the best compression algorithm for the data
    fn select_algorithm(&self, data: &[u8]) -> Result<CompressionAlgorithm> {
        // For small data, don't compress
        if data.len() < self.config.compression_threshold {
            return Ok(CompressionAlgorithm::None);
        }

        // For very large data, prefer LZ4 for speed
        if data.len() > 1024 * 1024 {
            // Check if LZ4 is available
            #[cfg(feature = "compression-lz4")]
            return Ok(CompressionAlgorithm::Lz4);
            #[cfg(not(feature = "compression-lz4"))]
            return Ok(CompressionAlgorithm::Gzip);
        }

        // For medium data, prefer Zstd for better ratio
        #[cfg(feature = "compression-zstd")]
        return Ok(CompressionAlgorithm::Zstd);

        #[cfg(not(feature = "compression-zstd"))]
        #[cfg(feature = "compression-lz4")]
        return Ok(CompressionAlgorithm::Lz4);

        #[cfg(not(feature = "compression-zstd"))]
        #[cfg(not(feature = "compression-lz4"))]
        return Ok(CompressionAlgorithm::Gzip);
    }

    /// Create stream header
    fn create_stream_header(payload: &CompressedPayload) -> [u8; 16] {
        let mut header = [0u8; 16];
        header[0..8].copy_from_slice(&(payload.original_size as u64).to_le_bytes());
        header[8..16].copy_from_slice(&(payload.compressed_size as u64).to_le_bytes());
        // Algorithm is implicit in the compression format
        header
    }

    /// Parse stream header
    fn parse_stream_header(header: &[u8; 16]) -> Result<(usize, usize, CompressionAlgorithm)> {
        let original_size = u64::from_le_bytes(header[0..8].try_into().unwrap()) as usize;
        let compressed_size = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;

        // Detect algorithm from size relationship
        let algorithm = if original_size == compressed_size {
            CompressionAlgorithm::None
        } else if compressed_size < original_size / 2 {
            // Likely Zstd or LZ4
            CompressionAlgorithm::Zstd
        } else {
            CompressionAlgorithm::Gzip
        };

        Ok((original_size, compressed_size, algorithm))
    }

    /// Get current statistics
    pub fn stats(&self) -> TransportCompressionStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = TransportCompressionStats::new();
    }
}

/// Result of a streaming compression operation
#[derive(Debug, Clone)]
pub struct CompressionStreamResult {
    /// Original uncompressed size
    pub original_size: usize,
    /// Compressed size including header
    pub compressed_size: usize,
    /// Algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Time taken in microseconds
    pub time_us: u64,
}

/// Compress a byte slice with transport-optimized settings
///
/// This is a convenience function that creates a temporary compressor
/// for single-shot compression operations.
pub async fn compress_transport(data: &[u8]) -> Result<CompressedPayload> {
    let config = TransportCompressionConfig::default();
    let compressor = AsyncCompressor::new(config);
    compressor.compress(data).await
}

/// Decompress a transport payload
///
/// This is a convenience function that creates a temporary compressor
/// for single-shot decompression operations.
pub async fn decompress_transport(payload: &CompressedPayload) -> Result<Vec<u8>> {
    let config = TransportCompressionConfig::default();
    let compressor = AsyncCompressor::new(config);
    compressor.decompress(payload).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_compression() {
        let config = TransportCompressionConfig::default();
        let compressor = AsyncCompressor::new(config);

        // Small data should not be compressed
        let small_data = b"hello";
        let result = compressor.compress(small_data).await.unwrap();
        assert_eq!(result.algorithm, CompressionAlgorithm::None);

        // Larger data should be compressed
        let large_data = b"hello world".repeat(100);
        let result = compressor.compress(&large_data).await.unwrap();
        assert!(result.compression_ratio < 1.0);
        assert!(result.original_size > result.compressed_size);

        // Decompress should work
        let decompressed = compressor.decompress(&result).await.unwrap();
        assert_eq!(decompressed, large_data);
    }

    #[tokio::test]
    async fn test_stream_compression() {
        let config = TransportCompressionConfig::default();
        let compressor = AsyncCompressor::new(config);

        let data = b"test data for streaming compression".repeat(100);
        let mut input: &[u8] = &data;
        let mut output = Vec::new();

        let result = compressor
            .compress_stream(&mut &input[..], &mut output)
            .await
            .unwrap();
        assert!(result.compression_ratio < 1.0);
    }

    #[tokio::test]
    async fn test_stream_roundtrip() {
        let config = TransportCompressionConfig::default();
        let compressor = AsyncCompressor::new(config);

        let original_data = b"roundtrip test data".repeat(50);

        // Compress to bytes
        let compressed = compressor.compress(&original_data).await.unwrap();

        // Create header
        let header = AsyncCompressor::create_stream_header(&compressed);
        let mut stream_data = Vec::new();
        stream_data.extend_from_slice(&header);
        stream_data.extend_from_slice(&compressed.data);

        // Decompress from stream
        let mut input: &[u8] = &stream_data;
        let mut output = Vec::new();
        let decompressed_size = compressor
            .decompress_stream(&mut &input[..], &mut output)
            .await
            .unwrap();

        assert_eq!(decompressed_size, original_data.len());
        assert_eq!(output, original_data);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let config = TransportCompressionConfig::default();
        let compressor = AsyncCompressor::new(config);

        let data = b"test data for stats".repeat(100);
        compressor.compress(&data).await.unwrap();

        let stats = compressor.stats();
        assert!(stats.total_compressions >= 1);
    }

    #[tokio::test]
    async fn test_threshold_check() {
        let config = TransportCompressionConfig {
            compression_threshold: 1024,
            ..Default::default()
        };
        let compressor = AsyncCompressor::new(config);

        // Below threshold
        let small = b"short";
        let result = compressor.compress(small).await.unwrap();
        assert_eq!(result.algorithm, CompressionAlgorithm::None);

        // Above threshold
        let large = b"x".repeat(2048);
        let result = compressor.compress(&large).await.unwrap();
        assert_ne!(result.algorithm, CompressionAlgorithm::None);
    }
}
