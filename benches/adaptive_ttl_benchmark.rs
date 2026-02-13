//! Adaptive TTL Performance Benchmarks
//!
//! Comprehensive benchmarks comparing static vs adaptive TTL cache performance.
//! Measures effectiveness across different access patterns and workloads.
//!
//! **Key Metrics:**
//! - Cache hit rate (%)
//! - Average TTL per access pattern
//! - Memory usage (bytes, entries)
//! - Response time (P50, P95, P99)
//!
//! **Performance Targets:**
//! - Hit rate improvement: >10% vs static TTL
//! - Memory overhead: <20% vs static TTL
//!
//! **Run with:** `cargo bench --bench adaptive_ttl_benchmark`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memory_storage_turso::cache::{AdaptiveTTLCache, TTLConfig};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use zipf::ZipfDistribution;

/// Create a tokio runtime for benchmarks
fn rt() -> &'static Runtime {
    static RUNTIME: once_cell::sync::Lazy<Runtime> =
        once_cell::sync::Lazy::new(|| Runtime::new().expect("Failed to create runtime"));
    &RUNTIME
}

/// Test value type (simulates cache payload)
#[derive(Debug, Clone)]
struct TestValue {
    id: u32,
    data: Vec<u8>,
    timestamp: i64,
}

impl TestValue {
    /// Create a test value with specified size
    fn new(id: u32, size: usize) -> Self {
        Self {
            id,
            data: vec![0xDE; size],
            timestamp: chrono::Utc::now().timestamp_micros(),
        }
    }

    /// Create small test value (128 bytes)
    fn small(id: u32) -> Self {
        Self::new(id, 128)
    }

    /// Create medium test value (1 KB)
    fn medium(id: u32) -> Self {
        Self::new(id, 1024)
    }

    /// Create large test value (8 KB)
    fn large(id: u32) -> Self {
        Self::new(id, 8192)
    }

    /// Approximate size in bytes
    fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }
}

// ========== Access Pattern Generators ==========

/// Sequential access pattern (0, 1, 2, ..., n, 0, 1, ...)
fn sequential_pattern(count: usize, iterations: usize) -> Vec<u32> {
    (0..iterations).map(|i| (i % count) as u32).collect()
}

/// Random access pattern
fn random_pattern(count: usize, iterations: usize, seed: u64) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..iterations)
        .map(|_| rng.gen_range(0u32..count as u32))
        .collect()
}

/// Zipfian access pattern (realistic workload: few hot items, many cold)
fn zipf_pattern(count: usize, iterations: usize, exponent: f64) -> Vec<u32> {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let zipf = ZipfDistribution::new(count, exponent).unwrap();
    (0..iterations)
        .map(|_| zipf.sample(&mut rng) as u32)
        .collect()
}

/// Hot-cold pattern (20% hot keys accessed 80% of the time)
fn hot_cold_pattern(count: usize, iterations: usize) -> Vec<u32> {
    let hot_count = count / 5;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    (0..iterations)
        .map(|_| {
            if rng.gen_bool(0.8) {
                // Hot key
                rng.gen_range(0u32..hot_count as u32)
            } else {
                // Cold key
                rng.gen_range(hot_count as u32..count as u32)
            }
        })
        .collect()
}

// ========== Benchmark Helpers ==========

/// Run a workload and collect metrics
async fn run_workload(
    cache: &AdaptiveTTLCache<u32, TestValue>,
    pattern: &[u32],
) -> (f64, Duration, Duration, Duration) {
    let mut latencies = Vec::with_capacity(pattern.len());
    let mut hits = 0u64;
    let mut misses = 0u64;

    for &key in pattern {
        let op_start = Instant::now();
        if cache.get(&key).await.is_some() {
            hits += 1;
        } else {
            misses += 1;
        }
        latencies.push(op_start.elapsed());
    }

    let hit_rate = if hits + misses > 0 {
        hits as f64 / (hits + misses) as f64
    } else {
        0.0
    };

    // Calculate percentiles
    latencies.sort();
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() * 95) / 100];
    let p99 = latencies[(latencies.len() * 99) / 100];

    (hit_rate, p50, p95, p99)
}

// ========== Static vs Adaptive Comparison ==========

/// Compare static TTL vs adaptive TTL baseline
fn bench_static_vs_adaptive_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_vs_adaptive");
    group.sample_size(20);

    // Test with different cache sizes
    for size in [100, 500, 1000].iter() {
        // Static TTL baseline
        group.bench_with_input(BenchmarkId::new("static", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let config = TTLConfig {
                        enable_adaptive_ttl: false,
                        base_ttl: Duration::from_secs(300),
                        enable_background_cleanup: false,
                        ..TTLConfig::default()
                    };
                    let cache: AdaptiveTTLCache<u32, TestValue> =
                        rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                    // Populate cache
                    rt().block_on(async {
                        for i in 0..size {
                            cache.insert(i as u32, TestValue::small(i as u32)).await;
                        }
                    });

                    (cache, sequential_pattern(size, size * 10))
                },
                |(cache, pattern)| {
                    rt().block_on(async {
                        let result = run_workload(&cache, &pattern).await;
                        black_box(result);
                    });
                },
                criterion::BatchSize::LargeInput,
            );
        });

        // Adaptive TTL
        group.bench_with_input(BenchmarkId::new("adaptive", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let config = TTLConfig {
                        enable_adaptive_ttl: true,
                        base_ttl: Duration::from_secs(300),
                        enable_background_cleanup: false,
                        ..TTLConfig::default()
                    };
                    let cache: AdaptiveTTLCache<u32, TestValue> =
                        rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                    // Populate cache
                    rt().block_on(async {
                        for i in 0..size {
                            cache.insert(i, TestValue::small(i as u32)).await;
                        }
                    });

                    (cache, sequential_pattern(size, size * 10))
                },
                |(cache, pattern)| {
                    rt().block_on(async {
                        let result = run_workload(&cache, &pattern).await;
                        black_box(result);
                    });
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ========== Access Pattern Benchmarks ==========

/// Benchmark Zipfian distribution (realistic workload)
fn bench_zipf_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipf_pattern");
    group.sample_size(20);

    let cache_size = 1000;
    let iterations = 10000;

    // Test different Zipf exponents
    for exponent in [0.5, 0.8, 1.0, 1.2].iter() {
        group.bench_with_input(
            BenchmarkId::new("adaptive", exponent),
            exponent,
            |b, &exponent| {
                b.iter_batched(
                    || {
                        let config = TTLConfig {
                            enable_adaptive_ttl: true,
                            base_ttl: Duration::from_secs(300),
                            max_entries: cache_size,
                            enable_background_cleanup: false,
                            ..TTLConfig::default()
                        };
                        let cache: AdaptiveTTLCache<u32, TestValue> =
                            rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                        rt().block_on(async {
                            for i in 0..cache_size {
                                cache.insert(i as u32, TestValue::medium(i as u32)).await;
                            }
                        });

                        (cache, zipf_pattern(cache_size, iterations, exponent))
                    },
                    |(cache, pattern)| {
                        rt().block_on(async {
                            let result = run_workload(&cache, &pattern).await;
                            black_box(result);
                        });
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark sequential access pattern
fn bench_sequential_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_pattern");
    group.sample_size(20);

    let cache_size = 500;
    let iterations = 5000;

    group.bench_function("static", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: false,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, sequential_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("adaptive", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, sequential_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Benchmark random access pattern
fn bench_random_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_pattern");
    group.sample_size(20);

    let cache_size = 1000;
    let iterations = 10000;

    group.bench_function("static", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: false,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, random_pattern(cache_size, iterations, 42))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("adaptive", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, random_pattern(cache_size, iterations, 42))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Benchmark hot-cold pattern
fn bench_hot_cold_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_cold_pattern");
    group.sample_size(20);

    let cache_size = 1000;
    let iterations = 10000;

    group.bench_function("static", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: false,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, hot_cold_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("adaptive", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, hot_cold_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let result = run_workload(&cache, &pattern).await;
                    black_box(result);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== Hit Rate Analysis ==========

/// Detailed hit rate measurement across patterns
fn bench_hit_rate_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_rate_analysis");
    group.sample_size(10);

    let cache_size = 500;
    let iterations = 5000;

    // Run separate benchmarks for each pattern to avoid closure type issues
    group.bench_function("sequential", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, sequential_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut hits = 0;
                    let mut misses = 0;
                    for &key in &pattern {
                        if cache.get(&key).await.is_some() {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    let hit_rate = hits as f64 / (hits + misses) as f64;
                    black_box(hit_rate);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("random", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, random_pattern(cache_size, iterations, 42))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut hits = 0;
                    let mut misses = 0;
                    for &key in &pattern {
                        if cache.get(&key).await.is_some() {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    let hit_rate = hits as f64 / (hits + misses) as f64;
                    black_box(hit_rate);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("zipf_0.8", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, zipf_pattern(cache_size, iterations, 0.8))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut hits = 0;
                    let mut misses = 0;
                    for &key in &pattern {
                        if cache.get(&key).await.is_some() {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    let hit_rate = hits as f64 / (hits + misses) as f64;
                    black_box(hit_rate);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("hot_cold", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, hot_cold_pattern(cache_size, iterations))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut hits = 0;
                    let mut misses = 0;
                    for &key in &pattern {
                        if cache.get(&key).await.is_some() {
                            hits += 1;
                        } else {
                            misses += 1;
                        }
                    }
                    let hit_rate = hits as f64 / (hits + misses) as f64;
                    black_box(hit_rate);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== Memory Overhead ==========

/// Memory usage comparison: static vs adaptive
fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");
    group.sample_size(20);

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("static", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let config = TTLConfig {
                        enable_adaptive_ttl: false,
                        base_ttl: Duration::from_secs(300),
                        max_entries: size,
                        enable_background_cleanup: false,
                        ..TTLConfig::default()
                    };
                    let cache: AdaptiveTTLCache<u32, TestValue> =
                        rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                    rt().block_on(async {
                        for i in 0..size {
                            cache.insert(i as u32, TestValue::medium(i as u32)).await;
                        }
                    });

                    cache
                },
                |cache| {
                    rt().block_on(async {
                        let len = cache.len().await;
                        let peak = cache.stats().peak_entries;
                        black_box((len, peak));
                    });
                },
                criterion::BatchSize::LargeInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("adaptive", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let config = TTLConfig {
                        enable_adaptive_ttl: true,
                        base_ttl: Duration::from_secs(300),
                        max_entries: size,
                        enable_background_cleanup: false,
                        ..TTLConfig::default()
                    };
                    let cache: AdaptiveTTLCache<u32, TestValue> =
                        rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                    rt().block_on(async {
                        for i in 0..size {
                            cache.insert(i as u32, TestValue::medium(i as u32)).await;
                        }
                    });

                    cache
                },
                |cache| {
                    rt().block_on(async {
                        let len = cache.len().await;
                        let peak = cache.stats().peak_entries;
                        black_box((len, peak));
                    });
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ========== Latency Percentiles ==========

/// Response time measurement (P50, P95, P99)
fn bench_latency_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_percentiles");
    group.sample_size(20);

    let cache_size = 1000;
    let iterations = 1000;

    group.bench_function("static_read", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: false,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, random_pattern(cache_size, iterations, 42))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut latencies = Vec::with_capacity(pattern.len());
                    for &key in &pattern {
                        let start = Instant::now();
                        let _ = cache.get(&key).await;
                        latencies.push(start.elapsed());
                    }
                    latencies.sort();
                    let p50 = latencies[latencies.len() / 2];
                    let p95 = latencies[(latencies.len() * 95) / 100];
                    let p99 = latencies[(latencies.len() * 99) / 100];
                    black_box((p50, p95, p99));
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("adaptive_read", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    max_entries: cache_size,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::small(i as u32)).await;
                    }
                });

                (cache, random_pattern(cache_size, iterations, 42))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    let mut latencies = Vec::with_capacity(pattern.len());
                    for &key in &pattern {
                        let start = Instant::now();
                        let _ = cache.get(&key).await;
                        latencies.push(start.elapsed());
                    }
                    latencies.sort();
                    let p50 = latencies[latencies.len() / 2];
                    let p95 = latencies[(latencies.len() * 95) / 100];
                    let p99 = latencies[(latencies.len() * 99) / 100];
                    black_box((p50, p95, p99));
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== TTL Adaptation Rate ==========

/// Measure how quickly TTL adapts to access patterns
fn bench_ttl_adaptation_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ttl_adaptation");
    group.sample_size(20);

    let iterations = 1000;

    group.bench_function("hot_key", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    hot_threshold: 5,
                    cold_threshold: 2,
                    adaptation_rate: 0.25,
                    max_entries: 100,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    cache.insert(0, TestValue::small(0)).await;
                });

                (cache, 0u32)
            },
            |(cache, key)| {
                rt().block_on(async {
                    // Access same key repeatedly to trigger TTL extension
                    for _ in 0..iterations {
                        let _ = cache.get(&key).await;
                    }
                    let final_ttl = cache.ttl(&key).await.unwrap();
                    black_box(final_ttl);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("cold_key", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    enable_adaptive_ttl: true,
                    base_ttl: Duration::from_secs(300),
                    hot_threshold: 5,
                    cold_threshold: 2,
                    adaptation_rate: 0.25,
                    max_entries: 100,
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                // Insert many keys, only access target once
                rt().block_on(async {
                    for i in 0..100 {
                        cache.insert(i, TestValue::small(i)).await;
                    }
                });

                (cache, 0u32)
            },
            |(cache, key)| {
                rt().block_on(async {
                    // Access key only once (should reduce TTL)
                    let _ = cache.get(&key).await;

                    // Wait a bit
                    tokio::time::sleep(Duration::from_millis(1)).await;

                    let final_ttl = cache.ttl(&key).await.unwrap();
                    black_box(final_ttl);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== Write Heavy Workload ==========

/// Performance under write-heavy workload
fn bench_write_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_heavy");
    group.sample_size(20);
    group.throughput(Throughput::Elements(1000));

    let config_write = TTLConfig::write_heavy();
    let config_default = TTLConfig::default();

    group.bench_function("write_heavy_config", |b| {
        b.iter_batched(
            || {
                let cache =
                    rt().block_on(async { AdaptiveTTLCache::new(config_write.clone()).unwrap() });

                (cache, 0u32)
            },
            |(cache, mut counter)| {
                rt().block_on(async {
                    for _ in 0..1000 {
                        cache.insert(counter, TestValue::small(counter)).await;
                        let _ = cache.get(&counter).await;
                        counter += 1;
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("default_config", |b| {
        b.iter_batched(
            || {
                let cache =
                    rt().block_on(async { AdaptiveTTLCache::new(config_default.clone()).unwrap() });

                (cache, 0u32)
            },
            |(cache, mut counter)| {
                rt().block_on(async {
                    for _ in 0..1000 {
                        cache.insert(counter, TestValue::small(counter)).await;
                        let _ = cache.get(&counter).await;
                        counter += 1;
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== Configuration Comparison ==========

/// Compare different configuration presets
fn bench_config_presets(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_presets");
    group.sample_size(20);

    let iterations: usize = 5000;
    let cache_size: usize = 1000;

    group.bench_function("default", |b| {
        let config = TTLConfig::default();
        b.iter_batched(
            || {
                let cache = rt().block_on(async { AdaptiveTTLCache::new(config.clone()).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, zipf_pattern(cache_size, iterations, 0.8))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    for &key in &pattern {
                        let _ = black_box(cache.get(&key).await);
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("high_hit_rate", |b| {
        let config = TTLConfig::high_hit_rate();
        b.iter_batched(
            || {
                let cache = rt().block_on(async { AdaptiveTTLCache::new(config.clone()).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, zipf_pattern(cache_size, iterations, 0.8))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    for &key in &pattern {
                        let _ = black_box(cache.get(&key).await);
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("memory_constrained", |b| {
        let config = TTLConfig::memory_constrained();
        b.iter_batched(
            || {
                let cache = rt().block_on(async { AdaptiveTTLCache::new(config.clone()).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, zipf_pattern(cache_size, iterations, 0.8))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    for &key in &pattern {
                        let _ = black_box(cache.get(&key).await);
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("write_heavy", |b| {
        let config = TTLConfig::write_heavy();
        b.iter_batched(
            || {
                let cache = rt().block_on(async { AdaptiveTTLCache::new(config.clone()).unwrap() });

                rt().block_on(async {
                    for i in 0..cache_size {
                        cache.insert(i as u32, TestValue::medium(i as u32)).await;
                    }
                });

                (cache, zipf_pattern(cache_size, iterations, 0.8))
            },
            |(cache, pattern)| {
                rt().block_on(async {
                    for &key in &pattern {
                        let _ = black_box(cache.get(&key).await);
                    }
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ========== Cleanup Performance ==========

/// Background cleanup performance
fn bench_cleanup_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cleanup");
    group.sample_size(20);

    let size = 1000;

    group.bench_function("manual_cleanup", |b| {
        b.iter_batched(
            || {
                let config = TTLConfig {
                    base_ttl: Duration::from_millis(100),
                    enable_background_cleanup: false,
                    ..TTLConfig::default()
                };
                let cache: AdaptiveTTLCache<u32, TestValue> =
                    rt().block_on(async { AdaptiveTTLCache::new(config).unwrap() });

                rt().block_on(async {
                    for i in 0..size {
                        cache.insert(i, TestValue::small(i)).await;
                    }
                    // Wait for entries to expire
                    tokio::time::sleep(Duration::from_millis(150)).await;
                });

                cache
            },
            |cache| {
                rt().block_on(async {
                    let removed = cache.cleanup_expired().await;
                    black_box(removed);
                });
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_static_vs_adaptive_baseline,
    bench_zipf_pattern,
    bench_sequential_pattern,
    bench_random_pattern,
    bench_hot_cold_pattern,
    bench_hit_rate_analysis,
    bench_memory_overhead,
    bench_latency_percentiles,
    bench_ttl_adaptation_rate,
    bench_write_heavy,
    bench_config_presets,
    bench_cleanup_performance,
);

criterion_main!(benches);
