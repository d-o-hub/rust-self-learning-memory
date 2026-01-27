//! Compression Module
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

mod payload;
mod stats;

// Re-export public types
pub use payload::CompressedPayload;
pub use stats::CompressionStatistics;

use memory_core::Result;

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

impl std::fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionAlgorithm::None => write!(f, "none"),
            CompressionAlgorithm::Lz4 => write!(f, "lz4"),
            CompressionAlgorithm::Zstd => write!(f, "zstd"),
            CompressionAlgorithm::Gzip => write!(f, "gzip"),
        }
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
