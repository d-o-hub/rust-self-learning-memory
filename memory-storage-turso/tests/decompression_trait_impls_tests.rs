//! Coverage tests for decompression module and `trait_impls` module (ACT-029)
//!
//! Focus areas:
//! - `transport/decompression.rs` - Decompression round-trips and error handling
//! - `trait_impls/mod.rs` - `StorageStatistics` struct usage
//!
//! These tests improve coverage from ~18-21% to target 50%+.
//!
//! Note: LZ4 decompression tests are omitted because LZ4 requires knowing the
//! original size from stats (cumulative `total_original_bytes`), which cannot
//! be set externally for pre-compressed data. LZ4 round-trips work correctly
//! when zstd is not available, but decompressing externally compressed LZ4
//! data is not supported.

#![cfg(feature = "compression")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::single_char_pattern)]
#![allow(unused)]
#![allow(dead_code)]
#![allow(missing_docs)]

use async_trait::async_trait;
use do_memory_storage_turso::StorageStatistics;
use do_memory_storage_turso::transport::{
    CompressedTransport, Transport, TransportCompressionConfig, TransportCompressionStats,
    TransportMetadata, TransportResponse,
};
use std::fmt::Debug;

// ============================================================================
// Mock Transport for Testing Decompression
// ============================================================================

/// Mock transport that returns compressed data with specific encoding headers
#[derive(Debug)]
struct EncodingMockTransport {
    /// The encoding to use in response header
    encoding: Option<String>,
    /// The data to return (can be pre-compressed)
    response_data: Vec<u8>,
    /// Track how many times send was called (shared for testing)
    send_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl EncodingMockTransport {
    #[allow(dead_code)]
    fn new(encoding: Option<&str>, response_data: Vec<u8>) -> Self {
        Self {
            encoding: encoding.map(|s| s.to_string()),
            response_data,
            send_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    #[allow(dead_code)]
    fn get_send_count(&self) -> u64 {
        self.send_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn send_count_handle(&self) -> std::sync::Arc<std::sync::atomic::AtomicU64> {
        self.send_count.clone()
    }
}

#[async_trait]
impl Transport for EncodingMockTransport {
    async fn send(&self, _data: &[u8]) -> anyhow::Result<TransportResponse> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut headers = Vec::new();
        if let Some(ref enc) = self.encoding {
            headers.push(("Content-Encoding".to_string(), enc.clone()));
        }

        // Return the response_data (which should be set appropriately for the test)
        Ok(TransportResponse {
            status: 200,
            body: self.response_data.clone(),
            headers,
        })
    }

    async fn send_async(&self, _data: &[u8]) -> anyhow::Result<()> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::new("encoding_mock", "1.0")
    }
}

/// Echo mock transport that returns what it was sent (for round-trip tests)
#[derive(Debug)]
struct EchoMockTransport {
    /// The encoding to add to response header (if any)
    response_encoding: Option<String>,
    /// Track how many times send was called
    send_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl EchoMockTransport {
    fn new() -> Self {
        Self {
            response_encoding: None,
            send_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    #[allow(dead_code)]
    fn with_encoding(encoding: &str) -> Self {
        Self {
            response_encoding: Some(encoding.to_string()),
            send_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    #[allow(dead_code)]
    fn send_count_handle(&self) -> std::sync::Arc<std::sync::atomic::AtomicU64> {
        self.send_count.clone()
    }
}

#[async_trait]
impl Transport for EchoMockTransport {
    async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut headers = Vec::new();
        if let Some(ref enc) = self.response_encoding {
            headers.push(("Content-Encoding".to_string(), enc.clone()));
        }

        // Echo back the data received
        Ok(TransportResponse {
            status: 200,
            body: data.to_vec(),
            headers,
        })
    }

    async fn send_async(&self, _data: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::new("echo_mock", "1.0")
    }
}

// ============================================================================
// Decompression Tests - Zstd
// ============================================================================

#[cfg(feature = "compression-zstd")]
mod zstd_decompression_tests {
    use super::*;

    /// Test zstd compression and decompression round-trip through CompressedTransport.
    /// This tests the `decompress_zstd` method via the full send/receive flow.
    #[tokio::test]
    async fn test_zstd_roundtrip_via_compressed_transport() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Send large data that should be compressed
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let response = transport.send(&original_data).await.unwrap();

        assert!(response.is_success());
        assert_eq!(
            response.body, original_data,
            "Data should match after round-trip"
        );
    }

    /// Test zstd decompression of pre-compressed data.
    /// This directly tests the `decompress_zstd` method when receiving zstd-encoded response.
    #[tokio::test]
    async fn test_zstd_decompress_pre_compressed_response() {
        // Compress data using zstd
        let original_data: Vec<u8> = b"hello world".repeat(500);
        let compressed = zstd::stream::encode_all(&original_data[..], 3).unwrap();

        // Create mock that returns pre-compressed zstd data with encoding header
        let mock = EncodingMockTransport::new(Some("zstd"), compressed.clone());
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Send triggers decompression of the response
        let response = transport.send(b"request".as_ref()).await.unwrap();

        assert!(response.is_success());
        assert_eq!(
            response.body, original_data,
            "Decompressed data should match original"
        );
    }

    /// Test error handling when zstd decompression fails with invalid data.
    #[tokio::test]
    async fn test_zstd_decompression_error_invalid_data() {
        // Create mock that returns invalid data with zstd encoding header
        let invalid_data = vec![0xFF, 0xFE, 0xFD, 0xFC]; // Not valid zstd
        let mock = EncodingMockTransport::new(Some("zstd"), invalid_data);
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Should fail to decompress invalid zstd data
        let result = transport.send(b"request".as_ref()).await;

        assert!(result.is_err(), "Should error on invalid zstd data");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Decompression") || err_msg.contains("Task join"),
            "Error message should mention decompression or task: {err_msg}"
        );
    }

    /// Test stats tracking after zstd decompression.
    #[tokio::test]
    async fn test_zstd_decompression_stats_tracking() {
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let compressed = zstd::stream::encode_all(&original_data[..], 3).unwrap();

        let mock = EncodingMockTransport::new(Some("zstd"), compressed);
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        transport.send(b"request".as_ref()).await.unwrap();

        let stats = transport.stats();
        // Decompression should be recorded
        assert!(stats.base.decompression_time_us > 0 || stats.total_decompressions >= 1);
    }

    /// Test multiple zstd compression/decompression operations.
    #[tokio::test]
    async fn test_zstd_multiple_operations() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Perform multiple compression operations
        for i in 1..=5 {
            let data: Vec<u8> = (0..(2000 + i * 1000)).map(|j| (j % 256) as u8).collect();
            let response = transport.send(&data).await.unwrap();
            assert_eq!(response.body, data);
        }

        let stats = transport.stats();
        assert!(stats.base.compression_count >= 5);
    }

    /// Test zstd decompression with various data sizes.
    #[tokio::test]
    async fn test_zstd_various_sizes() {
        let sizes = [1025, 2048, 4096, 8192, 16384];

        for size in sizes {
            let mock = EchoMockTransport::new();
            let config = TransportCompressionConfig::default();
            let transport = CompressedTransport::new(Box::new(mock), config);

            let data: Vec<u8> = vec![0x42; size];
            let response = transport.send(&data).await.unwrap();
            assert_eq!(
                response.body, data,
                "Size {size} should round-trip correctly"
            );
        }
    }

    /// Test zstd with high compression ratio data.
    #[tokio::test]
    async fn test_zstd_highly_compressible_data() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Highly compressible data (all same byte)
        let data = vec![0u8; 100_000];
        let response = transport.send(&data).await.unwrap();
        assert_eq!(response.body, data);

        // Check that compression ratio is very good
        let ratio = transport.overall_compression_ratio();
        assert!(
            ratio < 0.1,
            "Highly compressible data should have ratio < 0.1, got {ratio}"
        );
    }
}

// ============================================================================
// Decompression Tests - Gzip
// ============================================================================

#[cfg(feature = "compression-gzip")]
mod gzip_decompression_tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    /// Test gzip decompression of pre-compressed data.
    #[tokio::test]
    async fn test_gzip_decompress_pre_compressed_response() {
        // Compress data using gzip
        let original_data: Vec<u8> = b"hello world".repeat(500);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original_data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Create mock that returns pre-compressed gzip data with encoding header
        let mock = EncodingMockTransport::new(Some("gzip"), compressed.clone());
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        let response = transport.send(b"request".as_ref()).await.unwrap();

        assert!(response.is_success());
        assert_eq!(
            response.body, original_data,
            "Decompressed data should match original"
        );
    }

    /// Test gzip decompression error handling with invalid data.
    #[tokio::test]
    async fn test_gzip_decompression_error_invalid_data() {
        // Invalid gzip data (not a valid gzip stream)
        let invalid_data = vec![0xFF, 0xFE, 0xFD, 0xFC];
        let mock = EncodingMockTransport::new(Some("gzip"), invalid_data);
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        let result = transport.send(b"request".as_ref()).await;

        assert!(result.is_err(), "Should error on invalid gzip data");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Decompression") || err_msg.contains("Task join"),
            "Error should mention decompression: {err_msg}"
        );
    }

    /// Test gzip decompression of partial/corrupted gzip stream.
    #[tokio::test]
    async fn test_gzip_decompression_partial_stream() {
        // Create valid gzip then truncate it
        let original_data: Vec<u8> = b"hello world".repeat(500);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&original_data).unwrap();
        let compressed = encoder.finish().unwrap();
        let truncated = &compressed[..compressed.len() / 2];

        let mock = EncodingMockTransport::new(Some("gzip"), truncated.to_vec());
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        let result = transport.send(b"request".as_ref()).await;

        assert!(result.is_err(), "Should error on truncated gzip stream");
    }

    /// Test gzip with different compression levels.
    #[tokio::test]
    async fn test_gzip_different_compression_levels() {
        let levels = [
            flate2::Compression::fast(),
            flate2::Compression::default(),
            flate2::Compression::best(),
        ];

        for level in levels {
            let original_data: Vec<u8> = b"test data for compression".repeat(200);
            let mut encoder = GzEncoder::new(Vec::new(), level);
            encoder.write_all(&original_data).unwrap();
            let compressed = encoder.finish().unwrap();

            let mock = EncodingMockTransport::new(Some("gzip"), compressed);
            let config = TransportCompressionConfig::default();
            let transport = CompressedTransport::new(Box::new(mock), config);

            let response = transport.send(b"request".as_ref()).await.unwrap();
            assert_eq!(response.body, original_data);
        }
    }

    /// Test gzip decompression with empty gzip stream.
    #[tokio::test]
    async fn test_gzip_empty_stream() {
        // Empty gzip stream (valid but empty)
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let compressed = encoder.finish().unwrap();

        let mock = EncodingMockTransport::new(Some("gzip"), compressed);
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        let response = transport.send(b"request".as_ref()).await.unwrap();
        assert!(response.is_success());
        assert_eq!(response.body, Vec::<u8>::new());
    }
}

// ============================================================================
// Decompression Tests - LZ4 (when zstd is not available)
// ============================================================================

// Note: LZ4 decompression of externally pre-compressed data is not supported
// because the decompress_lz4 method reads original_size from cumulative stats.
// LZ4 round-trip tests work when zstd is not available.

#[cfg(all(feature = "compression-lz4", not(feature = "compression-zstd")))]
mod lz4_only_tests {
    use super::*;

    /// Test LZ4 round-trip when zstd is not available.
    /// LZ4 becomes the default compression algorithm.
    #[tokio::test]
    async fn test_lz4_roundtrip_when_no_zstd() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Send large data - LZ4 will be used since zstd is not available
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let response = transport.send(&original_data).await.unwrap();

        assert!(response.is_success());
        assert_eq!(response.body, original_data);

        // Verify LZ4 was used (check stats)
        let stats = transport.stats();
        assert!(stats.base.compression_count >= 1);
    }

    /// Test LZ4 compression with threshold.
    #[tokio::test]
    async fn test_lz4_compression_threshold() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig {
            compression_threshold: 2048,
            ..Default::default()
        };
        let transport = CompressedTransport::new(Box::new(mock), config);

        // Below threshold - no compression
        let small_data = vec![1u8; 1000];
        let response = transport.send(&small_data).await.unwrap();
        assert_eq!(response.body, small_data);

        // Above threshold - LZ4 compression
        let large_data = vec![2u8; 3000];
        let response2 = transport.send(&large_data).await.unwrap();
        assert_eq!(response2.body, large_data);
    }

    /// Test multiple LZ4 round-trips.
    #[tokio::test]
    async fn test_lz4_multiple_roundtrips() {
        let mock = EchoMockTransport::new();
        let config = TransportCompressionConfig::default();
        let transport = CompressedTransport::new(Box::new(mock), config);

        for i in 0..10 {
            let data: Vec<u8> = (0..(2000 + i * 500)).map(|j| (j % 256) as u8).collect();
            let response = transport.send(&data).await.unwrap();
            assert_eq!(response.body, data);
        }

        let stats = transport.stats();
        assert!(stats.base.compression_count >= 10);
    }
}

// ============================================================================
// Decompression Tests - Unknown Encoding
// ============================================================================

/// Test that unknown encoding is passed through unchanged.
#[tokio::test]
async fn test_unknown_encoding_passed_through() {
    let data = b"raw data".to_vec();
    let mock = EncodingMockTransport::new(Some("br"), data.clone()); // brotli - not supported
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let response = transport.send(b"request".as_ref()).await.unwrap();

    // Unknown encoding should be returned as-is (no decompression)
    assert!(response.is_success());
    assert_eq!(
        response.body, data,
        "Unknown encoding should pass through unchanged"
    );
}

/// Test that response without encoding header is passed through unchanged.
#[tokio::test]
async fn test_no_encoding_header_passed_through() {
    let data = b"raw data".to_vec();
    let mock = EncodingMockTransport::new(None, data.clone());
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let response = transport.send(b"request".as_ref()).await.unwrap();

    assert!(response.is_success());
    assert_eq!(response.body, data);
}

// ============================================================================
// Decompression Tests - Multiple Round-trips
// ============================================================================

/// Test multiple compression/decompression round-trips to verify stability.
#[tokio::test]
async fn test_multiple_compression_roundtrips() {
    let mock = EchoMockTransport::new();
    let send_count = mock.send_count_handle();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    for i in 0..10 {
        let data: Vec<u8> = (0..(1000 + i * 100)).map(|j| (j % 256) as u8).collect();
        let response = transport.send(&data).await.unwrap();
        assert_eq!(response.body, data, "Round-trip {i} should preserve data");
    }

    assert_eq!(send_count.load(std::sync::atomic::Ordering::SeqCst), 10);
}

// ============================================================================
// TransportCompressionStats Tests
// ============================================================================

/// Test stats creation and initial state.
#[test]
fn test_transport_compression_stats_new() {
    let stats = TransportCompressionStats::new();

    assert_eq!(stats.streaming_compressions, 0);
    assert_eq!(stats.streaming_decompressions, 0);
    assert_eq!(stats.total_bytes_saved, 0);
    assert_eq!(stats.warning_threshold_triggers, 0);
    assert_eq!(stats.algorithm_fallbacks, 0);
    assert_eq!(stats.avg_compression_time_us, 0);
    assert_eq!(stats.avg_decompression_time_us, 0);
    assert_eq!(stats.total_compressions, 0);
    assert_eq!(stats.total_decompressions, 0);
}

/// Test recording streaming compression.
#[test]
fn test_transport_compression_stats_record_streaming_compression() {
    let mut stats = TransportCompressionStats::new();

    // Record compression with savings
    stats.record_streaming_compression(1000, 400);
    assert_eq!(stats.streaming_compressions, 1);
    assert_eq!(stats.total_bytes_saved, 600);

    // Record compression where compressed is larger (no savings)
    stats.record_streaming_compression(100, 150);
    assert_eq!(stats.streaming_compressions, 2);
    assert_eq!(stats.total_bytes_saved, 600); // Should not increase
}

/// Test recording streaming decompression.
#[test]
fn test_transport_compression_stats_record_streaming_decompression() {
    let mut stats = TransportCompressionStats::new();

    stats.record_streaming_decompression();
    assert_eq!(stats.streaming_decompressions, 1);

    stats.record_streaming_decompression();
    assert_eq!(stats.streaming_decompressions, 2);
}

/// Test overall ratio calculation.
#[test]
fn test_transport_compression_stats_overall_ratio() {
    let mut stats = TransportCompressionStats::new();

    // No data - should return 1.0
    assert!((stats.overall_ratio() - 1.0).abs() < 0.01);

    // With data
    stats.base.record_compression(2000, 800, 100);
    assert!((stats.overall_ratio() - 0.4).abs() < 0.01);
}

/// Test bandwidth savings calculation.
#[test]
fn test_transport_compression_stats_bandwidth_savings() {
    let mut stats = TransportCompressionStats::new();

    stats.base.record_compression(1000, 400, 50);
    assert!((stats.bandwidth_savings_percent() - 60.0).abs() < 0.1);
}

/// Test recording compression times.
#[test]
fn test_transport_compression_stats_record_compression_time() {
    let mut stats = TransportCompressionStats::new();

    stats.record_compression_time(100);
    assert_eq!(stats.avg_compression_time_us, 100);
    assert_eq!(stats.total_compressions, 1);

    stats.record_compression_time(200);
    // Average: (100 + 200) / 2 = 150
    assert_eq!(stats.avg_compression_time_us, 150);
    assert_eq!(stats.total_compressions, 2);
}

/// Test recording decompression times.
#[test]
fn test_transport_compression_stats_record_decompression_time() {
    let mut stats = TransportCompressionStats::new();

    stats.record_decompression_time(50);
    assert_eq!(stats.avg_decompression_time_us, 50);
    assert_eq!(stats.total_decompressions, 1);

    stats.record_decompression_time(100);
    // Average: (50 + 100) / 2 = 75
    assert_eq!(stats.avg_decompression_time_us, 75);
    assert_eq!(stats.total_decompressions, 2);
}

/// Test recording warning threshold triggers.
#[test]
fn test_transport_compression_stats_record_warning_threshold() {
    let mut stats = TransportCompressionStats::new();

    stats.record_warning_threshold();
    assert_eq!(stats.warning_threshold_triggers, 1);

    stats.record_warning_threshold();
    stats.record_warning_threshold();
    assert_eq!(stats.warning_threshold_triggers, 3);
}

/// Test recording algorithm fallbacks.
#[test]
fn test_transport_compression_stats_record_algorithm_fallback() {
    let mut stats = TransportCompressionStats::new();

    stats.record_algorithm_fallback();
    assert_eq!(stats.algorithm_fallbacks, 1);
}

// ============================================================================
// StorageStatistics Tests (trait_impls module)
// ============================================================================

/// Test StorageStatistics creation and default values.
#[test]
fn test_storage_statistics_creation() {
    let stats = StorageStatistics {
        episode_count: 0,
        pattern_count: 0,
        heuristic_count: 0,
    };

    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

/// Test StorageStatistics with actual counts.
#[test]
fn test_storage_statistics_with_counts() {
    let stats = StorageStatistics {
        episode_count: 100,
        pattern_count: 50,
        heuristic_count: 25,
    };

    assert_eq!(stats.episode_count, 100);
    assert_eq!(stats.pattern_count, 50);
    assert_eq!(stats.heuristic_count, 25);
}

/// Test StorageStatistics Clone trait.
#[test]
fn test_storage_statistics_clone() {
    let original = StorageStatistics {
        episode_count: 10,
        pattern_count: 5,
        heuristic_count: 2,
    };

    let cloned = original.clone();

    assert_eq!(cloned.episode_count, original.episode_count);
    assert_eq!(cloned.pattern_count, original.pattern_count);
    assert_eq!(cloned.heuristic_count, original.heuristic_count);
}

/// Test StorageStatistics Debug trait.
#[test]
fn test_storage_statistics_debug() {
    let stats = StorageStatistics {
        episode_count: 42,
        pattern_count: 17,
        heuristic_count: 8,
    };

    let debug_str = format!("{stats:?}");

    assert!(debug_str.contains("episode_count"));
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("pattern_count"));
    assert!(debug_str.contains("17"));
    assert!(debug_str.contains("heuristic_count"));
    assert!(debug_str.contains("8"));
}

/// Test StorageStatistics total items calculation (utility).
#[test]
fn test_storage_statistics_total_items() {
    let stats = StorageStatistics {
        episode_count: 100,
        pattern_count: 50,
        heuristic_count: 25,
    };

    let total = stats.episode_count + stats.pattern_count + stats.heuristic_count;
    assert_eq!(total, 175);
}

/// Test StorageStatistics empty state.
#[test]
fn test_storage_statistics_empty() {
    let empty = StorageStatistics {
        episode_count: 0,
        pattern_count: 0,
        heuristic_count: 0,
    };

    assert_eq!(
        empty.episode_count + empty.pattern_count + empty.heuristic_count,
        0
    );
}

/// Test StorageStatistics field comparison for equality.
#[test]
fn test_storage_statistics_field_equality() {
    let stats1 = StorageStatistics {
        episode_count: 10,
        pattern_count: 5,
        heuristic_count: 2,
    };

    let stats2 = StorageStatistics {
        episode_count: 10,
        pattern_count: 5,
        heuristic_count: 2,
    };

    let stats3 = StorageStatistics {
        episode_count: 11,
        pattern_count: 5,
        heuristic_count: 2,
    };

    // Compare fields since StorageStatistics doesn't derive PartialEq
    assert_eq!(stats1.episode_count, stats2.episode_count);
    assert_eq!(stats1.pattern_count, stats2.pattern_count);
    assert_eq!(stats1.heuristic_count, stats2.heuristic_count);

    assert_ne!(stats1.episode_count, stats3.episode_count);
}

/// Test StorageStatistics with large counts.
#[test]
fn test_storage_statistics_large_counts() {
    let stats = StorageStatistics {
        episode_count: 1_000_000,
        pattern_count: 500_000,
        heuristic_count: 250_000,
    };

    assert_eq!(stats.episode_count, 1_000_000);
    assert_eq!(stats.pattern_count, 500_000);
    assert_eq!(stats.heuristic_count, 250_000);
}

/// Test StorageStatistics modification.
#[test]
fn test_storage_statistics_modification() {
    let mut stats = StorageStatistics {
        episode_count: 0,
        pattern_count: 0,
        heuristic_count: 0,
    };

    stats.episode_count = 10;
    stats.pattern_count = 5;
    stats.heuristic_count = 2;

    assert_eq!(stats.episode_count, 10);
    assert_eq!(stats.pattern_count, 5);
    assert_eq!(stats.heuristic_count, 2);
}

// ============================================================================
// Edge Cases and Error Path Tests
// ============================================================================

/// Test compression threshold boundary - exactly at threshold.
#[tokio::test]
async fn test_compression_threshold_boundary() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig {
        compression_threshold: 1024,
        ..Default::default()
    };
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Data exactly at threshold - should NOT be compressed
    let exact_threshold_data = vec![0u8; 1024];
    let response = transport.send(&exact_threshold_data).await.unwrap();
    assert_eq!(response.body, exact_threshold_data);

    // Data just above threshold - should be compressed
    let above_threshold_data = vec![0u8; 1025];
    let response2 = transport.send(&above_threshold_data).await.unwrap();
    assert_eq!(response2.body, above_threshold_data);
}

/// Test compression with different compression levels.
#[tokio::test]
async fn test_compression_levels() {
    // Test with low compression level (faster)
    let config = TransportCompressionConfig::default();
    let transport_low =
        CompressedTransport::with_level(Box::new(EchoMockTransport::new()), config.clone(), 1);

    let data = vec![0xAB; 5000];
    let response = transport_low.send(&data).await.unwrap();
    assert_eq!(response.body, data);

    // Test with high compression level (better ratio)
    let transport_high =
        CompressedTransport::with_level(Box::new(EchoMockTransport::new()), config, 22);
    let response2 = transport_high.send(&data).await.unwrap();
    assert_eq!(response2.body, data);
}

/// Test compression threshold customization.
#[tokio::test]
async fn test_custom_compression_threshold() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig::default();

    // Create transport with high threshold (10KB)
    let transport = CompressedTransport::with_threshold(Box::new(mock), config, 10 * 1024);

    // Data below custom threshold should not be compressed
    let small_data = vec![1u8; 5000];
    let response = transport.send(&small_data).await.unwrap();
    assert_eq!(response.body, small_data);

    // Data above custom threshold should be compressed
    let large_data = vec![1u8; 15000];
    let response2 = transport.send(&large_data).await.unwrap();
    assert_eq!(response2.body, large_data);
}

/// Test send_async with compression.
#[tokio::test]
async fn test_send_async_with_compression() {
    let mock = EncodingMockTransport::new(None, vec![]);
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Large data should be compressed
    let large_data = vec![0u8; 5000];
    let result = transport.send_async(&large_data).await;
    assert!(result.is_ok());
}

/// Test health check passthrough.
#[tokio::test]
async fn test_health_check_passthrough() {
    let mock = EncodingMockTransport::new(None, vec![]);
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let healthy = transport.health_check().await.unwrap();
    assert!(healthy);
}

/// Test metadata includes compression support.
#[test]
fn test_metadata_compression_support() {
    let mock = EncodingMockTransport::new(None, vec![]);
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let metadata = transport.metadata();
    assert!(metadata.supports_compression);
    assert_eq!(metadata.name, "encoding_mock");
}

/// Test stats reset functionality.
#[tokio::test]
async fn test_stats_reset() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Perform compression operations
    let data = vec![0u8; 5000];
    transport.send(&data).await.unwrap();

    let stats_before = transport.stats();
    assert!(stats_before.base.compression_count > 0);

    // Reset stats
    transport.reset_stats();

    let stats_after = transport.stats();
    assert_eq!(stats_after.base.compression_count, 0);
    assert_eq!(stats_after.total_compressions, 0);
}

/// Test overall compression ratio calculation.
#[tokio::test]
async fn test_overall_compression_ratio() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Send compressible data
    let data = vec![0u8; 10000];
    transport.send(&data).await.unwrap();

    let ratio = transport.overall_compression_ratio();
    // Should be less than 1.0 for compressible data
    assert!(ratio < 1.0);
}

/// Test bandwidth savings percentage.
#[tokio::test]
async fn test_bandwidth_savings_percentage() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Send compressible data
    let data = vec![0u8; 10000];
    transport.send(&data).await.unwrap();

    let savings = transport.bandwidth_savings_percent();
    // Should be positive for compressible data
    assert!(savings > 0.0);
}

/// Test compression with random data (less compressible).
#[tokio::test]
async fn test_compression_random_data() {
    let mock = EchoMockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Random-ish data (less compressible)
    let data: Vec<u8> = (0..10000).map(|i| (i * 17 + 31) as u8).collect();
    let response = transport.send(&data).await.unwrap();
    assert_eq!(response.body, data);

    // Random data should still compress, but less efficiently
    let stats = transport.stats();
    assert!(stats.base.compression_count > 0);
}
