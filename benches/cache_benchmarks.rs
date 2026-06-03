//! Cache Performance Benchmarks for TursoStorage
//!
//! Benchmarks comparing cached vs non-cached storage operations.
//! Measures latency improvements, hit rates, and memory usage.
//!
//! Run with: `cargo bench --bench cache_benchmarks`

#![allow(clippy::excessive_nesting)]
#![allow(deprecated)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::default_trait_access)]
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use do_memory_benches::TokioExecutor;
use do_memory_core::{Episode, Evidence, Heuristic, Pattern, TaskContext, TaskType};
use do_memory_storage_turso::{CacheConfig, CachedTursoStorage, TursoConfig, TursoStorage};
use rand::distr::{Distribution, Uniform};
use rand::seq::SliceRandom;
use std::hint::black_box;
use tempfile::TempDir;
use uuid::Uuid;

/// Get a reference to the global tokio runtime for setup logic
fn rt() -> &'static tokio::runtime::Runtime {
    static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
        once_cell::sync::Lazy::new(|| {
            tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
        });
    &RUNTIME
}

/// Create a test Turso storage instance
async fn create_test_turso_storage_async() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("benchmark.db");

    let storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
        .await
        .expect("Failed to create turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    (storage, dir)
}

/// Create a test episode with realistic data
fn create_test_episode(id: Uuid) -> Episode {
    create_test_episode_with_size(id, 0)
}

/// Create a test episode with specific number of steps (for payload size testing)
fn create_test_episode_with_size(id: Uuid, num_steps: usize) -> Episode {
    let mut steps = Vec::with_capacity(num_steps);
    for i in 0..num_steps {
        steps.push(do_memory_core::episode::ExecutionStep {
            step_number: i + 1,
            timestamp: chrono::Utc::now(),
            tool: format!("tool_{}", i % 10),
            action: format!("Step action {}", i),
            parameters_json: "{}".to_string(),
            result: None,
            latency_ms: 10,
            tokens_used: None,
            metadata: std::collections::HashMap::new(),
        });
    }

    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Benchmark episode {}", id),
        context: TaskContext {
            domain: "benchmark".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: do_memory_core::types::ComplexityLevel::Moderate,
            tags: vec!["performance".to_string(), "test".to_string()],
        },
        steps,
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
        checkpoints: Vec::new(),
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
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with 10 episodes
                    let ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let episode = create_test_episode(*id);
                        cached.store_episode_cached(&episode).await.unwrap();
                    }

                    // Prime the cache
                    for id in &ids {
                        let _ = cached.get_episode_cached(*id).await.unwrap();
                    }

                    (cached, ids, dir)
                })
            },
            |setup| async move {
                let (cached, ids, _dir) = setup;
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Additional Benchmarks ==========

/// Benchmark cold read latency (first access after storage)
fn bench_episode_cold_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_cold_read");
    group.sample_size(30);

    group.bench_function("cold_access", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    let episode_id = Uuid::new_v4();
                    let episode = create_test_episode(episode_id);
                    cached.store_episode_cached(&episode).await.unwrap();

                    (cached, episode_id, dir)
                })
            },
            |setup| async move {
                let (cached, episode_id, _dir) = setup;
                // Benchmark first access - cache miss + fill
                let _ = cached.get_episode_cached(episode_id).await.unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark mixed access patterns (80% hits, 20% misses)
fn bench_mixed_access_80_20(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_access_80_20");
    group.sample_size(30);

    group.bench_function("100_accesses", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // 20 unique episodes
                    let ids: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let episode = create_test_episode(*id);
                        cached.store_episode_cached(&episode).await.unwrap();
                    }

                    // Prime the cache for all 20 episodes
                    for id in &ids {
                        let _ = cached.get_episode_cached(*id).await.unwrap();
                    }

                    // Prepare 100 accesses:
                    // - 80 hits (randomly from the 20 primed IDs)
                    // - 20 misses (randomly generated new IDs)
                    let mut rng = rand::rng();
                    let mut access_ids = Vec::with_capacity(100);
                    let die = Uniform::new(0, ids.len()).unwrap();
                    for _ in 0..80 {
                        access_ids.push(ids[die.sample(&mut rng)]);
                    }
                    for _ in 0..20 {
                        access_ids.push(Uuid::new_v4());
                    }

                    // Shuffle access IDs
                    access_ids.shuffle(&mut rng);

                    (cached, access_ids, dir)
                })
            },
            |setup| async move {
                let (cached, access_ids, _dir) = setup;
                for id in access_ids {
                    let _ = cached.get_episode_cached(id).await.unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Compare cache overhead for small vs large episodes
fn bench_payload_size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_size_comparison");
    group.sample_size(20);

    group.bench_function("small_episode_cached", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
                    let id = Uuid::new_v4();
                    let ep = create_test_episode_with_size(id, 0);
                    cached.store_episode_cached(&ep).await.unwrap();
                    let _ = cached.get_episode_cached(id).await.unwrap(); // Prime

                    (cached, id, dir)
                })
            },
            |setup| async move {
                let (cached, id, _dir) = setup;
                for _ in 0..10 {
                    let _ = cached.get_episode_cached(id).await.unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("large_episode_cached", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
                    let id = Uuid::new_v4();
                    // Large episode with 100 steps
                    let ep = create_test_episode_with_size(id, 100);
                    cached.store_episode_cached(&ep).await.unwrap();
                    let _ = cached.get_episode_cached(id).await.unwrap(); // Prime

                    (cached, id, dir)
                })
            },
            |setup| async move {
                let (cached, id, _dir) = setup;
                for _ in 0..10 {
                    let _ = cached.get_episode_cached(id).await.unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark cached episode retrieval with 50 episodes
fn bench_episode_retrieval_cached_50(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_cached_50");
    group.sample_size(30);

    group.bench_function("50_episodes", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with 50 episodes
                    let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let episode = create_test_episode(*id);
                        cached.store_episode_cached(&episode).await.unwrap();
                    }

                    // Prime the cache
                    for id in &ids {
                        let _ = cached.get_episode_cached(*id).await.unwrap();
                    }

                    (cached, ids, dir)
                })
            },
            |setup| async move {
                let (cached, ids, _dir) = setup;
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark cached episode retrieval with 100 episodes
fn bench_episode_retrieval_cached_100(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_cached_100");
    group.sample_size(30);

    group.bench_function("100_episodes", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with 100 episodes
                    let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let episode = create_test_episode(*id);
                        cached.store_episode_cached(&episode).await.unwrap();
                    }

                    // Prime the cache
                    for id in &ids {
                        let _ = cached.get_episode_cached(*id).await.unwrap();
                    }

                    (cached, ids, dir)
                })
            },
            |setup| async move {
                let (cached, ids, _dir) = setup;
                for id in &ids {
                    let _ = cached.get_episode_cached(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark uncached episode retrieval for comparison
fn bench_episode_retrieval_uncached(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_retrieval_uncached");
    group.sample_size(30);

    group.bench_function("100_episodes", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;

                    // Pre-populate without cache
                    let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let episode = create_test_episode(*id);
                        storage.store_episode(&episode).await.unwrap();
                    }

                    (storage, ids, dir)
                })
            },
            |setup| async move {
                let (storage, ids, _dir) = setup;
                // Benchmark uncached retrieval
                for id in &ids {
                    let _ = storage.get_episode(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark episode write operations
fn bench_episode_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_write");
    group.sample_size(30);

    group.bench_function("10_episodes", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
                    let episodes: Vec<Episode> = (0..10)
                        .map(|_| create_test_episode(Uuid::new_v4()))
                        .collect();
                    (cached, episodes, dir)
                })
            },
            |setup| async move {
                let (cached, episodes, _dir) = setup;
                for episode in &episodes {
                    cached.store_episode_cached(episode).await.unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Pattern Benchmarks ==========

/// Benchmark cached pattern retrieval
fn bench_pattern_retrieval_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_retrieval_cached");
    group.sample_size(30);

    group.bench_function("50_patterns", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with 50 patterns
                    let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let pattern = create_test_pattern(*id);
                        cached.store_pattern_cached(&pattern).await.unwrap();
                    }

                    // Prime the cache
                    for id in &ids {
                        let _ = cached.get_pattern_cached(*id).await.unwrap();
                    }

                    (cached, ids, dir)
                })
            },
            |setup| async move {
                let (cached, ids, _dir) = setup;
                for id in &ids {
                    let _ = cached.get_pattern_cached(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Heuristic Benchmarks ==========

/// Benchmark cached heuristic retrieval
fn bench_heuristic_retrieval_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("heuristic_retrieval_cached");
    group.sample_size(30);

    group.bench_function("50_heuristics", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with 50 heuristics
                    let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
                    for id in &ids {
                        let heuristic = create_test_heuristic(*id);
                        cached.store_heuristic_cached(&heuristic).await.unwrap();
                    }

                    // Prime the cache
                    for id in &ids {
                        let _ = cached.get_heuristic_cached(*id).await.unwrap();
                    }

                    (cached, ids, dir)
                })
            },
            |setup| async move {
                let (cached, ids, _dir) = setup;
                for id in &ids {
                    let _ = cached.get_heuristic_cached(*id).await.unwrap();
                    black_box(id);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Cache Statistics Benchmarks ==========

/// Benchmark cache statistics calculation
fn bench_cache_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_stats");
    group.sample_size(50);

    group.bench_function("stats_calculation", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    for _ in 0..100 {
                        let episode = create_test_episode(Uuid::new_v4());
                        cached.store_episode_cached(&episode).await.unwrap();
                        let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
                        // Access again for hit
                        let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
                    }

                    (cached, dir)
                })
            },
            |setup| async move {
                let (cached, _dir) = setup;
                let stats = cached.stats();
                black_box(stats.hit_rate());
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark cache hit rate calculation
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");
    group.sample_size(50);

    group.bench_function("single_access", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with episodes
                    let episode_count = 50;
                    let mut episode_ids = Vec::with_capacity(episode_count);
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

                    (cached, dir)
                })
            },
            |setup| async move {
                let (cached, _dir) = setup;
                let stats = cached.stats();
                let hit_rate = stats.episode_hit_rate();
                black_box(hit_rate);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("multiple_accesses", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate with episodes
                    let episode_count = 50;
                    let mut episode_ids = Vec::with_capacity(episode_count);
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

                    (cached, dir)
                })
            },
            |setup| async move {
                let (cached, _dir) = setup;
                let stats = cached.stats();
                let hit_rate = stats.episode_hit_rate();
                black_box(hit_rate);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Cache Configuration Benchmarks ==========

/// Benchmark cache creation with different configurations
fn bench_cache_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_creation");
    group.sample_size(20);

    group.bench_function("disabled", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || rt().block_on(async { create_test_turso_storage_async().await }),
            |setup| async move {
                let (storage, _dir) = setup;
                let cached = CachedTursoStorage::new(
                    storage,
                    CacheConfig {
                        enable_episode_cache: false,
                        ..Default::default()
                    },
                );
                black_box(cached);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("small_100", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || rt().block_on(async { create_test_turso_storage_async().await }),
            |setup| async move {
                let (storage, _dir) = setup;
                let cached = CachedTursoStorage::new(
                    storage,
                    CacheConfig {
                        max_episodes: 100,
                        ..Default::default()
                    },
                );
                black_box(cached);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("medium_1000", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || rt().block_on(async { create_test_turso_storage_async().await }),
            |setup| async move {
                let (storage, _dir) = setup;
                let cached = CachedTursoStorage::new(
                    storage,
                    CacheConfig {
                        max_episodes: 1000,
                        ..Default::default()
                    },
                );
                black_box(cached);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("large_10000", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || rt().block_on(async { create_test_turso_storage_async().await }),
            |setup| async move {
                let (storage, _dir) = setup;
                let cached = CachedTursoStorage::new(storage, CacheConfig::default());
                black_box(cached);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark cache clear operation
fn bench_cache_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_clear");
    group.sample_size(20);

    group.bench_function("100_entries", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate cache with 100 episodes
                    for _ in 0..100 {
                        let episode = create_test_episode(Uuid::new_v4());
                        cached.store_episode_cached(&episode).await.unwrap();
                    }
                    (cached, dir)
                })
            },
            |setup| async move {
                let (cached, _dir) = setup;
                // Benchmark clear
                cached.clear_caches().await;
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("1000_entries", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    // Pre-populate cache with 1000 episodes
                    for _ in 0..1000 {
                        let episode = create_test_episode(Uuid::new_v4());
                        cached.store_episode_cached(&episode).await.unwrap();
                    }
                    (cached, dir)
                })
            },
            |setup| async move {
                let (cached, _dir) = setup;
                // Benchmark clear
                cached.clear_caches().await;
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== TursoConfig with Cache Benchmarks ==========

/// Benchmark TursoStorage with default cache configuration
fn bench_turso_with_cache_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("turso_with_cache");
    group.sample_size(20);

    group.bench_function("default_cache", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let dir = TempDir::new().unwrap();
                    let db_path = dir.path().join("benchmark.db");
                    (dir, db_path)
                })
            },
            |setup| async move {
                let (_dir, db_path) = setup;
                let config = TursoConfig::default();
                let storage = TursoStorage::with_config(
                    &format!("file:{}", db_path.to_string_lossy()),
                    "",
                    config,
                )
                .await
                .unwrap();

                let cached = storage.with_cache_default();
                black_box(cached);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ========== Latency Comparison Benchmarks ==========

/// Compare latency between first access (cache miss) and subsequent accesses (cache hit)
fn bench_cache_miss_vs_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_miss_vs_hit");
    group.sample_size(50);

    group.bench_function("first_access_miss", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    let episode_id = Uuid::new_v4();
                    let episode = create_test_episode(episode_id);
                    cached.store_episode_cached(&episode).await.unwrap();

                    (cached, episode_id, dir)
                })
            },
            |setup| async move {
                let (cached, episode_id, _dir) = setup;
                // First access - cache miss
                let _ = cached.get_episode_cached(episode_id).await.unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("subsequent_access_hit", |b| {
        b.to_async(TokioExecutor).iter_batched(
            || {
                rt().block_on(async {
                    let (storage, dir) = create_test_turso_storage_async().await;
                    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

                    let episode_id = Uuid::new_v4();
                    let episode = create_test_episode(episode_id);
                    cached.store_episode_cached(&episode).await.unwrap();

                    // First access - populate cache
                    let _ = cached.get_episode_cached(episode_id).await.unwrap();

                    (cached, episode_id, dir)
                })
            },
            |setup| async move {
                let (cached, episode_id, _dir) = setup;
                // Subsequent access - cache hit
                let _ = cached.get_episode_cached(episode_id).await.unwrap();
            },
            BatchSize::SmallInput,
        );
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
    bench_episode_cold_read,
    bench_mixed_access_80_20,
    bench_payload_size_comparison,
);

criterion_main!(benches);
