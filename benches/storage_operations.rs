//! Benchmarks for storage operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;
use test_utils::*;

fn create_bench_storage() -> (TursoStorage, TempDir) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("bench.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let storage = TursoStorage::from_database(db).unwrap();
        storage.initialize_schema().await.unwrap();
        (storage, dir)
    })
}

fn benchmark_store_episode(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (storage, _dir) = create_bench_storage();

    c.bench_function("store_episode", |b| {
        b.to_async(&runtime).iter(|| async {
            let episode = create_completed_episode("Benchmark", true);
            storage.store_episode(black_box(&episode)).await.unwrap();
        });
    });
}

fn benchmark_retrieve_episode(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (storage, _dir) = create_bench_storage();

    // Pre-populate with episodes
    runtime.block_on(async {
        for i in 0..100 {
            let episode = create_test_episode(&format!("Task {}", i));
            storage.store_episode(&episode).await.unwrap();
        }
    });

    c.bench_function("retrieve_episode", |b| {
        b.to_async(&runtime).iter(|| async {
            let episodes = create_test_episodes(1, "test");
            let id = episodes[0].episode_id;
            storage.get_episode(black_box(id)).await.unwrap();
        });
    });
}

fn benchmark_query_episodes(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("query_episodes_by_dataset_size");

    for dataset_size in [10, 100, 1000].iter() {
        let (storage, _dir) = create_bench_storage();

        // Pre-populate
        runtime.block_on(async {
            for i in 0..*dataset_size {
                let domain = if i % 2 == 0 { "web-api" } else { "cli-tool" };
                let context = create_test_context(domain, Some("rust"));
                let episode = create_test_episode_with_context(
                    &format!("Task {}", i),
                    context,
                    memory_core::TaskType::Testing,
                );
                storage.store_episode(&episode).await.unwrap();
            }
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(dataset_size),
            dataset_size,
            |b, _| {
                b.to_async(&runtime).iter(|| async {
                    let query = memory_storage_turso::EpisodeQuery {
                        domain: Some("web-api".to_string()),
                        limit: Some(10),
                        ..Default::default()
                    };
                    storage.query_episodes(black_box(&query)).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_writes(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_writes");

    // Only testing single write for now due to SQLite write locking limitations in local benchmarks
    for concurrency in [1].iter() {
        let (storage, _dir) = create_bench_storage();
        let storage = std::sync::Arc::new(storage);

        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &count| {
                b.to_async(&runtime).iter(|| async {
                    let handles: Vec<_> = (0..count)
                        .map(|i| {
                            let storage = storage.clone();
                            tokio::spawn(async move {
                                let episode = create_test_episode(&format!("Task {}", i));
                                storage.store_episode(&episode).await
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.await.unwrap().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_store_episode,
    benchmark_retrieve_episode,
    benchmark_query_episodes,
    benchmark_concurrent_writes
);
criterion_main!(benches);
