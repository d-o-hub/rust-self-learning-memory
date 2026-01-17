//! Internal cache state management
//!
//! This module contains the internal state structure for the LRU cache,
//! including the entry storage and LRU ordering queue.

use super::types::{CacheEntry, CacheMetrics};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Internal cache state
pub(crate) struct CacheState {
    /// Map of ID -> cache entry
    pub entries: HashMap<Uuid, CacheEntry>,
    /// LRU queue (front = oldest, back = newest)
    pub lru_queue: VecDeque<Uuid>,
    /// Metrics
    pub metrics: CacheMetrics,
}

impl CacheState {
    /// Create a new cache state
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_queue: VecDeque::new(),
            metrics: CacheMetrics::default(),
        }
    }

    /// Update metrics based on current state
    pub fn update_metrics(&mut self) {
        self.metrics.item_count = self.entries.len();
        self.metrics.total_size_bytes = self.entries.values().map(|e| e.size_bytes).sum();
        self.metrics.calculate_hit_rate();
    }

    /// Remove an entry by ID
    pub fn remove_entry(&mut self, id: &Uuid) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.remove(id) {
            // Remove from LRU queue
            self.lru_queue.retain(|&qid| qid != *id);
            Some(entry)
        } else {
            None
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_queue.clear();
        self.metrics = CacheMetrics::default();
    }
}

impl Default for CacheState {
    fn default() -> Self {
        Self::new()
    }
}
