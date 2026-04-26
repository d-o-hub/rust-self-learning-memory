use super::*;
use crate::compression::CompressionAlgorithm;

#[test]
fn test_config_default() {
    let config = TransportCompressionConfig::default();
    assert_eq!(config.compression_threshold, 1024);
    assert_eq!(config.auto_algorithm_selection, true);
    assert_eq!(config.preferred_algorithm, CompressionAlgorithm::Zstd);
    assert_eq!(config.max_compressed_size, 10 * 1024 * 1024);
    assert_eq!(config.enable_metrics, true);
    assert_eq!(config.warning_ratio_threshold, 0.9);
    assert_eq!(config.min_acceptable_ratio, 0.5);
}

#[test]
fn test_stats_new() {
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

#[test]
fn test_record_streaming_compression() {
    let mut stats = TransportCompressionStats::new();

    // Test recording with actual savings
    stats.record_streaming_compression(1000, 600);
    assert_eq!(stats.streaming_compressions, 1);
    assert_eq!(stats.total_bytes_saved, 400);

    // Test recording with no savings (compressed is larger)
    stats.record_streaming_compression(500, 550);
    assert_eq!(stats.streaming_compressions, 2);
    assert_eq!(stats.total_bytes_saved, 400); // Should not change
}

#[test]
fn test_record_streaming_decompression() {
    let mut stats = TransportCompressionStats::new();
    stats.record_streaming_decompression();
    stats.record_streaming_decompression();

    assert_eq!(stats.streaming_decompressions, 2);
}

#[test]
fn test_record_warning_threshold() {
    let mut stats = TransportCompressionStats::new();
    stats.record_warning_threshold();
    assert_eq!(stats.warning_threshold_triggers, 1);
}

#[test]
fn test_record_algorithm_fallback() {
    let mut stats = TransportCompressionStats::new();
    stats.record_algorithm_fallback();
    assert_eq!(stats.algorithm_fallbacks, 1);
}

#[test]
fn test_record_compression_time() {
    let mut stats = TransportCompressionStats::new();

    // First recording
    stats.record_compression_time(100);
    assert_eq!(stats.total_compressions, 1);
    assert_eq!(stats.avg_compression_time_us, 100);

    // Second recording
    stats.record_compression_time(200);
    assert_eq!(stats.total_compressions, 2);
    assert_eq!(stats.avg_compression_time_us, 150); // (100 + 200) / 2
}

#[test]
fn test_record_decompression_time() {
    let mut stats = TransportCompressionStats::new();

    // First recording
    stats.record_decompression_time(100);
    assert_eq!(stats.total_decompressions, 1);
    assert_eq!(stats.avg_decompression_time_us, 100);

    // Second recording
    stats.record_decompression_time(200);
    assert_eq!(stats.total_decompressions, 2);
    assert_eq!(stats.avg_decompression_time_us, 150);
}

#[test]
fn test_ratio_and_savings() {
    let mut stats = TransportCompressionStats::new();
    // Simulate base stats changing for ratios
    stats.base.total_original_bytes = 1000;
    stats.base.total_compressed_bytes = 600;

    // Ratio should be 600 / 1000 = 0.6
    assert_eq!(stats.overall_ratio(), 0.6);

    // Savings should be (1 - 0.6) * 100 = 40%
    assert_eq!(stats.bandwidth_savings_percent(), 40.0);
}

#[test]
fn test_transport_compression_error_display() {
    assert_eq!(
        TransportCompressionError::AlgorithmNotAvailable("lz4".to_string()).to_string(),
        "Compression algorithm not available: lz4"
    );
    assert_eq!(
        TransportCompressionError::DecompressionFailed("bad data".to_string()).to_string(),
        "Decompression failed: bad data"
    );
    assert_eq!(
        TransportCompressionError::SizeMismatch {
            expected: 100,
            actual: 50
        }
        .to_string(),
        "Payload size mismatch: expected 100, got 50"
    );
    assert_eq!(
        TransportCompressionError::ThresholdExceeded(5000).to_string(),
        "Compression threshold exceeded: 5000 bytes"
    );
    assert_eq!(
        TransportCompressionError::InvalidHeader.to_string(),
        "Invalid compressed data header"
    );
    assert_eq!(
        TransportCompressionError::StreamFailed("io error".to_string()).to_string(),
        "Streaming operation failed: io error"
    );
}

#[test]
fn test_compression_stream_result_creation() {
    let result = CompressionStreamResult {
        original_size: 1000,
        compressed_size: 500,
        algorithm: CompressionAlgorithm::Zstd,
        compression_ratio: 0.5,
        time_us: 150,
    };

    assert_eq!(result.original_size, 1000);
    assert_eq!(result.compressed_size, 500);
    assert_eq!(result.algorithm, CompressionAlgorithm::Zstd);
    assert_eq!(result.compression_ratio, 0.5);
    assert_eq!(result.time_us, 150);
}
