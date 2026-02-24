//! Adaptive TTL Cache Implementation
//!
//! This module provides a generic adaptive TTL cache that:
//! - Adjusts TTL based on access patterns (hot items get longer TTL, cold items get shorter)
//! - Performs background cleanup of expired entries
//! - Tracks statistics (hit rate, avg TTL, evictions)
//! - Provides thread-safe operations
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AdaptiveTTLCache<K, V>                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │   entries   │  │    stats    │  │   cleanup_task      │  │
//! │  │  HashMap    │  │ CacheStats  │  │   (background)      │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                    ┌─────────┴─────────┐
//!                    ▼                   ▼
//!           ┌────────────────┐  ┌────────────────┐
//!           │  CacheEntry<V> │  │   TTLConfig    │
//!           │ - value: V     │  │ - base_ttl     │
//!           │ - created_at   │  │ - min/max_ttl  │
//!           │ - access_count │  │ - thresholds   │
//!           │ - last_access  │  │ - adaptation   │
//!           └────────────────┘  └────────────────┘
//! ```

use super::ttl_config::{TTLConfig, TTLConfigError};
#[path = "adaptive_ttl_stats.rs"]
mod stats;
pub use stats::{CacheStats, CacheStatsSnapshot};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{debug, info, trace};

/// A cache entry with metadata for adaptive TTL management
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    /// The cached value
    pub value: V,
    /// When the entry was created
    pub created_at: Instant,
    /// Number of times this entry has been accessed
    pub access_count: u64,
    /// When the entry was last accessed
    pub last_accessed: Instant,
    /// Current TTL for this entry (adaptive)
    pub current_ttl: Duration,
    /// Access timestamps for pattern analysis (sliding window)
    access_history: Vec<Instant>,
}

impl<V> CacheEntry<V> {
    /// Create a new cache entry with the given value and initial TTL
    pub fn new(value: V, initial_ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            access_count: 0,
            last_accessed: now,
            current_ttl: initial_ttl,
            access_history: Vec::with_capacity(20),
        }
    }

    /// Record an access to this entry
    pub fn record_access(&mut self, config: &TTLConfig) {
        let now = Instant::now();
        self.access_count += 1;
        self.last_accessed = now;

        // Add to access history
        self.access_history.push(now);

        // Trim access history to window size
        let window_start = now - Duration::from_secs(config.access_window_secs);
        self.access_history.retain(|&t| t >= window_start);

        // Update TTL based on access pattern
        if config.enable_adaptive_ttl {
            let window_accesses = self.access_history.len() as u64;
            self.current_ttl = config.calculate_ttl(self.current_ttl, window_accesses);
        }
    }

    /// Check if this entry has expired
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > self.current_ttl
    }

    /// Get the remaining TTL for this entry
    pub fn remaining_ttl(&self) -> Duration {
        let elapsed = Instant::now().duration_since(self.created_at);
        self.current_ttl.saturating_sub(elapsed)
    }

    /// Get the access frequency (accesses per minute) over the window
    pub fn access_frequency(&self, window_secs: u64) -> f64 {
        if self.access_history.is_empty() {
            return 0.0;
        }

        let window_duration = Duration::from_secs(window_secs);
        let actual_window = Instant::now()
            .duration_since(self.created_at)
            .min(window_duration);

        if actual_window.as_secs() == 0 {
            return self.access_history.len() as f64;
        }

        let accesses = self.access_history.len() as f64;
        let minutes = actual_window.as_secs_f64() / 60.0;
        accesses / minutes
    }

    /// Reset the entry with a new value and TTL
    pub fn reset(&mut self, value: V, ttl: Duration) {
        let now = Instant::now();
        self.value = value;
        self.created_at = now;
        self.access_count = 0;
        self.last_accessed = now;
        self.current_ttl = ttl;
        self.access_history.clear();
    }
}

/// Adaptive TTL cache with generic key and value types
///
/// This cache automatically adjusts the TTL of entries based on their access patterns:
/// - Frequently accessed items (hot) get extended TTL
/// - Rarely accessed items (cold) get reduced TTL
/// - Expired entries are cleaned up in the background
pub struct AdaptiveTTLCache<K, V> {
    /// Cache entries
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Cache configuration
    config: TTLConfig,
    /// Cache statistics
    stats: Arc<CacheStats>,
    /// Background cleanup task handle
    cleanup_task: Option<JoinHandle<()>>,
}

impl<K, V> AdaptiveTTLCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new adaptive TTL cache with the given configuration
    ///
    /// # Errors
    ///
    /// Returns `TTLConfigError` if the configuration is invalid
    pub fn new(config: TTLConfig) -> Result<Self, TTLConfigError> {
        config.validate()?;

        let entries = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(CacheStats::new());

        // Start background cleanup task if enabled
        let cleanup_task = if config.enable_background_cleanup {
            Some(Self::start_cleanup_task(
                Arc::clone(&entries),
                Arc::clone(&stats),
                config.cleanup_interval,
                config.max_entries,
            ))
        } else {
            None
        };

        info!(
            "AdaptiveTTLCache initialized: max_entries={}, base_ttl={:?}, cleanup_interval={:?}",
            config.max_entries, config.base_ttl, config.cleanup_interval
        );

        Ok(Self {
            entries,
            config,
            stats,
            cleanup_task,
        })
    }

    /// Create a new cache with default configuration
    pub fn default_config() -> Result<Self, TTLConfigError> {
        Self::new(TTLConfig::default())
    }

    /// Get a value from the cache
    ///
    /// Returns `Some(value)` if the key exists and hasn't expired,
    /// `None` otherwise.
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                // Entry has expired, remove it
                trace!("Entry expired for key, removing");
                self.stats.record_ttl_expiration();
                entries.remove(key);
                self.stats.update_entry_count(entries.len());
                return None;
            }

            // Record the access and update TTL
            entry.record_access(&self.config);
            self.stats.record_hit();
            self.stats.record_ttl_adaptation(entry.current_ttl);

            Some(entry.value.clone())
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Insert a value into the cache
    ///
    /// If the key already exists, the value is updated and the entry is reset.
    pub async fn insert(&self, key: K, value: V) {
        let mut entries = self.entries.write().await;

        // Check if we need to evict entries
        if entries.len() >= self.config.max_entries && !entries.contains_key(&key) {
            self.evict_oldest(&mut entries).await;
        }

        let entry = CacheEntry::new(value, self.config.base_ttl);
        entries.insert(key, entry);
        self.stats.update_entry_count(entries.len());

        debug!(
            "Inserted entry, cache size: {}/{}",
            entries.len(),
            self.config.max_entries
        );
    }

    /// Remove a value from the cache
    ///
    /// Returns `true` if the key existed and was removed, `false` otherwise.
    pub async fn remove(&self, key: &K) -> bool {
        let mut entries = self.entries.write().await;

        if entries.remove(key).is_some() {
            self.stats.record_removal();
            self.stats.update_entry_count(entries.len());
            true
        } else {
            false
        }
    }

    /// Check if a key exists in the cache (and hasn't expired)
    pub async fn contains(&self, key: &K) -> bool {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Get the current TTL for an entry
    pub async fn ttl(&self, key: &K) -> Option<Duration> {
        let entries = self.entries.read().await;
        entries.get(key).map(|e| e.current_ttl)
    }

    /// Get the remaining TTL for an entry
    pub async fn remaining_ttl(&self, key: &K) -> Option<Duration> {
        let entries = self.entries.read().await;
        entries.get(key).map(|e| e.remaining_ttl())
    }

    /// Get the access count for an entry
    pub async fn access_count(&self, key: &K) -> Option<u64> {
        let entries = self.entries.read().await;
        entries.get(key).map(|e| e.access_count)
    }

    /// Get the number of entries in the cache
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let count = entries.len();
        entries.clear();
        self.stats.update_entry_count(0);
        info!("Cleared {} entries from cache", count);
    }

    /// Get a snapshot of the current statistics
    pub fn stats(&self) -> CacheStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get the cache configuration
    pub fn config(&self) -> &TTLConfig {
        &self.config
    }

    /// Manually trigger cleanup of expired entries
    ///
    /// Returns the number of entries removed.
    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let before_count = entries.len();

        entries.retain(|_key, entry| {
            if entry.is_expired() {
                self.stats.record_ttl_expiration();
                false
            } else {
                true
            }
        });

        let removed = before_count - entries.len();
        self.stats.update_entry_count(entries.len());
        self.stats.record_cleanup();

        if removed > 0 {
            debug!("Cleaned up {} expired entries", removed);
        }

        removed
    }

    /// Get all keys in the cache (that haven't expired)
    pub async fn keys(&self) -> Vec<K> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Evict the oldest entry (LRU eviction)
    async fn evict_oldest(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
        {
            if let Some(entry) = entries.remove(&oldest_key) {
                // Estimate bytes (simplified - just use size of value)
                let estimated_bytes = std::mem::size_of_val(&entry.value) as u64;
                self.stats.record_eviction(estimated_bytes);
                debug!("Evicted oldest entry");
            }
        }
    }

    /// Start the background cleanup task
    fn start_cleanup_task(
        entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        stats: Arc<CacheStats>,
        interval_duration: Duration,
        _max_entries: usize,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(interval_duration.as_secs()));

            loop {
                ticker.tick().await;

                let mut entries_guard = entries.write().await;
                let before_count = entries_guard.len();

                // Remove expired entries
                entries_guard.retain(|_key, entry| !entry.is_expired());
                let expired_count = before_count - entries_guard.len();
                for _ in 0..expired_count {
                    stats.record_ttl_expiration();
                }

                let removed = before_count - entries_guard.len();
                stats.update_entry_count(entries_guard.len());
                stats.record_cleanup();

                if removed > 0 {
                    debug!("Background cleanup removed {} expired entries", removed);
                }

                drop(entries_guard);
            }
        })
    }

    /// Stop the background cleanup task
    pub fn stop_cleanup(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
            info!("Background cleanup task stopped");
        }
    }
}

impl<K, V> Drop for AdaptiveTTLCache<K, V> {
    fn drop(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
#[path = "adaptive_ttl_tests.rs"]
mod tests;
