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
