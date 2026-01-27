//! Snapshot types for cache statistics

use super::config::PressureLevel;

/// Snapshot of cache statistics at a point in time
#[derive(Debug, Clone)]
pub struct AdaptiveTtlStatsSnapshot {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions
    pub evictions: u64,
    /// Evictions due to memory pressure
    pub pressure_evictions: u64,
    /// Evictions due to size limit
    pub size_evictions: u64,
    /// Evictions due to TTL expiration
    pub ttl_evictions: u64,
    /// TTL adaptations
    pub ttl_adaptations: u64,
    /// Hit rate percentage
    pub hit_rate_percent: f64,
    /// Current pressure level
    pub pressure_level: PressureLevel,
    /// Number of hot items
    pub hot_item_count: usize,
    /// Number of cold items
    pub cold_item_count: usize,
    /// Base cache hits
    pub base_hits: u64,
    /// Base cache misses
    pub base_misses: u64,
    /// Base evictions
    pub base_evictions: u64,
    /// Base expirations
    pub base_expirations: u64,
}

impl AdaptiveTtlStatsSnapshot {
    /// Calculate overall hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if cache is effective (hit rate > 40%)
    pub fn is_effective(&self) -> bool {
        self.hit_rate() > 40.0
    }
}
