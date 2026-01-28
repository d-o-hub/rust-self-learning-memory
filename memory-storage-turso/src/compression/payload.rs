//! # Compressed Payload Implementation
//!
//! This module contains the core compression and decompression logic
//! for the CompressedPayload type.

use crate::compression::CompressionAlgorithm;
use memory_core::Result;

/// A compressed payload with metadata for decompression
///
/// # Example
///
/// ```rust
/// use memory_storage_turso::compression::CompressedPayload;
///
/// let data = b"hello world".repeat(100);
/// let compressed = CompressedPayload::compress(&data, 1024).unwrap();
/// assert!(compressed.compression_ratio < 1.0);
/// let decompressed = compressed.decompress().unwrap();
/// assert_eq!(data, decompressed);
/// ```
#[derive(Debug, Clone)]
pub struct CompressedPayload {
    /// Original size before compression
    pub original_size: usize,
    /// Size after compression
    pub compressed_size: usize,
    /// Compression ratio (compressed/original), lower is better
    pub compression_ratio: f64,
    /// The compressed data
    pub data: Vec<u8>,
    /// Algorithm used for compression
    pub algorithm: CompressionAlgorithm,
}

impl CompressedPayload {
    /// Compress data using the best algorithm for the payload
    ///
    /// # Arguments
    ///
    /// * `data` - The data to compress
    /// * `threshold` - Minimum size to trigger compression (smaller payloads are stored uncompressed)
    ///
    /// # Returns
    ///
    /// A `CompressedPayload` containing the compressed data and metadata
    ///
    /// # Algorithm Selection
    ///
    /// - Payloads smaller than `threshold` are not compressed
    /// - For larger payloads, Zstd is tried first (best ratio)
    /// - If Zstd achieves < 50% ratio, LZ4 is used as fallback
    pub fn compress(data: &[u8], threshold: usize) -> Result<Self> {
        // For small payloads, skip compression
        if data.len() < threshold {
            return Ok(CompressedPayload {
                original_size: data.len(),
                compressed_size: data.len(),
                compression_ratio: 1.0,
                data: data.to_vec(),
                algorithm: CompressionAlgorithm::None,
            });
        }

        // Try Zstd first (best ratio)
        let zstd_result = Self::compress_zstd(data);
        match zstd_result {
            Ok(zstd) => {
                if zstd.compression_ratio >= 0.5 {
                    return Ok(zstd);
                }
                // Fall back to LZ4 if Zstd doesn't achieve good ratio
                Self::compress_lz4(data)
            }
            Err(_) => {
                // Zstd not available, fall back to LZ4
                Self::compress_lz4(data)
            }
        }
    }

    /// Compress using LZ4 algorithm
    pub fn compress_lz4(data: &[u8]) -> Result<Self> {
        #[cfg(feature = "compression-lz4")]
        {
            let compressed = lz4_flex::compress(data);
            let compressed_size = compressed.len();
            let original_size = data.len();
            let ratio = if original_size > 0 {
                compressed_size as f64 / original_size as f64
            } else {
                1.0
            };

            Ok(CompressedPayload {
                original_size,
                compressed_size,
                compression_ratio: ratio,
                data: compressed,
                algorithm: CompressionAlgorithm::Lz4,
            })
        }

        #[cfg(not(feature = "compression-lz4"))]
        {
            // Fall back to gzip if LZ4 not available
            Self::compress_gzip(data)
        }
    }

    /// Compress using Zstd algorithm
    pub fn compress_zstd(data: &[u8]) -> Result<Self> {
        #[cfg(feature = "compression-zstd")]
        {
            let compressed = zstd::stream::encode_all(data, 0)?;
            let compressed_size = compressed.len();
            let original_size = data.len();
            let ratio = if original_size > 0 {
                compressed_size as f64 / original_size as f64
            } else {
                1.0
            };

            Ok(CompressedPayload {
                original_size,
                compressed_size,
                compression_ratio: ratio,
                data: compressed,
                algorithm: CompressionAlgorithm::Zstd,
            })
        }

        #[cfg(not(feature = "compression-zstd"))]
        {
            Err(memory_core::Error::Storage(
                "Zstd compression not available (enable compression-zstd feature)".to_string(),
            ))
        }
    }

    /// Compress using Gzip algorithm
    pub fn compress_gzip(data: &[u8]) -> Result<Self> {
        #[cfg(feature = "compression-gzip")]
        {
            use flate2::{bufread::GzEncoder, Compression};
            use std::io::Read;

            let compression = Compression::default();
            let mut encoder = GzEncoder::new(data, compression);
            let mut compressed = Vec::new();
            encoder.read_to_end(&mut compressed)?;

            let compressed_size = compressed.len();
            let original_size = data.len();
            let ratio = if original_size > 0 {
                compressed_size as f64 / original_size as f64
            } else {
                1.0
            };

            Ok(CompressedPayload {
                original_size,
                compressed_size,
                compression_ratio: ratio,
                data: compressed,
                algorithm: CompressionAlgorithm::Gzip,
            })
        }

        #[cfg(not(feature = "compression-gzip"))]
        {
            // No compression available, return uncompressed
            Ok(CompressedPayload {
                original_size: data.len(),
                compressed_size: data.len(),
                compression_ratio: 1.0,
                data: data.to_vec(),
                algorithm: CompressionAlgorithm::None,
            })
        }
    }

    /// Decompress the payload
    ///
    /// # Returns
    ///
    /// The original uncompressed data
    pub fn decompress(&self) -> Result<Vec<u8>> {
        match self.algorithm {
            CompressionAlgorithm::None => Ok(self.data.clone()),
            CompressionAlgorithm::Lz4 => self.decompress_lz4(),
            CompressionAlgorithm::Zstd => self.decompress_zstd(),
            CompressionAlgorithm::Gzip => self.decompress_gzip(),
        }
    }

    /// Decompress LZ4 data
    fn decompress_lz4(&self) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-lz4")]
        {
            lz4_flex::decompress(&self.data, self.original_size).map_err(|e| {
                memory_core::Error::Storage(format!("LZ4 decompression failed: {}", e))
            })
        }

        #[cfg(not(feature = "compression-lz4"))]
        {
            Err(memory_core::Error::Storage(
                "LZ4 decompression not available (enable compression-lz4 feature)".to_string(),
            ))
        }
    }

    /// Decompress Zstd data
    fn decompress_zstd(&self) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-zstd")]
        {
            zstd::stream::decode_all(&self.data[..]).map_err(|e| {
                memory_core::Error::Storage(format!("Zstd decompression failed: {}", e))
            })
        }

        #[cfg(not(feature = "compression-zstd"))]
        {
            Err(memory_core::Error::Storage(
                "Zstd decompression not available (enable compression-zstd feature)".to_string(),
            ))
        }
    }

    /// Decompress Gzip data
    fn decompress_gzip(&self) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-gzip")]
        {
            use flate2::bufread::GzDecoder;
            use std::io::Read;

            let mut decoder = GzDecoder::new(&self.data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                memory_core::Error::Storage(format!("Gzip decompression failed: {}", e))
            })?;
            Ok(decompressed)
        }

        #[cfg(not(feature = "compression-gzip"))]
        {
            Err(memory_core::Error::Storage(
                "Gzip decompression not available (enable compression-gzip feature)".to_string(),
            ))
        }
    }

    /// Get the bandwidth savings percentage
    ///
    /// Returns the percentage of bandwidth saved (e.g., 40.0 for 40% savings)
    pub fn bandwidth_savings_percent(&self) -> f64 {
        (1.0 - self.compression_ratio) * 100.0
    }
}
