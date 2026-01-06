//! # Query Cache Implementation
//!
//! LRU cache with TTL for query results to improve retrieval performance.
//! Target: 2-3x speedup for repeated queries with ≥40% cache hit rate.

use crate::episode::Episode;
use crate::retrieval::cache::types::{
    CacheKey, CacheMetrics, CachedResult, DEFAULT_CACHE_TTL, DEFAULT_MAX_ENTRIES,
};
use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Query cache with LRU eviction and TTL
pub struct QueryCache {
    /// LRU cache storage
    cache: Arc<RwLock<LruCache<u64, CachedResult>>>,
    /// Domain index: maps domain -> set of cache key hashes
    domain_index: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    /// Lazy invalidation: set of cache key hashes marked for removal
    /// Entries are not removed immediately, but filtered on access
    invalidated_hashes: Arc<RwLock<HashSet<u64>>>,
    /// Cache metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Default TTL for new entries
    default_ttl: Duration,
    /// Maximum number of entries
    max_entries: usize,
}

impl QueryCache {
    /// Create a new query cache with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity_and_ttl(DEFAULT_MAX_ENTRIES, DEFAULT_CACHE_TTL)
    }

    /// Create a new query cache with custom capacity and TTL
    #[must_use]
    pub fn with_capacity_and_ttl(capacity: usize, ttl: Duration) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());
        let metrics = CacheMetrics {
            capacity,
            ..Default::default()
        };

        Self {
            cache: Arc::new(RwLock::new(cache)),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            invalidated_hashes: Arc::new(RwLock::new(HashSet::new())),
            metrics: Arc::new(RwLock::new(metrics)),
            default_ttl: ttl,
            max_entries: capacity,
        }
    }

    /// Get a cached query result
    #[must_use]
    pub fn get(&self, key: &CacheKey) -> Option<Vec<Episode>> {
        let key_hash = key.compute_hash();

        // Fast path: Check if this entry is marked for lazy invalidation
        {
            let invalidated = self.invalidated_hashes.read().expect(
                "QueryCache: invalidated_hashes lock poisoned - this indicates a panic in invalidation tracking",
            );
            if invalidated.contains(&key_hash) {
                // Entry is invalidated - count as miss and return None
                let mut metrics = self.metrics.write().expect(
                    "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
                );
                metrics.misses += 1;
                return None;
            }
        }

        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );

        // Check if entry exists and is not expired
        if let Some(result) = cache.get(&key_hash) {
            // Check if expired
            if result.is_expired() {
                // Remove expired entry
                cache.pop(&key_hash);
                metrics.misses += 1;
                metrics.evictions += 1;
                metrics.size = cache.len();
                return None;
            }

            // Cache hit
            metrics.hits += 1;
            Some(result.episodes.clone())
        } else {
            // Cache miss
            metrics.misses += 1;
            metrics.size = cache.len();
            None
        }
    }

    /// Store a query result in the cache
    pub fn put(&self, key: CacheKey, episodes: Vec<Episode>) {
        let key_hash = key.compute_hash();
        let cached_result = CachedResult {
            episodes,
            cached_at: Instant::now(),
            ttl: self.default_ttl,
        };

        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");

        // Check if this is an update to an existing entry
        let was_present = cache.contains(&key_hash);

        // Add to cache
        cache.put(key_hash, cached_result);

        // Update domain index if domain is specified
        if let Some(ref domain) = key.domain {
            let mut domain_index = self.domain_index.write().expect(
                "QueryCache: domain_index lock poisoned - this indicates a panic in domain tracking",
            );
            domain_index
                .entry(domain.clone())
                .or_default()
                .insert(key_hash);
        }

        // Update metrics
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );
        metrics.size = cache.len();

        // If this was an update (not a new entry), don't count as eviction
        if was_present {
            return;
        }

        // Check if we evicted something
        if cache.len() > self.max_entries {
            metrics.evictions += 1;
        }
    }

    /// Invalidate all cached entries (use for cross-domain changes)
    pub fn invalidate_all(&self) {
        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");
        let count = cache.len();
        cache.clear();

        // Clear domain index
        let mut domain_index = self.domain_index.write().expect(
            "QueryCache: domain_index lock poisoned - this indicates a panic in domain tracking",
        );
        domain_index.clear();

        // Clear invalidation set
        let mut invalidated = self.invalidated_hashes.write().expect(
            "QueryCache: invalidated_hashes lock poisoned - this indicates a panic in invalidation tracking",
        );
        invalidated.clear();

        // Update metrics
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );
        metrics.size = 0;
        metrics.invalidations += count as u64;
    }

    /// Invalidate entries for a specific domain
    ///
    /// This is more efficient than `invalidate_all()` for multi-domain workloads
    /// because it only clears entries for the specified domain.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain to invalidate
    pub fn invalidate_domain(&self, domain: &str) {
        let domain_index = self.domain_index.read().expect(
            "QueryCache: domain_index lock poisoned - this indicates a panic in domain tracking",
        );

        if let Some(hashes) = domain_index.get(domain) {
            let count = hashes.len();

            // Mark entries for lazy invalidation
            let mut invalidated = self.invalidated_hashes.write().expect(
                "QueryCache: invalidated_hashes lock poisoned - this indicates a panic in invalidation tracking",
            );
            for &hash in hashes {
                invalidated.insert(hash);
            }
            drop(invalidated);

            // Remove from domain index
            let mut domain_index = self.domain_index.write().expect(
                "QueryCache: domain_index lock poisoned - this indicates a panic in domain tracking",
            );
            domain_index.remove(domain);

            // Update metrics
            let mut metrics = self.metrics.write().expect(
                "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
            );
            metrics.invalidations += count as u64;
        }
        // If domain not found, no-op (already cleared or never existed)
    }

    /// Get current cache metrics
    #[must_use]
    pub fn metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );
        metrics.clone()
    }

    /// Clear all metrics
    pub fn clear_metrics(&self) {
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );
        *metrics = CacheMetrics {
            capacity: self.max_entries,
            ..Default::default()
        };
    }

    /// Get cache size (number of entries)
    ///
    /// Note: This returns the physical size of the cache, which may include
    /// entries that are marked for lazy invalidation. These entries will be
    /// filtered out when accessed via `get()`.
    #[must_use]
    pub fn size(&self) -> usize {
        self.cache
            .read()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code")
            .len()
    }

    /// Get effective cache size (excluding invalidated entries)
    ///
    /// This returns the logical size of the cache, excluding entries that
    /// are marked for lazy invalidation.
    #[must_use]
    pub fn effective_size(&self) -> usize {
        let cache_size = self
            .cache
            .read()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code")
            .len();
        let invalidated_size = self
            .invalidated_hashes
            .read()
            .expect("QueryCache: invalidated_hashes lock poisoned - this indicates a panic in invalidation tracking")
            .len();
        cache_size.saturating_sub(invalidated_size)
    }

    /// Check if cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache
            .read()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code")
            .is_empty()
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}
