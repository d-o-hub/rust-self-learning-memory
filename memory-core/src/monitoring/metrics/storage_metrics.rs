use parking_lot::RwLock;

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

#[cfg(test)]
mod tests {
    use super::*;

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
