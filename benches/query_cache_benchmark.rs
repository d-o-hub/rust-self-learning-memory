//! Query Cache Performance Benchmarks (v0.1.12)
//!
//! Benchmarks for query caching system to validate:
//! - Cache hit/miss latency
//! - Memory overhead
//! - Concurrent access performance
//! - Eviction behavior
//!
//! Run with: `cargo bench --bench query_cache_benchmark`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memory_core::retrieval::{CacheKey, QueryCache};
use memory_core::{Episode, ExecutionResult, ExecutionStep, TaskContext, TaskType};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Create a test episode with realistic data
fn create_test_episode(id: usize) -> Episode {
    let mut episode = Episode {
        episode_id: Uuid::new_v4(),
        task_type: TaskType::CodeGeneration,
        task_description: format!("Implement feature {}", id),
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: Some("axum".to_string()),
            ..Default::default()
        },
        start_time: chrono::Utc::now(),
        end_time: Some(chrono::Utc::now()),
        steps: vec![],
        outcome: Some(memory_core::TaskOutcome::Success {
            verdict: "Feature implemented".to_string(),
            artifacts: vec!["src/feature.rs".to_string()],
        }),
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        metadata: std::collections::HashMap::new(),
        tags: Vec::new(),
    };

    // Add 20 execution steps to make it realistic
    for i in 0..20 {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("tool_{}", i % 6),
            format!("Step {} action", i),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {} completed successfully", i),
        });
        episode.steps.push(step);
    }

    episode
}

/// Benchmark cache hit performance (best case)
fn bench_cache_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit");

    // Benchmark with different episode counts
    for episode_count in [1, 5, 10, 20].iter() {
        let cache = QueryCache::new();
        let key = CacheKey::new("test query".to_string())
            .with_domain(Some("web-api".to_string()))
            .with_limit(10);

        let episodes: Vec<Arc<Episode>> = (0..*episode_count)
            .map(|i| Arc::new(create_test_episode(i)))
            .collect();

        cache.put(key.clone(), episodes);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, _| {
                b.iter(|| {
                    let result = cache.get(black_box(&key));
                    assert!(result.is_some());
                    result
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache miss performance (worst case)
fn bench_cache_miss(c: &mut Criterion) {
    let cache = QueryCache::new();

    c.bench_function("cache_miss", |b| {
        b.iter(|| {
            let key = CacheKey::new(black_box("unique query".to_string()))
                .with_domain(Some("web-api".to_string()))
                .with_limit(10);
            let result = cache.get(&key);
            assert!(result.is_none());
            result
        });
    });
}

/// Benchmark cache put operation
fn bench_cache_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_put");

    for episode_count in [1, 5, 10, 20].iter() {
        let episodes: Vec<Arc<Episode>> = (0..*episode_count)
            .map(|i| Arc::new(create_test_episode(i)))
            .collect();

        group.throughput(Throughput::Elements(*episode_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, _| {
                let cache = QueryCache::new();
                let mut query_id = 0;

                b.iter(|| {
                    let key = CacheKey::new(format!("query_{}", query_id))
                        .with_domain(Some("web-api".to_string()))
                        .with_limit(10);
                    query_id += 1;

                    cache.put(black_box(key), black_box(episodes.clone()));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark LRU eviction performance
fn bench_cache_eviction(c: &mut Criterion) {
    let cache = QueryCache::with_capacity_and_ttl(100, Duration::from_secs(60));
    let episodes: Vec<Arc<Episode>> = (0..5).map(|i| Arc::new(create_test_episode(i))).collect();

    // Fill cache to capacity
    for i in 0..100 {
        let key = CacheKey::new(format!("query_{}", i))
            .with_domain(Some("web-api".to_string()))
            .with_limit(10);
        cache.put(key, episodes.clone());
    }

    c.bench_function("cache_eviction", |b| {
        let mut query_id = 100;

        b.iter(|| {
            let key = CacheKey::new(format!("query_{}", query_id))
                .with_domain(Some("web-api".to_string()))
                .with_limit(10);
            query_id += 1;

            // This should trigger LRU eviction
            cache.put(black_box(key), black_box(episodes.clone()));
        });
    });
}

/// Benchmark cache invalidation
fn bench_cache_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_invalidation");

    for cache_size in [10, 100, 1000, 5000].iter() {
        let cache = QueryCache::new();
        let episodes: Vec<Arc<Episode>> =
            (0..5).map(|i| Arc::new(create_test_episode(i))).collect();

        // Fill cache with entries
        for i in 0..*cache_size {
            let key = CacheKey::new(format!("query_{}", i))
                .with_domain(Some("web-api".to_string()))
                .with_limit(10);
            cache.put(key, episodes.clone());
        }

        group.throughput(Throughput::Elements(*cache_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(cache_size),
            cache_size,
            |b, _| {
                b.iter(|| {
                    cache.invalidate_all();

                    // Refill for next iteration
                    for i in 0..*cache_size {
                        let key = CacheKey::new(format!("query_{}", i))
                            .with_domain(Some("web-api".to_string()))
                            .with_limit(10);
                        cache.put(key.clone(), episodes.clone());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent cache access
fn bench_concurrent_access(c: &mut Criterion) {
    use std::thread;

    let cache = Arc::new(QueryCache::new());
    let episodes: Vec<Arc<Episode>> = (0..5).map(|i| Arc::new(create_test_episode(i))).collect();

    // Pre-populate cache
    for i in 0..100 {
        let key = CacheKey::new(format!("query_{}", i))
            .with_domain(Some("web-api".to_string()))
            .with_limit(10);
        cache.put(key, episodes.clone());
    }

    c.bench_function("concurrent_access_4_threads", |b| {
        b.iter(|| {
            #[allow(clippy::excessive_nesting)]
            let mut handles = vec![];

            // Spawn 4 threads doing mixed read/write operations
            for thread_id in 0..4 {
                let cache_clone = Arc::clone(&cache);
                let episodes_clone = episodes.clone();

                #[allow(clippy::excessive_nesting)]
                let handle = thread::spawn(move || {
                    for i in 0..25 {
                        let query_id = thread_id * 25 + i;
                        let key = CacheKey::new(format!("query_{}", query_id))
                            .with_domain(Some("web-api".to_string()))
                            .with_limit(10);

                        // 75% reads, 25% writes
                        if i % 4 == 0 {
                            cache_clone.put(key, episodes_clone.clone());
                        } else {
                            let _ = cache_clone.get(&key);
                        }
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

/// Benchmark metrics collection overhead
fn bench_metrics_collection(c: &mut Criterion) {
    let cache = QueryCache::new();

    c.bench_function("metrics_collection", |b| {
        b.iter(|| {
            let metrics = cache.metrics();
            black_box(metrics);
        });
    });
}

criterion_group!(
    benches,
    bench_cache_hit,
    bench_cache_miss,
    bench_cache_put,
    bench_cache_eviction,
    bench_cache_invalidation,
    bench_concurrent_access,
    bench_metrics_collection
);

criterion_main!(benches);
