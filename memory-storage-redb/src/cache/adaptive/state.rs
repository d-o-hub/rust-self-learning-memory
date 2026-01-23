//! Internal state for adaptive cache

use super::entry::AdaptiveCacheEntry;
use super::types::AdaptiveCacheMetrics;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Internal state for adaptive cache
pub(crate) struct AdaptiveCacheState<V> {
    /// Map of ID -> cache entry
    pub entries: HashMap<Uuid, AdaptiveCacheEntry<V>>,
    /// LRU queue (front = oldest, back = newest)
    pub lru_queue: VecDeque<Uuid>,
    /// Metrics
    pub metrics: AdaptiveCacheMetrics,
}

impl<V> AdaptiveCacheState<V> {
    /// Create a new cache state
    pub(crate) fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_queue: VecDeque::new(),
            metrics: AdaptiveCacheMetrics::default(),
        }
    }

    /// Update metrics based on current state
    pub(crate) fn update_metrics(&mut self, hot_threshold: usize, cold_threshold: usize) {
        let mut hot_count = 0;
        let mut cold_count = 0;

        for entry in self.entries.values() {
            if entry.is_hot(hot_threshold) {
                hot_count += 1;
            }
            if entry.is_cold(cold_threshold) {
                cold_count += 1;
            }
        }

        self.metrics.hot_item_count = hot_count;
        self.metrics.cold_item_count = cold_count;
        self.metrics.base.item_count = self.entries.len();
        self.metrics.base.calculate_hit_rate();
    }

    /// Remove an entry by ID
    pub(crate) fn remove_entry(&mut self, id: &Uuid) -> Option<AdaptiveCacheEntry<V>> {
        if let Some(entry) = self.entries.remove(id) {
            self.lru_queue.retain(|&qid| qid != *id);
            Some(entry)
        } else {
            None
        }
    }

    /// Clear all entries
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.lru_queue.clear();
        self.metrics = AdaptiveCacheMetrics::default();
    }
}

impl<V> Default for AdaptiveCacheState<V> {
    fn default() -> Self {
        Self::new()
    }
}
