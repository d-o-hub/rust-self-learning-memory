//! # Prometheus Metrics for Redb Cache
//!
//! This module provides Prometheus-compatible metrics for the redb cache layer.
//! It tracks cache hits, misses, and evictions in Prometheus format.
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_redb::metrics::RedbMetrics;
//!
//! let metrics = RedbMetrics::new();
//! metrics.record_cache_hit();
//! metrics.record_cache_miss();
//! let output = metrics.export_metrics();
//! ```

use parking_lot::RwLock;
use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;

/// Metrics for redb cache operations
#[derive(Debug)]
pub struct RedbMetrics {
    /// Total cache hits
    cache_hits: AtomicU64,
    /// Total cache misses
    cache_misses: AtomicU64,
    /// Total cache evictions
    cache_evictions: AtomicU64,
    /// Total cache expirations
    cache_expirations: AtomicU64,
    /// Total items in cache
    cache_items: AtomicU64,
    /// Total bytes used by cache
    cache_bytes: AtomicU64,
    /// Last export timestamp
    last_export: RwLock<u64>,
    /// Export count
    export_count: RwLock<u64>,
}

impl RedbMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_evictions: AtomicU64::new(0),
            cache_expirations: AtomicU64::new(0),
            cache_items: AtomicU64::new(0),
            cache_bytes: AtomicU64::new(0),
            last_export: RwLock::new(0),
            export_count: RwLock::new(0),
        }
    }

    /// Record a cache hit
    #[inline]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    #[inline]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache eviction
    #[inline]
    pub fn record_cache_eviction(&self) {
        self.cache_evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache expiration
    #[inline]
    pub fn record_cache_expiration(&self) {
        self.cache_expirations.fetch_add(1, Ordering::Relaxed);
    }

    /// Update cache size metrics
    #[inline]
    pub fn update_cache_size(&self, items: usize, bytes: usize) {
        self.cache_items.store(items as u64, Ordering::Relaxed);
        self.cache_bytes.store(bytes as u64, Ordering::Relaxed);
    }

    /// Get cache hit count
    #[inline]
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get cache miss count
    #[inline]
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Get cache eviction count
    #[inline]
    pub fn cache_evictions(&self) -> u64 {
        self.cache_evictions.load(Ordering::Relaxed)
    }

    /// Get cache expiration count
    #[inline]
    pub fn cache_expirations(&self) -> u64 {
        self.cache_expirations.load(Ordering::Relaxed)
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits();
        let misses = self.cache_misses();
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get cache item count
    #[inline]
    pub fn cache_items(&self) -> u64 {
        self.cache_items.load(Ordering::Relaxed)
    }

    /// Get cache bytes
    #[inline]
    pub fn cache_bytes(&self) -> u64 {
        self.cache_bytes.load(Ordering::Relaxed)
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> String {
        let mut output = String::with_capacity(2048);

        let hits = self.cache_hits();
        let misses = self.cache_misses();
        let evictions = self.cache_evictions();
        let expirations = self.cache_expirations();
        let items = self.cache_items();
        let bytes = self.cache_bytes();
        let hit_rate = self.cache_hit_rate();

        // Cache hits
        writeln!(output, "# HELP redb_cache_hits_total Total cache hits").ok();
        writeln!(output, "# TYPE redb_cache_hits_total counter").ok();
        writeln!(output, "redb_cache_hits_total {}", hits).ok();

        // Cache misses
        writeln!(
            output,
            "\n# HELP redb_cache_misses_total Total cache misses"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_misses_total counter").ok();
        writeln!(output, "redb_cache_misses_total {}", misses).ok();

        // Cache hit rate
        writeln!(output, "\n# HELP redb_cache_hit_rate Cache hit rate (0-1)").ok();
        writeln!(output, "# TYPE redb_cache_hit_rate gauge").ok();
        writeln!(output, "redb_cache_hit_rate {:.4}", hit_rate).ok();

        // Cache evictions
        writeln!(
            output,
            "\n# HELP redb_cache_evictions_total Total cache evictions"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_evictions_total counter").ok();
        writeln!(output, "redb_cache_evictions_total {}", evictions).ok();

        // Cache expirations
        writeln!(
            output,
            "\n# HELP redb_cache_expirations_total Total cache expirations"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_expirations_total counter").ok();
        writeln!(output, "redb_cache_expirations_total {}", expirations).ok();

        // Cache items
        writeln!(
            output,
            "\n# HELP redb_cache_items Current number of items in cache"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_items gauge").ok();
        writeln!(output, "redb_cache_items {}", items).ok();

        // Cache bytes
        writeln!(
            output,
            "\n# HELP redb_cache_bytes Total bytes used by cache"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_bytes gauge").ok();
        writeln!(output, "redb_cache_bytes {}", bytes).ok();

        // Update export statistics
        {
            let mut count = self.export_count.write();
            *count += 1;
            let mut last = self.last_export.write();
            *last = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }

        debug!("Exported {} bytes of redb metrics", output.len());
        output
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.cache_evictions.store(0, Ordering::Relaxed);
        self.cache_expirations.store(0, Ordering::Relaxed);
        self.cache_items.store(0, Ordering::Relaxed);
        self.cache_bytes.store(0, Ordering::Relaxed);
        *self.export_count.write() = 0;
        *self.last_export.write() = 0;
    }
}

impl Default for RedbMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = RedbMetrics::new();
        assert_eq!(metrics.cache_hits(), 0);
        assert_eq!(metrics.cache_misses(), 0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_hit() {
        let metrics = RedbMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.cache_hits(), 2);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_eviction() {
        let metrics = RedbMetrics::new();
        metrics.record_cache_eviction();
        metrics.record_cache_eviction();

        assert_eq!(metrics.cache_evictions(), 2);
    }

    #[test]
    fn test_cache_size_update() {
        let metrics = RedbMetrics::new();
        metrics.update_cache_size(100, 1024000);

        assert_eq!(metrics.cache_items(), 100);
        assert_eq!(metrics.cache_bytes(), 1024000);
    }

    #[test]
    fn test_export_format() {
        let metrics = RedbMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.update_cache_size(50, 50000);

        let output = metrics.export_metrics();

        assert!(output.contains("redb_cache_hits_total"));
        assert!(output.contains("redb_cache_misses_total"));
        assert!(output.contains("redb_cache_hit_rate"));
        assert!(output.contains("redb_cache_items"));
        assert!(output.contains("redb_cache_bytes"));
    }

    #[test]
    fn test_reset() {
        let metrics = RedbMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.reset();

        assert_eq!(metrics.cache_hits(), 0);
        assert_eq!(metrics.cache_misses(), 0);
    }
}
