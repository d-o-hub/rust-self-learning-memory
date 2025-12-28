//! Performance metrics for embedding providers

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Performance metrics for an embedding provider
#[derive(Debug, Default)]
pub struct ProviderMetrics {
    // Request counts
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    retried_requests: AtomicU64,

    // Processing metrics
    total_items_embedded: AtomicU64,
    total_tokens_used: AtomicU64,

    // Latency tracking (stored as sum for average calculation)
    total_latency_ms: AtomicU64,

    // Circuit breaker metrics
    circuit_breaker_opens: AtomicU64,
    circuit_breaker_closes: AtomicU64,

    // Compression metrics
    bytes_sent_uncompressed: AtomicU64,
    bytes_sent_compressed: AtomicU64,
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub retried_requests: u64,
    pub total_items_embedded: u64,
    pub total_tokens_used: u64,
    pub average_latency_ms: u64,
    pub circuit_breaker_opens: u64,
    pub circuit_breaker_closes: u64,
    pub bytes_sent_uncompressed: u64,
    pub bytes_sent_compressed: u64,
}

impl ProviderMetrics {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request attempt
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_success(&self, items: u64, tokens: u64, latency_ms: u64) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_items_embedded
            .fetch_add(items, Ordering::Relaxed);
        self.total_tokens_used.fetch_add(tokens, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.retried_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record circuit breaker opening
    pub fn record_circuit_breaker_open(&self) {
        self.circuit_breaker_opens.fetch_add(1, Ordering::Relaxed);
    }

    /// Record circuit breaker closing
    pub fn record_circuit_breaker_close(&self) {
        self.circuit_breaker_closes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record compression metrics
    pub fn record_compression(&self, uncompressed: u64, compressed: u64) {
        self.bytes_sent_uncompressed
            .fetch_add(uncompressed, Ordering::Relaxed);
        self.bytes_sent_compressed
            .fetch_add(compressed, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let total_latency_ms = self.total_latency_ms.load(Ordering::Relaxed);

        let average_latency_ms = if successful_requests > 0 {
            total_latency_ms / successful_requests
        } else {
            0
        };

        MetricsSnapshot {
            total_requests,
            successful_requests,
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            retried_requests: self.retried_requests.load(Ordering::Relaxed),
            total_items_embedded: self.total_items_embedded.load(Ordering::Relaxed),
            total_tokens_used: self.total_tokens_used.load(Ordering::Relaxed),
            average_latency_ms,
            circuit_breaker_opens: self.circuit_breaker_opens.load(Ordering::Relaxed),
            circuit_breaker_closes: self.circuit_breaker_closes.load(Ordering::Relaxed),
            bytes_sent_uncompressed: self.bytes_sent_uncompressed.load(Ordering::Relaxed),
            bytes_sent_compressed: self.bytes_sent_compressed.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.retried_requests.store(0, Ordering::Relaxed);
        self.total_items_embedded.store(0, Ordering::Relaxed);
        self.total_tokens_used.store(0, Ordering::Relaxed);
        self.total_latency_ms.store(0, Ordering::Relaxed);
        self.circuit_breaker_opens.store(0, Ordering::Relaxed);
        self.circuit_breaker_closes.store(0, Ordering::Relaxed);
        self.bytes_sent_uncompressed.store(0, Ordering::Relaxed);
        self.bytes_sent_compressed.store(0, Ordering::Relaxed);
    }
}

impl MetricsSnapshot {
    /// Calculate success rate as a percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate failure rate as a percentage
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.failed_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate retry rate as a percentage
    #[must_use]
    pub fn retry_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.retried_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate compression ratio as a percentage
    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        if self.bytes_sent_uncompressed == 0 {
            return 0.0;
        }
        (self.bytes_sent_compressed as f64 / self.bytes_sent_uncompressed as f64) * 100.0
    }

    /// Calculate bandwidth saved by compression
    #[must_use]
    pub fn bytes_saved(&self) -> u64 {
        self.bytes_sent_uncompressed
            .saturating_sub(self.bytes_sent_compressed)
    }

    /// Estimate cost based on token usage (`OpenAI` pricing)
    ///
    /// # Arguments
    /// * `cost_per_million_tokens` - Cost per 1M tokens (e.g., 0.02 for text-embedding-3-small)
    #[must_use]
    pub fn estimated_cost(&self, cost_per_million_tokens: f64) -> f64 {
        (self.total_tokens_used as f64 / 1_000_000.0) * cost_per_million_tokens
    }

    /// Calculate average items per request
    #[must_use]
    pub fn average_batch_size(&self) -> f64 {
        if self.successful_requests == 0 {
            return 0.0;
        }
        self.total_items_embedded as f64 / self.successful_requests as f64
    }

    /// Format metrics as a human-readable string
    #[must_use]
    pub fn format(&self) -> String {
        format!(
            r"Provider Metrics:
  Requests:        {} total ({} success, {} failed)
  Success Rate:    {:.2}%
  Retry Rate:      {:.2}%
  Items Embedded:  {}
  Tokens Used:     {}
  Avg Latency:     {}ms
  Avg Batch Size:  {:.1} items
  Circuit Breaker: {} opens, {} closes
  Compression:     {:.1}% ratio ({} bytes saved)",
            self.total_requests,
            self.successful_requests,
            self.failed_requests,
            self.success_rate(),
            self.retry_rate(),
            self.total_items_embedded,
            self.total_tokens_used,
            self.average_latency_ms,
            self.average_batch_size(),
            self.circuit_breaker_opens,
            self.circuit_breaker_closes,
            self.compression_ratio(),
            self.bytes_saved()
        )
    }
}

/// Helper to measure latency
pub struct LatencyTimer {
    start: Instant,
}

impl LatencyTimer {
    /// Start a new latency timer
    #[must_use]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = ProviderMetrics::new();

        metrics.record_request();
        metrics.record_success(10, 1000, 100);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.successful_requests, 1);
        assert_eq!(snapshot.total_items_embedded, 10);
        assert_eq!(snapshot.total_tokens_used, 1000);
        assert_eq!(snapshot.average_latency_ms, 100);
    }

    #[test]
    fn test_success_rate() {
        let metrics = ProviderMetrics::new();

        metrics.record_request();
        metrics.record_success(1, 100, 50);
        metrics.record_request();
        metrics.record_success(1, 100, 50);
        metrics.record_request();
        metrics.record_failure();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.success_rate(), 66.666_666_666_666_66);
        assert_eq!(snapshot.failure_rate(), 33.333_333_333_333_33);
    }

    #[test]
    fn test_compression_metrics() {
        let metrics = ProviderMetrics::new();

        metrics.record_compression(1000, 300);
        metrics.record_compression(2000, 600);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.bytes_sent_uncompressed, 3000);
        assert_eq!(snapshot.bytes_sent_compressed, 900);
        assert_eq!(snapshot.compression_ratio(), 30.0);
        assert_eq!(snapshot.bytes_saved(), 2100);
    }

    #[test]
    fn test_estimated_cost() {
        let metrics = ProviderMetrics::new();

        metrics.record_request();
        metrics.record_success(100, 10_000, 100);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.estimated_cost(0.02), 0.0002); // $0.02 per 1M tokens
    }

    #[test]
    fn test_average_batch_size() {
        let metrics = ProviderMetrics::new();

        metrics.record_request();
        metrics.record_success(100, 1000, 50);
        metrics.record_request();
        metrics.record_success(200, 2000, 60);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.average_batch_size(), 150.0);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = ProviderMetrics::new();

        metrics.record_request();
        metrics.record_success(10, 1000, 100);
        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.successful_requests, 0);
    }

    #[test]
    fn test_latency_timer() {
        let timer = LatencyTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
