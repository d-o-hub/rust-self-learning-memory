//! # System Telemetry
//!
//! Minimal viable telemetry for the memory management system to enable data-driven decisions.
//!
//! This module provides lightweight metrics collection for:
//! - Sandbox usage tracking
//! - Storage operation monitoring
//! - System health metrics
//!
//! ## Metrics Types
//!
//! - **Counters**: Monotonically increasing values (e.g., total executions)
//! - **Histograms**: Distributions of values (e.g., operation durations)
//! - **Gauges**: Point-in-time values (e.g., current cache size)
//!
//! ## Example
//!
//! ```no_run
//! use memory_core::telemetry::{Telemetry, TelemetryConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let telemetry = Telemetry::new(TelemetryConfig::default());
//!
//!     // Track sandbox execution
//!     telemetry.counter("sandbox_executions").increment();
//!
//!     // Track operation duration
//!     let timer = telemetry.histogram("operation_duration_ms").start_timer();
//!     // ... perform operation ...
//!     timer.observe(Duration::from_millis(100));
//!
//!     // Set gauge value
//!     telemetry.gauge("cache_size").set(42);
//!
//!     // Get metrics summary
//!     let summary = telemetry.get_summary().await;
//!     println!("{:#?}", summary);
//! }
//! ```

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod summary;

use counter::Counter;
use gauge::Gauge;
use histogram::Histogram;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use summary::TelemetrySummary;
use tokio::sync::RwLock;

/// Configuration for the telemetry system
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// How often to log metrics (in seconds)
    pub log_interval_secs: u64,
    /// Number of buckets for histograms (power-of-two sized)
    pub histogram_buckets: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_interval_secs: 60,
            histogram_buckets: 16,
        }
    }
}

/// Telemetry system for tracking system metrics
///
/// Provides thread-safe in-memory metrics collection with periodic logging.
#[derive(Clone)]
pub struct Telemetry {
    /// Configuration
    config: TelemetryConfig,
    /// Counter metrics
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    /// Histogram metrics
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    /// Gauge metrics
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Telemetry {
    /// Create a new telemetry instance with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(TelemetryConfig::default())
    }

    /// Create a new telemetry instance with custom configuration
    #[must_use]
    pub fn with_config(config: TelemetryConfig) -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get or create a counter metric
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the counter metric
    ///
    /// # Returns
    ///
    /// Counter handle for incrementing
    pub fn counter(&self, name: &str) -> CounterHandle {
        if !self.config.enabled {
            return CounterHandle::disabled();
        }

        CounterHandle {
            name: name.to_string(),
            counters: Arc::clone(&self.counters),
        }
    }

    /// Get or create a histogram metric
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the histogram metric
    ///
    /// # Returns
    ///
    /// Histogram handle for recording observations
    pub fn histogram(&self, name: &str) -> HistogramHandle {
        if !self.config.enabled {
            return HistogramHandle::disabled();
        }

        HistogramHandle {
            name: name.to_string(),
            histograms: Arc::clone(&self.histograms),
            buckets: self.config.histogram_buckets,
        }
    }

    /// Get or create a gauge metric
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the gauge metric
    ///
    /// # Returns
    ///
    /// Gauge handle for setting values
    pub fn gauge(&self, name: &str) -> GaugeHandle {
        if !self.config.enabled {
            return GaugeHandle::disabled();
        }

        GaugeHandle {
            name: name.to_string(),
            gauges: Arc::clone(&self.gauges),
        }
    }

    /// Get a summary of all metrics
    ///
    /// Returns aggregated statistics for all tracked metrics.
    pub async fn get_summary(&self) -> TelemetrySummary {
        let counters = self.counters.read().await;
        let histograms = self.histograms.read().await;
        let gauges = self.gauges.read().await;

        TelemetrySummary {
            counters: counters
                .iter()
                .map(|(name, counter)| (name.clone(), counter.get()))
                .collect(),
            histograms: histograms
                .iter()
                .map(|(name, histogram)| (name.clone(), histogram.get_stats()))
                .collect(),
            gauges: gauges
                .iter()
                .map(|(name, gauge)| (name.clone(), gauge.get()))
                .collect(),
        }
    }

    /// Log current metrics to tracing
    ///
    /// Outputs current metric values to the logging system.
    pub async fn log_metrics(&self) {
        let summary = self.get_summary().await;

        tracing::info!("=== Telemetry Metrics ===");

        // Log counters
        for (name, value) in &summary.counters {
            tracing::info!("  [counter] {}: {}", name, value);
        }

        // Log histograms
        for (name, stats) in &summary.histograms {
            tracing::info!(
                "  [histogram] {}: count={}, p50={:.2}, p95={:.2}, p99={:.2}",
                name,
                stats.count,
                stats.p50,
                stats.p95,
                stats.p99,
            );
        }

        // Log gauges
        for (name, value) in &summary.gauges {
            tracing::info!("  [gauge] {}: {}", name, value);
        }

        tracing::info!("========================");
    }

    /// Clear all metrics
    ///
    /// Resets all tracked metrics to initial state. Use with caution.
    pub async fn clear(&self) {
        let mut counters = self.counters.write().await;
        counters.clear();

        let mut histograms = self.histograms.write().await;
        histograms.clear();

        let mut gauges = self.gauges.write().await;
        gauges.clear();
    }
}

/// Handle for incrementing a counter metric
#[derive(Clone)]
pub struct CounterHandle {
    name: String,
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    enabled: bool,
}

impl CounterHandle {
    fn disabled() -> Self {
        Self {
            name: String::new(),
            counters: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        }
    }

    /// Increment the counter by 1
    pub fn increment(&self) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let counters = Arc::clone(&self.counters);
        tokio::spawn(async move {
            let mut counters = counters.write().await;
            let counter = counters.entry(name).or_insert_with(Counter::new);
            counter.increment();
        });
    }

    /// Increment the counter by a specific value
    pub fn increment_by(&self, value: u64) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let counters = Arc::clone(&self.counters);
        tokio::spawn(async move {
            let mut counters = counters.write().await;
            let counter = counters.entry(name).or_insert_with(Counter::new);
            counter.increment_by(value);
        });
    }

    /// Get the current counter value
    pub async fn get(&self) -> u64 {
        if !self.enabled {
            return 0;
        }

        let counters = self.counters.read().await;
        counters.get(&self.name).map(|c| c.get()).unwrap_or(0)
    }
}

/// Timer handle for histogram operations
#[derive(Clone)]
pub struct TimerHandle {
    name: String,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    start: std::time::Instant,
    enabled: bool,
}

impl TimerHandle {
    fn disabled() -> Self {
        Self {
            name: String::new(),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            start: std::time::Instant::now(),
            enabled: false,
        }
    }

    /// Record the elapsed duration to the histogram
    pub fn observe(self, duration: Duration) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let histograms = Arc::clone(&self.histograms);
        tokio::spawn(async move {
            let mut histograms = histograms.write().await;
            let histogram = histograms.entry(name).or_insert_with(|| Histogram::new(16));
            histogram.observe(duration);
        });
    }
}

/// Handle for recording histogram observations
#[derive(Clone)]
pub struct HistogramHandle {
    name: String,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    buckets: usize,
    enabled: bool,
}

impl HistogramHandle {
    fn disabled() -> Self {
        Self {
            name: String::new(),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            buckets: 16,
            enabled: false,
        }
    }

    /// Start a timer for this histogram
    #[must_use]
    pub fn start_timer(&self) -> TimerHandle {
        TimerHandle {
            name: self.name.clone(),
            histograms: Arc::clone(&self.histograms),
            start: std::time::Instant::now(),
            enabled: self.enabled,
        }
    }

    /// Record a duration observation
    pub fn observe(&self, duration: Duration) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let histograms = Arc::clone(&self.histograms);
        let buckets = self.buckets;
        tokio::spawn(async move {
            let mut histograms = histograms.write().await;
            let histogram = histograms.entry(name).or_insert_with(|| Histogram::new(buckets));
            histogram.observe(duration);
        });
    }

    /// Record a value in milliseconds
    pub fn observe_ms(&self, millis: u64) {
        self.observe(Duration::from_millis(millis));
    }

    /// Get histogram statistics
    pub async fn get_stats(&self) -> Option<HistogramStats> {
        if !self.enabled {
            return None;
        }

        let histograms = self.histograms.read().await;
        histograms.get(&self.name).map(|h| h.get_stats())
    }
}

/// Handle for setting gauge values
#[derive(Clone)]
pub struct GaugeHandle {
    name: String,
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
    enabled: bool,
}

impl GaugeHandle {
    fn disabled() -> Self {
        Self {
            name: String::new(),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: f64) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let gauges = Arc::clone(&self.gauges);
        tokio::spawn(async move {
            let mut gauges = gauges.write().await;
            let gauge = gauges.entry(name).or_insert_with(Gauge::new);
            gauge.set(value);
        });
    }

    /// Increment the gauge by 1
    pub fn increment(&self) {
        self.increment_by(1.0);
    }

    /// Decrement the gauge by 1
    pub fn decrement(&self) {
        self.increment_by(-1.0);
    }

    /// Increment the gauge by a specific value
    pub fn increment_by(&self, value: f64) {
        if !self.enabled {
            return;
        }

        let name = self.name.clone();
        let gauges = Arc::clone(&self.gauges);
        tokio::spawn(async move {
            let mut gauges = gauges.write().await;
            let gauge = gauges.entry(name).or_insert_with(Gauge::new);
            gauge.add(value);
        });
    }

    /// Get the current gauge value
    pub async fn get(&self) -> f64 {
        if !self.enabled {
            return 0.0;
        }

        let gauges = self.gauges.read().await;
        gauges.get(&self.name).map(|g| g.get()).unwrap_or(0.0)
    }
}

/// Statistics for a histogram metric
#[derive(Debug, Clone)]
pub struct HistogramStats {
    /// Number of observations
    pub count: u64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Average value
    pub avg: f64,
    /// 50th percentile
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter() {
        let telemetry = Telemetry::new();
        let counter = telemetry.counter("test_counter");

        counter.increment();
        counter.increment_by(5);

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.get().await, 6);
    }

    #[tokio::test]
    async fn test_histogram() {
        let telemetry = Telemetry::new();
        let histogram = telemetry.histogram("test_histogram");

        histogram.observe_ms(10);
        histogram.observe_ms(20);
        histogram.observe_ms(30);

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = histogram.get_stats().await.unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[tokio::test]
    async fn test_histogram_timer() {
        let telemetry = Telemetry::new();
        let histogram = telemetry.histogram("test_timer");

        let timer = histogram.start_timer();
        tokio::time::sleep(Duration::from_millis(50)).await;
        timer.observe(timer.start.elapsed());

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = histogram.get_stats().await.unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.avg >= 45.0); // Allow some variance
    }

    #[tokio::test]
    async fn test_gauge() {
        let telemetry = Telemetry::new();
        let gauge = telemetry.gauge("test_gauge");

        gauge.set(42.0);
        gauge.increment();
        gauge.decrement();

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(gauge.get().await, 42.0);
    }

    #[tokio::test]
    async fn test_summary() {
        let telemetry = Telemetry::new();

        telemetry.counter("c1").increment();
        telemetry.gauge("g1").set(1.0);
        telemetry.histogram("h1").observe_ms(100);

        tokio::time::sleep(Duration::from_millis(100)).await;

        let summary = telemetry.get_summary().await;
        assert_eq!(summary.counters.get("c1"), Some(&1u64));
        assert_eq!(summary.gauges.get("g1"), Some(&1.0));
        assert!(summary.histograms.contains_key("h1"));
    }

    #[tokio::test]
    async fn test_disabled_telemetry() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };
        let telemetry = Telemetry::with_config(config);

        telemetry.counter("test").increment();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let summary = telemetry.get_summary().await;
        assert!(!summary.counters.contains_key("test"));
    }

    #[tokio::test]
    async fn test_clear() {
        let telemetry = Telemetry::new();

        telemetry.counter("test").increment();

        tokio::time::sleep(Duration::from_millis(100)).await;

        telemetry.clear().await;

        let summary = telemetry.get_summary().await;
        assert!(!summary.counters.contains_key("test"));
    }
}
