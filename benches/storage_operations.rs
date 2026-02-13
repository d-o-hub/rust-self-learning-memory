//! Benchmarks for storage operations

#![allow(clippy::excessive_nesting)]
#![allow(deprecated)]
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_benches::TokioExecutor;
use memory_core::types::{TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use std::collections::HashMap;
use std::hint::black_box;
use uuid::Uuid;

fn benchmark_hashmap_storage(c: &mut Criterion) {
    c.bench_function("hashmap_storage", |b| {
        b.iter(|| {
            let mut storage = HashMap::new();

            // Simulate storing episode data
            for i in 0..black_box(100) {
                let key = Uuid::new_v4().to_string();
                let value = format!("episode_data_{}_with_steps_and_metadata", i);
                storage.insert(key, value);
            }

            black_box(storage.len());
        });
    });
}

fn benchmark_vector_storage(c: &mut Criterion) {
    c.bench_function("vector_storage", |b| {
        b.iter(|| {
            let mut storage = Vec::new();

            // Simulate storing multiple episodes
            for i in 0..black_box(1000) {
                let episode = format!("episode_{}_with_complete_lifecycle_data", i);
                storage.push(episode);
            }

            // Simulate retrieval operations
            let retrieved: Vec<_> = storage
                .iter()
                .filter(|ep| ep.contains("lifecycle"))
                .take(50)
                .collect();

            black_box(retrieved.len());
        });
    });
}

fn benchmark_bulk_episode_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_episode_operations");
    group.sample_size(10);

    for episode_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                #[allow(clippy::excessive_nesting)]
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();
                    let mut episode_ids = Vec::new();

                    // Create and complete episodes
                    for i in 0..count {
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
                                    verdict: format!("Bulk episode {} completed", i),
                                    artifacts: vec![],
                                },
                            )
                            .await
                            .expect("Failed to complete episode");

                        episode_ids.push(episode_id);
                    }

                    black_box(episode_ids.len());
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_operations", |b| {
        b.iter(|| {
            #[allow(clippy::excessive_nesting)]
            rt.block_on(async {
                // Simulate concurrent storage operations
                let mut handles = Vec::new();
                for i in 0..black_box(10) {
                    let handle = tokio::spawn(async move {
                        let mut local_data = Vec::new();
                        for j in 0..50 {
                            local_data.push(format!("concurrent_item_{}_{}", i, j));
                        }
                        local_data
                    });
                    handles.push(handle);
                }

                let mut all_data = Vec::new();
                for handle in handles {
                    let data = handle.await.expect("Task failed");
                    all_data.extend(data);
                }

                black_box(all_data.len());
            });
        });
    });
}

fn benchmark_redb_episode_retrieval(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("redb_episode_retrieval", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (memory, _temp_dir) = setup_temp_memory().await;
                let context = create_benchmark_context();
                let results = memory
                    .retrieve_relevant_context(
                        "Implement feature with processing".to_string(),
                        context,
                        black_box(10),
                    )
                    .await;

                black_box(results.len());
            });
        });
    });
}

fn benchmark_storage_initialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("redb_storage_init", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
            let db_path = temp_dir.path().join("init_benchmark.redb");

            let storage = rt.block_on(async {
                black_box(
                    RedbStorage::new(&db_path)
                        .await
                        .expect("Failed to create storage"),
                )
            });

            // Force drop to clean up
            drop(storage);
            drop(temp_dir);
        });
    });
}

criterion_group!(
    benches,
    benchmark_hashmap_storage,
    benchmark_vector_storage,
    benchmark_concurrent_operations,
    benchmark_bulk_episode_operations,
    benchmark_redb_episode_retrieval,
    benchmark_storage_initialization
);
criterion_main!(benches);
