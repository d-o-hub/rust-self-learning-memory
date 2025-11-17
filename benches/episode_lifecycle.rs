//! Comprehensive benchmarks for episode lifecycle operations
//!
//! This benchmark suite measures performance across the complete episode lifecycle:
//! - Episode creation (start_episode)
//! - Step logging with varying step counts (log_step)
//! - Episode completion and analysis (complete_episode)
//! - Full lifecycle operations (create → log steps → complete)
//! - Episode retrieval (retrieve_relevant_context)
//! - Pattern extraction and scoring performance

use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, Criterion,
};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    generate_large_episode_description, generate_many_execution_steps, setup_temp_memory,
    setup_temp_turso_memory,
};
use memory_core::types::{TaskOutcome, TaskType};

/// Benchmark episode creation performance (start_episode)
///
/// Measures the time to create new episodes with realistic task descriptions
/// and context information.
fn benchmark_episode_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_creation");
    group.sample_size(50);

    for batch_size in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    for i in 0..size {
                        let episode_id = memory
                            .start_episode(
                                generate_episode_description(i),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        black_box(episode_id);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark step logging performance with varying step counts (log_step)
///
/// Measures the performance impact of logging different numbers of execution steps
/// per episode, including both small and large step counts.
fn benchmark_step_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("step_logging");
    group.sample_size(30);

    for step_count in [1, 5, 10, 25, 50, 100].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    let episode_id = memory
                        .start_episode(
                            generate_large_episode_description(0),
                            context,
                            TaskType::CodeGeneration,
                        )
                        .await;

                    let steps = generate_execution_steps(count);
                    for step in steps {
                        memory.log_step(episode_id, step).await;
                    }

                    black_box(episode_id);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark episode completion performance (complete_episode)
///
/// Measures the time to complete episodes, including reward calculation,
/// reflection generation, and pattern extraction.
fn benchmark_episode_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_completion");
    group.sample_size(30);

    for step_count in [5, 25, 50].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    let episode_id = memory
                        .start_episode(
                            generate_large_episode_description(0),
                            context,
                            TaskType::CodeGeneration,
                        )
                        .await;

                    let steps = generate_execution_steps(count);
                    for step in steps {
                        memory.log_step(episode_id, step).await;
                    }

                    memory
                        .complete_episode(
                            episode_id,
                            TaskOutcome::Success {
                                verdict:
                                    "Episode completed successfully with comprehensive analysis"
                                        .to_string(),
                                artifacts: vec![
                                    "result.txt".to_string(),
                                    "analysis.json".to_string(),
                                ],
                            },
                        )
                        .await
                        .expect("Failed to complete episode");

                    black_box(episode_id);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full lifecycle operations (create → log steps → complete)
///
/// Measures end-to-end performance of complete episode lifecycles with
/// realistic workloads and varying complexity.
fn benchmark_full_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_lifecycle");
    group.sample_size(20);

    for (episode_count, steps_per_episode) in [(1, 10), (5, 25), (10, 50)].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::new(
                format!("{}_episodes_{}_steps", episode_count, steps_per_episode),
                episode_count,
            ),
            &(episode_count, steps_per_episode),
            |b, &(episodes, steps)| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    for i in 0..*episodes {
                        let episode_id = memory
                            .start_episode(
                                generate_large_episode_description(0),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(*steps);
                        for step in &steps {
                            memory.log_step(episode_id, step.clone()).await;
                        }

                        memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!(
                                        "Episode {} completed with {} steps",
                                        i,
                                        steps.len()
                                    ),
                                    artifacts: vec![format!("episode_{}_result.txt", i)],
                                },
                            )
                            .await
                            .expect("Failed to complete episode");

                        black_box(episode_id);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark episode retrieval performance (retrieve_relevant_context)
///
/// Measures performance of retrieving relevant episodes from memory,
/// including filtering, ranking, and limiting results.
fn benchmark_episode_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval");
    group.sample_size(50);

    // Pre-populate memory with episodes for retrieval testing

    for total_episodes in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(total_episodes),
            total_episodes,
            |b, &episode_count| {
                b.to_async(FuturesExecutor).iter_custom(|iters| async move {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    // Pre-populate with episodes
                    for i in 0..episode_count {
                        let episode_id = memory
                            .start_episode(
                                generate_episode_description(i),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(5);
                        for step in steps {
                            memory.log_step(episode_id, step).await;
                        }

                        memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!("Episode {} completed", i),
                                    artifacts: vec![],
                                },
                            )
                            .await
                            .expect("Failed to complete episode");
                    }

                    // Now benchmark retrieval
                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        let results = memory
                            .retrieve_relevant_context(
                                "Implement feature with processing".to_string(),
                                context.clone(),
                                black_box(10),
                            )
                            .await;

                        black_box(results);
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark episode scoring and pattern extraction performance
///
/// Measures the performance of reward calculation, reflection generation,
/// and pattern extraction during episode completion.
fn benchmark_scoring_and_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("scoring_and_patterns");
    group.sample_size(30);

    for step_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    let episode_id = memory
                        .start_episode(
                            generate_large_episode_description(0),
                            context,
                            TaskType::CodeGeneration,
                        )
                        .await;

                    let steps = generate_execution_steps(count);
                    for step in steps {
                        memory.log_step(episode_id, step).await;
                    }

                    // This will trigger scoring, reflection, and pattern extraction
                    memory
                        .complete_episode(
                            episode_id,
                            TaskOutcome::Success {
                                verdict: format!("Complex episode with {} steps completed", count),
                                artifacts: vec![
                                    "comprehensive_result.txt".to_string(),
                                    "detailed_analysis.json".to_string(),
                                    "performance_metrics.csv".to_string(),
                                ],
                            },
                        )
                        .await
                        .expect("Failed to complete episode");

                    black_box(episode_id);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent episode operations
///
/// Measures performance when multiple episodes are being processed concurrently,
/// simulating real-world usage patterns.
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.sample_size(20);

    for concurrent_episodes in [5, 10, 20].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(concurrent_episodes),
            concurrent_episodes,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();

                    let mut handles = Vec::new();

                    for i in 0..count {
                        let memory_clone = memory.clone();
                        let context_clone = context.clone();

                        let handle = tokio::spawn(async move {
                            let episode_id = memory_clone
                                .start_episode(
                                    generate_episode_description(i),
                                    context_clone,
                                    TaskType::CodeGeneration,
                                )
                                .await;

                            let steps = generate_execution_steps(10);
                            for step in &steps {
                                memory_clone.log_step(episode_id, step.clone()).await;
                            }

                            memory_clone
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Concurrent episode {} completed", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");

                            episode_id
                        });

                        handles.push(handle);
                    }

                    for handle in handles {
                        let episode_id = handle.await.expect("Task failed");
                        black_box(episode_id);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory pressure scenarios
///
/// Tests performance under memory pressure with large episodes and many operations.
fn benchmark_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");
    group.sample_size(10);

    group.bench_function("large_episode_lifecycle", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_memory().await;
            let context = create_benchmark_context();

            let episode_id = memory
                .start_episode(
                    generate_large_episode_description(0),
                    context,
                    TaskType::CodeGeneration,
                )
                .await;

            // Generate a very large number of steps
            let steps = generate_many_execution_steps(500);
            for step in steps {
                memory.log_step(episode_id, step).await;
            }

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Large episode with 500 steps completed successfully".to_string(),
                        artifacts: vec![
                            "large_result.txt".to_string(),
                            "comprehensive_analysis.json".to_string(),
                            "performance_report.pdf".to_string(),
                            "debug_logs.txt".to_string(),
                        ],
                    },
                )
                .await
                .expect("Failed to complete episode");

            black_box(episode_id);
        });
    });

    group.finish();
}

/// Benchmark storage backend performance comparison
///
/// Compares performance between different storage backends (redb vs Turso).
fn benchmark_storage_backends(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_backends");
    group.sample_size(20);

    // Benchmark redb storage
    group.bench_function("redb_lifecycle", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_memory().await;
            let context = create_benchmark_context();

            let episode_id = memory
                .start_episode(
                    generate_episode_description(0),
                    context,
                    TaskType::CodeGeneration,
                )
                .await;

            let steps = generate_execution_steps(10);
            for step in steps {
                memory.log_step(episode_id, step).await;
            }

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Episode completed with redb storage".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .expect("Failed to complete episode");

            black_box(episode_id);
        });
    });

    // Benchmark Turso storage (if available)
    group.bench_function("turso_lifecycle", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_turso_memory().await;
            let context = create_benchmark_context();

            let episode_id = memory
                .start_episode(
                    generate_episode_description(0),
                    context,
                    TaskType::CodeGeneration,
                )
                .await;

            let steps = generate_execution_steps(10);
            for step in steps {
                memory.log_step(episode_id, step).await;
            }

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Episode completed with Turso storage".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .expect("Failed to complete episode");

            black_box(episode_id);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_episode_creation,
    benchmark_step_logging,
    benchmark_episode_completion,
    benchmark_full_lifecycle,
    benchmark_episode_retrieval,
    benchmark_scoring_and_patterns,
    benchmark_concurrent_operations,
    benchmark_memory_pressure,
    benchmark_storage_backends
);
criterion_main!(benches);
