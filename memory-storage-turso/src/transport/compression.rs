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

mod compressor;
mod types;

use crate::compression::CompressedPayload;
pub use compressor::AsyncCompressor;
pub use types::{
    CompressionStreamResult, TransportCompressionConfig, TransportCompressionError,
    TransportCompressionStats,
};

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
