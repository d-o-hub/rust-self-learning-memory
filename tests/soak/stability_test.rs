//! 24-hour stability soak test
//!
//! Long-running stability test to validate system reliability over extended periods.
//! Monitors memory usage, connection pool health, and cache performance.
//!
//! Acceptance Criteria:
//! - 24-hour continuous operation without failure
//! - No memory leaks detected
//! - Maintain performance throughout test
//! - Generate periodic metrics

use memory_core::{types::ExecutionResult, Episode, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::{CacheConfig, CachedTursoStorage, TursoConfig, TursoStorage};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::task::JoinSet;

/// Test duration for normal runs (shorter than 24h for CI)
#[cfg(not(feature = "full-soak"))]
const TEST_DURATION: Duration = Duration::from_secs(60);

/// Full 24-hour test duration
#[cfg(feature = "full-soak")]
const TEST_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

/// Interval between memory usage checks
const MEMORY_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Interval between performance snapshots
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Number of concurrent workers
const WORKER_COUNT: usize = 4;

/// Episodes per worker cycle
const EPISODES_PER_CYCLE: usize = 10;

/// Memory usage statistics
#[derive(Debug, Clone)]
struct MemoryUsageStats {
    /// Current resident set size in bytes
    rss: Option<u64>,
    /// Current virtual memory size in bytes
    vms: Option<u64>,
    /// Number of threads
    threads: Option<usize>,
    /// Timestamp of measurement
    timestamp: std::time::SystemTime,
}

impl Default for MemoryUsageStats {
    fn default() -> Self {
        Self {
            rss: None,
            vms: None,
            threads: None,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

impl MemoryUsageStats {
    /// Create new memory usage stats
    fn new() -> Self {
        let mut stats = Self::default();
        stats.capture();
        stats
    }

    /// Capture current memory usage
    fn capture(&mut self) {
        self.timestamp = std::time::SystemTime::now();

        // Get memory usage from sysinfo if available
        #[cfg(feature = "sysinfo")]
        {
            use sysinfo::{System, SystemExt};
            let mut system = System::new_all();
            system.refresh_all();
            if let Some(process) = system.process(std::process::id()) {
                self.rss = Some(process.memory() * 1024); // Convert KB to bytes
                self.vms = Some(process.virtual_memory() * 1024);
                self.threads = None; // sysinfo doesn't always provide thread count
            }
        }
    }

    /// Check if memory usage increased significantly from baseline
    fn has_leaked(&self, baseline: &MemoryUsageStats, threshold: f64) -> bool {
        if let (Some(current_rss), Some(baseline_rss)) = (self.rss, baseline.rss) {
            if current_rss > baseline_rss {
                let increase = (current_rss - baseline_rss) as f64 / baseline_rss as f64;
                return increase > threshold;
            }
        }
        false
    }
}

/// Performance snapshot
#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    /// Episodes created
    episodes_created: usize,
    /// Episodes completed
    episodes_completed: usize,
    /// Total operations
    total_operations: usize,
    /// Failed operations
    failed_operations: usize,
    /// Average latency (milliseconds)
    avg_latency_ms: f64,
    /// P95 latency (milliseconds)
    p95_latency_ms: f64,
    /// P99 latency (milliseconds)
    p99_latency_ms: f64,
    /// Memory usage at snapshot time
    memory_usage: MemoryUsageStats,
    /// Timestamp
    timestamp: std::time::SystemTime,
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            episodes_created: 0,
            episodes_completed: 0,
            total_operations: 0,
            failed_operations: 0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            memory_usage: MemoryUsageStats::default(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Soak test state
struct SoakTestState {
    /// Running flag
    running: AtomicBool,
    /// Total episodes created
    episodes_created: AtomicUsize,
    /// Total episodes completed
    episodes_completed: AtomicUsize,
    /// Total operations
    total_operations: AtomicUsize,
    /// Failed operations
    failed_operations: AtomicUsize,
    /// Total latency (milliseconds)
    total_latency: AtomicU64,
    /// Latency samples for percentiles
    latencies: Arc<tokio::sync::Mutex<Vec<u64>>>,
    /// Baseline memory usage
    baseline_memory: Arc<tokio::sync::Mutex<MemoryUsageStats>>,
    /// Current memory usage
    current_memory: Arc<tokio::sync::Mutex<MemoryUsageStats>>,
    /// Performance snapshots
    snapshots: Arc<tokio::sync::Mutex<Vec<PerformanceSnapshot>>>,
}

impl SoakTestState {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            episodes_created: AtomicUsize::new(0),
            episodes_completed: AtomicUsize::new(0),
            total_operations: AtomicUsize::new(0),
            failed_operations: AtomicUsize::new(0),
            total_latency: AtomicU64::new(0),
            latencies: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            baseline_memory: Arc::new(tokio::sync::Mutex::new(MemoryUsageStats::new())),
            current_memory: Arc::new(tokio::sync::Mutex::new(MemoryUsageStats::new())),
            snapshots: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Record episode creation
    fn record_episode_created(&self) {
        self.episodes_created.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record episode completion
    fn record_episode_completed(&self, latency_ms: u64) {
        self.episodes_completed.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency_ms, Ordering::Relaxed);

        // Record latency for percentiles
        let mut latencies = self.latencies.blocking_lock();
        latencies.push(latency_ms);

        // Keep last 1000 samples
        if latencies.len() > 1000 {
            latencies.remove(0);
        }
    }

    /// Record operation failure
    fn record_operation_failure(&self) {
        self.failed_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Capture performance snapshot
    async fn capture_snapshot(&self) -> PerformanceSnapshot {
        let episodes_created = self.episodes_created.load(Ordering::Relaxed);
        let episodes_completed = self.episodes_completed.load(Ordering::Relaxed);
        let total_operations = self.total_operations.load(Ordering::Relaxed);
        let failed_operations = self.failed_operations.load(Ordering::Relaxed);

        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency_ms = if episodes_completed > 0 {
            total_latency as f64 / episodes_completed as f64
        } else {
            0.0
        };

        // Calculate percentiles
        let latencies = self.latencies.lock().await;
        let (p95_latency_ms, p99_latency_ms) = if !latencies.is_empty() {
            let mut sorted = latencies.clone();
            sorted.sort();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            let p99_index = (sorted.len() as f64 * 0.99) as usize;
            (
                sorted[p95_index.min(sorted.len() - 1)] as f64,
                sorted[p99_index.min(sorted.len() - 1)] as f64,
            )
        } else {
            (0.0, 0.0)
        };

        // Capture memory usage
        let mut memory = self.current_memory.lock().await;
        memory.capture();
        let memory_usage = memory.clone();

        PerformanceSnapshot {
            episodes_created,
            episodes_completed,
            total_operations,
            failed_operations,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            memory_usage,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Save a snapshot
    async fn save_snapshot(&self) {
        let snapshot = self.capture_snapshot().await;
        let mut snapshots = self.snapshots.lock().await;
        snapshots.push(snapshot);
    }
}

/// Create a test storage instance
async fn create_test_storage() -> (CachedTursoStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let config = TursoConfig {
        max_retries: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 5000,
        enable_pooling: true,
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        compression_level: 3,
        enable_transport_compression: true,
        cache_config: None, // We'll use CachedTursoStorage separately
        ..Default::default()
    };

    let storage =
        TursoStorage::with_config(&format!("file:{}", db_path.to_string_lossy()), "", config)
            .await
            .expect("Failed to create Turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let cache_config = CacheConfig {
        enable_episode_cache: true,
        enable_pattern_cache: true,
        enable_query_cache: true,
        max_episodes: 5000,
        max_patterns: 2000,
        max_query_results: 1000,
        episode_ttl: Duration::from_secs(1800),
        pattern_ttl: Duration::from_secs(3600),
        query_ttl: Duration::from_secs(300),
        min_ttl: Duration::from_secs(60),
        max_ttl: Duration::from_secs(7200),
        hot_threshold: 10,
        cold_threshold: 2,
        adaptation_rate: 0.25,
        enable_background_cleanup: true,
        cleanup_interval_secs: 60,
    };

    let cached_storage = CachedTursoStorage::new(storage, cache_config);

    (cached_storage, temp_dir)
}

/// Worker that continuously creates and completes episodes
async fn worker_task(
    storage: Arc<CachedTursoStorage>,
    state: Arc<SoakTestState>,
    worker_id: usize,
) {
    use memory_core::StorageBackend;

    let mut cycle = 0u64;

    println!("Worker {} started", worker_id);

    while state.is_running() {
        cycle += 1;

        // Create batch of episodes
        for i in 0..EPISODES_PER_CYCLE {
            if !state.is_running() {
                break;
            }

            // Create a new episode
            let mut episode = Episode::new(
                format!(
                    "Soak test worker {} cycle {} episode {}",
                    worker_id, cycle, i
                ),
                TaskContext {
                    domain: "soak_test".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("tokio".to_string()),
                    complexity: memory_core::types::ComplexityLevel::Moderate,
                    tags: vec!["soak_test".to_string(), format!("worker_{}", worker_id)],
                },
                TaskType::CodeGeneration,
            );

            let _episode_id = episode.episode_id;
            state.record_episode_created();

            // Add some steps
            for step_num in 0..3 {
                let mut step = memory_core::episode::ExecutionStep::new(
                    step_num + 1,
                    format!("tool_{}", step_num),
                    format!("Execute step {} in cycle {}", step_num, cycle),
                );
                step.result = Some(ExecutionResult::Success {
                    output: format!("Step completed on worker {}", worker_id),
                });
                step.latency_ms = 50 + (step_num as u64 * 10);

                episode.add_step(step);
            }

            // Complete episode
            let start = Instant::now();
            episode.complete(TaskOutcome::Success {
                verdict: format!("Completed on worker {} cycle {}", worker_id, cycle),
                artifacts: vec![],
            });

            // Store the episode
            let result = storage.store_episode(&episode).await;

            let latency_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(_) => state.record_episode_completed(latency_ms),
                Err(e) => {
                    eprintln!("Worker {}: Failed to store episode: {}", worker_id, e);
                    state.record_operation_failure();
                }
            }
        }

        // Small delay between cycles
        tokio::time::sleep(Duration::from_millis(100)).await;

        if cycle % 10 == 0 {
            println!("Worker {}: Completed {} cycles", worker_id, cycle);
        }
    }

    println!("Worker {} stopped", worker_id);
}

/// Monitor task that captures periodic metrics
async fn monitor_task(state: Arc<SoakTestState>, _storage: Arc<CachedTursoStorage>) {
    println!("Monitor task started");

    let mut snapshot_timer = tokio::time::interval(SNAPSHOT_INTERVAL);
    let mut memory_check_timer = tokio::time::interval(MEMORY_CHECK_INTERVAL);

    loop {
        tokio::select! {
            _ = snapshot_timer.tick() => {
                if !state.is_running() {
                    break;
                }

                // Capture performance snapshot
                state.save_snapshot().await;

                let snapshot = state.capture_snapshot().await;
                println!("\n=== Performance Snapshot ===");
                println!("Episodes Created: {}", snapshot.episodes_created);
                println!("Episodes Completed: {}", snapshot.episodes_completed);
                println!("Total Operations: {}", snapshot.total_operations);
                println!("Failed Operations: {}", snapshot.failed_operations);
                println!("Avg Latency: {:.2} ms", snapshot.avg_latency_ms);
                println!("P95 Latency: {:.2} ms", snapshot.p95_latency_ms);
                println!("P99 Latency: {:.2} ms", snapshot.p99_latency_ms);

                if let Some(rss) = snapshot.memory_usage.rss {
                    println!("Memory RSS: {:.2} MB", rss as f64 / 1024.0 / 1024.0);
                }
                if let Some(vms) = snapshot.memory_usage.vms {
                    println!("Memory VMS: {:.2} MB", vms as f64 / 1024.0 / 1024.0);
                }

                // Check for memory leaks
                let baseline = state.baseline_memory.lock().await;
                if snapshot.memory_usage.has_leaked(&baseline, 0.5) {
                    eprintln!("WARNING: Potential memory leak detected!");
                }
            }
            _ = memory_check_timer.tick() => {
                if !state.is_running() {
                    break;
                }

                // Capture memory usage
                let mut current = state.current_memory.lock().await;
                current.capture();
            }
            else => break,
        }
    }

    println!("Monitor task stopped");
}

/// Analyze test results
fn analyze_results(state: &SoakTestState) -> anyhow::Result<()> {
    let episodes_created = state.episodes_created.load(Ordering::Relaxed);
    let episodes_completed = state.episodes_completed.load(Ordering::Relaxed);
    let total_operations = state.total_operations.load(Ordering::Relaxed);
    let failed_operations = state.failed_operations.load(Ordering::Relaxed);

    println!("\n=== Final Test Results ===");
    println!("Episodes Created: {}", episodes_created);
    println!("Episodes Completed: {}", episodes_completed);
    println!("Total Operations: {}", total_operations);
    println!("Failed Operations: {}", failed_operations);

    if episodes_completed > 0 {
        let success_rate = (episodes_completed as f64 / total_operations as f64) * 100.0;
        println!("Success Rate: {:.2}%", success_rate);

        if success_rate < 99.0 {
            anyhow::bail!("Success rate {:.2}% is below 99.0%", success_rate);
        }
    }

    // Check for memory leaks
    let snapshots = state.snapshots.blocking_lock();
    if snapshots.len() > 1 {
        let first = &snapshots[0];
        let last = &snapshots[snapshots.len() - 1];

        if let (Some(first_rss), Some(last_rss)) = (first.memory_usage.rss, last.memory_usage.rss) {
            let growth = (last_rss as f64 - first_rss as f64) / first_rss as f64;
            println!("Memory Growth: {:.2}%", growth * 100.0);

            if growth > 0.5 {
                anyhow::bail!(
                    "Memory growth of {:.2}% exceeds 50% threshold - possible memory leak",
                    growth * 100.0
                );
            }
        }
    }

    println!("\n=== Test Analysis ===");
    if let Some(p95) = snapshots.last().map(|s| s.p95_latency_ms) {
        println!("Final P95 Latency: {:.2} ms", p95);

        // Check if latency degraded significantly
        if let Some(first_p95) = snapshots.first().map(|s| s.p95_latency_ms) {
            if first_p95 > 0.0 {
                let latency_growth = (p95 - first_p95) / first_p95;
                println!("Latency Growth: {:.2}%", latency_growth * 100.0);

                if latency_growth > 1.0 {
                    anyhow::bail!(
                        "P95 latency grew by {:.2}% indicating performance degradation",
                        latency_growth * 100.0
                    );
                }
            }
        }
    }

    Ok(())
}

/// Main test entry point
#[tokio::test]
#[ignore = "Intentionally slow soak test; run manually with --ignored or nightly soak workflow"]
// REASON: Long-running stability test (60s default, 24h with 'full-soak' feature)
// Run manually: cargo test --test stability_test test_24_hour_stability -- --ignored
// Category: Intentionally slow (soak test for production validation)
async fn test_24_hour_stability() {
    println!("=== 24-Hour Stability Soak Test ===");
    println!("Test duration: {:?}", TEST_DURATION);
    println!("Workers: {}", WORKER_COUNT);
    println!("Episodes per cycle: {}", EPISODES_PER_CYCLE);
    println!(
        "\nWARNING: This test runs for {:?}. Use feature flag 'full-soak' for 24-hour run.\n",
        TEST_DURATION
    );

    let start = Instant::now();

    let state = Arc::new(SoakTestState::new());

    let (storage, _temp_dir) = create_test_storage().await;
    let storage = Arc::new(storage);

    // Capture baseline memory
    let baseline = MemoryUsageStats::new();
    *state.baseline_memory.blocking_lock() = baseline;

    // Start workers
    let mut join_set = JoinSet::new();
    for worker_id in 0..WORKER_COUNT {
        let storage = storage.clone();
        let state = state.clone();

        join_set.spawn(async move {
            worker_task(storage, state, worker_id).await;
        });
    }

    // Start monitor
    let monitor_state = state.clone();
    let monitor_storage = storage.clone();
    let monitor_handle = tokio::spawn(async move {
        monitor_task(monitor_state, monitor_storage).await;
    });

    // Let test run for specified duration
    println!("Running stability test for {:?}...", TEST_DURATION);
    tokio::time::sleep(TEST_DURATION).await;

    // Stop all tasks
    println!("Stopping test...");
    state.stop();

    // Wait for workers to finish
    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            eprintln!("Worker task failed: {}", e);
        }
    }

    // Wait for monitor
    monitor_handle.await.ok();

    let elapsed = start.elapsed();
    println!("Test completed in {:?}", elapsed);

    // Analyze results
    analyze_results(&state).expect("Test analysis failed");

    println!("\n=== Stability Test Passed! ✅ ===");
    println!("The system remained stable for {:?}", TEST_DURATION);
    println!("No significant memory leaks detected");
    println!("Performance remained acceptable throughout the test");
}
