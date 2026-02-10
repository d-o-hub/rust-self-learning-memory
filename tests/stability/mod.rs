//! Stability test framework for 24-hour validation
//!
//! This module provides the foundation for long-running stability tests including:
//! - Test harness for long-running operations
//! - Metric collection and analysis
//! - Memory leak detection
//! - Performance monitoring
//! - Graceful degradation handling
//!
//! # Usage
//!
//! ```rust
//! use stability::{StabilityTest, TestConfig, TestMetrics};
//!
//! #[tokio::test]
//! #[ignore = "Run manually for 24h soak test"]
//! async fn test_24h_stability() {
//!     let config = TestConfig::default();
//!     let mut test = StabilityTest::new(config);
//!     
//!     test.run(|metrics| async {
//!         // Your test operations here
//!         metrics.record_operation(true, Duration::from_millis(10));
//!     }).await;
//! }
//! ```

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::interval;

/// Default test duration for 24-hour soak test
pub const DEFAULT_24H_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

/// Default test duration for CI/development (1 minute)
pub const DEFAULT_CI_DURATION: Duration = Duration::from_secs(60);

/// Default interval for metric snapshots
pub const DEFAULT_SNAPSHOT_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Default interval for memory checks
pub const DEFAULT_MEMORY_CHECK_INTERVAL: Duration = Duration::from_secs(60); // 1 minute

/// Maximum latency samples to keep in memory
const MAX_LATENCY_SAMPLES: usize = 100_000;

/// Configuration for stability tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Test duration
    pub duration: Duration,
    /// Interval between metric snapshots
    pub snapshot_interval: Duration,
    /// Interval between memory checks
    pub memory_check_interval: Duration,
    /// Number of concurrent workers
    pub worker_count: usize,
    /// Memory leak threshold (percentage growth)
    pub memory_leak_threshold: f64,
    /// Performance degradation threshold (percentage increase in latency)
    pub performance_degradation_threshold: f64,
    /// Minimum acceptable success rate (0.0 - 1.0)
    pub min_success_rate: f64,
    /// Enable memory monitoring
    pub monitor_memory: bool,
    /// Enable performance monitoring
    pub monitor_performance: bool,
    /// Print progress updates
    pub print_progress: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            duration: DEFAULT_CI_DURATION,
            snapshot_interval: DEFAULT_SNAPSHOT_INTERVAL,
            memory_check_interval: DEFAULT_MEMORY_CHECK_INTERVAL,
            worker_count: 4,
            memory_leak_threshold: 50.0, // 50% growth threshold
            performance_degradation_threshold: 100.0, // 100% latency increase
            min_success_rate: 0.95,      // 95% success rate required
            monitor_memory: true,
            monitor_performance: true,
            print_progress: true,
        }
    }
}

impl TestConfig {
    /// Create configuration for full 24-hour soak test
    pub fn for_24h_test() -> Self {
        Self {
            duration: DEFAULT_24H_DURATION,
            snapshot_interval: Duration::from_secs(300), // 5 minutes
            memory_check_interval: Duration::from_secs(60), // 1 minute
            worker_count: 8,
            ..Default::default()
        }
    }

    /// Create configuration for quick CI test
    pub fn for_ci_test() -> Self {
        Self {
            duration: Duration::from_secs(60),             // 1 minute
            snapshot_interval: Duration::from_secs(10),    // 10 seconds
            memory_check_interval: Duration::from_secs(5), // 5 seconds
            worker_count: 4,
            ..Default::default()
        }
    }
}

/// Individual operation result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OperationResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Operation latency
    pub latency: Duration,
    /// Timestamp
    pub timestamp: Instant,
    /// Optional error message
    pub error: Option<String>,
}

/// Memory usage sample
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemorySample {
    /// Resident set size in MB
    pub rss_mb: f64,
    /// Virtual memory size in MB
    pub vms_mb: f64,
    /// Timestamp
    pub timestamp: Instant,
}

/// Performance snapshot at a point in time
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Total operations at snapshot time
    pub total_operations: usize,
    /// Successful operations
    pub successful_operations: usize,
    /// Failed operations
    pub failed_operations: usize,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// P50 latency in milliseconds
    pub p50_latency_ms: f64,
    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,
    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,
    /// Current throughput (ops/sec)
    pub throughput: f64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Memory usage at snapshot time
    pub memory_mb: f64,
    /// Elapsed time since test start
    pub elapsed: Duration,
}

/// Comprehensive test metrics
pub struct TestMetrics {
    /// Total operations attempted
    total_operations: AtomicUsize,
    /// Successful operations
    successful_operations: AtomicUsize,
    /// Failed operations
    failed_operations: AtomicUsize,
    /// Total latency in microseconds
    total_latency_us: AtomicU64,
    /// Latency samples for percentile calculation
    latency_samples: Arc<Mutex<VecDeque<u64>>>,
    /// Memory usage samples
    memory_samples: Arc<Mutex<VecDeque<MemorySample>>>,
    /// Performance snapshots
    snapshots: Arc<Mutex<Vec<PerformanceSnapshot>>>,
    /// Test start time
    start_time: Instant,
    /// Test running flag
    running: AtomicBool,
}

impl TestMetrics {
    /// Create new test metrics
    pub fn new() -> Self {
        Self {
            total_operations: AtomicUsize::new(0),
            successful_operations: AtomicUsize::new(0),
            failed_operations: AtomicUsize::new(0),
            total_latency_us: AtomicU64::new(0),
            latency_samples: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LATENCY_SAMPLES))),
            memory_samples: Arc::new(Mutex::new(VecDeque::new())),
            snapshots: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            running: AtomicBool::new(true),
        }
    }

    /// Check if test is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the test
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Record a successful operation
    pub fn record_success(&self, latency: Duration) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.successful_operations.fetch_add(1, Ordering::Relaxed);

        let latency_us = latency.as_micros() as u64;
        self.total_latency_us
            .fetch_add(latency_us, Ordering::Relaxed);

        // Store sample
        let samples = self.latency_samples.clone();
        tokio::spawn(async move {
            let mut samples = samples.lock().await;
            samples.push_back(latency_us);
            // Keep only last N samples
            while samples.len() > MAX_LATENCY_SAMPLES {
                samples.pop_front();
            }
        });
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.failed_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an operation result
    pub fn record_operation(&self, success: bool, latency: Duration) {
        if success {
            self.record_success(latency);
        } else {
            self.record_failure();
        }
    }

    /// Record memory usage
    pub async fn record_memory(&self, rss_mb: f64, vms_mb: f64) {
        let mut samples = self.memory_samples.lock().await;
        samples.push_back(MemorySample {
            rss_mb,
            vms_mb,
            timestamp: Instant::now(),
        });
    }

    /// Save a performance snapshot
    pub async fn save_snapshot(&self) {
        let snapshot = self.capture_snapshot().await;
        let mut snapshots = self.snapshots.lock().await;
        snapshots.push(snapshot);
    }

    /// Capture current performance snapshot
    pub async fn capture_snapshot(&self) -> PerformanceSnapshot {
        let total = self.total_operations.load(Ordering::Relaxed);
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);
        let total_latency = self.total_latency_us.load(Ordering::Relaxed);

        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        let avg_latency_ms = if successful > 0 {
            (total_latency as f64 / successful as f64) / 1000.0
        } else {
            0.0
        };

        let throughput = if elapsed_secs > 0.0 {
            total as f64 / elapsed_secs
        } else {
            0.0
        };

        let success_rate = if total > 0 {
            successful as f64 / total as f64
        } else {
            0.0
        };

        // Calculate percentiles
        let samples = self.latency_samples.lock().await;
        let (p50, p95, p99) = if samples.len() >= 2 {
            let mut sorted: Vec<u64> = samples.iter().copied().collect();
            drop(samples); // Release lock before sorting
            sorted.sort_unstable();

            let p50_idx = ((sorted.len() as f64 * 0.50) as usize).min(sorted.len() - 1);
            let p95_idx = ((sorted.len() as f64 * 0.95) as usize).min(sorted.len() - 1);
            let p99_idx = ((sorted.len() as f64 * 0.99) as usize).min(sorted.len() - 1);

            (
                sorted[p50_idx] as f64 / 1000.0,
                sorted[p95_idx] as f64 / 1000.0,
                sorted[p99_idx] as f64 / 1000.0,
            )
        } else {
            (avg_latency_ms, avg_latency_ms, avg_latency_ms)
        };

        // Get current memory usage
        let memory_samples = self.memory_samples.lock().await;
        let memory_mb = memory_samples.back().map(|s| s.rss_mb).unwrap_or(0.0);

        PerformanceSnapshot {
            total_operations: total,
            successful_operations: successful,
            failed_operations: failed,
            avg_latency_ms,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            throughput,
            success_rate,
            memory_mb,
            elapsed,
        }
    }

    /// Get total operations
    pub fn total_operations(&self) -> usize {
        self.total_operations.load(Ordering::Relaxed)
    }

    /// Get successful operations
    pub fn successful_operations(&self) -> usize {
        self.successful_operations.load(Ordering::Relaxed)
    }

    /// Get failed operations
    pub fn failed_operations(&self) -> usize {
        self.failed_operations.load(Ordering::Relaxed)
    }

    /// Get test duration
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations();
        let successful = self.successful_operations();
        if total > 0 {
            successful as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get throughput (ops/sec)
    #[allow(dead_code)]
    pub fn throughput(&self) -> f64 {
        let total = self.total_operations();
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Check for memory leak
    pub async fn check_memory_leak(&self, threshold_percent: f64) -> Option<String> {
        let samples = self.memory_samples.lock().await;
        if samples.len() < 10 {
            return None;
        }

        let first = samples.front().unwrap();
        let last = samples.back().unwrap();

        if first.rss_mb > 0.0 {
            let growth_percent = ((last.rss_mb - first.rss_mb) / first.rss_mb) * 100.0;
            if growth_percent > threshold_percent {
                return Some(format!(
                    "Memory leak detected: {:.1}% growth ({} MB -> {} MB)",
                    growth_percent, first.rss_mb, last.rss_mb
                ));
            }
        }

        None
    }

    /// Check for performance degradation
    pub async fn check_performance_degradation(&self, threshold_percent: f64) -> Option<String> {
        let snapshots = self.snapshots.lock().await;
        if snapshots.len() < 3 {
            return None;
        }

        let first = snapshots.first().unwrap();
        let last = snapshots.last().unwrap();

        if first.avg_latency_ms > 0.0 {
            let degradation =
                ((last.avg_latency_ms - first.avg_latency_ms) / first.avg_latency_ms) * 100.0;
            if degradation > threshold_percent {
                return Some(format!(
                    "Performance degradation detected: {:.1}% latency increase ({:.2} ms -> {:.2} ms)",
                    degradation, first.avg_latency_ms, last.avg_latency_ms
                ));
            }
        }

        None
    }

    /// Print summary report
    pub async fn print_summary(&self, test_name: &str) {
        let snapshot = self.capture_snapshot().await;

        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!(
            "║           {} Stability Test Summary           ║",
            test_name
        );
        println!("╚════════════════════════════════════════════════════════════╝");
        println!("  Duration:          {:?}", snapshot.elapsed);
        println!("  Total Operations:  {}", snapshot.total_operations);
        println!("  Successful:        {}", snapshot.successful_operations);
        println!("  Failed:            {}", snapshot.failed_operations);
        println!("  Success Rate:      {:.2}%", snapshot.success_rate * 100.0);
        println!("  Throughput:        {:.2} ops/sec", snapshot.throughput);
        println!();
        println!("  Latency Metrics:");
        println!("    Average:         {:.2} ms", snapshot.avg_latency_ms);
        println!("    P50:             {:.2} ms", snapshot.p50_latency_ms);
        println!("    P95:             {:.2} ms", snapshot.p95_latency_ms);
        println!("    P99:             {:.2} ms", snapshot.p99_latency_ms);
        println!();
        println!("  Memory Usage:      {:.1} MB", snapshot.memory_mb);
        println!("══════════════════════════════════════════════════════════════\n");
    }
}

impl Default for TestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Stability test harness
pub struct StabilityTest {
    /// Test configuration
    config: TestConfig,
    /// Test metrics
    metrics: Arc<TestMetrics>,
}

impl StabilityTest {
    /// Create a new stability test with the given configuration
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(TestMetrics::new()),
        }
    }

    /// Get reference to metrics
    #[allow(dead_code)]
    pub fn metrics(&self) -> Arc<TestMetrics> {
        self.metrics.clone()
    }

    /// Run the stability test with the given operation function
    #[allow(clippy::excessive_nesting)]
    pub async fn run<F, Fut>(&self, operation: F)
    where
        F: Fn(Arc<TestMetrics>) -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        println!("\n=== Starting Stability Test ===");
        println!("Duration: {:?}", self.config.duration);
        println!("Workers: {}", self.config.worker_count);
        println!();

        // Start memory monitoring
        let memory_handle = if self.config.monitor_memory {
            let metrics = self.metrics.clone();
            let check_interval = self.config.memory_check_interval;
            Some(tokio::spawn(async move {
                let mut interval = interval(check_interval);
                loop {
                    interval.tick().await;

                    if !metrics.is_running() {
                        break;
                    }

                    let (rss, vms) = get_memory_usage();
                    metrics.record_memory(rss, vms).await;
                }
            }))
        } else {
            None
        };

        // Start snapshot monitoring
        let snapshot_handle = if self.config.monitor_performance {
            let metrics = self.metrics.clone();
            let snapshot_interval = self.config.snapshot_interval;
            let print_progress = self.config.print_progress;
            Some(tokio::spawn(async move {
                let mut interval = interval(snapshot_interval);
                let mut snapshot_count = 0;

                loop {
                    interval.tick().await;

                    if !metrics.is_running() {
                        break;
                    }

                    metrics.save_snapshot().await;
                    snapshot_count += 1;

                    if print_progress {
                        let snapshot = metrics.capture_snapshot().await;
                        println!(
                            "[Snapshot #{}] Ops: {}, Success: {:.1}%, Throughput: {:.1} ops/sec, Memory: {:.1} MB",
                            snapshot_count,
                            snapshot.total_operations,
                            snapshot.success_rate * 100.0,
                            snapshot.throughput,
                            snapshot.memory_mb
                        );
                    }
                }
            }))
        } else {
            None
        };

        // Start worker tasks
        let mut worker_handles = vec![];
        let duration = self.config.duration;
        for _ in 0..self.config.worker_count {
            let metrics = self.metrics.clone();

            let handle = tokio::spawn({
                let operation = operation.clone();
                async move {
                    while metrics.is_running() {
                        // Check duration
                        if metrics.elapsed() >= duration {
                            break;
                        }

                        operation(metrics.clone()).await;
                    }
                }
            });

            worker_handles.push(handle);
        }

        // Let the test run
        tokio::time::sleep(self.config.duration).await;

        // Stop the test
        self.metrics.stop();

        // Wait for all tasks to complete
        for handle in worker_handles {
            handle.await.ok();
        }

        if let Some(handle) = memory_handle {
            handle.await.ok();
        }
        if let Some(handle) = snapshot_handle {
            handle.await.ok();
        }

        // Print final summary
        self.metrics.print_summary("Stability").await;

        // Validate results
        self.validate().await;
    }

    /// Validate test results against criteria
    pub async fn validate(&self) {
        let mut errors = vec![];

        // Check success rate
        let success_rate = self.metrics.success_rate();
        if success_rate < self.config.min_success_rate {
            errors.push(format!(
                "Success rate {:.2}% below minimum {:.2}%",
                success_rate * 100.0,
                self.config.min_success_rate * 100.0
            ));
        }

        // Check for memory leak
        if self.config.monitor_memory {
            if let Some(warning) = self
                .metrics
                .check_memory_leak(self.config.memory_leak_threshold)
                .await
            {
                errors.push(warning);
            }
        }

        // Check for performance degradation
        if self.config.monitor_performance {
            if let Some(warning) = self
                .metrics
                .check_performance_degradation(self.config.performance_degradation_threshold)
                .await
            {
                errors.push(warning);
            }
        }

        if errors.is_empty() {
            println!("✅ All validation criteria passed!");
        } else {
            println!("\n❌ Validation failed:");
            for error in &errors {
                println!("  - {}", error);
            }
            panic!("Stability test validation failed");
        }
    }
}

/// Get current memory usage in MB
///
/// Returns (RSS, VMS) memory usage in megabytes
#[allow(clippy::excessive_nesting)]
pub fn get_memory_usage() -> (f64, f64) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
            let mut rss_kb = 0.0;
            let mut vms_kb = 0.0;

            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        rss_kb = parts[1].parse::<f64>().unwrap_or(0.0);
                    }
                } else if line.starts_with("VmSize:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        vms_kb = parts[1].parse::<f64>().unwrap_or(0.0);
                    }
                }
            }

            return (rss_kb / 1024.0, vms_kb / 1024.0);
        }
    }

    // Fallback: return zeros if we can't determine memory usage
    (0.0, 0.0)
}

/// Create a 24-hour stability test
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::test]
/// #[ignore = "Run manually for 24h soak test"]
/// async fn test_24h_stability() {
///     let test = stability::create_24h_test();
///     
///     test.run(|metrics| async move {
///         // Your test operations here
///         let start = Instant::now();
///         // ... perform operation ...
///         metrics.record_operation(true, start.elapsed());
///     }).await;
/// }
/// ```
pub fn create_24h_test() -> StabilityTest {
    StabilityTest::new(TestConfig::for_24h_test())
}

/// Create a quick CI stability test
pub fn create_ci_test() -> StabilityTest {
    StabilityTest::new(TestConfig::for_ci_test())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        let metrics = TestMetrics::new();

        metrics.record_success(Duration::from_millis(10));
        metrics.record_success(Duration::from_millis(20));
        metrics.record_failure();

        assert_eq!(metrics.total_operations(), 3);
        assert_eq!(metrics.successful_operations(), 2);
        assert_eq!(metrics.failed_operations(), 1);
        assert!((metrics.success_rate() - 0.6667).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_memory_leak_detection() {
        let metrics = TestMetrics::new();

        // Record initial memory
        metrics.record_memory(100.0, 200.0).await;

        // Record growth beyond threshold
        metrics.record_memory(160.0, 300.0).await;

        // Check should detect leak (>50% growth)
        let leak = metrics.check_memory_leak(50.0).await;
        assert!(leak.is_some());
        assert!(leak.unwrap().contains("60%"));
    }

    #[tokio::test]
    async fn test_snapshot_capture() {
        let metrics = TestMetrics::new();

        // Record some operations
        metrics.record_success(Duration::from_millis(10));
        metrics.record_success(Duration::from_millis(20));
        metrics.record_success(Duration::from_millis(30));

        // Record memory
        metrics.record_memory(100.0, 200.0).await;

        // Capture snapshot
        let snapshot = metrics.capture_snapshot().await;

        assert_eq!(snapshot.total_operations, 3);
        assert_eq!(snapshot.successful_operations, 3);
        assert_eq!(snapshot.failed_operations, 0);
        assert!((snapshot.success_rate - 1.0).abs() < 0.01);
        assert!(snapshot.avg_latency_ms > 0.0);
        assert!(snapshot.p50_latency_ms > 0.0);
    }
}
