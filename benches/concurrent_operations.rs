//! Concurrent operations benchmarks for async Rust applications
//!
//! Tests concurrent read/write operations across multiple storage backends
//! following patterns from rust-storage-bench and YCSB workloads.

use criterion::{
    async_executor::FuturesExecutor, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use futures::future::join_all;
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_core::types::{TaskOutcome, TaskType};
use rand::prelude::*;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// YCSB-like workload patterns for concurrent operations
#[derive(Debug, Clone)]
enum WorkloadPattern {
    /// Workload A: Update heavy (50% reads, 50% writes)
    UpdateHeavy,
    /// Workload B: Read mostly (95% reads, 5% writes)
    ReadMostly,
    /// Workload C: Read only (100% reads)
    ReadOnly,
    /// Workload D: Read latest (95% reads of recent data, 5% writes)
    ReadLatest,
    /// Workload E: Short ranges (95% inserts, 5% reads of recent data)
    ShortRanges,
}

impl WorkloadPattern {
    fn read_ratio(&self) -> f64 {
        match self {
            Self::UpdateHeavy => 0.5,
            Self::ReadMostly => 0.95,
            Self::ReadOnly => 1.0,
            Self::ReadLatest => 0.95,
            Self::ShortRanges => 0.05,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::UpdateHeavy => "update_heavy",
            Self::ReadMostly => "read_mostly",
            Self::ReadOnly => "read_only",
            Self::ReadLatest => "read_latest",
            Self::ShortRanges => "short_ranges",
        }
    }
}

async fn setup_concurrent_benchmark_data(
    memory: &memory_core::memory::SelfLearningMemory,
    episode_count: usize,
) -> Vec<uuid::Uuid> {
    let context = create_benchmark_context();
    let mut episode_ids = Vec::with_capacity(episode_count);

    // Pre-populate with episodes for concurrent testing
    for i in 0..episode_count {
        let episode_id = memory
            .start_episode(
                generate_episode_description(i),
                context.clone(),
                TaskType::CodeGeneration,
            )
            .await;

        let steps = generate_execution_steps(3);
        for step in steps {
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Concurrent test episode {}", i),
                    artifacts: vec![],
                },
            )
            .await
            .expect("Failed to complete episode");

        episode_ids.push(episode_id);
    }

    episode_ids
}

async fn run_concurrent_workload(
    memory: Arc<memory_core::memory::SelfLearningMemory>,
    episode_ids: &[uuid::Uuid],
    pattern: WorkloadPattern,
    operations: usize,
    thread_id: usize,
) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(thread_id as u64);
    let context = create_benchmark_context();

    for _ in 0..operations {
        let choice: f64 = rng.gen();

        if choice < pattern.read_ratio() {
            // Read operation
            let random_idx = rng.gen_range(0..episode_ids.len());
            let _episode_id = episode_ids[random_idx];

            let _result = memory
                .retrieve_relevant_context(
                    format!("Query from thread {} episode {}", thread_id, random_idx),
                    context.clone(),
                    5,
                )
                .await;
        } else {
            // Write operation
            let episode_id = memory
                .start_episode(
                    format!(
                        "Concurrent write from thread {} op {}",
                        thread_id,
                        rng.gen::<u32>()
                    ),
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
                        verdict: "Concurrent write completed".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .expect("Failed to complete episode");
        }
    }
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.sample_size(10); // Reduce sample size for concurrent benchmarks
    group.measurement_time(std::time::Duration::from_secs(30)); // Longer measurement time

    let patterns = vec![
        WorkloadPattern::UpdateHeavy,
        WorkloadPattern::ReadMostly,
        WorkloadPattern::ReadOnly,
        WorkloadPattern::ReadLatest,
        WorkloadPattern::ShortRanges,
    ];

    let concurrency_levels = vec![1, 4, 8, 16];
    let operations_per_thread = 100;

    for pattern in patterns {
        for concurrency in &concurrency_levels {
            group.bench_with_input(
                BenchmarkId::new(
                    format!("{}_concurrency_{}", pattern.name(), concurrency),
                    format!("{}@{}", pattern.name(), concurrency),
                ),
                &(pattern.clone(), *concurrency),
                |b, (pattern, concurrency)| {
                    b.to_async(FuturesExecutor).iter(|| async {
                        let (memory, _temp_dir) = setup_temp_memory().await;
                        let memory = Arc::new(memory);

                        // Setup initial data
                        let episode_ids = setup_concurrent_benchmark_data(&memory, 1000).await;

                        // Run concurrent operations
                        let semaphore = Arc::new(Semaphore::new(*concurrency));
                        let mut handles = vec![];

                        for thread_id in 0..*concurrency {
                            let memory = memory.clone();
                            let episode_ids = episode_ids.clone();
                            let pattern = pattern.clone();
                            let semaphore = semaphore.clone();

                            let handle = tokio::spawn(async move {
                                #[allow(clippy::excessive_nesting)]
                                {
                                    let _permit = semaphore.acquire().await.unwrap();
                                    run_concurrent_workload(
                                        memory,
                                        &episode_ids,
                                        pattern,
                                        operations_per_thread,
                                        thread_id,
                                    )
                                    .await;
                                }
                            });

                            handles.push(handle);
                        }

                        // Wait for all operations to complete
                        join_all(handles).await;
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_async_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_throughput");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(20));

    for operations in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(operations),
            operations,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let memory = Arc::new(memory);
                    let context = create_benchmark_context();

                    // Run multiple async operations concurrently
                    let mut handles = vec![];

                    for i in 0..count {
                        let memory = memory.clone();
                        let context = context.clone();

                        let handle = tokio::spawn(async move {
                            #[allow(clippy::excessive_nesting)]
                            {
                                let episode_id = memory
                                    .start_episode(
                                        generate_episode_description(i),
                                        context,
                                        TaskType::CodeGeneration,
                                    )
                                    .await;

                                let steps = generate_execution_steps(1);
                                for step in steps {
                                    memory.log_step(episode_id, step).await;
                                }

                                memory
                                    .complete_episode(
                                        episode_id,
                                        TaskOutcome::Success {
                                            verdict: format!("Async throughput test {}", i),
                                            artifacts: vec![],
                                        },
                                    )
                                    .await
                                    .expect("Failed to complete episode");

                                episode_id
                            }
                        });

                        handles.push(handle);
                    }

                    // Wait for all operations
                    let _results = join_all(handles).await;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_concurrent_operations,
    benchmark_async_throughput
);
criterion_main!(benches);
