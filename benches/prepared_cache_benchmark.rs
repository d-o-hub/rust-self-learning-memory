//! Benchmarks for prepared statement cache performance
//!
//! These benchmarks measure the effectiveness of the prepared statement cache
//! and the performance improvement from connection-aware caching.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use memory_storage_turso::prepared::PreparedStatementCache;

/// Benchmark basic cache operations
fn bench_cache_basic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_basic");

    group.bench_function("cache_hit", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();
        let sql = "SELECT * FROM episodes WHERE id = ?";

        // Prime the cache
        cache.record_miss(conn_id, sql, 100);

        b.iter(|| {
            black_box(cache.is_cached(black_box(conn_id), black_box(sql)));
            cache.record_hit(conn_id, sql);
            black_box(());
        });
    });

    group.bench_function("cache_miss", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();

        b.iter(|| {
            let sql = format!("SELECT * FROM episodes WHERE id = {}", black_box(42));
            cache.record_miss(conn_id, &sql, 100);
            black_box(());
        });
    });

    group.finish();
}

/// Benchmark cache with multiple connections
fn bench_cache_multiple_connections(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_multi_conn");

    group.bench_function("10_connections", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_ids: Vec<_> = (0..10).map(|_| cache.get_connection_id()).collect();
        let sql = "SELECT * FROM episodes WHERE id = ?";

        // Prime all connections
        for &conn_id in &conn_ids {
            cache.record_miss(conn_id, sql, 100);
        }

        b.iter(|| {
            for &conn_id in &conn_ids {
                black_box(cache.is_cached(conn_id, sql));
            }
        });
    });

    group.bench_function("100_connections", |b| {
        let cache = PreparedStatementCache::with_config(
            memory_storage_turso::prepared::PreparedCacheConfig {
                max_size: 100,
                max_connections: 100,
                ..Default::default()
            },
        );
        let conn_ids: Vec<_> = (0..100).map(|_| cache.get_connection_id()).collect();
        let sql = "SELECT * FROM episodes WHERE id = ?";

        // Prime all connections
        for &conn_id in &conn_ids {
            cache.record_miss(conn_id, sql, 100);
        }

        b.iter(|| {
            for &conn_id in &conn_ids {
                black_box(cache.is_cached(conn_id, sql));
            }
        });
    });

    group.finish();
}

/// Benchmark cache eviction
fn bench_cache_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_eviction");

    group.bench_function("lru_eviction", |b| {
        let cache = PreparedStatementCache::new(10); // Small cache to force eviction
        let conn_id = cache.get_connection_id();

        b.iter(|| {
            // Add 15 statements (should trigger evictions)
            for i in 0..15 {
                let sql = format!("SELECT * FROM table_{} WHERE id = ?", i);
                cache.record_miss(conn_id, &sql, 100);
                black_box(());
            }
        });
    });

    group.finish();
}

/// Benchmark cache with varying SQL patterns
fn bench_cache_sql_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_sql_patterns");

    group.bench_function("repeated_queries", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();

        let queries = vec![
            "SELECT * FROM episodes WHERE id = ?",
            "SELECT * FROM patterns WHERE success_rate > ?",
            "SELECT COUNT(*) FROM episodes",
            "SELECT * FROM heuristics WHERE enabled = ?",
        ];

        // Prime cache
        for query in &queries {
            cache.record_miss(conn_id, query, 100);
        }

        b.iter(|| {
            for query in &queries {
                black_box(cache.is_cached(conn_id, query));
                cache.record_hit(conn_id, query);
                black_box(());
            }
        });
    });

    group.bench_function("parameterized_queries", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();

        let templates = vec![
            "SELECT * FROM episodes WHERE id = ?",
            "SELECT * FROM patterns WHERE pattern_type = ?",
            "SELECT * FROM heuristics WHERE value > ?",
        ];

        // Prime cache
        for template in &templates {
            cache.record_miss(conn_id, template, 100);
        }

        b.iter(|| {
            for template in &templates {
                black_box(cache.is_cached(conn_id, template));
                cache.record_hit(conn_id, template);
                black_box(());
            }
        });
    });

    group.finish();
}

/// Benchmark cache statistics
fn bench_cache_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_statistics");

    group.bench_function("stats_calculation", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();

        // Populate cache
        for i in 0..50 {
            let sql = format!("SELECT * FROM episodes WHERE id = {}", i);
            cache.record_miss(conn_id, &sql, 100);

            if i % 2 == 0 {
                cache.record_hit(conn_id, &sql);
            }
        }

        b.iter(|| {
            black_box(cache.stats());
        });
    });

    group.finish();
}

/// Benchmark cache cleanup
fn bench_cache_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_cleanup");

    group.bench_function("clear_connection", |b| {
        let cache = PreparedStatementCache::new(100);
        let conn_id = cache.get_connection_id();

        // Populate cache
        for i in 0..50 {
            let sql = format!("SELECT * FROM episodes WHERE id = {}", i);
            cache.record_miss(conn_id, &sql, 100);
        }

        b.iter(|| {
            let test_conn_id = black_box(conn_id);
            cache.clear_connection(test_conn_id);
            black_box(());
        });
    });

    group.bench_function("clear_all", |b| {
        let cache = PreparedStatementCache::new(100);

        // Populate multiple connections
        for _i in 0..10 {
            let conn_id = cache.get_connection_id();
            for j in 0..10 {
                let sql = format!("SELECT * FROM episodes WHERE id = {}", j);
                cache.record_miss(conn_id, &sql, 100);
            }
        }

        b.iter(|| {
            cache.clear();
            black_box(());
        });
    });

    group.finish();
}

/// Benchmark concurrent cache access
fn bench_cache_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_concurrent");

    group.throughput(Throughput::Elements(1000));

    group.bench_function("concurrent_access", |b| {
        let cache = std::sync::Arc::new(PreparedStatementCache::new(100));

        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| run_concurrent_bench(std::sync::Arc::clone(&cache)));
    });

    group.finish();
}

/// Helper function for concurrent benchmark to reduce nesting
async fn run_concurrent_bench(cache: std::sync::Arc<PreparedStatementCache>) {
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let cache = std::sync::Arc::clone(&cache);
            tokio::spawn(async move {
                let conn_id = cache.get_connection_id();
                for i in 0..100 {
                    let sql = format!("SELECT * FROM episodes WHERE id = {}", i);
                    cache.record_miss(conn_id, &sql, 100);
                    cache.is_cached(conn_id, &sql);
                    cache.record_hit(conn_id, &sql);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

criterion_group!(
    benches,
    bench_cache_basic_operations,
    bench_cache_multiple_connections,
    bench_cache_eviction,
    bench_cache_sql_patterns,
    bench_cache_statistics,
    bench_cache_cleanup,
    bench_cache_concurrent
);

criterion_main!(benches);
