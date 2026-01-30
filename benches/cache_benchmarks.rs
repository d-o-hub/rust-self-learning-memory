//! Cache Performance Benchmarks for TursoStorage
//!
//! Benchmarks comparing cached vs non-cached storage operations.
//! Measures latency improvements, hit rates, and memory usage.
//!
//! Run with: `cargo bench --bench cache_benchmarks`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_core::{Episode, Evidence, Heuristic, Pattern, TaskContext, TaskType};
use memory_storage_turso::{CacheConfig, CachedTursoStorage, TursoConfig, TursoStorage};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Create a tokio runtime for the benchmarks
fn rt() -> &'static Runtime {
    static RUNTIME: once_cell::sync::Lazy<Runtime> =
        once_cell::sync::Lazy::new(|| Runtime::new().expect("Failed to create runtime"));
    &RUNTIME
}

/// Create a test Turso storage instance
fn create_test_turso_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("benchmark.db");

    let storage = rt().block_on(async {
        TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
            .await
            .expect("Failed to create turso storage")
    });

    rt().block_on(async {
        storage
            .initialize_schema()
            .await
            .expect("Failed to initialize schema")
    });

    (storage, dir)
}

/// Create a test episode with realistic data
fn create_test_episode(id: Uuid) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Benchmark episode {}", id),
        context: TaskContext {
            domain: "benchmark".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: memory_core::types::ComplexityLevel::Moderate,
            tags: vec!["performance".to_string(), "test".to_string()],
        },
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        start_time: chrono::Utc::now(),
        end_time: None,
        metadata: std::collections::HashMap::new(),
        tags: Vec::new(),
    }
}

/// Create a test pattern
fn create_test_pattern(id: Uuid) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec![
            "tool1".to_string(),
            "tool2".to_string(),
            "tool3".to_string(),
        ],
        context: TaskContext {
            domain: "benchmark".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.85,
        avg_latency: chrono::Duration::milliseconds(50),
        occurrence_count: 25,
        effectiveness: Default::default(),
    }
}

/// Create a test heuristic
fn create_test_heuristic(id: Uuid) -> Heuristic {
    Heuristic {
        heuristic_id: id,
        condition: "benchmark_condition".to_string(),
        action: "benchmark_action".to_string(),
        confidence: 0.8,
        evidence: Evidence {
            episode_ids: vec![],
            success_rate: 0.8,
            sample_size: 15,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// ========== Episode Benchmarks ==========

/// Benchmark cached episode retrieval with 10 episodes
fn bench_episode_retrieval_cached_10(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_cached");
    group.sample_size(30);

    group.bench_function("10_episodes", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with 10 episodes
            let ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let episode = create_test_episode(*id);
                    cached.store_episode_cached(&episode).await.unwrap();
                }

                // Prime the cache
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                }

                // Benchmark cached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

/// Benchmark cached episode retrieval with 50 episodes
fn bench_episode_retrieval_cached_50(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_cached_50");
    group.sample_size(30);

    group.bench_function("50_episodes", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with 50 episodes
            let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let episode = create_test_episode(*id);
                    cached.store_episode_cached(&episode).await.unwrap();
                }

                // Prime the cache
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                }

                // Benchmark cached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

/// Benchmark cached episode retrieval with 100 episodes
fn bench_episode_retrieval_cached_100(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_cached_100");
    group.sample_size(30);

    group.bench_function("100_episodes", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with 100 episodes
            let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let episode = create_test_episode(*id);
                    cached.store_episode_cached(&episode).await.unwrap();
                }

                // Prime the cache
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                }

                // Benchmark cached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

/// Benchmark uncached episode retrieval for comparison
fn bench_episode_retrieval_uncached(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_uncached");
    group.sample_size(30);

    group.bench_function("100_episodes", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();

            // Pre-populate without cache
            let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let episode = create_test_episode(*id);
                    storage.store_episode(&episode).await.unwrap();
                }

                // Benchmark uncached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = storage.get_episode(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

/// Benchmark episode write operations
fn bench_episode_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_write");
    group.sample_size(30);

    group.bench_function("10_episodes", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            rt().block_on(async {
                for i in 0..10 {
                    let episode = create_test_episode(Uuid::new_v4());
                    cached.store_episode_cached(&episode).await.unwrap();
                    black_box(i);
                }
            });
        });
    });

    group.finish();
}

// ========== Pattern Benchmarks ==========

/// Benchmark cached pattern retrieval
fn bench_pattern_retrieval_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_retrieval_cached");
    group.sample_size(30);

    group.bench_function("50_patterns", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with 50 patterns
            let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let pattern = create_test_pattern(*id);
                    cached.store_pattern_cached(&pattern).await.unwrap();
                }

                // Prime the cache
                for id in &ids {
                    let _ = cached.get_pattern_cached(*id).await.unwrap();
                }

                // Benchmark cached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = cached.get_pattern_cached(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

// ========== Heuristic Benchmarks ==========

/// Benchmark cached heuristic retrieval
fn bench_heuristic_retrieval_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("heuristic_retrieval_cached");
    group.sample_size(30);

    group.bench_function("50_heuristics", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with 50 heuristics
            let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
            rt().block_on(async {
                for id in &ids {
                    let heuristic = create_test_heuristic(*id);
                    cached.store_heuristic_cached(&heuristic).await.unwrap();
                }

                // Prime the cache
                for id in &ids {
                    let _ = cached.get_heuristic_cached(*id).await.unwrap();
                }

                // Benchmark cached retrieval
                let start = std::time::Instant::now();
                for id in &ids {
                    let _ = cached.get_heuristic_cached(*id).await.unwrap();
                    black_box(id);
                }
                black_box(start.elapsed());
            });
        });
    });

    group.finish();
}

// ========== Cache Statistics Benchmarks ==========

/// Benchmark cache statistics calculation
fn bench_cache_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_stats");
    group.sample_size(50);

    group.bench_function("stats_calculation", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            rt().block_on(async {
                for _ in 0..100 {
                    let episode = create_test_episode(Uuid::new_v4());
                    cached.store_episode_cached(&episode).await.unwrap();
                    let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
                    // Access again for hit
                    let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
                }

                let stats = cached.stats();
                black_box(stats.hit_rate());
            });
        });
    });

    group.finish();
}

/// Benchmark cache hit rate calculation
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");
    group.sample_size(50);

    group.bench_function("single_access", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with episodes
            let episode_count = 50;
            let mut episode_ids = Vec::with_capacity(episode_count);
            rt().block_on(async {
                for _ in 0..episode_count {
                    let id = Uuid::new_v4();
                    let episode = create_test_episode(id);
                    cached.store_episode_cached(&episode).await.unwrap();
                    episode_ids.push(id);
                }

                // Access each once (all misses)
                for id in &episode_ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                }

                let stats = cached.stats();
                let hit_rate = stats.episode_hit_rate();
                black_box(hit_rate);
            });
        });
    });

    group.bench_function("multiple_accesses", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            // Pre-populate with episodes
            let episode_count = 50;
            let mut episode_ids = Vec::with_capacity(episode_count);
            rt().block_on(async {
                for _ in 0..episode_count {
                    let id = Uuid::new_v4();
                    let episode = create_test_episode(id);
                    cached.store_episode_cached(&episode).await.unwrap();
                    episode_ids.push(id);
                }

                // Access each 5 times (mix of misses and hits)
                for id in episode_ids.iter().cycle().take(episode_ids.len() * 5) {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                }

                let stats = cached.stats();
                let hit_rate = stats.episode_hit_rate();
                black_box(hit_rate);
            });
        });
    });

    group.finish();
}

// ========== Cache Configuration Benchmarks ==========

/// Benchmark cache creation with different configurations
fn bench_cache_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_creation");
    group.sample_size(20);

    group.bench_function("disabled", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(
                storage,
                CacheConfig {
                    enable_episode_cache: false,
                    ..Default::default()
                },
            );
            black_box(cached);
        });
    });

    group.bench_function("small_100", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(
                storage,
                CacheConfig {
                    max_episodes: 100,
                    ..Default::default()
                },
            );
            black_box(cached);
        });
    });

    group.bench_function("medium_1000", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(
                storage,
                CacheConfig {
                    max_episodes: 1000,
                    ..Default::default()
                },
            );
            black_box(cached);
        });
    });

    group.bench_function("large_10000", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());
            black_box(cached);
        });
    });

    group.finish();
}

/// Benchmark cache clear operation
fn bench_cache_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_clear");
    group.sample_size(20);

    group.bench_function("100_entries", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            rt().block_on(async {
                // Pre-populate cache with 100 episodes
                for _ in 0..100 {
                    let episode = create_test_episode(Uuid::new_v4());
                    cached.store_episode_cached(&episode).await.unwrap();
                }

                // Benchmark clear
                cached.clear_caches().await;
            });
        });
    });

    group.bench_function("1000_entries", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            rt().block_on(async {
                // Pre-populate cache with 1000 episodes
                for _ in 0..1000 {
                    let episode = create_test_episode(Uuid::new_v4());
                    cached.store_episode_cached(&episode).await.unwrap();
                }

                // Benchmark clear
                cached.clear_caches().await;
            });
        });
    });

    group.finish();
}

// ========== TursoConfig with Cache Benchmarks ==========

/// Benchmark TursoStorage with default cache configuration
fn bench_turso_with_cache_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("turso_with_cache");
    group.sample_size(20);

    group.bench_function("default_cache", |b| {
        b.iter(|| {
            let dir = TempDir::new().unwrap();
            let db_path = dir.path().join("benchmark.db");

            let config = TursoConfig::default();
            let storage = rt().block_on(async {
                TursoStorage::with_config(
                    &format!("file:{}", db_path.to_string_lossy()),
                    "",
                    config,
                )
                .await
                .unwrap()
            });

            let cached = storage.with_cache_default();
            black_box(cached);
        });
    });

    group.finish();
}

// ========== Latency Comparison Benchmarks ==========

/// Compare latency between first access (cache miss) and subsequent accesses (cache hit)
fn bench_cache_miss_vs_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_miss_vs_hit");
    group.sample_size(50);

    group.bench_function("first_access_miss", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            let episode_id = Uuid::new_v4();
            let episode = create_test_episode(episode_id);

            rt().block_on(async {
                cached.store_episode_cached(&episode).await.unwrap();

                // First access - cache miss
                let start = std::time::Instant::now();
                let _ = cached.get_episode_cached(episode_id).await.unwrap();
                let miss_latency = start.elapsed();
                black_box(miss_latency);
            });
        });
    });

    group.bench_function("subsequent_access_hit", |b| {
        b.iter(|| {
            let (storage, _dir) = create_test_turso_storage();
            let cached = CachedTursoStorage::new(storage, CacheConfig::default());

            let episode_id = Uuid::new_v4();
            let episode = create_test_episode(episode_id);

            rt().block_on(async {
                cached.store_episode_cached(&episode).await.unwrap();

                // First access - populate cache
                let _ = cached.get_episode_cached(episode_id).await.unwrap();

                // Subsequent access - cache hit
                let start = std::time::Instant::now();
                let _ = cached.get_episode_cached(episode_id).await.unwrap();
                let hit_latency = start.elapsed();
                black_box(hit_latency);
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_episode_retrieval_cached_10,
    bench_episode_retrieval_cached_50,
    bench_episode_retrieval_cached_100,
    bench_episode_retrieval_uncached,
    bench_episode_write,
    bench_pattern_retrieval_cached,
    bench_heuristic_retrieval_cached,
    bench_cache_stats,
    bench_cache_hit_rate,
    bench_cache_creation,
    bench_cache_clear,
    bench_turso_with_cache_config,
    bench_cache_miss_vs_hit,
);

criterion_main!(benches);
