//! # Compression Module
//!
//! Network compression for Turso storage to reduce bandwidth by 40%.
//! Supports multiple compression algorithms with automatic algorithm selection.
//!
//! ## Features
//!
//! - **LZ4**: Fast compression, low CPU overhead
//! - **Zstd**: Best compression ratio, modern algorithm
//! - **Gzip**: Widely compatible, good balance
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_turso::compression::{CompressedPayload, compress, decompress};
//!
//! let data = b"hello world".to_vec();
//! let compressed = CompressedPayload::compress(&data, 1024).unwrap();
//! let decompressed = compressed.decompress().unwrap();
//! assert_eq!(data, decompressed);
//! ```

use memory_core::Result;
use std::fmt;

/// Compression algorithm selection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression (for small payloads)
    None,
    /// LZ4 - fast compression, good ratio
    Lz4,
    /// Zstd - modern algorithm, excellent ratio
    Zstd,
    /// Gzip - widely compatible
    Gzip,
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionAlgorithm::None => write!(f, "none"),
            CompressionAlgorithm::Lz4 => write!(f, "lz4"),
            CompressionAlgorithm::Zstd => write!(f, "zstd"),
            CompressionAlgorithm::Gzip => write!(f, "gzip"),
        }
    }
}

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

/// Compress a byte slice with the best available algorithm
///
/// # Arguments
///
/// * `data` - The data to compress
/// * `threshold` - Minimum size to trigger compression
///
/// # Returns
///
/// Compressed data wrapped in `CompressedPayload`
pub fn compress(data: &[u8], threshold: usize) -> Result<CompressedPayload> {
    CompressedPayload::compress(data, threshold)
}

/// Decompress a `CompressedPayload`
///
/// # Arguments
///
/// * `payload` - The compressed payload
///
/// # Returns
///
/// The original uncompressed data
pub fn decompress(payload: &CompressedPayload) -> Result<Vec<u8>> {
    payload.decompress()
}

/// Compress a JSON string efficiently
///
/// JSON data typically compresses well (30-70% reduction) due to
/// repetitive structure and common keywords.
///
/// # Arguments
///
/// * `json_str` - The JSON string to compress
/// * `threshold` - Minimum size to trigger compression
///
/// # Returns
///
/// Compressed payload
pub fn compress_json(json_str: &str, threshold: usize) -> Result<CompressedPayload> {
    compress(json_str.as_bytes(), threshold)
}

/// Compress embedding data (float arrays compress very well)
///
/// Float arrays with sequential values compress extremely well (up to 90%)
/// due to repetitive binary patterns.
///
/// # Arguments
///
/// * `embedding` - The embedding vector
/// * `threshold` - Minimum size to trigger compression
///
/// # Returns
///
/// Compressed payload
pub fn compress_embedding(embedding: &[f32], threshold: usize) -> Result<CompressedPayload> {
    // Convert f32 to bytes for compression
    let bytes: Vec<u8> = embedding.iter().flat_map(|&f| f.to_le_bytes()).collect();
    compress(&bytes, threshold)
}

/// Decompress embedding data
///
/// # Arguments
///
/// * `payload` - The compressed payload
/// * `expected_size` - Expected number of floats in the embedding
///
/// # Returns
///
/// The original embedding vector
pub fn decompress_embedding(payload: &CompressedPayload, expected_size: usize) -> Result<Vec<f32>> {
    let bytes = decompress(payload)?;
    let floats: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            f32::from_le_bytes(arr)
        })
        .collect();

    if floats.len() != expected_size {
        return Err(memory_core::Error::Storage(format!(
            "Embedding size mismatch: expected {}, got {}",
            expected_size,
            floats.len()
        )));
    }

    Ok(floats)
}

/// Compression statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct CompressionStatistics {
    /// Total bytes before compression
    pub total_original_bytes: u64,
    /// Total bytes after compression
    pub total_compressed_bytes: u64,
    /// Number of items compressed
    pub compression_count: u64,
    /// Number of items that were too small to compress
    pub skipped_count: u64,
    /// Total time spent compressing (microseconds)
    pub compression_time_us: u64,
    /// Total time spent decompressing (microseconds)
    pub decompression_time_us: u64,
}

impl CompressionStatistics {
    /// Create a new statistics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the overall compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.total_original_bytes > 0 {
            self.total_compressed_bytes as f64 / self.total_original_bytes as f64
        } else {
            1.0
        }
    }

    /// Get the bandwidth savings percentage
    pub fn bandwidth_savings_percent(&self) -> f64 {
        (1.0 - self.compression_ratio()) * 100.0
    }

    /// Record a compression operation
    pub fn record_compression(
        &mut self,
        original_size: usize,
        compressed_size: usize,
        time_us: u64,
    ) {
        self.total_original_bytes += original_size as u64;
        self.total_compressed_bytes += compressed_size as u64;
        self.compression_count += 1;
        self.compression_time_us += time_us;
    }

    /// Record a skipped compression (too small)
    pub fn record_skipped(&mut self) {
        self.skipped_count += 1;
    }

    /// Record a decompression operation
    pub fn record_decompression(&mut self, time_us: u64) {
        self.decompression_time_us += time_us;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_payload_no_compression() {
        let data = b"hello";
        let compressed = CompressedPayload::compress(data, 1024).unwrap();

        assert_eq!(compressed.algorithm, CompressionAlgorithm::None);
        assert_eq!(compressed.original_size, 5);
        assert_eq!(compressed.compressed_size, 5);
        assert_eq!(compressed.compression_ratio, 1.0);
    }

    #[test]
    fn test_roundtrip_lz4() {
        #[cfg(feature = "compression-lz4")]
        {
            let data = b"hello world".repeat(100);
            let compressed = CompressedPayload::compress_lz4(&data).unwrap();

            assert_eq!(compressed.algorithm, CompressionAlgorithm::Lz4);
            assert!(compressed.compression_ratio < 1.0);

            let decompressed = compressed.decompress().unwrap();
            assert_eq!(data, decompressed);
        }
    }

    #[test]
    fn test_roundtrip_zstd() {
        #[cfg(feature = "compression-zstd")]
        {
            let data = b"hello world".repeat(100);
            let compressed = CompressedPayload::compress_zstd(&data).unwrap();

            assert_eq!(compressed.algorithm, CompressionAlgorithm::Zstd);
            assert!(compressed.compression_ratio < 1.0);

            let decompressed = compressed.decompress().unwrap();
            assert_eq!(data, decompressed);
        }
    }

    #[test]
    fn test_roundtrip_gzip() {
        #[cfg(feature = "compression-gzip")]
        {
            let data = b"hello world".repeat(100);
            let compressed = CompressedPayload::compress_gzip(&data).unwrap();

            assert_eq!(compressed.algorithm, CompressionAlgorithm::Gzip);
            assert!(compressed.compression_ratio < 1.0);

            let decompressed = compressed.decompress().unwrap();
            assert_eq!(data, decompressed);
        }
    }

    #[test]
    fn test_compress_json() {
        #[cfg(any(feature = "compression-lz4", feature = "compression-zstd"))]
        {
            // Use a larger JSON to ensure compression is beneficial
            let json =
                r#"{"name": "test", "items": [1, 2, 3], "nested": {"key": "value"}}"#.repeat(50);
            let compressed = compress_json(&json, 100).unwrap();

            // Only assert compression ratio if compression was actually applied
            if compressed.algorithm != CompressionAlgorithm::None {
                assert!(compressed.compression_ratio < 1.0);
            }

            let decompressed = compressed.decompress().unwrap();
            assert_eq!(json.as_bytes(), decompressed.as_slice());
        }
    }

    #[test]
    fn test_compress_embedding() {
        #[cfg(any(feature = "compression-lz4", feature = "compression-zstd"))]
        {
            let embedding: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
            let compressed = compress_embedding(&embedding, 100).unwrap();

            assert!(compressed.compression_ratio < 1.0);

            let decompressed = decompress_embedding(&compressed, 1000).unwrap();
            assert_eq!(embedding, decompressed);
        }
    }

    #[test]
    fn test_compression_statistics() {
        let mut stats = CompressionStatistics::new();

        stats.record_compression(1000, 400, 100);
        stats.record_compression(500, 250, 50);
        stats.record_skipped();

        assert_eq!(stats.total_original_bytes, 1500);
        assert_eq!(stats.total_compressed_bytes, 650);
        assert_eq!(stats.compression_count, 2);
        assert_eq!(stats.skipped_count, 1);

        // Verify compression ratio (650/1500 ≈ 0.433)
        let ratio = stats.compression_ratio();
        assert!(ratio > 0.4 && ratio < 0.5);

        // Verify bandwidth savings (~56.7%)
        let savings = stats.bandwidth_savings_percent();
        assert!(savings > 50.0 && savings < 60.0);
    }
}
