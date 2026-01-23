//! # Query Cache Types
//!
//! Types for the query cache with LRU eviction and TTL.

use crate::episode::Episode;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default cache TTL (60 seconds)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(60);

/// Default maximum cache entries (10,000 queries)
pub const DEFAULT_MAX_ENTRIES: usize = 10_000;

/// Cache key combining query parameters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Query text or description
    pub query: String,
    /// Task domain filter (optional)
    pub domain: Option<Arc<str>>,
    /// Task type filter (optional)
    pub task_type: Option<String>,
    /// Time range start (unix timestamp, optional)
    pub time_start: Option<i64>,
    /// Time range end (unix timestamp, optional)
    pub time_end: Option<i64>,
    /// Maximum results to return
    pub limit: usize,
}

impl CacheKey {
    /// Create a new cache key
    #[must_use]
    pub fn new(query: String) -> Self {
        Self {
            query,
            domain: None,
            task_type: None,
            time_start: None,
            time_end: None,
            limit: 10,
        }
    }

    /// Set domain filter
    #[must_use]
    pub fn with_domain(mut self, domain: Option<String>) -> Self {
        self.domain = domain.map(|s| Arc::from(s.as_str()));
        self
    }

    /// Set task type filter
    #[must_use]
    pub fn with_task_type(mut self, task_type: Option<String>) -> Self {
        self.task_type = task_type;
        self
    }

    /// Set time range filter
    #[must_use]
    pub fn with_time_range(mut self, start: Option<i64>, end: Option<i64>) -> Self {
        self.time_start = start;
        self.time_end = end;
        self
    }

    /// Set result limit
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Compute hash for this cache key
    #[must_use]
    pub fn compute_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

/// Cached query result with expiration time
#[derive(Debug, Clone)]
pub struct CachedResult {
    /// Cached episodes (Arc for zero-copy retrieval)
    pub episodes: Arc<[Episode]>,
    /// Time when this entry was cached
    pub cached_at: Instant,
    /// Time-to-live for this entry
    pub ttl: Duration,
}

impl CachedResult {
    /// Check if this cached result has expired
    pub(crate) fn is_expired(&self) -> bool {
        self.cached_at.elapsed() >= self.ttl
    }
}

/// Cache metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache evictions
    pub evictions: u64,
    /// Total cache invalidations
    pub invalidations: u64,
    /// Current cache size (number of entries)
    pub size: usize,
    /// Maximum cache capacity
    pub capacity: usize,
}

impl CacheMetrics {
    /// Calculate cache hit rate (0.0 to 1.0)
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Check if cache is performing well (hit rate ≥ 40%)
    #[must_use]
    pub fn is_effective(&self) -> bool {
        self.hit_rate() >= 0.4
    }
}
