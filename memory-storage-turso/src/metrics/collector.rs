//! # Metrics Collector
//!
//! Collector for operation-specific metrics including latency histograms
//! and operation counts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use parking_lot::RwLock;
use tracing::debug;

use super::core::TursoMetrics;
use super::types::{CacheStats, LatencyStats, OperationMetrics, PoolStats};

/// Collector for operation-specific metrics
///
/// This struct tracks per-operation metrics including latency histograms
/// and operation counts.
#[derive(Debug)]
pub struct MetricsCollector {
    /// Core metrics
    metrics: Arc<TursoMetrics>,
    /// Operation-specific latency tracking
    operations: RwLock<HashMap<String, LatencyStats>>,
    /// Error tracking by type
    errors: RwLock<HashMap<String, u64>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(TursoMetrics::new()),
            operations: RwLock::new(HashMap::new()),
            errors: RwLock::new(HashMap::new()),
        }
    }

    /// Get reference to core metrics
    pub fn metrics(&self) -> &TursoMetrics {
        &self.metrics
    }

    /// Record a query with operation type and duration
    ///
    /// # Arguments
    ///
    /// * `operation` - Operation type string
    /// * `duration` - Query duration
    /// * `success` - Whether the query succeeded
    /// * `bytes_transferred` - Optional bytes read/written
    pub fn record_query(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
        bytes_transferred: Option<u64>,
    ) {
        let duration_us = duration.as_micros() as u64;

        // Update core metrics
        self.metrics.record_query(success, duration_us);

        if let Some(bytes) = bytes_transferred {
            if success {
                self.metrics.record_bytes_read(bytes);
            }
        }

        // Update operation-specific latency
        let mut operations = self.operations.write();
        let stats = operations
            .entry(operation.to_string())
            .or_insert_with(LatencyStats::new);
        stats.record(duration_us);
    }

    /// Record a cache operation
    pub fn record_cache(&self, hit: bool) {
        if hit {
            self.metrics.record_cache_hit();
        } else {
            self.metrics.record_cache_miss();
        }
    }

    /// Record a cache eviction
    pub fn record_cache_eviction(&self) {
        self.metrics.record_cache_eviction();
    }

    /// Record connection acquisition
    pub fn record_connection(&self, wait_time: Duration) {
        self.metrics
            .record_connection_acquisition(wait_time.as_micros() as u64);
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str) {
        let mut errors = self.errors.write();
        *errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Get metrics for a specific operation
    pub fn operation_metrics(&self, operation: &str) -> Option<OperationMetrics> {
        let operations = self.operations.read();
        operations.get(operation).map(|latency| OperationMetrics {
            operation: operation.to_string(),
            total_count: latency.count,
            success_count: latency.count, // We don't track failures per operation
            error_count: 0,
            latency: latency.clone(),
        })
    }

    /// Get all operation metrics
    pub fn all_operation_metrics(&self) -> Vec<OperationMetrics> {
        let operations = self.operations.read();
        operations
            .iter()
            .map(|(op, latency)| OperationMetrics {
                operation: op.clone(),
                total_count: latency.count,
                success_count: latency.count,
                error_count: 0,
                latency: latency.clone(),
            })
            .collect()
    }

    /// Get latency percentiles for an operation
    ///
    /// # Returns
    ///
    /// Tuple of (p50, p95, p99) in microseconds, or None if no data
    pub fn latency_percentiles(&self, operation: &str) -> Option<(u64, u64, u64)> {
        let operations = self.operations.read();
        operations.get(operation).map(|stats| stats.percentiles())
    }

    /// Get all latency percentiles
    pub fn all_latency_percentiles(&self) -> HashMap<String, (u64, u64, u64)> {
        let operations = self.operations.read();
        operations
            .iter()
            .map(|(op, stats)| (op.clone(), stats.percentiles()))
            .collect()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.metrics.cache_hits(),
            misses: self.metrics.cache_misses(),
            evictions: self.metrics.cache_evictions(),
            current_size: 0, // Would need to track separately
            max_size: 0,     // Would need to track separately
        }
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            active_connections: self.metrics.active_connections(),
            idle_connections: self.metrics.idle_connections(),
            total_connections: self.metrics.active_connections() + self.metrics.idle_connections(),
            wait_count: self.metrics.connection_acquisitions(),
            wait_time_us: 0, // Would need to track separately
        }
    }

    /// Get overall query success rate
    pub fn query_success_rate(&self) -> f64 {
        let total = self.metrics.total_queries();
        if total == 0 {
            0.0
        } else {
            self.metrics.successful_queries() as f64 / total as f64
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        // Reset atomic counters
        self.metrics.total_queries.store(0, Ordering::Relaxed);
        self.metrics.successful_queries.store(0, Ordering::Relaxed);
        self.metrics.failed_queries.store(0, Ordering::Relaxed);
        self.metrics.bytes_read.store(0, Ordering::Relaxed);
        self.metrics.bytes_written.store(0, Ordering::Relaxed);
        self.metrics
            .connection_acquisitions
            .store(0, Ordering::Relaxed);
        self.metrics
            .connection_wait_time_us
            .store(0, Ordering::Relaxed);
        self.metrics.cache_hits.store(0, Ordering::Relaxed);
        self.metrics.cache_misses.store(0, Ordering::Relaxed);
        self.metrics.cache_evictions.store(0, Ordering::Relaxed);

        // Reset operation maps
        let mut operations = self.operations.write();
        operations.clear();
        let mut errors = self.errors.write();
        errors.clear();

        debug!("Metrics collector reset");
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record some queries
        collector.record_query("episode_create", Duration::from_micros(100), true, None);
        collector.record_query("episode_create", Duration::from_micros(200), true, None);
        collector.record_query("episode_get", Duration::from_micros(50), true, None);

        // Get operation metrics
        let episode_create_metrics = collector.operation_metrics("episode_create").unwrap();
        assert_eq!(episode_create_metrics.total_count, 2);

        // Get percentiles
        let (p50, p95, p99) = collector.latency_percentiles("episode_create").unwrap();
        assert!(p50 > 0);
        assert!(p95 >= p50);
        assert!(p99 >= p95);

        // Check all operations
        let all_ops = collector.all_operation_metrics();
        assert_eq!(all_ops.len(), 2);
    }

    #[test]
    fn test_error_tracking() {
        let collector = MetricsCollector::new();

        collector.record_error("connection_timeout");
        collector.record_error("connection_timeout");
        collector.record_error("validation_error");

        let errors = collector.errors.read();
        assert_eq!(errors.get("connection_timeout"), Some(&2));
        assert_eq!(errors.get("validation_error"), Some(&1));
    }

    #[test]
    fn test_query_success_rate() {
        let collector = MetricsCollector::new();

        collector.record_query("test", Duration::from_micros(100), true, None);
        collector.record_query("test", Duration::from_micros(100), true, None);
        collector.record_query("test", Duration::from_micros(100), false, None);

        assert!((collector.query_success_rate() - 0.666).abs() < 0.01);
    }
}
