//! LRU cache with TTL expiration for redb storage
//!
//! This module provides an in-memory LRU (Least Recently Used) cache layer
//! that sits on top of redb storage, implementing:
//! - LRU eviction policy when cache is full
//! - TTL-based expiration with lazy and background cleanup
//! - Cache metrics tracking (hit rate, miss rate, evictions)
//! - Configurable size limits and TTL values
//!
//! # Example
//!
//! ```no_run
//! use memory_storage_redb::{CacheConfig, LRUCache};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = CacheConfig {
//!     max_size: 1000,
//!     default_ttl_secs: 3600,
//!     cleanup_interval_secs: 300,
//!     enable_background_cleanup: true,
//! };
//!
//! let cache = LRUCache::new(config);
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info};
use uuid::Uuid;

/// Configuration for the LRU cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of items in cache
    pub max_size: usize,
    /// Default TTL in seconds (0 = no expiration)
    pub default_ttl_secs: u64,
    /// Background cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable background cleanup task
    pub enable_background_cleanup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl_secs: 3600,     // 1 hour
            cleanup_interval_secs: 300, // 5 minutes
            enable_background_cleanup: true,
        }
    }
}

/// Metadata for a cached entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Last access time
    last_access: DateTime<Utc>,
    /// Expiration time (if TTL is set)
    expires_at: Option<DateTime<Utc>>,
    /// Approximate size in bytes
    size_bytes: usize,
}

impl CacheEntry {
    /// Create a new cache entry with TTL
    fn new(ttl_secs: u64, size_bytes: usize) -> Self {
        let now = Utc::now();
        let expires_at = if ttl_secs > 0 {
            Some(now + Duration::seconds(ttl_secs as i64))
        } else {
            None
        };

        Self {
            last_access: now,
            expires_at,
            size_bytes,
        }
    }

    /// Check if entry is expired
    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update last access time
    fn touch(&mut self) {
        self.last_access = Utc::now();
    }
}

/// Cache performance metrics
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions due to size limit
    pub evictions: u64,
    /// Total expirations due to TTL
    pub expirations: u64,
    /// Current number of items in cache
    pub item_count: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

impl CacheMetrics {
    /// Calculate hit rate
    fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Internal cache state
struct CacheState {
    /// Metadata for each cached item
    entries: HashMap<Uuid, CacheEntry>,
    /// LRU order: front = least recently used, back = most recently used
    lru_queue: VecDeque<Uuid>,
    /// Cache metrics
    metrics: CacheMetrics,
}

impl CacheState {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_queue: VecDeque::new(),
            metrics: CacheMetrics::default(),
        }
    }
}

/// LRU cache with TTL expiration
pub struct LRUCache {
    config: CacheConfig,
    state: Arc<RwLock<CacheState>>,
    cleanup_task: Option<JoinHandle<()>>,
}

impl LRUCache {
    /// Create a new LRU cache
    pub fn new(config: CacheConfig) -> Self {
        let state = Arc::new(RwLock::new(CacheState::new()));

        let cleanup_task = if config.enable_background_cleanup && config.cleanup_interval_secs > 0 {
            Some(Self::start_cleanup_task(
                Arc::clone(&state),
                config.cleanup_interval_secs,
            ))
        } else {
            None
        };

        info!(
            "Initialized LRU cache: max_size={}, ttl={}s, cleanup={}s",
            config.max_size, config.default_ttl_secs, config.cleanup_interval_secs
        );

        Self {
            config,
            state,
            cleanup_task,
        }
    }

    /// Record a cache access (hit or miss)
    pub async fn record_access(&self, id: Uuid, hit: bool, size_bytes: Option<usize>) -> bool {
        let mut state = self.state.write().await;

        if hit {
            // Cache hit: update access time and move to back of LRU queue
            if let Some(entry) = state.entries.get_mut(&id) {
                // Check if expired
                if entry.is_expired() {
                    debug!("Cache entry expired on access: {}", id);
                    state.metrics.expirations += 1;
                    state.metrics.misses += 1;

                    // Remove expired entry
                    state.entries.remove(&id);
                    state.lru_queue.retain(|&item_id| item_id != id);
                    state.metrics.item_count = state.entries.len();

                    state.metrics.calculate_hit_rate();
                    return false;
                }

                // Update access time
                entry.touch();

                // Move to back of LRU queue (most recently used)
                state.lru_queue.retain(|&item_id| item_id != id);
                state.lru_queue.push_back(id);

                state.metrics.hits += 1;
                state.metrics.calculate_hit_rate();
                true
            } else {
                // Entry not in metadata (shouldn't happen but handle gracefully)
                state.metrics.misses += 1;
                state.metrics.calculate_hit_rate();
                false
            }
        } else {
            // Cache miss: potentially add new entry
            state.metrics.misses += 1;

            if let Some(size) = size_bytes {
                // Check if we need to evict
                while state.entries.len() >= self.config.max_size && !state.lru_queue.is_empty() {
                    if let Some(evict_id) = state.lru_queue.pop_front() {
                        if let Some(evicted) = state.entries.remove(&evict_id) {
                            state.metrics.evictions += 1;
                            state.metrics.total_size_bytes = state
                                .metrics
                                .total_size_bytes
                                .saturating_sub(evicted.size_bytes);
                            debug!("Evicted LRU entry: {}", evict_id);
                        }
                    }
                }

                // Add new entry
                let entry = CacheEntry::new(self.config.default_ttl_secs, size);
                state.metrics.total_size_bytes += size;
                state.entries.insert(id, entry);
                state.lru_queue.push_back(id);
                state.metrics.item_count = state.entries.len();

                debug!("Added new cache entry: {} (size: {} bytes)", id, size);
            }

            state.metrics.calculate_hit_rate();
            false
        }
    }

    /// Remove an entry from the cache
    pub async fn remove(&self, id: Uuid) {
        let mut state = self.state.write().await;

        if let Some(entry) = state.entries.remove(&id) {
            state.lru_queue.retain(|&item_id| item_id != id);
            state.metrics.total_size_bytes = state
                .metrics
                .total_size_bytes
                .saturating_sub(entry.size_bytes);
            state.metrics.item_count = state.entries.len();
            debug!("Removed cache entry: {}", id);
        }
    }

    /// Check if an entry exists and is not expired
    pub async fn contains(&self, id: Uuid) -> bool {
        let state = self.state.read().await;

        if let Some(entry) = state.entries.get(&id) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Get current cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        let state = self.state.read().await;
        let metrics = state.metrics.clone();

        info!(
            "Cache metrics: hits={}, misses={}, hit_rate={:.2}%, evictions={}, expirations={}, items={}, size={} bytes",
            metrics.hits,
            metrics.misses,
            metrics.hit_rate * 100.0,
            metrics.evictions,
            metrics.expirations,
            metrics.item_count,
            metrics.total_size_bytes
        );

        metrics
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut state = self.state.write().await;
        state.entries.clear();
        state.lru_queue.clear();
        state.metrics.total_size_bytes = 0;
        state.metrics.item_count = 0;
        info!("Cache cleared");
    }

    /// Run cleanup: remove expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut state = self.state.write().await;

        let _now = Utc::now();
        let mut expired_ids = Vec::new();

        // Find expired entries
        for (id, entry) in state.entries.iter() {
            if entry.is_expired() {
                expired_ids.push(*id);
            }
        }

        // Remove expired entries
        let count = expired_ids.len();
        for id in expired_ids {
            if let Some(entry) = state.entries.remove(&id) {
                state.lru_queue.retain(|&item_id| item_id != id);
                state.metrics.total_size_bytes = state
                    .metrics
                    .total_size_bytes
                    .saturating_sub(entry.size_bytes);
                state.metrics.expirations += 1;
            }
        }

        state.metrics.item_count = state.entries.len();

        if count > 0 {
            debug!("Cleaned up {} expired cache entries", count);
        }

        count
    }

    /// Start background cleanup task
    fn start_cleanup_task(state: Arc<RwLock<CacheState>>, interval_secs: u64) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                let mut state_guard = state.write().await;
                let mut expired_ids = Vec::new();

                // Find expired entries
                for (id, entry) in state_guard.entries.iter() {
                    if entry.is_expired() {
                        expired_ids.push(*id);
                    }
                }

                // Remove expired entries
                let count = expired_ids.len();
                for id in expired_ids {
                    if let Some(entry) = state_guard.entries.remove(&id) {
                        state_guard.lru_queue.retain(|&item_id| item_id != id);
                        state_guard.metrics.total_size_bytes = state_guard
                            .metrics
                            .total_size_bytes
                            .saturating_sub(entry.size_bytes);
                        state_guard.metrics.expirations += 1;
                    }
                }

                state_guard.metrics.item_count = state_guard.entries.len();
                drop(state_guard);

                if count > 0 {
                    debug!("Background cleanup removed {} expired entries", count);
                }
            }
        })
    }

    /// Stop the background cleanup task
    pub fn stop_cleanup(&mut self) {
        if let Some(handle) = self.cleanup_task.take() {
            handle.abort();
            info!("Stopped background cleanup task");
        }
    }
}

impl Drop for LRUCache {
    fn drop(&mut self) {
        self.stop_cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    fn create_test_cache(max_size: usize, ttl_secs: u64) -> LRUCache {
        let config = CacheConfig {
            max_size,
            default_ttl_secs: ttl_secs,
            cleanup_interval_secs: 1,
            enable_background_cleanup: false, // Disable for deterministic tests
        };
        LRUCache::new(config)
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = create_test_cache(100, 3600);
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 0);
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_hit_and_miss() {
        let cache = create_test_cache(100, 3600);
        let id = Uuid::new_v4();

        // First access: miss
        let hit = cache.record_access(id, false, Some(100)).await;
        assert!(!hit);

        // Second access: hit
        let hit = cache.record_access(id, true, None).await;
        assert!(hit);

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate, 0.5);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = create_test_cache(3, 3600);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();

        // Fill cache to capacity
        cache.record_access(id1, false, Some(100)).await;
        cache.record_access(id2, false, Some(100)).await;
        cache.record_access(id3, false, Some(100)).await;

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 3);

        // Add one more item, should evict id1 (least recently used)
        cache.record_access(id4, false, Some(100)).await;

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 3);
        assert_eq!(metrics.evictions, 1);

        // id1 should be evicted, others should still be in cache
        assert!(!cache.contains(id1).await);
        assert!(cache.contains(id2).await);
        assert!(cache.contains(id3).await);
        assert!(cache.contains(id4).await);
    }

    #[tokio::test]
    async fn test_lru_order_with_access() {
        let cache = create_test_cache(3, 3600);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();

        // Fill cache
        cache.record_access(id1, false, Some(100)).await;
        cache.record_access(id2, false, Some(100)).await;
        cache.record_access(id3, false, Some(100)).await;

        // Access id1, making it most recently used
        cache.record_access(id1, true, None).await;

        // Add id4, should evict id2 (now least recently used)
        cache.record_access(id4, false, Some(100)).await;

        assert!(cache.contains(id1).await);
        assert!(!cache.contains(id2).await); // Evicted
        assert!(cache.contains(id3).await);
        assert!(cache.contains(id4).await);
    }

    #[tokio::test]
    async fn test_ttl_expiration_on_access() {
        let cache = create_test_cache(10, 1); // 1 second TTL

        let id = Uuid::new_v4();
        cache.record_access(id, false, Some(100)).await;
        assert!(cache.contains(id).await);

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        // Access should detect expiration
        let hit = cache.record_access(id, true, None).await;
        assert!(!hit);

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.expirations, 1);
        assert!(!cache.contains(id).await);
    }

    #[tokio::test]
    async fn test_manual_cleanup() {
        let cache = create_test_cache(10, 1); // 1 second TTL

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        cache.record_access(id1, false, Some(100)).await;
        cache.record_access(id2, false, Some(100)).await;

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        let expired = cache.cleanup_expired().await;
        assert_eq!(expired, 2);

        assert!(!cache.contains(id1).await);
        assert!(!cache.contains(id2).await);
    }

    #[tokio::test]
    async fn test_background_cleanup() {
        let config = CacheConfig {
            max_size: 10,
            default_ttl_secs: 1,
            cleanup_interval_secs: 1,
            enable_background_cleanup: true,
        };
        let cache = LRUCache::new(config);

        let id = Uuid::new_v4();
        cache.record_access(id, false, Some(100)).await;
        assert!(cache.contains(id).await);

        // Wait for expiration and cleanup
        sleep(TokioDuration::from_secs(3)).await;

        assert!(!cache.contains(id).await);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = create_test_cache(10, 3600);

        for _ in 0..5 {
            let id = Uuid::new_v4();
            cache.record_access(id, false, Some(100)).await;
        }

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 5);

        cache.clear().await;

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 0);
        assert_eq!(metrics.total_size_bytes, 0);
    }

    #[tokio::test]
    async fn test_cache_size_tracking() {
        let cache = create_test_cache(10, 3600);

        cache.record_access(Uuid::new_v4(), false, Some(100)).await;
        cache.record_access(Uuid::new_v4(), false, Some(200)).await;
        cache.record_access(Uuid::new_v4(), false, Some(150)).await;

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.total_size_bytes, 450);
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache = create_test_cache(10, 3600);
        let id = Uuid::new_v4();

        cache.record_access(id, false, Some(100)).await;
        assert!(cache.contains(id).await);

        cache.remove(id).await;
        assert!(!cache.contains(id).await);

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 0);
    }

    #[tokio::test]
    async fn test_cache_metrics_accuracy() {
        let cache = create_test_cache(10, 3600);
        let id = Uuid::new_v4();

        // 1 miss
        cache.record_access(id, false, Some(100)).await;

        // 3 hits
        cache.record_access(id, true, None).await;
        cache.record_access(id, true, None).await;
        cache.record_access(id, true, None).await;

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 3);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate, 0.75); // 3/4
    }

    #[tokio::test]
    async fn test_zero_ttl_no_expiration() {
        let cache = create_test_cache(10, 0); // No TTL

        let id = Uuid::new_v4();
        cache.record_access(id, false, Some(100)).await;

        // Wait a bit
        sleep(TokioDuration::from_secs(2)).await;

        // Should still be valid
        assert!(cache.contains(id).await);
        let hit = cache.record_access(id, true, None).await;
        assert!(hit);
    }

    #[tokio::test]
    async fn test_edge_case_size_one() {
        let cache = create_test_cache(1, 3600); // Size = 1

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        cache.record_access(id1, false, Some(100)).await;
        assert!(cache.contains(id1).await);

        // Adding second item should evict first
        cache.record_access(id2, false, Some(100)).await;
        assert!(!cache.contains(id1).await);
        assert!(cache.contains(id2).await);

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.item_count, 1);
        assert_eq!(metrics.evictions, 1);
    }
}
