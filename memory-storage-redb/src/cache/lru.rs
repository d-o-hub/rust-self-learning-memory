//! LRU cache implementation with TTL expiration
//!
//! This module implements the main LRUCache struct with methods for:
//! - Recording cache hits/misses
//! - LRU eviction when cache is full
//! - TTL-based expiration
//! - Background cleanup task

use super::state::CacheState;
use super::types::{CacheConfig, CacheEntry, CacheMetrics};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{debug, info};
use uuid::Uuid;

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
                    state.remove_entry(&id);
                    state.update_metrics();
                    return false;
                }

                // Update access time
                entry.touch();

                // Move to back of LRU queue (most recently used)
                state.lru_queue.retain(|&qid| qid != id);
                state.lru_queue.push_back(id);

                state.metrics.hits += 1;
                state.update_metrics();
                true
            } else {
                // Entry not found - treat as miss
                state.metrics.misses += 1;
                state.update_metrics();
                false
            }
        } else {
            // Cache miss: add new entry
            state.metrics.misses += 1;

            let size = size_bytes.unwrap_or(0);
            let entry = CacheEntry::new(self.config.default_ttl_secs, size);

            // Check if we need to evict
            if state.entries.len() >= self.config.max_size {
                // Evict oldest entry (front of queue)
                if let Some(oldest_id) = state.lru_queue.pop_front() {
                    state.entries.remove(&oldest_id);
                    state.metrics.evictions += 1;
                    debug!("Evicted LRU entry: {}", oldest_id);
                }
            }

            // Add new entry
            state.entries.insert(id, entry);
            state.lru_queue.push_back(id);

            state.update_metrics();
            false
        }
    }

    /// Remove an entry from the cache
    pub async fn remove(&self, id: Uuid) {
        let mut state = self.state.write().await;
        state.remove_entry(&id);
        state.update_metrics();
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
        state.metrics.clone()
    }

    /// Clear all entries from cache
    pub async fn clear(&self) {
        let mut state = self.state.write().await;
        state.clear();
    }

    /// Manually cleanup expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut state = self.state.write().await;
        let mut expired_ids = Vec::new();

        // Find expired entries
        for (id, entry) in &state.entries {
            if entry.is_expired() {
                expired_ids.push(*id);
            }
        }

        // Remove them
        let count = expired_ids.len();
        for id in expired_ids {
            state.remove_entry(&id);
            state.metrics.expirations += 1;
        }

        state.update_metrics();

        if count > 0 {
            debug!("Cleaned up {} expired cache entries", count);
        }

        count
    }

    /// Start background cleanup task
    fn start_cleanup_task(state: Arc<RwLock<CacheState>>, interval_secs: u64) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(interval_secs));
            loop {
                ticker.tick().await;

                let mut state_guard = state.write().await;
                let mut expired_ids = Vec::new();

                // Find expired entries
                for (id, entry) in &state_guard.entries {
                    if entry.is_expired() {
                        expired_ids.push(*id);
                    }
                }

                // Remove them
                let count = expired_ids.len();
                for id in expired_ids {
                    state_guard.remove_entry(&id);
                    state_guard.metrics.expirations += 1;
                }

                state_guard.update_metrics();
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

impl Drop for LRUCache {
    fn drop(&mut self) {
        self.stop_cleanup();
    }
}
