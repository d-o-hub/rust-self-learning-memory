//! Cache statistics and messages for the advanced query cache

use super::query_cache_types::{QueryKey, QueryType};
use std::collections::HashMap;

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct AdvancedCacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions due to capacity
    pub evictions: u64,
    /// Total expirations due to TTL
    pub expirations: u64,
    /// Total invalidations due to table changes
    pub invalidations: u64,
    /// Current cache size
    pub current_size: usize,
    /// Hot query count
    pub hot_queries: usize,
    /// Refresh operations performed
    pub refreshes: u64,
    /// Hit rate by query type
    pub hit_rate_by_type: HashMap<QueryType, f64>,
}

impl AdvancedCacheStats {
    /// Calculate overall hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Merge stats from another instance
    pub fn merge(&mut self, other: &AdvancedCacheStats) {
        self.hits += other.hits;
        self.misses += other.misses;
        self.evictions += other.evictions;
        self.expirations += other.expirations;
        self.invalidations += other.invalidations;
        self.refreshes += other.refreshes;
    }
}

/// Invalidation message for cache maintenance
#[derive(Debug, Clone)]
pub enum InvalidationMessage {
    /// Invalidate by table dependency
    TableChanged(super::query_cache_types::TableDependency),
    /// Invalidate specific query key
    InvalidateKey(QueryKey),
    /// Invalidate all queries
    InvalidateAll,
    /// Shutdown the invalidation handler
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        let mut stats = AdvancedCacheStats::default();
        stats.hits = 10;
        stats.misses = 5;

        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_stats_merge() {
        let mut stats1 = AdvancedCacheStats {
            hits: 10,
            misses: 5,
            ..Default::default()
        };
        let stats2 = AdvancedCacheStats {
            hits: 5,
            misses: 3,
            ..Default::default()
        };

        stats1.merge(&stats2);
        assert_eq!(stats1.hits, 15);
        assert_eq!(stats1.misses, 8);
    }
}
