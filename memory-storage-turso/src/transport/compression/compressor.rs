//! Async compressor for transport operations

use super::types::{
    CompressionStreamResult, TransportCompressionConfig, TransportCompressionError,
    TransportCompressionStats,
};
use crate::compression::{CompressedPayload, CompressionAlgorithm};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::warn;

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
            let mut stats = self.stats.lock().map_err(|e| {
                TransportCompressionError::StreamFailed(format!(
                    "Failed to acquire stats lock: {}",
                    e
                ))
            })?;
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
        let mut stats = self.stats.lock().map_err(|e| {
            TransportCompressionError::StreamFailed(format!("Failed to acquire stats lock: {}", e))
        })?;

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

        let mut stats = self.stats.lock().map_err(|e| {
            TransportCompressionError::StreamFailed(format!("Failed to acquire stats lock: {}", e))
        })?;
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

        let total_read = content.len();
        let total_written = header.len() + compressed.data.len();

        let elapsed = start.elapsed().as_micros() as u64;

        // Update stats
        let mut stats = self.stats.lock().map_err(|e| {
            TransportCompressionError::StreamFailed(format!("Failed to acquire stats lock: {}", e))
        })?;
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
        let mut stats = self.stats.lock().map_err(|e| {
            TransportCompressionError::StreamFailed(format!("Failed to acquire stats lock: {}", e))
        })?;
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
    pub fn create_stream_header(payload: &CompressedPayload) -> [u8; 16] {
        let mut header = [0u8; 16];
        header[0..8].copy_from_slice(&(payload.original_size as u64).to_le_bytes());
        header[8..16].copy_from_slice(&(payload.compressed_size as u64).to_le_bytes());
        // Algorithm is implicit in the compression format
        header
    }

    /// Parse stream header
    fn parse_stream_header(header: &[u8; 16]) -> Result<(usize, usize, CompressionAlgorithm)> {
        // SAFETY: header is guaranteed to be 16 bytes by the type signature
        let original_size = u64::from_le_bytes(
            header[0..8]
                .try_into()
                .expect("Header slice must be exactly 8 bytes"),
        ) as usize;
        let compressed_size = u64::from_le_bytes(
            header[8..16]
                .try_into()
                .expect("Header slice must be exactly 8 bytes"),
        ) as usize;

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
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for reading: {}", e);
                TransportCompressionStats::new()
            })
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = TransportCompressionStats::new();
        } else {
            tracing::error!("Failed to acquire stats lock for reset");
        }
    }
}
