//! # Prometheus Metrics Export Module
//!
//! This module provides Prometheus-compatible metrics export for Turso storage operations.
//! It collects latency, throughput, and error metrics and formats them for Prometheus scraping.
//!
//! ## Features
//!
//! - **Latency Histograms**: P50, P95, P99 latency tracking per operation
//! - **Throughput Counters**: Operations per second tracking
//! - **Error Rates**: Error tracking by type
//! - **Prometheus Format**: Standard Prometheus exposition format
//! - **Configurable Export**: HTTP endpoint or file-based export
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_turso::metrics::export::{PrometheusExporter, ExportConfig};
//!
//! let config = ExportConfig::default();
//! let exporter = PrometheusExporter::new(config);
//! let metrics_text = exporter.export_metrics();
//! ```

use std::fmt::Write;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tracing::{debug, info};

use crate::metrics::collector::MetricsCollector;

// Submodules
mod config;
mod http;
mod types;

pub use config::{ExportConfig, ExportFormat, ExportTarget};
pub use http::MetricsHttpServer;
pub use types::{ExportedMetric, MetricType, MetricValue};

/// Prometheus-compatible metrics exporter
///
/// Collects metrics from the MetricsCollector and exports them in Prometheus format.
/// Supports both HTTP endpoint and file-based export.
#[derive(Debug)]
pub struct PrometheusExporter {
    config: ExportConfig,
    collector: MetricsCollector,
    last_export: RwLock<Instant>,
    export_count: RwLock<u64>,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter with the given configuration
    pub fn new(config: ExportConfig) -> Self {
        info!(
            "Creating Prometheus exporter with format={:?}, target={:?}",
            config.format, config.target
        );

        Self {
            config,
            collector: MetricsCollector::new(),
            last_export: RwLock::new(Instant::now()),
            export_count: RwLock::new(0),
        }
    }

    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(ExportConfig::default())
    }

    /// Get reference to the metrics collector
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }

    /// Record a query operation for metrics collection
    pub fn record_query(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
        bytes_transferred: Option<u64>,
    ) {
        self.collector
            .record_query(operation, duration, success, bytes_transferred);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.collector.record_cache(true);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.collector.record_cache(false);
    }

    /// Record a cache eviction
    pub fn record_cache_eviction(&self) {
        self.collector.record_cache_eviction();
    }

    /// Record connection acquisition
    pub fn record_connection(&self, wait_time: Duration) {
        self.collector.record_connection(wait_time);
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str) {
        self.collector.record_error(error_type);
    }

    /// Export metrics in Prometheus format
    ///
    /// Returns a string containing all metrics in Prometheus exposition format.
    pub fn export_metrics(&self) -> String {
        let mut output = String::with_capacity(4096);

        // Add header
        writeln!(
            &mut output,
            "# HELP turso_storage_metrics Turso storage metrics\n# TYPE turso_storage_metrics gauge"
        )
        .ok();

        // Export operation metrics
        self.export_operation_metrics(&mut output);

        // Export cache metrics
        self.export_cache_metrics(&mut output);

        // Export pool metrics
        self.export_pool_metrics(&mut output);

        // Export throughput metrics
        self.export_throughput_metrics(&mut output);

        // Export error metrics
        self.export_error_metrics(&mut output);

        // Update export statistics
        {
            let mut count = self.export_count.write();
            *count += 1;
            let mut last = self.last_export.write();
            *last = Instant::now();
        }

        debug!("Exported {} bytes of metrics", output.len());
        output
    }

    /// Export operation-specific metrics
    fn export_operation_metrics(&self, output: &mut String) {
        let operations = self.collector.all_operation_metrics();

        // Header for operation metrics
        writeln!(output, "# Operation latency metrics").ok();
        writeln!(
            output,
            "# HELP turso_operation_latency_microseconds Operation latency in microseconds"
        )
        .ok();
        writeln!(
            output,
            "# TYPE turso_operation_latency_microseconds summary"
        )
        .ok();

        for op in operations {
            let (p50, p95, p99) = op.latency.percentiles();

            // Export quantiles
            writeln!(
                output,
                "turso_operation_latency_microseconds{{operation=\"{}\",quantile=\"0.5\"}} {}",
                op.operation, p50
            )
            .ok();
            writeln!(
                output,
                "turso_operation_latency_microseconds{{operation=\"{}\",quantile=\"0.95\"}} {}",
                op.operation, p95
            )
            .ok();
            writeln!(
                output,
                "turso_operation_latency_microseconds{{operation=\"{}\",quantile=\"0.99\"}} {}",
                op.operation, p99
            )
            .ok();

            // Export count
            writeln!(
                output,
                "turso_operation_count{{operation=\"{}\"}} {}",
                op.operation, op.total_count
            )
            .ok();

            // Export success rate
            let success_rate = op.success_rate();
            writeln!(
                output,
                "turso_operation_success_rate{{operation=\"{}\"}} {:.4}",
                op.operation, success_rate
            )
            .ok();
        }
    }

    /// Export cache metrics
    fn export_cache_metrics(&self, output: &mut String) {
        let cache_stats = self.collector.cache_stats();
        let hit_rate = cache_stats.hit_rate();

        writeln!(output, "\n# Cache metrics").ok();
        writeln!(output, "# HELP turso_cache_hits Total cache hits").ok();
        writeln!(output, "# TYPE turso_cache_hits counter").ok();
        writeln!(output, "turso_cache_hits {}", cache_stats.hits).ok();

        writeln!(output, "# HELP turso_cache_misses Total cache misses").ok();
        writeln!(output, "# TYPE turso_cache_misses counter").ok();
        writeln!(output, "turso_cache_misses {}", cache_stats.misses).ok();

        writeln!(output, "# HELP turso_cache_hit_rate Cache hit rate (0-1)").ok();
        writeln!(output, "# TYPE turso_cache_hit_rate gauge").ok();
        writeln!(output, "turso_cache_hit_rate {:.4}", hit_rate).ok();

        writeln!(output, "# HELP turso_cache_evictions Total cache evictions").ok();
        writeln!(output, "# TYPE turso_cache_evictions counter").ok();
        writeln!(output, "turso_cache_evictions {}", cache_stats.evictions).ok();
    }

    /// Export pool metrics
    fn export_pool_metrics(&self, output: &mut String) {
        let pool_stats = self.collector.pool_stats();
        let utilization = pool_stats.utilization();

        writeln!(output, "\n# Connection pool metrics").ok();
        writeln!(
            output,
            "# HELP turso_pool_active_connections Active connections"
        )
        .ok();
        writeln!(output, "# TYPE turso_pool_active_connections gauge").ok();
        writeln!(
            output,
            "turso_pool_active_connections {}",
            pool_stats.active_connections
        )
        .ok();

        writeln!(
            output,
            "# HELP turso_pool_idle_connections Idle connections"
        )
        .ok();
        writeln!(output, "# TYPE turso_pool_idle_connections gauge").ok();
        writeln!(
            output,
            "turso_pool_idle_connections {}",
            pool_stats.idle_connections
        )
        .ok();

        writeln!(
            output,
            "# HELP turso_pool_total_connections Total connections"
        )
        .ok();
        writeln!(output, "# TYPE turso_pool_total_connections gauge").ok();
        writeln!(
            output,
            "turso_pool_total_connections {}",
            pool_stats.total_connections
        )
        .ok();

        writeln!(
            output,
            "# HELP turso_pool_utilization Pool utilization (0-1)"
        )
        .ok();
        writeln!(output, "# TYPE turso_pool_utilization gauge").ok();
        writeln!(output, "turso_pool_utilization {:.4}", utilization).ok();

        writeln!(output, "# HELP turso_pool_wait_count Connection wait count").ok();
        writeln!(output, "# TYPE turso_pool_wait_count counter").ok();
        writeln!(output, "turso_pool_wait_count {}", pool_stats.wait_count).ok();
    }

    /// Export throughput metrics
    fn export_throughput_metrics(&self, output: &mut String) {
        let metrics = self.collector.metrics();

        writeln!(output, "\n# Throughput metrics").ok();
        writeln!(output, "# HELP turso_total_queries Total queries executed").ok();
        writeln!(output, "# TYPE turso_total_queries counter").ok();
        writeln!(output, "turso_total_queries {}", metrics.total_queries()).ok();

        writeln!(output, "# HELP turso_successful_queries Successful queries").ok();
        writeln!(output, "# TYPE turso_successful_queries counter").ok();
        writeln!(
            output,
            "turso_successful_queries {}",
            metrics.successful_queries()
        )
        .ok();

        writeln!(output, "# HELP turso_failed_queries Failed queries").ok();
        writeln!(output, "# TYPE turso_failed_queries counter").ok();
        writeln!(output, "turso_failed_queries {}", metrics.failed_queries()).ok();

        writeln!(output, "# HELP turso_bytes_read Total bytes read").ok();
        writeln!(output, "# TYPE turso_bytes_read counter").ok();
        writeln!(output, "turso_bytes_read {}", metrics.bytes_read()).ok();

        writeln!(output, "# HELP turso_bytes_written Total bytes written").ok();
        writeln!(output, "# TYPE turso_bytes_written counter").ok();
        writeln!(output, "turso_bytes_written {}", metrics.bytes_written()).ok();
    }

    /// Export error metrics
    fn export_error_metrics(&self, output: &mut String) {
        writeln!(output, "\n# Error metrics").ok();
        writeln!(
            output,
            "# HELP turso_query_success_rate Overall query success rate"
        )
        .ok();
        writeln!(output, "# TYPE turso_query_success_rate gauge").ok();
        writeln!(
            output,
            "turso_query_success_rate {:.4}",
            self.collector.query_success_rate()
        )
        .ok();
    }

    /// Get export statistics
    pub fn export_stats(&self) -> ExportStats {
        ExportStats {
            export_count: *self.export_count.read(),
            last_export: *self.last_export.read(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.collector.reset();
        info!("Prometheus exporter metrics reset");
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new(ExportConfig::default())
    }
}

/// Export statistics
#[derive(Debug, Clone)]
pub struct ExportStats {
    /// Number of times metrics have been exported
    pub export_count: u64,
    /// Timestamp of last export
    pub last_export: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_exporter_creation() {
        let exporter = PrometheusExporter::default();
        assert_eq!(exporter.export_stats().export_count, 0);
    }

    #[test]
    fn test_export_metrics_format() {
        let exporter = PrometheusExporter::default();

        // Record some metrics
        exporter.record_query(
            "episode_create",
            Duration::from_micros(100),
            true,
            Some(1024),
        );
        exporter.record_query("episode_get", Duration::from_micros(50), true, None);
        exporter.record_cache_hit();
        exporter.record_cache_miss();

        // Export and verify format
        let output = exporter.export_metrics();

        // Check for Prometheus format markers
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("turso_operation_latency_microseconds"));
        assert!(output.contains("turso_cache_hits"));
        assert!(output.contains("turso_cache_misses"));

        // Check that export count was updated
        assert_eq!(exporter.export_stats().export_count, 1);
    }

    #[test]
    fn test_operation_metrics_export() {
        let exporter = PrometheusExporter::default();

        exporter.record_query("test_op", Duration::from_micros(100), true, None);
        exporter.record_query("test_op", Duration::from_micros(200), true, None);

        let output = exporter.export_metrics();

        assert!(output.contains("turso_operation_count{operation=\"test_op\"}"));
        assert!(output.contains("turso_operation_success_rate{operation=\"test_op\"}"));
    }

    #[test]
    fn test_cache_metrics_export() {
        let exporter = PrometheusExporter::default();

        exporter.record_cache_hit();
        exporter.record_cache_hit();
        exporter.record_cache_miss();

        let output = exporter.export_metrics();

        assert!(output.contains("turso_cache_hits 2"));
        assert!(output.contains("turso_cache_misses 1"));
        assert!(output.contains("turso_cache_hit_rate"));
    }

    #[test]
    fn test_reset() {
        let exporter = PrometheusExporter::default();

        exporter.record_query("test", Duration::from_micros(100), true, None);
        exporter.export_metrics();

        assert_eq!(exporter.export_stats().export_count, 1);

        exporter.reset();

        // After reset, metrics should be cleared
        let output = exporter.export_metrics();
        assert!(!output.contains("turso_operation_count{operation=\"test\"}"));
    }
}
