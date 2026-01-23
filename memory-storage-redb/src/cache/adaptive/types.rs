//! Adaptive cache configuration and metrics types

use std::time::Duration;

/// Configuration for adaptive TTL cache
#[derive(Debug, Clone)]
pub struct AdaptiveCacheConfig {
    /// Maximum number of items in cache
    pub max_size: usize,
    /// Default TTL when entry is first created
    pub default_ttl: Duration,
    /// Minimum TTL that entries can be reduced to
    pub min_ttl: Duration,
    /// Maximum TTL that entries can be increased to
    pub max_ttl: Duration,
    /// Access count threshold above which an item is considered "hot"
    pub hot_threshold: usize,
    /// Access count threshold below which an item is considered "cold"
    pub cold_threshold: usize,
    /// How fast TTL adapts (0.0 = no adaptation, 1.0 = instant change)
    pub adaptation_rate: f64,
    /// Number of accesses to track per entry in sliding window
    pub window_size: usize,
    /// Background cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable background cleanup task
    pub enable_background_cleanup: bool,
}

impl Default for AdaptiveCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Duration::from_secs(1800), // 30 minutes
            min_ttl: Duration::from_secs(300),      // 5 minutes
            max_ttl: Duration::from_secs(7200),     // 2 hours
            hot_threshold: 10,                      // 10+ accesses = hot
            cold_threshold: 2,                      // 2 or fewer accesses = cold
            adaptation_rate: 0.25,                  // 25% adjustment per window
            window_size: 20,                        // Track last 20 accesses
            cleanup_interval_secs: 60,              // 1 minute
            enable_background_cleanup: true,
        }
    }
}

/// Metrics specific to adaptive cache performance
#[derive(Debug, Clone, Default)]
pub struct AdaptiveCacheMetrics {
    /// Total number of hot items in cache
    pub hot_item_count: usize,
    /// Total number of cold items in cache
    pub cold_item_count: usize,
    /// Total number of TTL increases
    pub ttl_increases: u64,
    /// Total number of TTL decreases
    pub ttl_decreases: u64,
    /// Total number of TTL adjustments that hit bounds
    pub ttl_bound_hits: u64,
    /// Base metrics from standard cache
    pub base: super::super::CacheMetrics,
}

impl AdaptiveCacheMetrics {
    /// Calculate hit rate from base metrics
    pub fn hit_rate(&self) -> f64 {
        self.base.hit_rate
    }

    /// Check if cache is effective
    pub fn is_effective(&self) -> bool {
        self.base.hit_rate > 0.4
    }
}
