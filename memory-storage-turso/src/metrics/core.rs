//! # Core Metrics
//!
//! Core metrics structures using atomic counters for high-performance collection.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Core metrics for Turso storage
///
/// This struct holds atomic counters for high-performance metrics collection.
#[derive(Debug, Default)]
pub struct TursoMetrics {
    /// Total queries executed
    total_queries: AtomicU64,
    /// Total successful queries
    successful_queries: AtomicU64,
    /// Total failed queries
    failed_queries: AtomicU64,
    /// Total bytes read
    bytes_read: AtomicU64,
    /// Total bytes written
    bytes_written: AtomicU64,
    /// Total connection acquisitions
    connection_acquisitions: AtomicU64,
    /// Total connection wait time (microseconds)
    connection_wait_time_us: AtomicU64,
    /// Cache hits
    cache_hits: AtomicU64,
    /// Cache misses
    cache_misses: AtomicU64,
    /// Cache evictions
    cache_evictions: AtomicU64,
    /// Active connections
    active_connections: AtomicU32,
    /// Idle connections
    idle_connections: AtomicU32,
}

impl TursoMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a query execution
    pub fn record_query(&self, success: bool, duration_us: u64) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_queries.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }
        // Latency is recorded separately in MetricsCollector
    }

    /// Record bytes read from database
    pub fn record_bytes_read(&self, bytes: u64) {
        self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record bytes written to database
    pub fn record_bytes_written(&self, bytes: u64) {
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record connection acquisition
    pub fn record_connection_acquisition(&self, wait_time_us: u64) {
        self.connection_acquisitions.fetch_add(1, Ordering::Relaxed);
        self.connection_wait_time_us
            .fetch_add(wait_time_us, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache eviction
    pub fn record_cache_eviction(&self) {
        self.cache_evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active connections count
    pub fn set_active_connections(&self, count: u32) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    /// Update idle connections count
    pub fn set_idle_connections(&self, count: u32) {
        self.idle_connections.store(count, Ordering::Relaxed);
    }

    /// Get total query count
    pub fn total_queries(&self) -> u64 {
        self.total_queries.load(Ordering::Relaxed)
    }

    /// Get successful query count
    pub fn successful_queries(&self) -> u64 {
        self.successful_queries.load(Ordering::Relaxed)
    }

    /// Get failed query count
    pub fn failed_queries(&self) -> u64 {
        self.failed_queries.load(Ordering::Relaxed)
    }

    /// Get cache hit count
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get cache miss count
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Get cache hit rate
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

    /// Get active connections count
    pub fn active_connections(&self) -> u32 {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Get idle connections count
    pub fn idle_connections(&self) -> u32 {
        self.idle_connections.load(Ordering::Relaxed)
    }

    /// Get bytes read
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read.load(Ordering::Relaxed)
    }

    /// Get bytes written
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }

    /// Get connection acquisitions
    pub fn connection_acquisitions(&self) -> u64 {
        self.connection_acquisitions.load(Ordering::Relaxed)
    }

    /// Get connection wait time
    pub fn connection_wait_time_us(&self) -> u64 {
        self.connection_wait_time_us.load(Ordering::Relaxed)
    }

    /// Get cache evictions count
    pub fn cache_evictions(&self) -> u64 {
        self.cache_evictions.load(Ordering::Relaxed)
    }

    /// Reset all atomic counters to zero
    #[allow(clippy::rest_pat_in_fully_bound_struct)]
    pub fn reset(&self) {
        self.total_queries.store(0, Ordering::Relaxed);
        self.successful_queries.store(0, Ordering::Relaxed);
        self.failed_queries.store(0, Ordering::Relaxed);
        self.bytes_read.store(0, Ordering::Relaxed);
        self.bytes_written.store(0, Ordering::Relaxed);
        self.connection_acquisitions.store(0, Ordering::Relaxed);
        self.connection_wait_time_us.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.cache_evictions.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        self.idle_connections.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turso_metrics() {
        let metrics = TursoMetrics::new();

        metrics.record_query(true, 100);
        metrics.record_query(true, 200);
        metrics.record_query(false, 50);

        assert_eq!(metrics.total_queries(), 3);
        assert_eq!(metrics.successful_queries(), 2);
        assert_eq!(metrics.failed_queries(), 1);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = TursoMetrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.cache_hits(), 2);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_connection_metrics() {
        let metrics = TursoMetrics::new();

        metrics.record_connection_acquisition(100);
        metrics.record_connection_acquisition(200);

        assert_eq!(metrics.connection_acquisitions(), 2);
        assert_eq!(metrics.connection_wait_time_us(), 300);
    }

    #[test]
    fn test_connection_tracking() {
        let metrics = TursoMetrics::new();

        metrics.set_active_connections(5);
        metrics.set_idle_connections(3);

        assert_eq!(metrics.active_connections(), 5);
        assert_eq!(metrics.idle_connections(), 3);
    }

    #[test]
    fn test_bytes_metrics() {
        let metrics = TursoMetrics::new();

        metrics.record_bytes_read(1024);
        metrics.record_bytes_written(512);

        assert_eq!(metrics.bytes_read(), 1024);
        assert_eq!(metrics.bytes_written(), 512);
    }
}
