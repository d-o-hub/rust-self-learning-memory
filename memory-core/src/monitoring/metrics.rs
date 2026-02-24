//! # Unified Prometheus Metrics
//!
//! This module provides unified Prometheus metrics export for both
//! memory-storage-redb (cache layer) and memory-storage-turso (durable storage).
//!
//! ## Features
//!
//! - **Cache Metrics**: Hit rate, misses, evictions from redb
//! - **Storage Latency**: Operation latency histograms from Turso
//! - **Prometheus Format**: Standard Prometheus exposition format
//! - **HTTP Endpoint**: Built-in /metrics endpoint
//!
//! ## Usage
//!
//! ```rust
//! use memory_core::monitoring::metrics::MetricsRegistry;
//!
//! let registry = MetricsRegistry::new();
//! ```

#![allow(
    clippy::uninlined_format_args,
    clippy::must_use_candidate,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::derivable_impls,
    clippy::unwrap_or_default
)]

use std::fmt::Write;
use std::sync::Arc;
use tracing::debug;

mod http_server;
mod storage_metrics;

pub use http_server::MetricsHttpServer;
pub use storage_metrics::{CacheStats, OperationLatency, RedbMetrics, TursoStorageMetrics};

/// Unified metrics registry for all storage backends
#[derive(Debug)]
pub struct MetricsRegistry {
    /// Redb cache metrics
    redb_metrics: Arc<RedbMetrics>,
    /// Turso storage metrics  
    turso_metrics: Arc<TursoStorageMetrics>,
    /// Last export timestamp
    last_export: parking_lot::RwLock<u64>,
    /// Export count
    export_count: parking_lot::RwLock<u64>,
}

impl Clone for MetricsRegistry {
    fn clone(&self) -> Self {
        Self {
            redb_metrics: self.redb_metrics.clone(),
            turso_metrics: self.turso_metrics.clone(),
            last_export: parking_lot::RwLock::new(*self.last_export.read()),
            export_count: parking_lot::RwLock::new(*self.export_count.read()),
        }
    }
}

impl MetricsRegistry {
    /// Create new metrics registry
    pub fn new() -> Self {
        Self {
            redb_metrics: Arc::new(RedbMetrics::new()),
            turso_metrics: Arc::new(TursoStorageMetrics::new()),
            last_export: parking_lot::RwLock::new(0),
            export_count: parking_lot::RwLock::new(0),
        }
    }

    /// Get redb metrics reference
    pub fn redb(&self) -> &Arc<RedbMetrics> {
        &self.redb_metrics
    }

    /// Get turso metrics reference
    pub fn turso(&self) -> &Arc<TursoStorageMetrics> {
        &self.turso_metrics
    }

    /// Export all metrics in Prometheus format
    pub fn export_metrics(&self) -> String {
        let mut output = String::with_capacity(4096);

        // Header
        writeln!(
            output,
            "# HELP memory_storage_metrics Unified memory storage metrics"
        )
        .ok();
        writeln!(output, "# TYPE memory_storage_metrics gauge").ok();

        // Export redb metrics
        self.export_redb_metrics(&mut output);

        // Export turso metrics
        self.export_turso_metrics(&mut output);

        debug!("Exported {} bytes of unified metrics", output.len());
        output
    }

    /// Export redb cache metrics
    fn export_redb_metrics(&self, output: &mut String) {
        let hits = self.redb_metrics.cache_hits();
        let misses = self.redb_metrics.cache_misses();
        let evictions = self.redb_metrics.cache_evictions();
        let expirations = self.redb_metrics.cache_expirations();
        let items = self.redb_metrics.cache_items();
        let bytes = self.redb_metrics.cache_bytes();
        let hit_rate = self.redb_metrics.cache_hit_rate();

        writeln!(output, "\n# Redb cache metrics").ok();

        writeln!(output, "# HELP redb_cache_hits_total Total cache hits").ok();
        writeln!(output, "# TYPE redb_cache_hits_total counter").ok();
        writeln!(output, "redb_cache_hits_total {}", hits).ok();

        writeln!(
            output,
            "\n# HELP redb_cache_misses_total Total cache misses"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_misses_total counter").ok();
        writeln!(output, "redb_cache_misses_total {}", misses).ok();

        writeln!(output, "\n# HELP redb_cache_hit_rate Cache hit rate (0-1)").ok();
        writeln!(output, "# TYPE redb_cache_hit_rate gauge").ok();
        writeln!(output, "redb_cache_hit_rate {:.4}", hit_rate).ok();

        writeln!(
            output,
            "\n# HELP redb_cache_evictions_total Total cache evictions"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_evictions_total counter").ok();
        writeln!(output, "redb_cache_evictions_total {}", evictions).ok();

        writeln!(
            output,
            "\n# HELP redb_cache_expirations_total Total cache expirations"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_expirations_total counter").ok();
        writeln!(output, "redb_cache_expirations_total {}", expirations).ok();

        writeln!(
            output,
            "\n# HELP redb_cache_items Current number of items in cache"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_items gauge").ok();
        writeln!(output, "redb_cache_items {}", items).ok();

        writeln!(
            output,
            "\n# HELP redb_cache_bytes Total bytes used by cache"
        )
        .ok();
        writeln!(output, "# TYPE redb_cache_bytes gauge").ok();
        writeln!(output, "redb_cache_bytes {}", bytes).ok();
    }

    /// Export turso storage metrics
    fn export_turso_metrics(&self, output: &mut String) {
        let ops = self.turso_metrics.operations();

        writeln!(output, "\n# Turso storage metrics").ok();

        writeln!(
            output,
            "# HELP turso_storage_operations_total Total storage operations"
        )
        .ok();
        writeln!(output, "# TYPE turso_storage_operations_total counter").ok();

        writeln!(
            output,
            "# HELP turso_storage_operation_duration_ms Operation latency in milliseconds"
        )
        .ok();
        writeln!(output, "# TYPE turso_storage_operation_duration_ms summary").ok();

        for (op_name, stats) in ops {
            let (p50, p95, p99) = stats.percentiles_ms();
            let count = stats.count();

            writeln!(
                output,
                "turso_storage_operations_total{{operation=\"{}\"}} {}",
                op_name, count
            )
            .ok();
            writeln!(
                output,
                "turso_storage_operation_duration_ms{{operation=\"{}\",quantile=\"0.5\"}} {}",
                op_name, p50
            )
            .ok();
            writeln!(
                output,
                "turso_storage_operation_duration_ms{{operation=\"{}\",quantile=\"0.95\"}} {}",
                op_name, p95
            )
            .ok();
            writeln!(
                output,
                "turso_storage_operation_duration_ms{{operation=\"{}\",quantile=\"0.99\"}} {}",
                op_name, p99
            )
            .ok();
        }

        // Export cache stats from turso
        let cache_stats = self.turso_metrics.cache_stats();
        writeln!(output, "\n# Turso cache metrics").ok();

        writeln!(output, "# HELP turso_cache_hits_total Total cache hits").ok();
        writeln!(output, "# TYPE turso_cache_hits_total counter").ok();
        writeln!(output, "turso_cache_hits_total {}", cache_stats.hits).ok();

        writeln!(
            output,
            "\n# HELP turso_cache_misses_total Total cache misses"
        )
        .ok();
        writeln!(output, "# TYPE turso_cache_misses_total counter").ok();
        writeln!(output, "turso_cache_misses_total {}", cache_stats.misses).ok();

        let hit_rate = cache_stats.hit_rate();
        writeln!(output, "\n# HELP turso_cache_hit_rate Cache hit rate (0-1)").ok();
        writeln!(output, "# TYPE turso_cache_hit_rate gauge").ok();
        writeln!(output, "turso_cache_hit_rate {:.4}", hit_rate).ok();
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.redb_metrics.reset();
        self.turso_metrics.reset();
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        let output = registry.export_metrics();

        assert!(output.contains("redb_cache_hits_total"));
        assert!(output.contains("turso_storage_operations_total"));
    }
}
