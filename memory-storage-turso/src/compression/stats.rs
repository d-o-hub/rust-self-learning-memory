//! # Compression Statistics
//!
//! This module provides statistics tracking for compression operations.

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
    /// Number of failed compression operations
    pub failed_count: u64,
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

    /// Record a failed compression operation
    pub fn record_failed(&mut self) {
        self.failed_count += 1;
    }
}
