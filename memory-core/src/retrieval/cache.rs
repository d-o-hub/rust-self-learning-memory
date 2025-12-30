//! Query cache for episodic memory retrieval
//!
//! Provides LRU caching with TTL for query results to improve retrieval performance.
//! Target: 2-3x speedup for repeated queries with ≥40% cache hit rate.
//!
//! ## Supported Workloads
//!
//! This cache is optimized for **interactive/CLI workloads** with moderate query rates:
//! - **Ideal**: 1-100 queries per second
//! - **Use cases**: Agent development, testing, interactive memory queries
//! - **Episode completion rate**: <1 episode/minute for optimal cache effectiveness
//!
//! ### High-Throughput Workloads
//!
//! For streaming/batch workloads (>100 QPS) or high episode completion rates:
//! - Cache effectiveness may decrease due to frequent invalidation
//! - Consider implementing domain-based invalidation (see `GitHub` issue)
//! - Adjust TTL based on your episode completion frequency
//!
//! ## Design Decisions
//!
//! ### Full Cache Invalidation
//!
//! The cache invalidates **all entries** when a new episode completes. This is:
//! - **Conservative**: Ensures no stale results
//! - **Simple**: No complex invalidation logic
//! - **Trade-off**: Lower hit rate in high-throughput scenarios
//!
//! **Future improvement**: Domain-based invalidation to only clear affected queries
//!
//! ### Thread Safety
//!
//! Uses `Arc<RwLock<>>` for thread-safe concurrent access:
//! - Multiple readers can access cache simultaneously
//! - Writers block all readers (but operations are fast)
//! - Lock poisoning is unlikely but handled with descriptive error messages
//!
//! ## Performance Characteristics
//!
//! - **Cache hit latency**: ~1-5µs
//! - **Cache miss overhead**: ~2µs (hash computation)
//! - **Memory overhead**: ~10KB per cached query (10 episodes × 1KB each)
//! - **Maximum memory**: ~100MB (10,000 entries × 10KB)
//!
//! ## Example Usage
//!
//! ```
//! use memory_core::retrieval::{QueryCache, CacheKey};
//!
//! let cache = QueryCache::new();
//!
//! // Create cache key
//! let key = CacheKey::new("implement authentication".to_string())
//!     .with_domain(Some("web-api".to_string()))
//!     .with_limit(5);
//!
//! // Check cache
//! if let Some(episodes) = cache.get(&key) {
//!     println!("Cache hit! Found {} episodes", episodes.len());
//! }
//!
//! // Populate cache
//! cache.put(key, episodes);
//!
//! // Monitor performance
//! let metrics = cache.metrics();
//! println!("Hit rate: {:.1}%", metrics.hit_rate() * 100.0);
//! assert!(metrics.is_effective(), "Cache hit rate should be ≥40%");
//! ```

use crate::episode::Episode;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
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
    pub domain: Option<String>,
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
        self.domain = domain;
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
struct CachedResult {
    /// Cached episodes
    episodes: Vec<Episode>,
    /// Time when this entry was cached
    cached_at: Instant,
    /// Time-to-live for this entry
    ttl: Duration,
}

impl CachedResult {
    /// Check if this cached result has expired
    fn is_expired(&self) -> bool {
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

/// Query cache with LRU eviction and TTL
pub struct QueryCache {
    /// LRU cache storage
    cache: Arc<RwLock<LruCache<u64, CachedResult>>>,
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
            metrics: Arc::new(RwLock::new(metrics)),
            default_ttl: ttl,
            max_entries: capacity,
        }
    }

    /// Get episodes from cache
    #[must_use]
    pub fn get(&self, key: &CacheKey) -> Option<Vec<Episode>> {
        let hash = key.compute_hash();
        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );

        if let Some(result) = cache.get(&hash) {
            // Check if expired
            if result.is_expired() {
                // Remove expired entry
                cache.pop(&hash);
                metrics.misses += 1;
                metrics.evictions += 1;
                return None;
            }

            // Cache hit
            metrics.hits += 1;
            Some(result.episodes.clone())
        } else {
            // Cache miss
            metrics.misses += 1;
            None
        }
    }

    /// Put episodes into cache
    pub fn put(&self, key: CacheKey, episodes: Vec<Episode>) {
        let hash = key.compute_hash();
        let result = CachedResult {
            episodes,
            cached_at: Instant::now(),
            ttl: self.default_ttl,
        };

        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );

        // Track eviction if cache is at capacity
        if cache.len() >= self.max_entries && !cache.contains(&hash) {
            metrics.evictions += 1;
        }

        cache.put(hash, result);
        metrics.size = cache.len();
    }

    /// Invalidate all cached entries
    ///
    /// Should be called when new episodes are inserted to ensure fresh results
    pub fn invalidate_all(&self) {
        let mut cache = self
            .cache
            .write()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code");
        let mut metrics = self.metrics.write().expect(
            "QueryCache: metrics lock poisoned - this indicates a panic in metrics tracking",
        );

        let size = cache.len();
        cache.clear();
        metrics.invalidations += size as u64;
        metrics.size = 0;
    }

    /// Invalidate entries matching a domain filter
    ///
    /// # TODO: Domain-Based Invalidation (v0.1.13+)
    ///
    /// Current implementation invalidates ALL entries (conservative).
    /// Future optimization: Only invalidate entries matching the domain.
    ///
    /// See: `plans/GITHUB_ISSUE_domain_based_cache_invalidation.md`
    ///
    /// **Trigger**: Implement if cache hit rate <30% in production after 2 weeks
    pub fn invalidate_domain(&self, _domain: &str) {
        // TODO: Track domain per cache key and invalidate selectively
        // For now, invalidate all (safe but suboptimal)
        self.invalidate_all();
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
    #[must_use]
    pub fn size(&self) -> usize {
        self.cache
            .read()
            .expect("QueryCache: cache lock poisoned - this indicates a panic in cache code")
            .len()
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono;

    fn create_test_episode(id: &str) -> Episode {
        use crate::types::{TaskContext, TaskType};
        use std::collections::HashMap;
        use uuid::Uuid;

        Episode {
            episode_id: Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()),
            task_type: TaskType::CodeGeneration,
            task_description: "test task".to_string(),
            context: TaskContext::default(),
            start_time: chrono::Utc::now(),
            end_time: None,
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_cache_hit() {
        let cache = QueryCache::new();
        let key = CacheKey::new("test query".to_string());
        let episodes = vec![create_test_episode("ep1")];

        // Cache miss initially
        assert!(cache.get(&key).is_none());

        // Put in cache
        cache.put(key.clone(), episodes.clone());

        // Cache hit
        let result = cache.get(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);

        // Check metrics
        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::with_capacity_and_ttl(100, Duration::from_millis(10));
        let key = CacheKey::new("test query".to_string());
        let episodes = vec![create_test_episode("ep1")];

        cache.put(key.clone(), episodes);

        // Immediate hit
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(15));

        // Should be expired
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = QueryCache::new();
        let key1 = CacheKey::new("query1".to_string());
        let key2 = CacheKey::new("query2".to_string());

        cache.put(key1.clone(), vec![create_test_episode("ep1")]);
        cache.put(key2.clone(), vec![create_test_episode("ep2")]);

        assert_eq!(cache.size(), 2);

        // Invalidate all
        cache.invalidate_all();

        assert_eq!(cache.size(), 0);
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_none());

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 2);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = QueryCache::with_capacity_and_ttl(2, DEFAULT_CACHE_TTL);

        let key1 = CacheKey::new("query1".to_string());
        let key2 = CacheKey::new("query2".to_string());
        let key3 = CacheKey::new("query3".to_string());

        cache.put(key1.clone(), vec![create_test_episode("ep1")]);
        cache.put(key2.clone(), vec![create_test_episode("ep2")]);

        // Cache should have 2 entries
        assert_eq!(cache.size(), 2);

        // Add third entry, should evict oldest (key1)
        cache.put(key3.clone(), vec![create_test_episode("ep3")]);

        assert_eq!(cache.size(), 2);
        assert!(cache.get(&key1).is_none()); // Evicted
        assert!(cache.get(&key2).is_some()); // Still present
        assert!(cache.get(&key3).is_some()); // Newly added

        let metrics = cache.metrics();
        assert_eq!(metrics.evictions, 1);
    }

    #[test]
    fn test_cache_key_with_filters() {
        let key1 = CacheKey::new("test".to_string())
            .with_domain(Some("web".to_string()))
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        let key2 = CacheKey::new("test".to_string())
            .with_domain(Some("web".to_string()))
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        let key3 = CacheKey::new("test".to_string())
            .with_domain(Some("data".to_string())) // Different domain
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        // Same keys should have same hash
        assert_eq!(key1.compute_hash(), key2.compute_hash());

        // Different keys should have different hash
        assert_ne!(key1.compute_hash(), key3.compute_hash());
    }

    #[test]
    fn test_metrics_effectiveness() {
        let cache = QueryCache::new();
        let key = CacheKey::new("test".to_string());
        let episodes = vec![create_test_episode("ep1")];

        cache.put(key.clone(), episodes);

        // Generate hits
        for _ in 0..10 {
            let _ = cache.get(&key);
        }

        let metrics = cache.metrics();
        assert!(metrics.is_effective()); // Should be > 40% hit rate
        assert!(metrics.hit_rate() > 0.9); // Should be ~90% (10 hits, 1 miss)
    }
}
