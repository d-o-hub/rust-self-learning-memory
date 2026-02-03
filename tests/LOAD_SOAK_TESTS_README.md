# Load and Soak Tests

This directory contains comprehensive load and soak tests for validating the self-learning memory system's performance and stability.

## Overview

### Load Tests (`benches/load_tests.rs`)

Load tests measure system performance under various concurrent workloads:

1. **Concurrent Episode Creation**
   - 100 concurrent episode creations
   - Measures throughput (episodes/second)
   - Measures latency percentiles (P50, P95, P99)

2. **Relationship Stress Test**
   - Creates 1000 episodes
   - Creates 5000 relationships between them
   - Queries relationship graph
   - Measures query performance

3. **Pattern Extraction Load**
   - Creates episodes with steps
   - Triggers pattern extraction
   - Measures extraction throughput

4. **Memory Pressure Test**
   - Creates 10,000 episodes
   - Monitors memory usage
   - Verifies no memory leaks

5. **Mixed Workload**
   - Combination of read and write operations
   - 67% reads, 33% writes
   - Simulates realistic usage patterns

### Soak Tests (`benches/soak_tests.rs`)

Soak tests validate long-term stability:

1. **24-Hour Stability Test**
   - Continuous operations for 24 hours
   - Logs metrics every hour
   - Verifies no crashes or memory leaks
   - Run with: `cargo bench --bench soak_tests --features full-soak`

2. **Connection Pool Stability**
   - Continuous connection acquire/release
   - Monitors pool metrics over time
   - Detects connection leaks

3. **Memory Leak Detection**
   - Monitors memory usage over time
   - Detects memory growth patterns
   - Validates garbage collection

4. **Performance Degradation Detection**
   - Monitors latency trends over time
   - Detects performance degradation
   - Validates sustained performance

### Stability Framework (`tests/stability/mod.rs`)

A reusable framework for building stability tests:

- **TestConfig**: Configure test duration, workers, thresholds
- **TestMetrics**: Collect and analyze performance metrics
- **StabilityTest**: Run tests with automatic monitoring

## Running the Tests

### Load Tests

```bash
# Run all load tests
cd benches && cargo bench --bench load_tests

# Run specific load test
cd benches && cargo bench --bench load_tests -- concurrent_episode_creation
cd benches && cargo bench --bench load_tests -- relationship_stress
cd benches && cargo bench --bench load_tests -- pattern_extraction
cd benches && cargo bench --bench load_tests -- memory_pressure
```

### Soak Tests

```bash
# Run short soak tests (default - 1-5 minutes)
cd benches && cargo bench --bench soak_tests

# Run full 24-hour soak test
cd benches && cargo bench --bench soak_tests --features full-soak

# Run specific soak test
cd benches && cargo bench --bench soak_tests -- 24h_stability
cd benches && cargo bench --bench soak_tests -- connection_pool
cd benches && cargo bench --bench soak_tests -- memory_leak
cd benches && cargo bench --bench soak_tests -- performance_degradation
```

### Stability Framework Tests

```bash
# Run CI stability test (1 minute)
cargo test --test stability_24h -- test_ci_stability

# Run custom stability test (10 seconds)
cargo test --test stability_24h -- test_custom_stability

# Run full 24-hour stability test
cargo test --test stability_24h -- test_24h_stability --ignored
```

## Test Configuration

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `debug`, `info`)
- `TEST_DURATION`: Override default test duration (in seconds)

### Features

- `full-soak`: Enable full 24-hour soak tests (default is shortened for CI)

## Metrics Collected

### Performance Metrics

- **Throughput**: Operations per second
- **Latency**: P50, P95, P99 percentiles
- **Success Rate**: Percentage of successful operations

### Memory Metrics

- **RSS**: Resident Set Size (MB)
- **VMS**: Virtual Memory Size (MB)
- **Growth**: Memory growth percentage

### Stability Metrics

- **Uptime**: Continuous operation time
- **Error Count**: Number of failed operations
- **Degradation**: Performance change over time

## Interpreting Results

### Success Criteria

- **Success Rate**: > 95%
- **Memory Growth**: < 50% over test duration
- **Performance Degradation**: < 100% latency increase
- **No Crashes**: Zero panics or unexpected errors

### Example Output

```
╔════════════════════════════════════════════════════════════╗
║           24-Hour Stability Soak Test Summary           ║
╚════════════════════════════════════════════════════════════╝
  Duration:          24h 0m 0s
  Total Operations:  8640000
  Successful:        8639280
  Failed:            720
  Success Rate:      99.99%
  Throughput:        100.00 ops/sec

  Latency Metrics:
    Average:         2.50 ms
    P50:             2.10 ms
    P95:             5.20 ms
    P99:             8.50 ms

  Memory Usage:      256.0 MB
══════════════════════════════════════════════════════════════
```

## Continuous Integration

The tests are designed to run in CI with shortened durations:

```yaml
# Example GitHub Actions workflow
- name: Run Load Tests
  run: cd benches && cargo bench --bench load_tests

- name: Run Soak Tests (Short)
  run: cd benches && cargo bench --bench soak_tests

- name: Run Stability Tests
  run: cargo test --test stability_24h -- test_ci_stability
```

## Troubleshooting

### High Memory Usage

If memory usage grows significantly:
1. Check for memory leaks in storage backends
2. Verify cache eviction policies
3. Review connection pool configuration

### Low Throughput

If throughput is lower than expected:
1. Check CPU and I/O utilization
2. Verify database connection limits
3. Review concurrent operation settings

### Test Failures

If tests fail:
1. Check `RUST_LOG=debug` output
2. Review metrics snapshots
3. Analyze failure patterns

## Adding New Tests

### Load Test Example

```rust
fn benchmark_my_load_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_my_test");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("my_operation", |b| {
        b.to_async(TokioExecutor).iter_custom(|iters| async move {
            // Your test implementation
        });
    });

    group.finish();
}
```

### Stability Test Example

```rust
#[tokio::test]
async fn test_my_stability() {
    let config = TestConfig {
        duration: Duration::from_secs(300),
        worker_count: 4,
        ..Default::default()
    };

    let test = StabilityTest::new(config);

    test.run(|metrics: Arc<TestMetrics>| async move {
        let start = Instant::now();
        // Your operation
        metrics.record_operation(true, start.elapsed());
    }).await;
}
```

## See Also

- `tests/load/` - Additional load tests
- `tests/soak/` - Additional soak tests
- `benches/` - Performance benchmarks
- `docs/PERFORMANCE.md` - Performance documentation
