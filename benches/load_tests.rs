//! Load testing benchmarks for the self-learning memory system
//!
//! This benchmark suite measures system performance under various load conditions:
//! - Concurrent episode creation (100 concurrent operations)
//! - Relationship stress testing (1000 episodes, 5000 relationships)
//! - Pattern extraction load testing
//! - Memory pressure testing (10,000 episodes)
//!
//! Run with: `cargo bench --bench load_tests`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_benches::TokioExecutor;
use memory_core::types::{TaskOutcome, TaskType};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Metrics collector for load test measurements
#[derive(Debug, Clone)]
struct LoadTestMetrics {
    /// Total operations completed
    operations_completed: usize,
    /// Total operations failed
    operations_failed: usize,
    /// Latency measurements in microseconds
    latencies_us: Vec<u64>,
    /// Test start time
    start_time: Instant,
    /// Test end time
    end_time: Option<Instant>,
}

impl LoadTestMetrics {
    fn new() -> Self {
        Self {
            operations_completed: 0,
            operations_failed: 0,
            latencies_us: Vec::new(),
            start_time: Instant::now(),
            end_time: None,
        }
    }

    fn record_success(&mut self, latency: Duration) {
        self.operations_completed += 1;
        self.latencies_us.push(latency.as_micros() as u64);
    }

    fn record_failure(&mut self) {
        self.operations_failed += 1;
    }

    fn finish(&mut self) {
        self.end_time = Some(Instant::now());
    }

    fn duration(&self) -> Duration {
        self.end_time
            .map(|end| end.duration_since(self.start_time))
            .unwrap_or_else(|| self.start_time.elapsed())
    }

    fn throughput(&self) -> f64 {
        let duration_secs = self.duration().as_secs_f64();
        if duration_secs > 0.0 {
            self.operations_completed as f64 / duration_secs
        } else {
            0.0
        }
    }

    fn percentile(&self, p: f64) -> Option<u64> {
        if self.latencies_us.is_empty() {
            return None;
        }
        let mut sorted = self.latencies_us.clone();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64 * p) as usize).min(sorted.len() - 1);
        Some(sorted[index])
    }

    fn p50(&self) -> Option<u64> {
        self.percentile(0.50)
    }

    fn p95(&self) -> Option<u64> {
        self.percentile(0.95)
    }

    fn p99(&self) -> Option<u64> {
        self.percentile(0.99)
    }

    fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Load Test Results ===", test_name);
        println!("Operations Completed: {}", self.operations_completed);
        println!("Operations Failed: {}", self.operations_failed);
        println!("Duration: {:?}", self.duration());
        println!("Throughput: {:.2} ops/sec", self.throughput());
        if let Some(p50) = self.p50() {
            println!("P50 Latency: {} µs", p50);
        }
        if let Some(p95) = self.p95() {
            println!("P95 Latency: {} µs", p95);
        }
        if let Some(p99) = self.p99() {
            println!("P99 Latency: {} µs", p99);
        }
        println!("===============================\n");
    }
}

/// Benchmark concurrent episode creation with 100 concurrent operations
fn benchmark_concurrent_episode_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_concurrent_episode_creation");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Test with different concurrency levels
    for concurrency in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(TokioExecutor).iter_custom(|iters| async move {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let (memory, _temp_dir) = setup_temp_memory().await;
                        let memory = Arc::new(memory);
                        let context = create_benchmark_context();

                        let mut metrics = LoadTestMetrics::new();
                        let semaphore = Arc::new(Semaphore::new(concurrency));
                        let mut handles = vec![];

                        // Spawn concurrent episode creation tasks
                        for i in 0..concurrency {
                            let memory = memory.clone();
                            let context = context.clone();
                            let semaphore = semaphore.clone();

                            let handle = tokio::spawn(async move {
                                let _permit = semaphore.acquire().await.unwrap();
                                let start = Instant::now();

                                let episode_id = memory
                                    .start_episode(
                                        generate_episode_description(i),
                                        context,
                                        TaskType::CodeGeneration,
                                    )
                                    .await;

                                let steps = generate_execution_steps(3);
                                for step in steps {
                                    memory.log_step(episode_id, step).await;
                                }

                                let result = memory
                                    .complete_episode(
                                        episode_id,
                                        TaskOutcome::Success {
                                            verdict: format!("Concurrent episode {}", i),
                                            artifacts: vec![],
                                        },
                                    )
                                    .await;

                                let latency = start.elapsed();

                                (result.is_ok(), latency)
                            });

                            handles.push(handle);
                        }

                        // Collect results
                        for handle in handles {
                            match handle.await {
                                Ok((success, latency)) => {
                                    if success {
                                        metrics.record_success(latency);
                                    } else {
                                        metrics.record_failure();
                                    }
                                }
                                Err(_) => {
                                    metrics.record_failure();
                                }
                            }
                        }

                        metrics.finish();
                        total_duration += metrics.duration();
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark relationship stress test with 1000 episodes and 5000 relationships
fn benchmark_relationship_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_relationship_stress");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(60));

    group.bench_function("create_and_query_relationships", |b| {
        b.to_async(TokioExecutor).iter_custom(|iters| async move {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let (memory, _temp_dir) = setup_temp_memory().await;
                let memory = Arc::new(memory);
                let context = create_benchmark_context();

                let start = Instant::now();

                // Create 1000 episodes
                let mut episode_ids = Vec::with_capacity(1000);
                for i in 0..1000 {
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
                                verdict: format!("Relationship test episode {}", i),
                                artifacts: vec![],
                            },
                        )
                        .await
                        .expect("Failed to complete episode");

                    episode_ids.push(episode_id);
                }

                // Create 5000 relationships between episodes
                let mut relationship_handles = vec![];
                for i in 0..5000 {
                    let memory = memory.clone();
                    let source_idx = i % episode_ids.len();
                    let target_idx = (i + 1) % episode_ids.len();
                    let source_id = episode_ids[source_idx];
                    let target_id = episode_ids[target_idx];

                    let handle = tokio::spawn(async move {
                        memory
                            .create_relationship(
                                source_id,
                                target_id,
                                memory_core::types::RelationshipType::DependsOn,
                                Some(format!("Relationship {}", i)),
                            )
                            .await
                    });

                    relationship_handles.push(handle);
                }

                // Wait for all relationships to be created
                let relationship_results = join_all(relationship_handles).await;
                let successful_relationships = relationship_results
                    .iter()
                    .filter(|r| r.as_ref().map(|inner| inner.is_ok()).unwrap_or(false))
                    .count();

                // Query relationship graph
                let query_start = Instant::now();
                for episode_id in episode_ids.iter().take(100) {
                    let _related = memory.get_related_episodes(*episode_id, None).await;
                }
                let query_duration = query_start.elapsed();

                let total_elapsed = start.elapsed();
                println!(
                    "Created {} episodes, {} relationships, queried in {:?}",
                    1000, successful_relationships, query_duration
                );

                total_duration += total_elapsed;
            }

            total_duration
        });
    });

    group.finish();
}

/// Benchmark pattern extraction load testing
fn benchmark_pattern_extraction_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_pattern_extraction");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    for episode_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                b.to_async(TokioExecutor).iter_custom(|iters| async move {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let (memory, _temp_dir) = setup_temp_memory().await;
                        let memory = Arc::new(memory);
                        let context = create_benchmark_context();

                        let start = Instant::now();

                        // Create episodes with steps
                        let mut episode_ids = Vec::with_capacity(count);
                        for i in 0..count {
                            let episode_id = memory
                                .start_episode(
                                    generate_episode_description(i),
                                    context.clone(),
                                    TaskType::CodeGeneration,
                                )
                                .await;

                            // Add varying number of steps
                            let step_count = 3 + (i % 5);
                            let steps = generate_execution_steps(step_count);
                            for step in steps {
                                memory.log_step(episode_id, step).await;
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Pattern extraction episode {}", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");

                            episode_ids.push(episode_id);
                        }

                        // Trigger pattern extraction by querying
                        let extraction_start = Instant::now();
                        for _ in 0..10 {
                            let _patterns = memory
                                .analyze_patterns(memory_core::types::PatternFilter::default())
                                .await;
                        }
                        let extraction_duration = extraction_start.elapsed();

                        let total_elapsed = start.elapsed();
                        println!(
                            "Created {} episodes, pattern extraction took {:?}",
                            count, extraction_duration
                        );

                        total_duration += total_elapsed;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory pressure test with 10,000 episodes
fn benchmark_memory_pressure_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_memory_pressure");
    group.sample_size(3);
    group.measurement_time(Duration::from_secs(120));

    for episode_count in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                b.to_async(TokioExecutor).iter_custom(|iters| async move {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let (memory, _temp_dir) = setup_temp_memory().await;
                        let memory = Arc::new(memory);
                        let context = create_benchmark_context();

                        let start = Instant::now();
                        let mut metrics = LoadTestMetrics::new();

                        // Create episodes in batches to manage memory
                        let batch_size = 100;
                        for batch_start in (0..count).step_by(batch_size) {
                            let batch_end = (batch_start + batch_size).min(count);
                            let mut handles = vec![];

                            for i in batch_start..batch_end {
                                let memory = memory.clone();
                                let context = context.clone();

                                let handle = tokio::spawn(async move {
                                    let episode_start = Instant::now();

                                    let episode_id = memory
                                        .start_episode(
                                            generate_episode_description(i),
                                            context,
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
                                                verdict: format!("Memory pressure episode {}", i),
                                                artifacts: vec![],
                                            },
                                        )
                                        .await;

                                    let latency = episode_start.elapsed();
                                    (result.is_ok(), latency)
                                });

                                handles.push(handle);
                            }

                            // Collect batch results
                            for handle in handles {
                                match handle.await {
                                    Ok((success, latency)) => {
                                        if success {
                                            metrics.record_success(latency);
                                        } else {
                                            metrics.record_failure();
                                        }
                                    }
                                    Err(_) => {
                                        metrics.record_failure();
                                    }
                                }
                            }

                            // Small delay between batches to allow memory cleanup
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }

                        metrics.finish();
                        let total_elapsed = start.elapsed();

                        metrics.print_summary(&format!("Memory Pressure ({} episodes)", count));

                        total_duration += total_elapsed;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mixed workload with read and write operations
fn benchmark_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_mixed_workload");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("read_write_mixed", |b| {
        b.to_async(TokioExecutor).iter_custom(|iters| async move {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let (memory, _temp_dir) = setup_temp_memory().await;
                let memory = Arc::new(memory);
                let context = create_benchmark_context();

                // Pre-populate with some episodes
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
                                verdict: format!("Pre-populated episode {}", i),
                                artifacts: vec![],
                            },
                        )
                        .await
                        .expect("Failed to complete episode");

                    episode_ids.push(episode_id);
                }

                let start = Instant::now();
                let mut handles = vec![];

                // Mix of read and write operations
                for i in 0..200 {
                    let memory = memory.clone();
                    let context = context.clone();
                    let episode_ids = episode_ids.clone();

                    let handle = tokio::spawn(async move {
                        if i % 3 == 0 {
                            // Write operation (33%)
                            let episode_id = memory
                                .start_episode(
                                    format!("Mixed workload write {}", i),
                                    context,
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
                                        verdict: format!("Write {}", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .map(|_| ())
                        } else {
                            // Read operation (67%)
                            let random_idx = i % episode_ids.len();
                            let _result = memory
                                .retrieve_relevant_context(format!("Query {}", i), context, 5)
                                .await;
                            Ok(())
                        }
                    });

                    handles.push(handle);
                }

                // Wait for all operations
                let results = join_all(handles).await;
                let successful = results.iter().filter(|r| r.is_ok()).count();

                let total_elapsed = start.elapsed();
                println!(
                    "Mixed workload: {} operations, {} successful in {:?}",
                    200, successful, total_elapsed
                );

                total_duration += total_elapsed;
            }

            total_duration
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_concurrent_episode_creation,
    benchmark_relationship_stress,
    benchmark_pattern_extraction_load,
    benchmark_memory_pressure_load,
    benchmark_mixed_workload
);
criterion_main!(benches);
