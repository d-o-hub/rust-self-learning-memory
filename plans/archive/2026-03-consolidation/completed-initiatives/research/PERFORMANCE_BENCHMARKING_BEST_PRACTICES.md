# Performance Benchmarking Best Practices

**Document Type**: Research / Best Practices
**Version**: 1.0
**Created**: 2025-12-25
**Purpose**: Establish comprehensive performance benchmarking methodology for memory system

---

## Executive Summary

Establish best practices for performance benchmarking in self-learning memory system, drawing from existing benchmarks, Rust ecosystem standards, and performance engineering principles.

**Key Principles**: Reproducibility, Relevance, Comprehensiveness, Automation, Actionability

---

## Current Benchmarking State

### Existing Benchmarks
**Location**: `benches/`
**Performance Baselines**:
| Operation | Target (P95) | Actual (P95) | Margin | Status |
|-----------|-------------|--------------|--------|--------|
| Episode Creation | <50ms | 2.56 µs | 19,531x faster | ✅ |
| Step Logging | <20ms | 1.13 µs | 17,699x faster | ✅ |
| Episode Completion | <500ms | 3.82 µs | 130,890x faster | ✅ |
| Pattern Extraction | <1000ms | 10.43 µs | 95,880x faster | ✅ |
| Memory Retrieval | <100ms | 721.01 µs | 138x faster | ✅ |
| Concurrent Ops | >100 eps/s | 9.96ms per op | 502x faster | ✅ |

### Benchmarking Tools
- **Criterion.rs**: Statistical benchmarking framework
- **Cargo Bench**: Rust's built-in benchmarking

---

## Benchmarking Methodology

### 1. Benchmark Categories

#### 1.1 Microbenchmarks (Unit Level)
**Purpose**: Measure individual function performance
**Example**:
```rust
#[bench]
fn bench_episode_creation(b: &mut Bencher) {
    let memory = setup_memory();
    b.iter(|| {
        let episode = create_test_episode();
        memory.start_episode(episode).unwrap()
    });
}
```

#### 1.2 Macrobenchmarks (Integration Level)
**Purpose**: Measure realistic workflows
**Example**:
```rust
#[bench]
fn bench_episode_lifecycle(b: &mut Bencher) {
    let memory = setup_memory();
    b.iter(|| {
        let episode = memory.start_episode(create_test_episode()).unwrap();
        for step in create_test_steps(50) {
            memory.log_step(episode.id, step).unwrap();
        }
        memory.complete_episode(episode.id, create_outcome()).unwrap();
    });
}
```

#### 1.3 Scalability Benchmarks
**Purpose**: Measure performance scaling with data volume
**Example**:
```rust
fn bench_scalability() {
    for size in [100, 1000, 10000, 100000] {
        let dataset = generate_dataset(size);
        let duration = bench_retrieval(&dataset);
        println!("Size: {}, Duration: {:?}", size, duration);
    }
}
```

#### 1.4 Concurrency Benchmarks
**Purpose**: Measure performance under concurrent load
**Example**:
```rust
#[bench]
fn bench_concurrent_storage(b: &mut Bencher) {
    let memory = Arc::new(setup_memory());
    let concurrency = 10;

    b.iter(|| {
        let handles: Vec<_> = (0..concurrency)
            .map(|_| {
                let memory = memory.clone();
                thread::spawn(move || memory.store_episode(create_test_episode()))
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    });
}
```

#### 1.5 Memory Benchmarks
**Purpose**: Measure memory usage and leaks
**Example**:
```rust
fn bench_memory_pressure() {
    let memory = setup_memory();
    let baseline = memory_usage();

    for i in 0..1000 {
        memory.store_episode(create_test_episode()).unwrap();
    }

    let peak = memory_usage();
    assert!(peak - baseline < MAX_GROWTH);
}
```

---

## Benchmarking Design Principles

### 2.1 Representativeness
**Principle**: Benchmarks should reflect real-world usage
**Guidelines**:
- Use realistic data (not random strings)
- Simulate actual workflow patterns
- Include I/O operations (network, disk)
- Model typical access patterns

### 2.2 Reproducibility
**Principle**: Results must be consistent across runs
**Guidelines**:
- Use deterministic data generation
- Fix random seeds
- Control environment variables
- Document system specifications

**Example**:
```rust
fn setup_benchmark() -> SelfLearningMemory {
    let mut rng = StdRng::seed_from_u64(42); // Fixed seed
    let data = generate_test_data(&mut rng, 1000);
    // ...
}
```

### 2.3 Statistical Validity
**Principle**: Results must be statistically significant
**Guidelines**:
- Use Criterion.rs for statistical analysis
- Run sufficient iterations (100-1000)
- Measure confidence intervals
- Identify outliers

**Criterion.rs Example**:
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("retrieval");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let memory = setup_with_episodes(size);
            b.iter(|| memory.query_memory("test", 10));
        });
    }

    group.finish();
}
```

---

## Performance Metrics

### 3.1 Latency Metrics
- **P50**: Median latency (50th percentile)
- **P95**: 95th percentile (common SLA target)
- **P99**: 99th percentile (tail latency)
- **Mean**: Average latency
- **Min/Max**: Range of latencies

### 3.2 Throughput Metrics
- **Operations/sec**: Number of operations per second
- **Episodes/sec**: Episodes stored/retrieved per second
- **Patterns/sec**: Patterns extracted per second

### 3.3 Resource Metrics
- **Memory Usage**: Peak and average memory consumption
- **CPU Usage**: CPU time and utilization
- **Disk I/O**: Read/write operations and bytes
- **Network I/O**: Request/response bytes (for Turso)

---

## Benchmarking Environment

### 4.1 Hardware Configuration
**Document baseline configuration**:
```toml
[hardware]
cpu = "AMD EPYC 7763 @ 2.45GHz"
cores = 64
memory = "256GB RAM"
disk = "NVMe SSD (Samsung 980 PRO)"
```

### 4.2 Environment Control
```bash
# Disable turbo boost for consistent CPU frequency
sudo cpupower frequency-set -g performance

# Set process priority
nice -n 0 cargo bench

# Disable swap for memory benchmarks
sudo swapoff -a

# Flush filesystem caches
sync; echo 3 | sudo tee /proc/sys/vm/drop_caches
```

---

## Continuous Benchmarking

### 5.1 CI/CD Integration
```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks
on: [push, pull_request]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --all -- --save-baseline main
      - name: Compare with baseline
        run: cargo bench --all -- --baseline main
```

### 5.2 Regression Detection
```rust
const MAX_REGRESSION_PCT: f64 = 10.0;

fn check_regression(current: Duration, baseline: Duration) -> Result<()> {
    let regression_pct = ((current.as_nanos() as f64 - baseline.as_nanos() as f64)
        / baseline.as_nanos() as f64) * 100.0;

    if regression_pct > MAX_REGRESSION_PCT {
        return Err(Error::PerformanceRegression { regression_pct });
    }
    Ok(())
}
```

---

## Benchmark Analysis

### 6.1 Performance Profiling
```bash
# Use flamegraph for CPU profiling
cargo flamegraph --bench episode_lifecycle

# Use heaptrack for memory profiling
heaptrack cargo bench --bench memory_pressure
```

### 6.2 Trend Analysis
**Track performance over time**:
```
Metric: Episode Creation Latency (P95)
Date        | Value    | Trend
2025-12-20 | 2.56µs   | Baseline
2025-12-21 | 2.58µs   | +0.78% ✓
2025-12-22 | 3.12µs   | +21.9% ⚠️
```

---

## Benchmarking Checklist

### Before Benchmarking
- [ ] Environment configured (CPU, memory, disk)
- [ ] Hardware specifications documented
- [ ] System state optimized (caches flushed, swap off)
- [ ] Benchmark data prepared

### During Benchmarking
- [ ] Warm-up runs completed
- [ ] Sufficient iterations executed
- [ ] Outliers identified and handled
- [ ] Results collected for all metrics

### After Benchmarking
- [ ] Results validated (sanity check)
- [ ] Statistical analysis performed
- [ ] Comparison with baseline documented
- [ ] Results archived

---

## Best Practices Summary

### DO:
✅ Use Criterion.rs for statistical benchmarking
✅ Benchmark realistic workloads and data
✅ Document benchmark environment and configuration
✅ Isolate benchmarks from each other
✅ Run warm-up iterations
✅ Store benchmark results for trend analysis
✅ Detect performance regressions in CI/CD
✅ Profile bottlenecks for optimization

### DON'T:
✗ Benchmark with artificial data (random strings)
✗ Mix benchmarks with tests
✗ Ignore cold start effects
✗ Rely on single-run results
✗ Change benchmark environment mid-run
✗ Skip documentation of configuration

---

## References

- **Criterion.rs**: https://bheisler.github.io/criterion.rs/book/
- **Rust Benchmarking**: https://doc.rust-lang.org/1.81.0/book/ch11-01-testing.html#benchmark-functions
- **Flamegraph**: https://github.com/flamegraph-rs/flamegraph

---

**Document Status**: ✅ Best Practices Documented
**Last Updated**: 2025-12-25
**Next Review**: 2026-03-25 (quarterly review)
