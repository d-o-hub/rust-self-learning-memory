//! Types and configuration for transport compression

use crate::compression::{
    CompressedPayload, CompressionAlgorithm, CompressionStatistics as BaseCompressionStats,
};
use thiserror::Error;

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
