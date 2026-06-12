//! Cache statistics tracking with atomic counters.

use super::config::CacheStats;
use std::sync::atomic::{AtomicU64, Ordering};

/// Internal cache statistics with atomic counters
#[derive(Default)]
pub(crate) struct CacheStatsInner {
    pub(crate) episode_hits: AtomicU64,
    pub(crate) episode_misses: AtomicU64,
    pub(crate) pattern_hits: AtomicU64,
    pub(crate) pattern_misses: AtomicU64,
    pub(crate) heuristic_hits: AtomicU64,
    pub(crate) heuristic_misses: AtomicU64,
    pub(crate) query_hits: AtomicU64,
    pub(crate) query_misses: AtomicU64,
    pub(crate) evictions: AtomicU64,
    pub(crate) expirations: AtomicU64,
}

impl CacheStatsInner {
    /// Snapshot the current counters into a `CacheStats`.
    pub fn snapshot(&self) -> CacheStats {
        CacheStats {
            episode_hits: self.episode_hits.load(Ordering::Relaxed),
            episode_misses: self.episode_misses.load(Ordering::Relaxed),
            pattern_hits: self.pattern_hits.load(Ordering::Relaxed),
            pattern_misses: self.pattern_misses.load(Ordering::Relaxed),
            query_hits: self.query_hits.load(Ordering::Relaxed),
            query_misses: self.query_misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            expirations: self.expirations.load(Ordering::Relaxed),
        }
    }

    pub fn record_query_hit(&self) {
        self.query_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_query_miss(&self) {
        self.query_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_expiration(&self) {
        self.expirations.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_inner_default() {
        let stats = CacheStatsInner::default();
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.episode_hits, 0);
        assert_eq!(snapshot.query_hits, 0);
        assert_eq!(snapshot.evictions, 0);
        assert_eq!(snapshot.expirations, 0);
    }

    #[test]
    fn test_record_query_hit() {
        let stats = CacheStatsInner::default();
        stats.record_query_hit();
        stats.record_query_hit();
        assert_eq!(stats.query_hits.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_query_miss() {
        let stats = CacheStatsInner::default();
        stats.record_query_miss();
        assert_eq!(stats.query_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_eviction() {
        let stats = CacheStatsInner::default();
        stats.record_eviction();
        stats.record_eviction();
        stats.record_eviction();
        assert_eq!(stats.evictions.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_record_expiration() {
        let stats = CacheStatsInner::default();
        stats.record_expiration();
        assert_eq!(stats.expirations.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_snapshot_after_mutations() {
        let stats = CacheStatsInner::default();
        stats.record_query_hit();
        stats.record_query_hit();
        stats.record_query_miss();
        stats.record_eviction();
        stats.record_expiration();

        let snap = stats.snapshot();
        assert_eq!(snap.query_hits, 2);
        assert_eq!(snap.query_misses, 1);
        assert_eq!(snap.evictions, 1);
        assert_eq!(snap.expirations, 1);
    }
}
