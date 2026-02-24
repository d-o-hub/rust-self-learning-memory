use super::{CompressedTransport, TransportCompressionStats};
use crate::compression::CompressionAlgorithm;

#[derive(Debug, Clone)]
pub(super) struct CompressionResult {
    pub(super) data: Vec<u8>,
    pub(super) original_size: usize,
    pub(super) compressed_size: usize,
    pub(super) algorithm: CompressionAlgorithm,
    pub(super) compression_ratio: f64,
}

impl CompressedTransport {
    pub fn stats(&self) -> TransportCompressionStats {
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for reading: {}", e);
                TransportCompressionStats::new()
            })
    }

    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = TransportCompressionStats::new();
        } else {
            tracing::error!("Failed to acquire stats lock for reset");
        }
    }

    pub fn overall_compression_ratio(&self) -> f64 {
        self.stats
            .lock()
            .map(|guard| guard.overall_ratio())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for ratio: {}", e);
                0.0
            })
    }

    pub fn bandwidth_savings_percent(&self) -> f64 {
        self.stats
            .lock()
            .map(|guard| guard.bandwidth_savings_percent())
            .unwrap_or_else(|e| {
                tracing::error!("Failed to acquire stats lock for bandwidth: {}", e);
                0.0
            })
    }
}
