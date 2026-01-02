//! Simplified benchmarks for domain-based cache invalidation
//!
//! Validates performance claims:
//! - Domain invalidation latency: <100µs for <1000 entries
//! - Minimal put() overhead with domain tracking

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_core::episode::Episode;
use memory_core::retrieval::{CacheKey, QueryCache};
use memory_core::types::{TaskContext, TaskType};
use std::collections::HashMap;
use uuid::Uuid;

fn create_test_episode(id: u32, domain: &str) -> Episode {
    let mut context = TaskContext::default();
    context.domain = domain.to_string();

    Episode {
        episode_id: Uuid::new_v4(),
        task_type: TaskType::CodeGeneration,
        task_description: format!("Task {} in {}", id, domain),
        context,
        start_time: chrono::Utc::now(),
        end_time: None,
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        metadata: HashMap::new(),
    }
}

/// Benchmark: Domain invalidation latency (target: <100µs for <1000 entries)
fn bench_domain_invalidation_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain_invalidation_latency");
    
    for size in [100, 300, 600, 900].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let cache = QueryCache::new();
                    for i in 0..size {
                        let domain = match i % 3 {
                            0 => "web-api",
                            1 => "data-processing",
                            _ => "machine-learning",
                        };
                        let key = CacheKey::new(format!("query-{}", i))
                            .with_domain(Some(domain.to_string()));
                        let episodes = vec![create_test_episode(i, domain)];
                        cache.put(key, episodes);
                    }
                    cache
                },
                |cache| {
                    cache.invalidate_domain(black_box("web-api"));
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

/// Benchmark: Compare invalidate_all() vs invalidate_domain()
fn bench_invalidation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("invalidation_comparison");
    let size = 300;
    
    group.bench_function("invalidate_all_300_entries", |b| {
        b.iter_batched(
            || {
                let cache = QueryCache::new();
                for i in 0..size {
                    let domain = match i % 3 {
                        0 => "web-api",
                        1 => "data-processing",
                        _ => "machine-learning",
                    };
                    let key = CacheKey::new(format!("query-{}", i))
                        .with_domain(Some(domain.to_string()));
                    let episodes = vec![create_test_episode(i, domain)];
                    cache.put(key, episodes);
                }
                cache
            },
            |cache| {
                cache.invalidate_all();
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.bench_function("invalidate_domain_100_entries", |b| {
        b.iter_batched(
            || {
                let cache = QueryCache::new();
                for i in 0..size {
                    let domain = match i % 3 {
                        0 => "web-api",
                        1 => "data-processing",
                        _ => "machine-learning",
                    };
                    let key = CacheKey::new(format!("query-{}", i))
                        .with_domain(Some(domain.to_string()));
                    let episodes = vec![create_test_episode(i, domain)];
                    cache.put(key, episodes);
                }
                cache
            },
            |cache| {
                cache.invalidate_domain(black_box("web-api"));
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark: put() overhead with domain tracking
fn bench_put_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_overhead");
    
    group.bench_function("with_domain", |b| {
        let cache = QueryCache::new();
        let mut counter = 0u32;
        
        b.iter(|| {
            let key = CacheKey::new(format!("query-{}", counter))
                .with_domain(Some("web-api".to_string()));
            let episodes = vec![create_test_episode(counter, "web-api")];
            cache.put(key, episodes);
            counter += 1;
        });
    });
    
    group.bench_function("without_domain", |b| {
        let cache = QueryCache::new();
        let mut counter = 0u32;
        
        b.iter(|| {
            let key = CacheKey::new(format!("query-{}", counter));
            let episodes = vec![create_test_episode(counter, "general")];
            cache.put(key, episodes);
            counter += 1;
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_domain_invalidation_latency,
    bench_invalidation_comparison,
    bench_put_overhead,
);
criterion_main!(benches);
