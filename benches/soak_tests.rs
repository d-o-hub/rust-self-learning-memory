//! Soak testing benchmarks for 24-hour stability validation
//!
//! This benchmark suite provides long-running tests to validate system stability:
//! - 24-hour continuous operation test
//! - Connection pool stability monitoring
//! - Memory leak detection over time
//! - Performance degradation monitoring
//!
//! Run with: `cargo bench --bench soak_tests`
//! For full 24-hour test: `cargo bench --bench soak_tests --features full-soak`

#![allow(clippy::excessive_nesting)]
#![allow(deprecated)]
use criterion::{criterion_group, criterion_main, Criterion};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_benches::TokioExecutor;
use memory_core::types::{TaskOutcome, TaskType};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Test duration for normal benchmark runs (shorter for CI)
#[cfg(not(feature = "full-soak"))]
const SOAK_TEST_DURATION: Duration = Duration::from_secs(60); // 1 minute for CI

/// Full 24-hour test duration
#[cfg(feature = "full-soak")]
const SOAK_TEST_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

/// Interval between metric snapshots
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(60); // Every minute

/// Memory check interval
const MEMORY_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Metrics for soak test monitoring
#[derive(Debug, Clone)]
struct SoakTestMetrics {
    /// Total operations performed
    total_operations: Arc<AtomicUsize>,
    /// Successful operations
    successful_operations: Arc<AtomicUsize>,
    /// Failed operations
    failed_operations: Arc<AtomicUsize>,
    /// Total latency in microseconds
    total_latency_us: Arc<AtomicU64>,
    /// Latency samples for percentile calculation
    latency_samples: Arc<tokio::sync::Mutex<Vec<u64>>>,
    /// Memory usage samples in MB
    memory_samples: Arc<tokio::sync::Mutex<Vec<(f64, Instant)>>>,
    /// Test running flag
    running: Arc<AtomicBool>,
    /// Start time
    start_time: Instant,
}

impl SoakTestMetrics {
    fn new() -> Self {
        Self {
            total_operations: Arc::new(AtomicUsize::new(0)),
            successful_operations: Arc::new(AtomicUsize::new(0)),
            failed_operations: Arc::new(AtomicUsize::new(0)),
            total_latency_us: Arc::new(AtomicU64::new(0)),
            latency_samples: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            memory_samples: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(true)),
            start_time: Instant::now(),
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn record_operation(&self, success: bool, latency: Duration) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
            let latency_us = latency.as_micros() as u64;
            self.total_latency_us
                .fetch_add(latency_us, Ordering::Relaxed);

            // Store sample for percentile calculation
            let samples = self.latency_samples.clone();
            tokio::spawn(async move {
                let mut samples = samples.lock().await;
                samples.push(latency_us);
                // Keep only last 10000 samples to manage memory
                if samples.len() > 10000 {
                    samples.remove(0);
                }
            });
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
    }

    async fn record_memory(&self, memory_mb: f64) {
        let mut samples = self.memory_samples.lock().await;
        samples.push((memory_mb, Instant::now()));
    }

    fn get_success_rate(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        let successful = self.successful_operations.load(Ordering::Relaxed);
        if total > 0 {
            successful as f64 / total as f64
        } else {
            0.0
        }
    }

    fn get_throughput(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            0.0
        }
    }

    fn get_average_latency_ms(&self) -> f64 {
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let total_latency = self.total_latency_us.load(Ordering::Relaxed);
        if successful > 0 {
            (total_latency as f64 / successful as f64) / 1000.0
        } else {
            0.0
        }
    }

    async fn get_percentile_latency_ms(&self, percentile: f64) -> Option<f64> {
        let samples = self.latency_samples.lock().await;
        if samples.is_empty() {
            return None;
        }
        let mut sorted = samples.clone();
        drop(samples); // Release lock before sorting
        sorted.sort_unstable();
        let index = ((sorted.len() as f64 * percentile) as usize).min(sorted.len() - 1);
        Some(sorted[index] as f64 / 1000.0)
    }

    async fn check_memory_leak(&self, threshold_percent: f64) -> Option<String> {
        let samples = self.memory_samples.lock().await;
        if samples.len() < 10 {
            return None;
        }

        let first_memory = samples.first().unwrap().0;
        let last_memory = samples.last().unwrap().0;

        if first_memory > 0.0 {
            let growth_percent = ((last_memory - first_memory) / first_memory) * 100.0;
            if growth_percent > threshold_percent {
                return Some(format!(
                    "Potential memory leak detected: {:.1}% growth ({} MB -> {} MB)",
                    growth_percent, first_memory, last_memory
                ));
            }
        }

        None
    }

    async fn print_summary(&self, test_name: &str) {
        let total = self.total_operations.load(Ordering::Relaxed);
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);
        let duration = self.start_time.elapsed();

        println!("\n=== {} Soak Test Summary ===", test_name);
        println!("Duration: {:?}", duration);
        println!("Total Operations: {}", total);
        println!("Successful: {}", successful);
        println!("Failed: {}", failed);
        println!("Success Rate: {:.2}%", self.get_success_rate() * 100.0);
        println!("Throughput: {:.2} ops/sec", self.get_throughput());
        println!("Average Latency: {:.2} ms", self.get_average_latency_ms());

        if let Some(p50) = self.get_percentile_latency_ms(0.50).await {
            println!("P50 Latency: {:.2} ms", p50);
        }
        if let Some(p95) = self.get_percentile_latency_ms(0.95).await {
            println!("P95 Latency: {:.2} ms", p95);
        }
        if let Some(p99) = self.get_percentile_latency_ms(0.99).await {
            println!("P99 Latency: {:.2} ms", p99);
        }

        // Memory summary
        let memory_samples = self.memory_samples.lock().await;
        if !memory_samples.is_empty() {
            let first = memory_samples.first().unwrap().0;
            let last = memory_samples.last().unwrap().0;
            let max = memory_samples.iter().map(|(m, _)| *m).fold(0.0, f64::max);
            println!("\nMemory Usage:");
            println!("  Initial: {:.1} MB", first);
            println!("  Final: {:.1} MB", last);
            println!("  Peak: {:.1} MB", max);
            if first > 0.0 {
                let growth = ((last - first) / first) * 100.0;
                println!("  Growth: {:.1}%", growth);
            }
        }

        println!("===============================\n");
    }
}

/// Get current memory usage in MB
fn get_memory_usage_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/status
        if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f64>() {
                            return kb / 1024.0; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }

    // Fallback: return 0 if we can't determine memory usage
    0.0
}

/// 24-hour stability soak test benchmark
fn benchmark_24h_stability(c: &mut Criterion) {
    let mut group = c.benchmark_group("soak_24h_stability");
    group.sample_size(1);
    group.measurement_time(SOAK_TEST_DURATION);

    group.bench_function("continuous_operations", |b| {
        b.to_async(TokioExecutor).iter_custom(|_iters| async {
            println!("\n=== Starting 24-Hour Stability Soak Test ===");
            println!("Test duration: {:?}", SOAK_TEST_DURATION);
            println!("This test runs continuous operations to validate stability\n");

            let (memory, _temp_dir) = setup_temp_memory().await;
            let memory = Arc::new(memory);
            let metrics = Arc::new(SoakTestMetrics::new());

            // Start memory monitoring task
            let memory_metrics = metrics.clone();
            let memory_monitor = tokio::spawn(async move {
                let mut check_interval = interval(MEMORY_CHECK_INTERVAL);
                loop {
                    check_interval.tick().await;

                    if !memory_metrics.is_running() {
                        break;
                    }

                    let memory_mb = get_memory_usage_mb();
                    memory_metrics.record_memory(memory_mb).await;

                    // Check for memory leaks every 10 minutes
                    if let Some(warning) = memory_metrics.check_memory_leak(50.0).await {
                        println!("WARNING: {}", warning);
                    }
                }
            });

            // Start snapshot task
            let snapshot_metrics = metrics.clone();
            let snapshot_task = tokio::spawn(async move {
                let mut snapshot_interval = interval(SNAPSHOT_INTERVAL);
                let mut snapshot_count = 0;

                loop {
                    snapshot_interval.tick().await;

                    if !snapshot_metrics.is_running() {
                        break;
                    }

                    snapshot_count += 1;
                    println!("\n--- Snapshot #{} ---", snapshot_count);
                    println!("Elapsed: {:?}", snapshot_metrics.start_time.elapsed());
                    println!(
                        "Operations: {}",
                        snapshot_metrics.total_operations.load(Ordering::Relaxed)
                    );
                    println!(
                        "Success Rate: {:.2}%",
                        snapshot_metrics.get_success_rate() * 100.0
                    );
                    println!(
                        "Throughput: {:.2} ops/sec",
                        snapshot_metrics.get_throughput()
                    );
                    println!(
                        "Avg Latency: {:.2} ms",
                        snapshot_metrics.get_average_latency_ms()
                    );

                    if let Some(p95) = snapshot_metrics.get_percentile_latency_ms(0.95).await {
                        println!("P95 Latency: {:.2} ms", p95);
                    }

                    let memory_mb = get_memory_usage_mb();
                    println!("Memory Usage: {:.1} MB", memory_mb);
                    println!("--------------------\n");
                }
            });

            // Start worker tasks
            let mut worker_handles = vec![];
            for worker_id in 0..4 {
                let memory = memory.clone();
                let metrics = metrics.clone();

                let handle = tokio::spawn(async move {
                    let context = create_benchmark_context();
                    let mut operation_count = 0;

                    while metrics.is_running() {
                        // Check if test duration exceeded
                        if metrics.start_time.elapsed() >= SOAK_TEST_DURATION {
                            break;
                        }

                        let start = Instant::now();

                        // Create and complete an episode
                        let episode_id = memory
                            .start_episode(
                                format!("Soak test worker {} op {}", worker_id, operation_count),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(2);
                        for step in steps {
                            memory.log_step(episode_id, step).await;
                        }

                        let result = memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!(
                                        "Worker {} operation {}",
                                        worker_id, operation_count
                                    ),
                                    artifacts: vec![],
                                },
                            )
                            .await;

                        let latency = start.elapsed();
                        metrics.record_operation(result.is_ok(), latency);

                        operation_count += 1;

                        // Small delay to prevent overwhelming the system
                        if operation_count % 100 == 0 {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }

                    operation_count
                });

                worker_handles.push(handle);
            }

            // Let the test run for the specified duration
            tokio::time::sleep(SOAK_TEST_DURATION).await;

            // Stop all tasks
            metrics.stop();

            // Wait for workers to complete
            let mut total_operations = 0;
            for handle in worker_handles {
                if let Ok(count) = handle.await {
                    total_operations += count;
                }
            }

            // Wait for monitoring tasks
            memory_monitor.await.ok();
            snapshot_task.await.ok();

            // Print final summary
            metrics.print_summary("24-Hour Stability").await;

            println!("Total worker operations: {}", total_operations);
            println!("24-Hour Stability Test Complete\n");

            SOAK_TEST_DURATION
        });
    });

    group.finish();
}

/// Connection pool stability benchmark
fn benchmark_connection_pool_stability(c: &mut Criterion) {
    let mut group = c.benchmark_group("soak_connection_pool");
    group.sample_size(1);
    group.measurement_time(Duration::from_secs(300)); // 5 minute test

    group.bench_function("pool_acquire_release", |b| {
        b.to_async(TokioExecutor).iter_custom(|_iters| async {
            println!("\n=== Starting Connection Pool Stability Test ===");
            println!("Testing continuous connection acquire/release cycles\n");

            let (memory, _temp_dir) = setup_temp_memory().await;
            let memory = Arc::new(memory);

            let metrics = Arc::new(SoakTestMetrics::new());
            let test_duration = Duration::from_secs(300); // 5 minutes

            // Spawn multiple workers that continuously acquire and release connections
            let mut handles = vec![];
            for worker_id in 0..10 {
                let memory = memory.clone();
                let metrics = metrics.clone();

                let handle = tokio::spawn(async move {
                    let context = create_benchmark_context();
                    let mut operation_count = 0;

                    while metrics.is_running() {
                        if metrics.start_time.elapsed() >= test_duration {
                            break;
                        }

                        let start = Instant::now();

                        // Perform a series of operations that exercise the connection pool
                        let episode_id = memory
                            .start_episode(
                                format!("Pool test worker {} op {}", worker_id, operation_count),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        // Immediate retrieval to test connection reuse
                        let _result = memory
                            .retrieve_relevant_context(
                                format!("Query worker {} op {}", worker_id, operation_count),
                                context.clone(),
                                5,
                            )
                            .await;

                        // Complete the episode
                        let result = memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!("Pool test {}:{}", worker_id, operation_count),
                                    artifacts: vec![],
                                },
                            )
                            .await;

                        let latency = start.elapsed();
                        metrics.record_operation(result.is_ok(), latency);

                        operation_count += 1;

                        // Minimal delay to maximize connection churn
                        if operation_count % 50 == 0 {
                            tokio::time::sleep(Duration::from_millis(1)).await;
                        }
                    }

                    operation_count
                });

                handles.push(handle);
            }

            // Let the test run
            tokio::time::sleep(test_duration).await;
            metrics.stop();

            // Collect results
            let mut total_operations = 0;
            for handle in handles {
                if let Ok(count) = handle.await {
                    total_operations += count;
                }
            }

            metrics.print_summary("Connection Pool Stability").await;

            println!("Total pool operations: {}", total_operations);
            println!("Connection Pool Stability Test Complete\n");

            test_duration
        });
    });

    group.finish();
}

/// Memory leak detection benchmark
fn benchmark_memory_leak_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("soak_memory_leak");
    group.sample_size(1);
    group.measurement_time(Duration::from_secs(180)); // 3 minute test

    group.bench_function("memory_stability", |b| {
        b.to_async(TokioExecutor).iter_custom(|_iters| async {
            println!("\n=== Starting Memory Leak Detection Test ===");
            println!("Monitoring memory usage over time\n");

            let (memory, _temp_dir) = setup_temp_memory().await;
            let memory = Arc::new(memory);

            let metrics = Arc::new(SoakTestMetrics::new());
            let test_duration = Duration::from_secs(180); // 3 minutes

            // Record initial memory
            let initial_memory = get_memory_usage_mb();
            metrics.record_memory(initial_memory).await;
            println!("Initial memory: {:.1} MB", initial_memory);

            // Spawn memory monitoring
            let memory_metrics = metrics.clone();
            let monitor_handle = tokio::spawn(async move {
                let mut check_interval = interval(Duration::from_secs(10));
                loop {
                    check_interval.tick().await;

                    if !memory_metrics.is_running() {
                        break;
                    }

                    let memory_mb = get_memory_usage_mb();
                    memory_metrics.record_memory(memory_mb).await;

                    // Print memory status every 30 seconds
                    if memory_metrics.start_time.elapsed().as_secs() % 30 == 0 {
                        println!("Memory: {:.1} MB", memory_mb);
                    }
                }
            });

            // Spawn worker that creates and drops episodes
            let worker_metrics = metrics.clone();
            let worker_memory = memory.clone();
            let worker_handle = tokio::spawn(async move {
                let context = create_benchmark_context();
                let mut operation_count = 0;

                while worker_metrics.is_running() {
                    if worker_metrics.start_time.elapsed() >= test_duration {
                        break;
                    }

                    // Create batch of episodes
                    let batch_size = 50;
                    let mut episode_ids = Vec::with_capacity(batch_size);

                    for i in 0..batch_size {
                        let start = Instant::now();

                        let episode_id = worker_memory
                            .start_episode(
                                format!("Memory test episode {}", operation_count + i),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(2);
                        for step in steps {
                            worker_memory.log_step(episode_id, step).await;
                        }

                        let result = worker_memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!("Memory test {}", operation_count + i),
                                    artifacts: vec![],
                                },
                            )
                            .await;

                        let latency = start.elapsed();
                        worker_metrics.record_operation(result.is_ok(), latency);
                        episode_ids.push(episode_id);
                    }

                    operation_count += batch_size;

                    // Allow time for garbage collection
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                operation_count
            });

            // Let the test run
            tokio::time::sleep(test_duration).await;
            metrics.stop();

            // Wait for tasks
            let _ = worker_handle.await;
            monitor_handle.await.ok();

            // Analyze memory trend
            let memory_samples = metrics.memory_samples.lock().await;
            let final_memory = memory_samples.last().map(|(m, _)| *m).unwrap_or(0.0);

            metrics.print_summary("Memory Leak Detection").await;

            // Memory growth analysis
            if initial_memory > 0.0 {
                let growth = ((final_memory - initial_memory) / initial_memory) * 100.0;
                println!("Memory growth: {:.1}%", growth);

                if growth > 20.0 {
                    println!("WARNING: Significant memory growth detected!");
                } else if growth > 5.0 {
                    println!("NOTE: Moderate memory growth (may be normal)");
                } else {
                    println!("OK: Memory usage stable");
                }
            }

            println!("Memory Leak Detection Test Complete\n");

            test_duration
        });
    });

    group.finish();
}

/// Performance degradation detection benchmark
fn benchmark_performance_degradation(c: &mut Criterion) {
    let mut group = c.benchmark_group("soak_performance_degradation");
    group.sample_size(1);
    group.measurement_time(Duration::from_secs(120)); // 2 minute test

    group.bench_function("sustained_performance", |b| {
        b.to_async(TokioExecutor).iter_custom(|_iters| async {
            println!("\n=== Starting Performance Degradation Test ===");
            println!("Monitoring performance consistency over time\n");

            let (memory, _temp_dir) = setup_temp_memory().await;
            let memory = Arc::new(memory);

            let metrics = Arc::new(SoakTestMetrics::new());
            let test_duration = Duration::from_secs(120); // 2 minutes

            // Pre-populate with data
            let context = create_benchmark_context();
            let mut episode_ids = Vec::with_capacity(100);
            for i in 0..100 {
                let episode_id = memory
                    .start_episode(
                        generate_episode_description(i),
                        context.clone(),
                        TaskType::CodeGeneration,
                    )
                    .await;

                let steps = generate_execution_steps(2);
                for step in steps {
                    memory.log_step(episode_id, step).await;
                }

                memory
                    .complete_episode(
                        episode_id,
                        TaskOutcome::Success {
                            verdict: format!("Pre-populated {}", i),
                            artifacts: vec![],
                        },
                    )
                    .await
                    .expect("Failed to complete episode");

                episode_ids.push(episode_id);
            }

            // Spawn worker that performs consistent operations
            let worker_metrics = metrics.clone();
            let worker_memory = memory.clone();
            let worker_handle = tokio::spawn(async move {
                let context = create_benchmark_context();
                let mut operation_count = 0;

                while worker_metrics.is_running() {
                    if worker_metrics.start_time.elapsed() >= test_duration {
                        break;
                    }

                    let start = Instant::now();

                    // Mix of operations
                    if operation_count % 3 == 0 {
                        // Create new episode
                        let episode_id = worker_memory
                            .start_episode(
                                format!("Perf test episode {}", operation_count),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(2);
                        for step in steps {
                            worker_memory.log_step(episode_id, step).await;
                        }

                        let result = worker_memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!("Perf test {}", operation_count),
                                    artifacts: vec![],
                                },
                            )
                            .await;

                        let latency = start.elapsed();
                        worker_metrics.record_operation(result.is_ok(), latency);
                    } else {
                        // Query existing episodes
                        let _result = worker_memory
                            .retrieve_relevant_context(
                                format!("Perf test query {}", operation_count),
                                context.clone(),
                                5,
                            )
                            .await;

                        let latency = start.elapsed();
                        worker_metrics.record_operation(true, latency);
                    }

                    operation_count += 1;
                }

                operation_count
            });

            // Let the test run
            tokio::time::sleep(test_duration).await;
            metrics.stop();

            let _ = worker_handle.await;

            metrics.print_summary("Performance Degradation").await;

            // Analyze performance trend
            let latency_samples = metrics.latency_samples.lock().await;
            if latency_samples.len() >= 100 {
                let mid_point = latency_samples.len() / 2;
                let first_half: Vec<u64> = latency_samples[..mid_point].to_vec();
                let second_half: Vec<u64> = latency_samples[mid_point..].to_vec();

                let first_avg = first_half.iter().sum::<u64>() as f64 / first_half.len() as f64;
                let second_avg = second_half.iter().sum::<u64>() as f64 / second_half.len() as f64;

                let degradation = ((second_avg - first_avg) / first_avg) * 100.0;
                println!("First half avg latency: {:.2} µs", first_avg);
                println!("Second half avg latency: {:.2} µs", second_avg);
                println!("Performance change: {:.1}%", degradation);

                if degradation > 50.0 {
                    println!("WARNING: Significant performance degradation detected!");
                } else if degradation > 20.0 {
                    println!("NOTE: Moderate performance change");
                } else {
                    println!("OK: Performance stable");
                }
            }

            println!("Performance Degradation Test Complete\n");

            test_duration
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_24h_stability,
    benchmark_connection_pool_stability,
    benchmark_memory_leak_detection,
    benchmark_performance_degradation
);
criterion_main!(benches);
