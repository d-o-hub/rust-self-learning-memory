//! Multi-backend comparison benchmarks
//!
//! Compares performance across different storage backends:
//! - redb (embedded)
//! - Turso/libSQL (cloud)
//! - Future: other backends
//!
//! Following patterns from rust-storage-bench for fair comparisons.

use criterion::{
    async_executor::TokioExecutor, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
};
use memory_core::types::{StorageConfig, TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use memory_storage_turso::TursoStorage;
use std::path::Path;
use tempfile::TempDir;

#[derive(Debug, Clone)]
enum StorageBackend {
    Redb,
    Turso,
}

impl StorageBackend {
    fn name(&self) -> &'static str {
        match self {
            Self::Redb => "redb",
            Self::Turso => "turso",
        }
    }

    async fn setup_memory(&self) -> (memory_core::memory::SelfLearningMemory, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let storage_config = match self {
            Self::Redb => {
                let db_path = temp_dir.path().join("comparison.redb");
                StorageConfig::Redb {
                    path: db_path.to_string_lossy().to_string(),
                }
            }
            Self::Turso => {
                // For comparison, use a local SQLite file to simulate Turso
                // In production, this would use actual Turso connection
                let db_path = temp_dir.path().join("comparison.db");
                StorageConfig::Turso {
                    url: format!("file:{}", db_path.to_string_lossy()),
                    auth_token: None,
                }
            }
        };

        let memory_config = memory_core::types::MemoryConfig {
            storage: storage_config,
            ..Default::default()
        };

        let storage: Box<dyn memory_core::storage::StorageBackend> = match self {
            Self::Redb => {
                let db_path = temp_dir.path().join("comparison.redb");
                Box::new(RedbStorage::new(&db_path).expect("Failed to create redb storage"))
            }
            Self::Turso => {
                let db_path = temp_dir.path().join("comparison.db");
                Box::new(
                    TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), None)
                        .await
                        .expect("Failed to create turso storage"),
                )
            }
        };

        let memory = memory_core::memory::SelfLearningMemory::with_storage(storage, memory_config)
            .await
            .expect("Failed to create memory");

        (memory, temp_dir)
    }
}

async fn run_backend_comparison(
    backend: StorageBackend,
    operation: &str,
    operation_fn: impl Fn(
        memory_core::memory::SelfLearningMemory,
    ) -> futures::future::BoxFuture<'static, ()>,
) {
    let (memory, _temp_dir) = backend.setup_memory().await;
    operation_fn(memory).await;
}

fn benchmark_backend_write_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_write_performance");
    group.sample_size(20);

    let backends = vec![StorageBackend::Redb, StorageBackend::Turso];

    for backend in backends {
        group.bench_with_input(
            BenchmarkId::new("single_episode_write", backend.name()),
            &backend,
            |b, backend| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = backend.setup_memory().await;
                    let context = create_benchmark_context();

                    let episode_id = memory
                        .start_episode(
                            generate_episode_description(1),
                            context,
                            TaskType::CodeGeneration,
                        )
                        .await
                        .expect("Failed to create episode");

                    let steps = generate_execution_steps(3);
                    for step in steps {
                        memory
                            .log_step(episode_id, step)
                            .await
                            .expect("Failed to log step");
                    }

                    memory
                        .complete_episode(
                            episode_id,
                            TaskOutcome::Success {
                                verdict: "Backend write test".to_string(),
                                artifacts: vec![],
                            },
                        )
                        .await
                        .expect("Failed to complete episode");
                });
            },
        );
    }

    group.finish();
}

fn benchmark_backend_read_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_read_performance");
    group.sample_size(20);

    let backends = vec![StorageBackend::Redb, StorageBackend::Turso];

    for backend in backends {
        group.bench_with_input(
            BenchmarkId::new("episode_retrieval", backend.name()),
            &backend,
            |b, backend| {
                b.to_async(TokioExecutor).iter_custom(|iters| async move {
                    let mut total_time = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        let (memory, _temp_dir) = backend.setup_memory().await;
                        let context = create_benchmark_context();

                        // Pre-populate with data
                        for i in 0..10 {
                            let episode_id = memory
                                .start_episode(
                                    generate_episode_description(i),
                                    context.clone(),
                                    TaskType::CodeGeneration,
                                )
                                .await
                                .expect("Failed to create episode");

                            let steps = generate_execution_steps(2);
                            for step in steps {
                                memory
                                    .log_step(episode_id, step)
                                    .await
                                    .expect("Failed to log step");
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Read test episode {}", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");
                        }

                        // Time the read operation
                        let start = std::time::Instant::now();
                        let results = memory
                            .retrieve_relevant_context(
                                "Backend read performance test".to_string(),
                                context,
                                5,
                            )
                            .await
                            .expect("Failed to retrieve context");

                        total_time += start.elapsed();
                        criterion::black_box(results.len());
                    }

                    total_time
                });
            },
        );
    }

    group.finish();
}

fn benchmark_backend_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_bulk_operations");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    let backends = vec![StorageBackend::Redb, StorageBackend::Turso];
    let bulk_sizes = vec![10, 50, 100];

    for backend in backends {
        for &bulk_size in &bulk_sizes {
            group.bench_with_input(
                BenchmarkId::new(format!("bulk_write_{}", bulk_size), backend.name()),
                &(backend.clone(), bulk_size),
                |b, (backend, &size)| {
                    b.to_async(TokioExecutor).iter(|| async {
                        let (memory, _temp_dir) = backend.setup_memory().await;
                        let context = create_benchmark_context();

                        // Bulk write operations
                        let mut episode_ids = vec![];

                        for i in 0..size {
                            let episode_id = memory
                                .start_episode(
                                    generate_episode_description(i),
                                    context.clone(),
                                    TaskType::CodeGeneration,
                                )
                                .await
                                .expect("Failed to create episode");

                            let steps = generate_execution_steps(2);
                            for step in steps {
                                memory
                                    .log_step(episode_id, step)
                                    .await
                                    .expect("Failed to log step");
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Bulk write {}", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");

                            episode_ids.push(episode_id);
                        }

                        criterion::black_box(episode_ids.len());
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_backend_concurrent_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_concurrent_performance");
    group.sample_size(5);
    group.measurement_time(std::time::Duration::from_secs(45));

    let backends = vec![StorageBackend::Redb, StorageBackend::Turso];
    let concurrency_levels = vec![2, 4, 8];

    for backend in backends {
        for &concurrency in &concurrency_levels {
            group.bench_with_input(
                BenchmarkId::new(format!("concurrent_{}", concurrency), backend.name()),
                &(backend.clone(), concurrency),
                |b, (backend, &concurrency)| {
                    b.to_async(TokioExecutor).iter(|| async {
                        let (memory, _temp_dir) = backend.setup_memory().await;
                        let memory = std::sync::Arc::new(memory);

                        // Pre-populate with some data
                        let context = create_benchmark_context();
                        for i in 0..20 {
                            let episode_id = memory
                                .start_episode(
                                    generate_episode_description(i),
                                    context.clone(),
                                    TaskType::CodeGeneration,
                                )
                                .await
                                .expect("Failed to create episode");

                            let steps = generate_execution_steps(1);
                            for step in steps {
                                memory
                                    .log_step(episode_id, step)
                                    .await
                                    .expect("Failed to log step");
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Concurrent setup {}", i),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");
                        }

                        // Run concurrent operations
                        let mut handles = vec![];

                        for thread_id in 0..concurrency {
                            let memory = memory.clone();
                            let context = context.clone();

                            let handle = tokio::spawn(async move {
                                for i in 0..10 {
                                    // Mix of reads and writes
                                    if i % 2 == 0 {
                                        let _results = memory
                                            .retrieve_relevant_context(
                                                format!("Concurrent read {}:{}", thread_id, i),
                                                context.clone(),
                                                3,
                                            )
                                            .await
                                            .expect("Failed to retrieve context");
                                    } else {
                                        let episode_id = memory
                                            .start_episode(
                                                format!("Concurrent write {}:{}", thread_id, i),
                                                context.clone(),
                                                TaskType::CodeGeneration,
                                            )
                                            .await
                                            .expect("Failed to create episode");

                                        memory
                                            .complete_episode(
                                                episode_id,
                                                TaskOutcome::Success {
                                                    verdict: format!(
                                                        "Concurrent write {}:{}",
                                                        thread_id, i
                                                    ),
                                                    artifacts: vec![],
                                                },
                                            )
                                            .await
                                            .expect("Failed to complete episode");
                                    }
                                }
                            });

                            handles.push(handle);
                        }

                        // Wait for all concurrent operations
                        futures::future::join_all(handles).await;
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_backend_storage_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_storage_efficiency");
    group.sample_size(10);

    let backends = vec![StorageBackend::Redb, StorageBackend::Turso];
    let dataset_sizes = vec![100, 500];

    for backend in backends {
        for &size in &dataset_sizes {
            group.bench_with_input(
                BenchmarkId::new(format!("storage_efficiency_{}", size), backend.name()),
                &(backend.clone(), size),
                |b, (backend, &dataset_size)| {
                    b.to_async(TokioExecutor).iter_custom(|iters| async move {
                        let mut total_bytes = 0u64;

                        for _ in 0..iters {
                            let (memory, temp_dir) = backend.setup_memory().await;
                            let context = create_benchmark_context();

                            // Create dataset
                            for i in 0..dataset_size {
                                let episode_id = memory
                                    .start_episode(
                                        generate_episode_description(i),
                                        context.clone(),
                                        TaskType::CodeGeneration,
                                    )
                                    .await
                                    .expect("Failed to create episode");

                                let steps = generate_execution_steps(2);
                                for step in steps {
                                    memory
                                        .log_step(episode_id, step)
                                        .await
                                        .expect("Failed to log step");
                                }

                                memory
                                    .complete_episode(
                                        episode_id,
                                        TaskOutcome::Success {
                                            verdict: format!("Storage efficiency test {}", i),
                                            artifacts: vec![format!("file_{}.txt", i)],
                                        },
                                    )
                                    .await
                                    .expect("Failed to complete episode");
                            }

                            // Force flush/sync
                            memory.sync().await.expect("Failed to sync");

                            // Measure storage size
                            let storage_size =
                                fs_extra::dir::get_size(temp_dir.path()).unwrap_or(0);
                            total_bytes += storage_size;
                        }

                        // Return storage size as duration for Criterion
                        std::time::Duration::from_nanos(total_bytes)
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_backend_write_performance,
    benchmark_backend_read_performance,
    benchmark_backend_bulk_operations,
    benchmark_backend_concurrent_performance,
    benchmark_backend_storage_efficiency
);
criterion_main!(benches);
