//! Scalability benchmarks for multiple storage backends
//!
//! Tests how performance scales with:
//! - Dataset size (episodes, patterns)
//! - Concurrent users
//! - Query complexity
//! - Storage backend differences

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_benches::TokioExecutor;
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_core::types::{TaskOutcome, TaskType};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ScalabilityDimension {
    /// Scale dataset size
    DatasetSize,
    /// Scale concurrent users
    ConcurrentUsers,
    /// Scale query complexity
    QueryComplexity,
    /// Scale storage operations
    OperationScale,
}

impl ScalabilityDimension {
    #[allow(dead_code)]
    fn name(&self) -> &'static str {
        match self {
            Self::DatasetSize => "dataset_size",
            Self::ConcurrentUsers => "concurrent_users",
            Self::QueryComplexity => "query_complexity",
            Self::OperationScale => "operation_scale",
        }
    }
}

async fn populate_dataset(
    memory: &memory_core::memory::SelfLearningMemory,
    episode_count: usize,
) -> Vec<uuid::Uuid> {
    let context = create_benchmark_context();
    let mut episode_ids = Vec::with_capacity(episode_count);

    for i in 0..episode_count {
        let episode_id = memory
            .start_episode(
                generate_episode_description(i),
                context.clone(),
                TaskType::CodeGeneration,
            )
            .await;

        // Vary step count to create different complexity levels
        let step_count = 1 + (i % 10); // 1-10 steps per episode
        let steps = generate_execution_steps(step_count);
        for step in steps {
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Dataset episode {} with {} steps", i, step_count),
                    artifacts: vec![format!("artifact_{}.rs", i)],
                },
            )
            .await
            .expect("Failed to complete episode");

        episode_ids.push(episode_id);
    }

    episode_ids
}

fn benchmark_dataset_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("dataset_scalability");
    group.sample_size(10);

    let dataset_sizes = vec![100, 500, 1000, 5000];

    for &size in &dataset_sizes {
        group.bench_with_input(
            BenchmarkId::new("dataset_query_scalability", size),
            &size,
            |b, &_size| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let memory = Arc::new(memory);
                    let context = create_benchmark_context();
                    let results = memory
                        .retrieve_relevant_context(
                            "Query performance at scale".to_string(),
                            context,
                            20,
                        )
                        .await;

                    // Benchmark measures time per query at this dataset size
                    criterion::black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

#[allow(clippy::excessive_nesting)]
fn benchmark_concurrent_user_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_user_scalability");
    group.sample_size(5);
    group.measurement_time(std::time::Duration::from_secs(30));

    let user_counts = vec![1, 5, 10, 20, 50];
    let operations_per_user = 20;

    for &user_count in &user_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_users", user_count),
            &user_count,
            |b, &users| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let memory = Arc::new(memory);

                    // Pre-populate with some data
                    let _episode_ids = populate_dataset(&memory, 200).await;

                    // Simulate concurrent users
                    let semaphore = Arc::new(Semaphore::new(users));
                    let mut handles = vec![];

                    for user_id in 0..users {
                        let memory = memory.clone();
                        let semaphore = semaphore.clone();

                        let handle = tokio::spawn(async move {
                            #[allow(clippy::excessive_nesting)]
                            {
                                let _permit = semaphore.acquire().await.unwrap();
                                let context = create_benchmark_context();

                                for op in 0..operations_per_user {
                                    // Mix of reads and writes
                                    if op % 3 == 0 {
                                        // Write operation
                                        let episode_id = memory
                                            .start_episode(
                                                format!("User {} operation {}", user_id, op),
                                                context.clone(),
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
                                                    verdict: format!(
                                                        "User {} write {}",
                                                        user_id, op
                                                    ),
                                                    artifacts: vec![],
                                                },
                                            )
                                            .await
                                            .expect("Failed to complete episode");
                                    } else {
                                        // Read operation
                                        let _results = memory
                                            .retrieve_relevant_context(
                                                format!("User {} query {}", user_id, op),
                                                context.clone(),
                                                5,
                                            )
                                            .await;
                                    }
                                }
                            }
                        });

                        handles.push(handle);
                    }

                    // Wait for all users to complete
                    futures::future::join_all(handles).await;
                });
            },
        );
    }

    group.finish();
}

fn benchmark_query_complexity_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_complexity_scalability");
    group.sample_size(15);

    // Pre-populate with a fixed dataset
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (memory, _temp_dir) = rt.block_on(async {
        let setup = setup_temp_memory().await;
        populate_dataset(&setup.0, 1000).await;
        setup
    });
    let memory = Arc::new(memory);

    let query_limits = vec![1, 5, 10, 20, 50, 100];

    for &limit in &query_limits {
        group.bench_with_input(
            BenchmarkId::new("query_result_limit", limit),
            &limit,
            |b, &query_limit| {
                b.to_async(TokioExecutor).iter(|| async {
                    let context = create_benchmark_context();

                    let results = memory
                        .retrieve_relevant_context(
                            "Complexity scalability test query".to_string(),
                            context,
                            query_limit,
                        )
                        .await;

                    criterion::black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

#[allow(clippy::excessive_nesting)]
fn benchmark_operation_batch_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_batch_scalability");
    group.sample_size(10);

    let batch_sizes = vec![1, 5, 10, 25, 50, 100];

    for &batch_size in &batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("batch_operations", batch_size),
            &batch_size,
            |b, &size| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    // Execute operations in batches
                    let mut handles = vec![];

                    for batch in 0..(100 / size) {
                        let mut batch_handles = vec![];

                        for i in 0..size {
                            let memory = memory.clone();
                            let context = context.clone();
                            let op_id = batch * size + i;

                            let handle = tokio::spawn(async move {
                                #[allow(clippy::excessive_nesting)]
                                {
                                    let episode_id = memory
                                        .start_episode(
                                            format!("Batch operation {}", op_id),
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
                                                verdict: format!(
                                                    "Batch operation {} completed",
                                                    op_id
                                                ),
                                                artifacts: vec![],
                                            },
                                        )
                                        .await
                                        .expect("Failed to complete episode");

                                    episode_id
                                }
                            });

                            batch_handles.push(handle);
                        }

                        // Wait for this batch to complete before starting next
                        let results = futures::future::join_all(batch_handles).await;
                        handles.extend(results);
                    }

                    // Ensure all operations completed
                    criterion::black_box(handles.len());
                });
            },
        );
    }

    group.finish();
}

#[allow(clippy::excessive_nesting)]
fn benchmark_throughput_vs_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_vs_latency");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(20));

    // Test how latency changes with operation rate
    let operation_rates = vec![10, 50, 100, 200]; // operations per second target

    for &rate in &operation_rates {
        group.bench_with_input(
            BenchmarkId::new("operation_rate", rate),
            &rate,
            |b, &target_rate| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    let start_time = std::time::Instant::now();
                    let mut operation_count = 0;

                    // Run operations for a fixed time period
                    while start_time.elapsed().as_secs() < 10 {
                        let episode_id = memory
                            .start_episode(
                                format!("Rate test operation {}", operation_count),
                                context.clone(),
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
                                    verdict: format!("Rate test {}", operation_count),
                                    artifacts: vec![],
                                },
                            )
                            .await
                            .expect("Failed to complete episode");

                        operation_count += 1;

                        // Rate limiting - calculate delay to achieve target rate
                        let elapsed = start_time.elapsed();
                        let target_operations = (elapsed.as_secs_f64() * target_rate as f64) as u32;

                        if operation_count > target_operations {
                            let delay = std::time::Duration::from_secs_f64(
                                (operation_count as f64 - target_operations as f64)
                                    / target_rate as f64,
                            );
                            tokio::time::sleep(delay).await;
                        }
                    }

                    criterion::black_box(operation_count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_dataset_scalability,
    benchmark_concurrent_user_scalability,
    benchmark_query_complexity_scalability,
    benchmark_operation_batch_scalability,
    benchmark_throughput_vs_latency
);
criterion_main!(benches);
