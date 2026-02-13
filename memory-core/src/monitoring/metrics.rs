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

use parking_lot::RwLock;
use std::fmt::Write;
use std::sync::Arc;
use tracing::debug;

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

// ============================================================================
// Redb Metrics Types
// ============================================================================

/// Metrics for redb cache operations (mirrors memory_storage_redb::metrics::RedbMetrics)
#[derive(Debug, Default)]
pub struct RedbMetrics {
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,
    cache_evictions: std::sync::atomic::AtomicU64,
    cache_expirations: std::sync::atomic::AtomicU64,
    cache_items: std::sync::atomic::AtomicU64,
    cache_bytes: std::sync::atomic::AtomicU64,
}

impl RedbMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    #[inline]
    pub fn record_cache_hit(&self) {
        self.cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a cache miss
    #[inline]
    pub fn record_cache_miss(&self) {
        self.cache_misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a cache eviction
    #[inline]
    pub fn record_cache_eviction(&self) {
        self.cache_evictions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a cache expiration
    #[inline]
    pub fn record_cache_expiration(&self) {
        self.cache_expirations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update cache size metrics
    #[inline]
    pub fn update_cache_size(&self, items: usize, bytes: usize) {
        self.cache_items
            .store(items as u64, std::sync::atomic::Ordering::Relaxed);
        self.cache_bytes
            .store(bytes as u64, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get cache hit count
    #[inline]
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get cache miss count
    #[inline]
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get cache eviction count
    #[inline]
    pub fn cache_evictions(&self) -> u64 {
        self.cache_evictions
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get cache expiration count
    #[inline]
    pub fn cache_expirations(&self) -> u64 {
        self.cache_expirations
            .load(std::sync::atomic::Ordering::Relaxed)
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
        self.cache_items.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get cache bytes
    #[inline]
    pub fn cache_bytes(&self) -> u64 {
        self.cache_bytes.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.cache_hits
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_misses
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_evictions
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_expirations
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_items
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_bytes
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// Turso Storage Metrics Types
// ============================================================================

/// Operation latency statistics
#[derive(Debug, Clone, Default)]
pub struct OperationLatency {
    count: u64,
    total_ms: u64,
    p50: u64,
    p95: u64,
    p99: u64,
}

impl OperationLatency {
    /// Record a latency sample (in milliseconds)
    pub fn record(&mut self, latency_ms: u64) {
        self.count += 1;
        self.total_ms += latency_ms;

        // Simple percentile tracking (not accurate but lightweight)
        // For accurate percentiles, use a proper histogram library
        if self.count == 1 {
            self.p50 = latency_ms;
            self.p95 = latency_ms;
            self.p99 = latency_ms;
        } else {
            // Update percentiles with simple moving average
            self.p50 = self.p50 * 7 / 10 + latency_ms * 3 / 10;
            self.p95 = self.p95 * 9 / 10 + latency_ms / 10;
            self.p99 = self.p99.max(latency_ms);
        }
    }

    /// Get operation count
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get average latency in ms
    pub fn avg_ms(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.total_ms / self.count
        }
    }

    /// Get percentiles (p50, p95, p99) in ms
    pub fn percentiles_ms(&self) -> (u64, u64, u64) {
        (self.p50, self.p95, self.p99)
    }
}

/// Turso storage operation metrics
#[derive(Debug)]
pub struct TursoStorageMetrics {
    /// Operation-specific latency tracking
    operations: RwLock<std::collections::HashMap<String, OperationLatency>>,
    /// Cache hits
    cache_hits: std::sync::atomic::AtomicU64,
    /// Cache misses
    cache_misses: std::sync::atomic::AtomicU64,
}

impl TursoStorageMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(std::collections::HashMap::new()),
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            cache_misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Record a storage operation with latency
    pub fn record_operation(&self, operation: &str, latency_ms: u64) {
        let mut ops = self.operations.write();
        let stats = ops.entry(operation.to_string()).or_default();
        stats.record(latency_ms);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get all operation metrics
    pub fn operations(&self) -> std::collections::HashMap<String, OperationLatency> {
        let ops = self.operations.read();
        ops.clone()
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.cache_hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.cache_misses.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.operations.write().clear();
        self.cache_hits
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_misses
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for TursoStorageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
}

impl CacheStats {
    /// Get hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

// ============================================================================
// HTTP Server for /metrics Endpoint
// ============================================================================

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// HTTP server for serving Prometheus metrics at /metrics
pub struct MetricsHttpServer {
    registry: MetricsRegistry,
    handle: Option<JoinHandle<()>>,
}

impl MetricsHttpServer {
    /// Create new HTTP server
    pub fn new(registry: MetricsRegistry) -> Self {
        Self {
            registry,
            handle: None,
        }
    }

    /// Start the HTTP server on the given address
    pub async fn start(&mut self, addr: &str) -> std::io::Result<()> {
        let addr: SocketAddr = addr
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let listener = TcpListener::bind(addr).await?;
        info!("Metrics HTTP server listening on http://{}/metrics", addr);

        let registry = self.registry.clone();

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, peer_addr)) = listener.accept().await else {
                    error!("Failed to accept connection");
                    continue;
                };
                let registry = registry.clone();
                tokio::spawn(handle_connection(stream, peer_addr, registry));
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the HTTP server
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            info!("Metrics HTTP server stopped");
        }
    }
}

impl Drop for MetricsHttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Handle a single HTTP connection
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    registry: MetricsRegistry,
) {
    if let Err(e) = handle_connection_impl(&mut stream, peer_addr, &registry).await {
        tracing::warn!("Error handling connection from {}: {}", peer_addr, e);
    }
}

async fn handle_connection_impl(
    stream: &mut tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    registry: &MetricsRegistry,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    match path {
        "/metrics" => {
            let metrics = registry.export_metrics();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                metrics.len(),
                metrics
            );
            stream.write_all(response.as_bytes()).await?;
            info!("Served metrics to {}", peer_addr);
        }
        "/health" => {
            let body = "OK";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await?;
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }
    }

    Ok(())
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

    #[test]
    fn test_redb_metrics() {
        let metrics = RedbMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.update_cache_size(10, 1000);

        assert_eq!(metrics.cache_hits(), 1);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_turso_metrics() {
        let metrics = TursoStorageMetrics::new();

        metrics.record_operation("episode_get", 25);
        metrics.record_operation("episode_get", 50);
        metrics.record_operation("episode_get", 100);
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let ops = metrics.operations();
        assert!(ops.contains_key("episode_get"));

        let cache = metrics.cache_stats();
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_operation_latency() {
        let mut latency = OperationLatency::default();

        latency.record(10);
        latency.record(20);
        latency.record(30);

        assert_eq!(latency.count(), 3);
        assert!(latency.avg_ms() > 0);

        let (p50, p95, p99) = latency.percentiles_ms();
        assert!(p50 <= p95);
        assert!(p95 <= p99);
    }
}
