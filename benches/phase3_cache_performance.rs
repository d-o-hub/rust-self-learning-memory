//! Phase 3 Cache Performance Benchmarks
//!
//! Measures the performance improvements from:
//! - CachedTursoStorage (adaptive cache)
//! - PreparedStatementCache
//! - Batch operations

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memory_core::{Episode, Pattern, PatternId, StorageBackend, TaskContext, TaskType};
use memory_storage_turso::{CacheConfig, TursoStorage};
use std::hint::black_box;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Helper to create test storage
async fn create_test_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("bench.db");
    let storage = TursoStorage::new(&format!("file:{}", db_path.display()), "")
        .await
        .unwrap();
    storage.initialize_schema().await.unwrap();
    (storage, dir)
}

/// Create sample episodes for benchmarking
fn create_episodes(count: usize) -> Vec<Episode> {
    (0..count)
        .map(|i| {
            Episode::new(
                format!("Task {}", i),
                TaskContext::default(),
                TaskType::CodeGeneration,
            )
        })
        .collect()
}

/// Create sample patterns for benchmarking
fn create_patterns(count: usize) -> Vec<Pattern> {
    (0..count)
        .map(|i| Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: format!("condition_{}", i),
            action: format!("action_{}", i),
            outcome_stats: memory_core::types::OutcomeStats {
                success_count: 5,
                failure_count: 1,
                total_count: 6,
                avg_duration_secs: 0.5,
            },
            context: TaskContext::default(),
            effectiveness: memory_core::pattern::PatternEffectiveness::default(),
        })
        .collect()
}

/// Benchmark: Cached vs Uncached Episode Retrieval
fn bench_cache_episode_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (storage, _dir) = rt.block_on(create_test_storage());

    // Pre-populate with episodes
    let episodes = create_episodes(100);
    let episode_ids: Vec<_> = episodes.iter().map(|e| e.episode_id).collect();

    for episode in &episodes {
        rt.block_on(storage.store_episode(episode)).unwrap();
    }

    let mut group = c.benchmark_group("cache_episode_retrieval");

    // Benchmark uncached retrieval
    group.bench_function("uncached", |b| {
        b.iter(|| {
            for id in &episode_ids[0..10] {
                let _ = rt.block_on(storage.get_episode(black_box(*id)));
            }
        });
    });

    // Benchmark cached retrieval
    let cached = storage.with_cache_default();

    // Warm up the cache
    for id in &episode_ids[0..10] {
        let _ = rt.block_on(cached.get_episode(*id));
    }

    group.bench_function("cached", |b| {
        b.iter(|| {
            for id in &episode_ids[0..10] {
                let _ = rt.block_on(cached.get_episode(black_box(*id)));
            }
        });
    });

    // Report cache stats
    let stats = cached.stats();
    println!("\nCache Stats:");
    println!("  Hit rate: {:.2}%", stats.episode_hit_rate() * 100.0);
    println!(
        "  Hits: {}, Misses: {}",
        stats.episode_hits, stats.episode_misses
    );

    group.finish();
}

/// Benchmark: Cache Hit Rate Under Load
fn bench_cache_hit_rate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (storage, _dir) = rt.block_on(create_test_storage());

    // Pre-populate with episodes
    let episodes = create_episodes(1000);
    for episode in &episodes {
        rt.block_on(storage.store_episode(episode)).unwrap();
    }

    let cached = storage.with_cache(CacheConfig {
        max_episodes: 100,
        ..Default::default()
    });

    let mut group = c.benchmark_group("cache_hit_rate");
    group.sample_size(50);

    // Benchmark with varying working set sizes
    for working_set_size in [10, 50, 100, 200].iter() {
        let episode_ids: Vec<_> = episodes[0..*working_set_size]
            .iter()
            .map(|e| e.episode_id)
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(working_set_size),
            working_set_size,
            |b, _| {
                b.iter(|| {
                    for id in &episode_ids {
                        let _ = rt.block_on(cached.get_episode(black_box(*id)));
                    }
                });
            },
        );
    }

    let stats = cached.stats();
    println!("\nFinal Cache Stats:");
    println!("  Overall hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!(
        "  Episode hit rate: {:.2}%",
        stats.episode_hit_rate() * 100.0
    );

    group.finish();
}

/// Benchmark: Batch vs Individual Episode Storage
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_operations");
    group.sample_size(20);

    for size in [10, 50, 100].iter() {
        let (storage, _dir) = rt.block_on(create_test_storage());

        // Benchmark individual stores
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("individual", size), size, |b, &size| {
            b.iter(|| {
                let episodes = create_episodes(size);
                for episode in episodes {
                    let _ = rt.block_on(storage.store_episode(&episode));
                }
            });
        });

        // Benchmark batch stores
        let (storage, _dir) = rt.block_on(create_test_storage());
        group.bench_with_input(BenchmarkId::new("batch", size), size, |b, &size| {
            b.iter(|| {
                let episodes = create_episodes(size);
                let _ = rt.block_on(storage.store_episodes_batch(black_box(episodes)));
            });
        });
    }

    group.finish();
}

/// Benchmark: Prepared Statement Cache Performance
fn bench_prepared_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (storage, _dir) = rt.block_on(create_test_storage());

    // Pre-populate to trigger repeated queries
    let episodes = create_episodes(100);
    for episode in &episodes {
        rt.block_on(storage.store_episode(episode)).unwrap();
    }

    let mut group = c.benchmark_group("prepared_cache");

    group.bench_function("query_with_cache", |b| {
        b.iter(|| {
            for episode in &episodes[0..20] {
                let _ = rt.block_on(storage.get_episode(black_box(episode.episode_id)));
            }
        });
    });

    // Report prepared cache stats
    let stats = storage.prepared_cache_stats();
    println!("\nPrepared Cache Stats:");
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("  Hits: {}, Misses: {}", stats.hits, stats.misses);
    println!("  Cache size: {}", stats.current_size);
    println!("  Avg prep time: {:.2}µs", stats.avg_preparation_time_us);

    group.finish();
}

/// Benchmark: Pattern Cache Performance
fn bench_pattern_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (storage, _dir) = rt.block_on(create_test_storage());

    // Pre-populate with patterns
    let patterns = create_patterns(100);
    let pattern_ids: Vec<_> = patterns.iter().map(|p| p.id()).collect();

    for pattern in &patterns {
        rt.block_on(storage.store_pattern(pattern)).unwrap();
    }

    let cached = storage.with_cache_default();

    let mut group = c.benchmark_group("pattern_cache");

    // Warm up cache
    for id in &pattern_ids[0..10] {
        let _ = rt.block_on(cached.get_pattern(*id));
    }

    group.bench_function("cached_retrieval", |b| {
        b.iter(|| {
            for id in &pattern_ids[0..10] {
                let _ = rt.block_on(cached.get_pattern(black_box(*id)));
            }
        });
    });

    let stats = cached.stats();
    println!("\nPattern Cache Stats:");
    println!("  Hit rate: {:.2}%", stats.pattern_hit_rate() * 100.0);
    println!(
        "  Hits: {}, Misses: {}",
        stats.pattern_hits, stats.pattern_misses
    );

    group.finish();
}

/// Benchmark: Batch Query Operations
fn bench_batch_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (storage, _dir) = rt.block_on(create_test_storage());

    // Pre-populate
    let episodes = create_episodes(100);
    let episode_ids: Vec<_> = episodes.iter().map(|e| e.episode_id).collect();

    for episode in &episodes {
        rt.block_on(storage.store_episode(episode)).unwrap();
    }

    let mut group = c.benchmark_group("batch_queries");

    for size in [10, 50].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Individual queries
        group.bench_with_input(BenchmarkId::new("individual", size), size, |b, &size| {
            let ids = &episode_ids[0..size];
            b.iter(|| {
                for id in ids {
                    let _ = rt.block_on(storage.get_episode(black_box(*id)));
                }
            });
        });

        // Batch query
        group.bench_with_input(BenchmarkId::new("batch", size), size, |b, &size| {
            let ids = &episode_ids[0..size];
            b.iter(|| {
                let _ = rt.block_on(storage.get_episodes_batch(black_box(ids)));
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = phase3_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_cache_episode_retrieval,
        bench_cache_hit_rate,
        bench_batch_operations,
        bench_prepared_cache,
        bench_pattern_cache,
        bench_batch_queries
}

criterion_main!(phase3_benches);
