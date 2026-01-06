//! # Query Cache for Episodic Memory Retrieval
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
//! - Use **domain-based invalidation** for multi-domain workloads (see below)
//! - Adjust TTL based on your episode completion frequency
//!
//! ## Design Decisions
//!
//! ### Domain-Based Invalidation
//!
//! The cache supports **selective invalidation by domain** (v0.1.11+):
//! - **`invalidate_domain(domain)`**: Clears only entries for the specified domain
//! - **`invalidate_all()`**: Clears all entries (use when domain is unknown)
//! - **Benefits**: Higher cache hit rates in multi-domain workloads
//! - **Performance**: O(k) where k is number of entries in domain
//!
//! #### When to Use Domain-Based Invalidation
//!
//! - ✅ **Use `invalidate_domain()`**: Multi-domain agents, isolated domains
//! - ✅ **Use `invalidate_all()`**: Single domain, cross-domain changes, or uncertain scope
//!
//! #### Performance Comparison
//!
//! | Scenario | `invalidate_all()` | `invalidate_domain()` | Improvement |
//! |----------|-------------------|----------------------|-------------|
//! | 3 domains, invalidate 1 | 100% entries cleared | 33% entries cleared | +15-20% hit rate |
//! | Single domain | 100% entries cleared | 100% entries cleared | No difference |
//! | High-throughput multi-domain | <30% hit rate | 35-40% hit rate | +10-15% hit rate |
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
//! - **Domain invalidation**: <100µs for domains with <1000 entries
//! - **Memory overhead**: ~10KB per cached query + O(d × k) for domain index
//! - **Maximum memory**: ~100MB (10,000 entries × 10KB)
//!
//! ## Example Usage
//!
//! ### Basic Usage
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
//! // Check cache (miss initially)
//! assert!(cache.get(&key).is_none());
//!
//! // Populate cache
//! # let episodes = vec![];
//! cache.put(key.clone(), episodes);
//!
//! // Check cache again (hit)
//! if let Some(episodes) = cache.get(&key) {
//!     println!("Cache hit! Found {} episodes", episodes.len());
//! }
//!
//! // Monitor performance
//! let metrics = cache.metrics();
//! println!("Hit rate: {:.1}%", metrics.hit_rate() * 100.0);
//! // After 1 miss and 1 hit, hit rate is 50%
//! assert!(metrics.hit_rate() >= 0.4);
//! ```
//!
//! ### Domain-Based Invalidation (Multi-Domain Workloads)
//!
//! ```
//! use memory_core::retrieval::{QueryCache, CacheKey};
//!
//! let cache = QueryCache::new();
//!
//! // Cache queries from different domains
//! # let episodes = vec![];
//! let web_key = CacheKey::new("query".to_string())
//!     .with_domain(Some("web-api".to_string()));
//! let data_key = CacheKey::new("query".to_string())
//!     .with_domain(Some("data-processing".to_string()));
//!
//! cache.put(web_key.clone(), episodes.clone());
//! cache.put(data_key.clone(), episodes.clone());
//!
//! // When an episode completes in web-api domain, invalidate only that domain
//! cache.invalidate_domain("web-api");
//!
//! // web-api queries are cleared, but data-processing queries remain cached
//! assert!(cache.get(&web_key).is_none());
//! assert!(cache.get(&data_key).is_some());
//!
//! // For cross-domain changes, use invalidate_all()
//! cache.invalidate_all();
//! assert!(cache.get(&data_key).is_none());
//! ```

pub mod lru;
pub mod tests;
pub mod types;

// Re-export public API
pub use lru::QueryCache;
pub use types::{CacheKey, CacheMetrics, DEFAULT_CACHE_TTL, DEFAULT_MAX_ENTRIES};
