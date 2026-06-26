//! Adaptive LRU cache with intelligent TTL adjustment
//!
//! This module provides an intelligent caching layer that dynamically adjusts
//! TTL (Time-To-Live) based on access patterns:
//! - "Hot" items (frequently accessed) get extended TTL
//! - "Cold" items (rarely accessed) get reduced TTL
//! - Min/max TTL bounds prevent extreme values
//! - Exponential backoff smooths TTL adjustments
//!
//! ## Algorithm
//!
//! 1. Track access count per entry in a sliding window
//! 2. On each access, update TTL based on access frequency:
//!    - If access_count > hot_threshold: Increase TTL (max: max_ttl)
//!    - If access_count < cold_threshold: Decrease TTL (min: min_ttl)
//! 3. Use exponential backoff: new_ttl = current_ttl * (1 ± adaptation_rate)

mod entry;
mod state;
mod types;

pub use types::{AdaptiveCacheConfig, AdaptiveCacheMetrics};

use self::entry::AdaptiveCacheEntry;
use self::state::AdaptiveCacheState;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{debug, info};
use uuid::Uuid;

/// Adaptive LRU cache with intelligent TTL adjustment
///
/// This cache monitors access patterns and adjusts TTL dynamically:
/// - Frequently accessed items get longer TTL
/// - Rarely accessed items get shorter TTL
/// - Reduces memory waste on cold items
/// - Improves hit rate for hot items
///
/// ## Thread Safety
///
/// This cache is designed for single-threaded async access. All public methods
/// acquire the write lock internally. For higher concurrency, consider using
/// fine-grained locking strategies.
pub struct AdaptiveCache<V: Clone + Send + Sync + 'static> {
    config: AdaptiveCacheConfig,
    state: Arc<RwLock<AdaptiveCacheState<V>>>,
    cleanup_task: Option<JoinHandle<()>>,
}

impl<V: Clone + Send + Sync + 'static> AdaptiveCache<V> {
    /// Create a new adaptive cache with default configuration
    pub fn new(config: AdaptiveCacheConfig) -> Self {
        let state = Arc::new(RwLock::new(AdaptiveCacheState::<V>::new()));

        let cleanup_task = if config.enable_background_cleanup && config.cleanup_interval_secs > 0 {
            Some(Self::start_cleanup_task(
                Arc::clone(&state),
                config.cleanup_interval_secs,
            ))
        } else {
            None
        };

        info!(
            "Initialized adaptive cache: default_ttl={}s, min_ttl={}s, max_ttl={}s, hot={}, cold={}",
            config.default_ttl.as_secs(),
            config.min_ttl.as_secs(),
            config.max_ttl.as_secs(),
            config.hot_threshold,
            config.cold_threshold
        );

        Self {
            config,
            state,
            cleanup_task,
        }
    }

    /// Record a cache access (hit or miss)
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the cache entry
    /// * `hit` - Whether this was a cache hit (true) or miss (false)
    /// * `value` - Value to store (only used on cache miss)
    ///
    /// # Returns
    ///
    /// `true` if the entry was found and is valid, `false` otherwise
    pub async fn record_access(&self, id: Uuid, hit: bool, value: Option<V>) -> bool {
        let now = Instant::now();
        let mut state = self.state.write().await;

        if hit {
            // Cache hit: update access time and move to back of LRU queue
            if let Some(entry) = state.entries.get_mut(&id) {
                // Check if expired
                if entry.is_expired(now) {
                    debug!("Cache entry expired on access: {}", id);
                    state.metrics.base.expirations += 1;
                    state.metrics.base.misses += 1;

                    // Remove expired entry
                    state.remove_entry(&id);
                    state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
                    return false;
                }

                // Record access and update TTL
                entry.record_access(now, &self.config);

                // Move to back of LRU queue (most recently used)
                state.lru_queue.retain(|&qid| qid != id);
                state.lru_queue.push_back(id);

                state.metrics.base.hits += 1;
                state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
                true
            } else {
                // Entry not found - treat as miss
                state.metrics.base.misses += 1;
                state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
                false
            }
        } else {
            // Cache miss: add new entry
            state.metrics.base.misses += 1;

            if let Some(v) = value {
                // Check if we need to evict
                if state.entries.len() >= self.config.max_size {
                    // Evict oldest entry (front of queue)
                    if let Some(oldest_id) = state.lru_queue.pop_front() {
                        state.entries.remove(&oldest_id);
                        state.metrics.base.evictions += 1;
                        debug!("Evicted LRU entry: {}", oldest_id);
                    }
                }

                // Add new entry
                let entry = AdaptiveCacheEntry::new(v, self.config.default_ttl);
                state.entries.insert(id, entry);
                state.lru_queue.push_back(id);
            }

            state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
            false
        }
    }

    /// Get the current value for an entry without recording access
    pub async fn get(&self, id: Uuid) -> Option<V> {
        let state = self.state.read().await;
        state.entries.get(&id).map(|entry| entry.value.clone())
    }

    /// Get the current value and record access
    pub async fn get_and_record(&self, id: Uuid) -> Option<V> {
        let now = Instant::now();
        let mut state = self.state.write().await;

        if let Some(entry) = state.entries.get_mut(&id) {
            if !entry.is_expired(now) {
                // Clone value first before any further borrows
                let value = entry.value.clone();
                entry.record_access(now, &self.config);
                state.metrics.base.hits += 1;
                state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
                return Some(value);
            }
        }

        state.metrics.base.misses += 1;
        state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
        None
    }

    /// Remove an entry from the cache
    pub async fn remove(&self, id: Uuid) {
        let mut state = self.state.write().await;
        state.remove_entry(&id);
        state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);
    }

    /// Check if an entry exists and is not expired
    pub async fn contains(&self, id: Uuid) -> bool {
        let now = Instant::now();
        let state = self.state.read().await;
        if let Some(entry) = state.entries.get(&id) {
            !entry.is_expired(now)
        } else {
            false
        }
    }

    /// Get access count for an entry
    pub async fn access_count(&self, id: Uuid) -> Option<usize> {
        let state = self.state.read().await;
        state.entries.get(&id).map(|entry| entry.access_count())
    }

    /// Get current TTL for an entry
    pub async fn ttl(&self, id: Uuid) -> Option<Duration> {
        let state = self.state.read().await;
        state.entries.get(&id).map(|entry| entry.ttl())
    }

    /// Get current cache metrics
    pub async fn get_metrics(&self) -> AdaptiveCacheMetrics {
        let state = self.state.read().await;
        state.metrics.clone()
    }

    /// Clear all entries from cache
    pub async fn clear(&self) {
        let mut state = self.state.write().await;
        state.clear();
    }

    /// Manually cleanup expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut state = self.state.write().await;
        let mut expired_ids = Vec::new();

        // Find expired entries
        for (id, entry) in &state.entries {
            let ttl_secs = entry.ttl_seconds.load(Ordering::SeqCst);
            let created_at = entry.created_at;
            let expires_at = created_at + Duration::from_secs(ttl_secs);
            let is_expired = now >= expires_at;
            if is_expired {
                expired_ids.push(*id);
            }
        }

        // Remove them
        let count = expired_ids.len();
        for id in expired_ids {
            state.remove_entry(&id);
            state.metrics.base.expirations += 1;
        }

        state.update_metrics(self.config.hot_threshold, self.config.cold_threshold);

        if count > 0 {
            debug!("Cleaned up {} expired cache entries", count);
        }

        count
    }

    /// Get the number of hot items
    pub async fn hot_count(&self) -> usize {
        let state = self.state.read().await;
        state.metrics.hot_item_count
    }

    /// Get the number of cold items
    pub async fn cold_count(&self) -> usize {
        let state = self.state.read().await;
        state.metrics.cold_item_count
    }

    /// Get cache size (number of entries)
    pub async fn len(&self) -> usize {
        let state = self.state.read().await;
        state.entries.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Start background cleanup task
    fn start_cleanup_task(
        state: Arc<RwLock<AdaptiveCacheState<V>>>,
        interval_secs: u64,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(interval_secs));
            loop {
                ticker.tick().await;

                let now = Instant::now();
                let mut state_guard = state.write().await;
                let mut expired_ids = Vec::new();

                // Find expired entries
                for (id, entry) in &state_guard.entries {
                    let ttl_secs = entry.ttl_seconds.load(Ordering::SeqCst);
                    let expires_at = entry.created_at + Duration::from_secs(ttl_secs);
                    if now >= expires_at {
                        expired_ids.push(*id);
                    }
                }

                // Remove them
                let count = expired_ids.len();
                for id in expired_ids {
                    state_guard.remove_entry(&id);
                    state_guard.metrics.base.expirations += 1;
                }

                state_guard.update_metrics(10, 2); // Use defaults for metrics
                drop(state_guard);

                if count > 0 {
                    debug!("Background cleanup removed {} expired entries", count);
                }
            }
        })
    }

    /// Stop the background cleanup task
    pub fn stop_cleanup(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }
}

impl<V: Clone + Send + Sync + 'static> Drop for AdaptiveCache<V> {
    fn drop(&mut self) {
        self.stop_cleanup();
    }
}

/// Conversion from AdaptiveCacheConfig to standard CacheConfig
impl From<AdaptiveCacheConfig> for super::super::CacheConfig {
    fn from(config: AdaptiveCacheConfig) -> Self {
        Self {
            max_size: 1000,
            default_ttl_secs: config.default_ttl.as_secs(),
            cleanup_interval_secs: config.cleanup_interval_secs,
            enable_background_cleanup: config.enable_background_cleanup,
        }
    }
}
