//! Advanced Query Result Caching with Smart Invalidation
//!
//! Provides sophisticated caching for query results with:
//! - Dependency tracking for smart invalidation
//! - Configurable TTL per query type
//! - LRU eviction with size limits
//! - Cache hit/miss statistics
//! - Background refresh for hot queries
//! - Thread-safe concurrent access

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, trace};

#[path = "query_cache_types.rs"]
mod types;
pub use types::{
    AdvancedCacheStats, AdvancedQueryCacheConfig, CachedResult, InvalidationMessage, QueryKey,
    QueryType, TableDependency,
};

/// Advanced query cache with smart invalidation
pub struct AdvancedQueryCache {
    /// Cached results
    results: Arc<RwLock<HashMap<QueryKey, CachedResult>>>,
    /// Reverse index: table -> query keys that depend on it
    dependency_index: Arc<RwLock<HashMap<TableDependency, HashSet<QueryKey>>>>,
    /// LRU queue for eviction (front = oldest)
    lru_queue: Arc<RwLock<VecDeque<QueryKey>>>,
    /// Configuration
    config: AdvancedQueryCacheConfig,
    /// Statistics
    stats: Arc<RwLock<AdvancedCacheStats>>,
    /// Invalidation channel sender
    invalidation_tx: mpsc::UnboundedSender<InvalidationMessage>,
    /// Hot queries set (for background refresh)
    hot_queries: Arc<RwLock<HashSet<QueryKey>>>,
}

impl AdvancedQueryCache {
    /// Create a new advanced query cache
    pub fn new(
        config: AdvancedQueryCacheConfig,
    ) -> (Self, mpsc::UnboundedReceiver<InvalidationMessage>) {
        let (invalidation_tx, invalidation_rx) = mpsc::unbounded_channel();

        let cache = Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            dependency_index: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(RwLock::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(AdvancedCacheStats::default())),
            invalidation_tx,
            hot_queries: Arc::new(RwLock::new(HashSet::new())),
        };

        (cache, invalidation_rx)
    }

    /// Create with default configuration
    pub fn new_with_receiver() -> (Self, mpsc::UnboundedReceiver<InvalidationMessage>) {
        Self::new(AdvancedQueryCacheConfig::default())
    }

    /// Get cached result
    pub fn get(&self, key: &QueryKey) -> Option<Vec<u8>> {
        let results = self.results.read();

        if let Some(result) = results.get(key) {
            if result.is_expired() {
                drop(results);
                self.handle_expired(key);
                return None;
            }

            result.record_access();

            // Check if this is becoming a hot query
            if result.access_count() >= self.config.hot_threshold {
                let mut hot = self.hot_queries.write();
                hot.insert(key.clone());
            }

            // Update LRU queue
            self.update_lru(key.clone());

            // Update stats
            self.stats.write().hits += 1;

            trace!("Cache hit for query key: {:?}", key);
            Some(result.data.clone())
        } else {
            self.stats.write().misses += 1;
            trace!("Cache miss for query key: {:?}", key);
            None
        }
    }

    /// Store result in cache
    pub fn put(&self, key: QueryKey, data: Vec<u8>, dependencies: Vec<TableDependency>) {
        let ttl = self.config.ttl_for_type(key.query_type);

        // Evict if at capacity
        self.evict_if_needed();

        // Build dependency index
        if self.config.enable_dependency_tracking {
            let mut index = self.dependency_index.write();
            for dep in &dependencies {
                index.entry(dep.clone()).or_default().insert(key.clone());
            }
        }

        // Store result
        let result = CachedResult::new(data, ttl, dependencies, key.query_type);

        let mut results = self.results.write();
        results.insert(key.clone(), result);
        drop(results);

        // Update LRU queue
        self.lru_queue.write().push_back(key);

        // Update stats
        self.stats.write().current_size = self.results.read().len();

        debug!("Cached query result with TTL: {:?}", ttl);
    }

    /// Invalidate cache entries by table dependency
    pub fn invalidate_by_table(&self, table: &TableDependency) {
        if !self.config.enable_dependency_tracking {
            return;
        }

        let keys_to_invalidate: Vec<QueryKey> = {
            let index = self.dependency_index.read();
            index
                .get(table)
                .map(|keys| keys.iter().cloned().collect())
                .unwrap_or_default()
        };

        let mut invalidated = 0;
        for key in keys_to_invalidate {
            self.remove_entry(&key);
            invalidated += 1;
        }

        if invalidated > 0 {
            self.stats.write().invalidations += invalidated;
            info!(
                "Invalidated {} cache entries for table: {:?}",
                invalidated, table
            );
        }
    }

    /// Invalidate specific query key
    pub fn invalidate_key(&self, key: &QueryKey) {
        self.remove_entry(key);
    }

    /// Clear all cached results
    pub fn clear(&self) {
        self.results.write().clear();
        self.dependency_index.write().clear();
        self.lru_queue.write().clear();
        self.hot_queries.write().clear();
        self.stats.write().current_size = 0;

        info!("Cleared all query cache entries");
    }

    /// Get cache statistics
    pub fn stats(&self) -> AdvancedCacheStats {
        self.stats.read().clone()
    }

    /// Get hot queries that need background refresh
    pub fn get_hot_queries_needing_refresh(&self) -> Vec<QueryKey> {
        let results = self.results.read();
        let hot = self.hot_queries.read();

        hot.iter()
            .filter(|key| {
                results.get(key).is_some_and(|r| {
                    r.should_refresh(self.config.hot_threshold, self.config.refresh_interval)
                })
            })
            .cloned()
            .collect()
    }

    /// Mark a query as refreshed
    pub fn mark_refreshed(&self, key: &QueryKey) {
        let mut results = self.results.write();
        if let Some(result) = results.get_mut(key) {
            // Reset creation time to extend TTL
            result.created_at = Instant::now();
            self.stats.write().refreshes += 1;
        }
    }

    /// Get the invalidation sender
    pub fn invalidation_sender(&self) -> mpsc::UnboundedSender<InvalidationMessage> {
        self.invalidation_tx.clone()
    }

    /// Handle expired entry
    fn handle_expired(&self, key: &QueryKey) {
        self.remove_entry(key);
        self.stats.write().expirations += 1;
        trace!("Removed expired cache entry: {:?}", key);
    }

    /// Remove a cache entry and clean up dependencies
    fn remove_entry(&self, key: &QueryKey) {
        let result = self.results.write().remove(key);

        if let Some(result) = result {
            self.cleanup_dependency_index(key, &result.dependencies);
        }

        // Remove from LRU queue
        self.lru_queue.write().retain(|k| k != key);

        // Remove from hot queries
        self.hot_queries.write().remove(key);

        // Update stats
        self.stats.write().current_size = self.results.read().len();
    }

    /// Clean up dependency index for removed entry
    fn cleanup_dependency_index(&self, key: &QueryKey, dependencies: &[TableDependency]) {
        if !self.config.enable_dependency_tracking {
            return;
        }
        let mut index = self.dependency_index.write();
        for dep in dependencies {
            if let Some(keys) = index.get_mut(dep) {
                keys.remove(key);
                if keys.is_empty() {
                    index.remove(dep);
                }
            }
        }
    }

    /// Evict oldest entries if at capacity
    fn evict_if_needed(&self) {
        let current_size = self.results.read().len();

        if current_size >= self.config.max_queries {
            let keys_to_evict: Vec<QueryKey> = {
                let lru = self.lru_queue.read();
                lru.iter()
                    .take(current_size - self.config.max_queries + 1)
                    .cloned()
                    .collect()
            };

            for key in keys_to_evict {
                self.remove_entry(&key);
                self.stats.write().evictions += 1;
                debug!("Evicted LRU cache entry: {:?}", key);
            }
        }
    }

    /// Update LRU queue (move accessed key to back)
    fn update_lru(&self, key: QueryKey) {
        let mut lru = self.lru_queue.write();
        lru.retain(|k| k != &key);
        lru.push_back(key);
    }

    /// Clear expired entries
    pub fn clear_expired(&self) -> usize {
        let expired_keys: Vec<QueryKey> = {
            let results = self.results.read();
            results
                .iter()
                .filter(|(_, result)| result.is_expired())
                .map(|(key, _)| key.clone())
                .collect()
        };

        let count = expired_keys.len();
        for key in expired_keys {
            self.remove_entry(&key);
        }

        if count > 0 {
            self.stats.write().expirations += count as u64;
            debug!("Cleared {} expired cache entries", count);
        }

        count
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.results.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for AdvancedQueryCache {
    fn clone(&self) -> Self {
        Self {
            results: Arc::clone(&self.results),
            dependency_index: Arc::clone(&self.dependency_index),
            lru_queue: Arc::clone(&self.lru_queue),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            invalidation_tx: self.invalidation_tx.clone(),
            hot_queries: Arc::clone(&self.hot_queries),
        }
    }
}

#[cfg(test)]
#[path = "query_cache_tests.rs"]
mod tests;
