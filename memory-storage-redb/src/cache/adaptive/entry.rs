//! Adaptive cache entry with TTL tracking

use super::types::AdaptiveCacheConfig;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Internal cache entry with adaptive TTL tracking
#[derive(Debug)]
pub(crate) struct AdaptiveCacheEntry<V> {
    /// Cached value
    pub value: V,
    /// When entry was created
    pub created_at: Instant,
    /// Last access time (for LRU ordering)
    pub last_accessed: Instant,
    /// Current TTL in seconds (atomic for lock-free reads)
    pub ttl_seconds: AtomicU64,
    /// Access count (atomic for lock-free reads)
    pub access_count: AtomicUsize,
    /// Sliding window of recent access timestamps
    pub access_window: VecDeque<Instant>,
}

impl<V> AdaptiveCacheEntry<V> {
    /// Create a new cache entry
    pub(crate) fn new(value: V, default_ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            ttl_seconds: AtomicU64::new(default_ttl.as_secs()),
            access_count: AtomicUsize::new(0),
            access_window: VecDeque::new(),
        }
    }

    /// Record an access to this entry and update TTL
    pub(crate) fn record_access(&mut self, now: Instant, config: &AdaptiveCacheConfig) {
        self.last_accessed = now;

        // Add to access window
        self.access_window.push_back(now);
        if self.access_window.len() > config.window_size {
            self.access_window.pop_front();
        }

        // Increment access count
        let count = self.access_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Update TTL based on access frequency
        let current_ttl = self.ttl_seconds.load(Ordering::SeqCst);
        let new_ttl = self.calculate_adaptive_ttl(count, current_ttl, config);
        self.ttl_seconds.store(new_ttl, Ordering::SeqCst);
    }

    /// Calculate new TTL based on access pattern
    fn calculate_adaptive_ttl(
        &self,
        access_count: usize,
        current_ttl: u64,
        config: &AdaptiveCacheConfig,
    ) -> u64 {
        let mut new_ttl = current_ttl as f64;
        let adaptation = config.adaptation_rate;

        if access_count >= config.hot_threshold {
            // Hot item: increase TTL exponentially
            new_ttl *= 1.0 + adaptation;
        } else if access_count <= config.cold_threshold {
            // Cold item: decrease TTL exponentially
            new_ttl *= 1.0 - adaptation;
        }

        // Clamp to bounds
        let min_secs = config.min_ttl.as_secs() as f64;
        let max_secs = config.max_ttl.as_secs() as f64;
        new_ttl = new_ttl.clamp(min_secs, max_secs);

        new_ttl as u64
    }

    /// Check if entry has expired
    pub(crate) fn is_expired(&self, now: Instant) -> bool {
        let ttl_seconds = self.ttl_seconds.load(Ordering::SeqCst);
        if ttl_seconds == 0 {
            return false; // No expiration
        }
        let expires_at = self.created_at + Duration::from_secs(ttl_seconds);
        now >= expires_at
    }

    /// Get current TTL
    pub(crate) fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_seconds.load(Ordering::SeqCst))
    }

    /// Get access count
    pub(crate) fn access_count(&self) -> usize {
        self.access_count.load(Ordering::SeqCst)
    }

    /// Check if item is "hot"
    pub(crate) fn is_hot(&self, hot_threshold: usize) -> bool {
        self.access_count.load(Ordering::SeqCst) >= hot_threshold
    }

    /// Check if item is "cold"
    pub(crate) fn is_cold(&self, cold_threshold: usize) -> bool {
        self.access_count.load(Ordering::SeqCst) <= cold_threshold
    }
}

impl<V: Clone> Clone for AdaptiveCacheEntry<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            ttl_seconds: AtomicU64::new(self.ttl_seconds.load(Ordering::SeqCst)),
            access_count: AtomicUsize::new(self.access_count.load(Ordering::SeqCst)),
            access_window: self.access_window.clone(),
        }
    }
}
