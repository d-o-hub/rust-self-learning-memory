# Benchmark Analysis Guide

## Criterion Benchmarks

### Setup

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "episode_ops"
harness = false
```

### Basic Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_episode_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("episode_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let memory = setup_memory().await;
            black_box(
                memory.start_episode(
                    "Benchmark task".to_string(),
                    TaskContext::default(),
                    TaskType::CodeGeneration,
                ).await
            )
        });
    });
}

criterion_group!(benches, bench_episode_creation);
criterion_main!(benches);
```

## Benchmark Comparison

### Baseline Management

```bash
# Save baseline before changes
cargo bench --bench episode_ops -- --save-baseline main

# After making changes
cargo bench --bench episode_ops -- --baseline main

# Compare multiple baselines
cargo bench --bench episode_ops -- --baseline main --baseline develop
```

### Regression Detection

```rust
fn compare_benchmarks() {
    let baseline = load_baseline("main").unwrap();
    let current = load_baseline("current").unwrap();

    for (name, current_result) in current {
        if let Some(baseline_result) = baseline.get(&name) {
            let change = (current_result.mean - baseline_result.mean) / baseline_result.mean;

            if change > 0.10 {
                eprintln!("REGRESSION: {} is {:.1}% slower", name, change * 100);
            } else if change < -0.10 {
                eprintln!("IMPROVEMENT: {} is {:.1}% faster", name, -change * 100);
            }
        }
    }
}
```

## Performance Metrics

| Metric | Description | Interpretation |
|--------|-------------|----------------|
| mean | Average execution time | Overall performance |
| stddev | Variation in timing | Consistency |
| median | Middle value | Robust to outliers |
| change | % vs baseline | Regression detection |

## Common Benchmarks

```rust
// Episode lifecycle
fn bench_episode_lifecycle(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("episode_lifecycle", |b| {
        b.to_async(&rt).iter(|| async {
            let memory = setup_memory().await;
            let id = memory.start_episode("bench".into(), Default::default(), TaskType::CodeGeneration).await;
            memory.complete_episode(id, TaskOutcome::Success, None).await
        });
    });
}

// Pattern extraction
fn bench_pattern_extraction(c: &mut Criterion) {
    // Pattern extraction benchmarks
}

// Memory retrieval
fn bench_memory_retrieval(c: &mut Criterion) {
    // Context-based retrieval benchmarks
}
```
