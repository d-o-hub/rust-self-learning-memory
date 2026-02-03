# Load and Soak Tests

This directory contains comprehensive load and soak tests for validating production confidence in the memory management system.

## Overview

The test suite is designed to validate:
- Connection pool stability under heavy load
- Cache behavior and hit rates under load
- Batch operation throughput and transaction safety
- Long-term system stability (24-hour soak test)
- Rate limiter accuracy and recovery
- Memory leak detection
- Performance regression detection

## Test Structure

```
tests/
├── load/                          # Load tests (Day 1)
│   ├── connection_pool_test.rs    # 1000 concurrent connections, 10K queries
│   ├── cache_load_test.rs         # 100K episodes, cache hit rate > 80%
│   ├── batch_operations_test.rs   # 10K episodes, 50K patterns, throughput 200-300/sec
│   └── mod.rs
├── soak/                          # Soak tests (Day 2)
│   ├── stability_test.rs          # 24-hour stability test
│   ├── rate_limiter_test.rs       # Rate limiter accuracy and recovery
│   └── mod.rs
└── Cargo.toml
```

## Test Specifications

### Day 1: Load Tests

#### 1. Connection Pool Load Test (`connection_pool_test.rs`)

**Objective**: Validate connection pool behavior under heavy concurrent load.

**Test Configuration**:
- Max concurrent connections: 1000
- Total queries: 10,000
- Maximum test duration: 10 minutes

**Acceptance Criteria**:
- ✅ 1000 concurrent connections executed successfully
- ✅ 10,000 queries across connections
- ✅ No connection exhaustion (zero connection timeouts)
- ✅ Pool scaling behavior validated

**Expected Results**:
- Success rate: 100%
- Zero connection pool exhaustion events
- Proper pool expansion and contraction

**Run**: `cargo test --test connection_pool_test`

---

#### 2. Cache Load Test (`cache_load_test.rs`)

**Objective**: Validate cache behavior under heavy query load and memory pressure.

**Test Configuration**:
- Episodes to insert: 100,000
- Query iterations: 10,000
- Cache size: 10,000 episodes
- Maximum test duration: 5 minutes

**Acceptance Criteria**:
- ✅ 100,000 episodes inserted successfully
- ✅ Cache hit rate > 80%
- ✅ Cache eviction works under memory pressure
- ✅ Cache clear operations function correctly

**Expected Results**:
- Cache hit rate: > 80%
- Cache eviction properly removes old entries
- Recent episodes have higher hit rates than old episodes

**Run**: `cargo test --test cache_load_test`

---

#### 3. Batch Operations Load Test (`batch_operations_test.rs`)

**Objective**: Validate batch operation performance and transaction safety.

**Test Configuration**:
- Episodes to batch insert: 10,000
- Patterns to batch insert: 50,000
- Heuristics to batch insert: 1,000
- Batch size: 100
- Target throughput: 200-300 ops/sec
- Maximum test duration: 10 minutes

**Acceptance Criteria**:
- ✅ All batch operations complete successfully
- ✅ Meets throughput target (200-300 ops/sec)
- ✅ Transaction safety validated
- ✅ Data integrity verified

**Expected Results**:
- Episode throughput: ≥ 200 ops/sec
- Pattern throughput: ≥ 200 ops/sec
- Zero data loss or corruption
- Accurate episode and pattern counts

**Run**: `cargo test --test batch_operations_test`

---

### Day 2: Soak Tests

#### 4. 24-Hour Stability Test (`stability_test.rs`)

**Objective**: Validate long-term system stability and detect memory leaks.

**Test Configuration**:
- Runtime: 24 hours (or 60s for CI/fast testing)
- Worker count: 4 concurrent workers
- Episodes per cycle: 10
- Memory check interval: 30 seconds
- Performance snapshot interval: 5 minutes

**Acceptance Criteria**:
- ✅ 24-hour continuous operation without failure
- ✅ No memory leaks detected (< 50% growth)
- ✅ Performance remains stable (P95 latency < 100% growth)
- ✅ Periodic metrics generated

**Expected Results**:
- Success rate: > 99%
- Memory growth: < 50%
- P95 latency growth: < 100%
- System remains stable throughout

**Run (CI mode)**: `cargo test --test stability_test`

**Run (full 24-hour)**: `cargo test --test stability_test -- --ignored --features full-soak`

---

#### 5. Rate Limiter Soak Test (`rate_limiter_test.rs`)

**Objective**: Validate rate limiter accuracy and recovery mechanisms.

**Test Configuration**:
- Rate limit: 100 requests/second
- Burst capacity: 150 requests
- Sustained load duration: 60 seconds
- Burst load duration: 10 seconds
- Recovery test duration: 30 seconds

**Acceptance Criteria**:
- ✅ Rate limiter accuracy > 95%
- ✅ Burst handling within tolerance
- ✅ Rate limiter recovers after limit
- ✅ System remains stable

**Expected Results**:
- Rate limiting accuracy: > 95%
- Burst capacity utilization: > 80%
- Recovery success rate: > 80%

**Run**: `cargo test --test rate_limiter_test`

---

### Day 3: Analysis

#### 6. Memory Profiling

**Objective**: Detect memory leaks and analyze allocation patterns.

**Tools**:
- Valgrind (if available)
- System memory monitoring
- Rust heap profiling

**Procedure**:
1. Run tests under memory profiler
2. Check for memory leaks
3. Analyze allocation patterns
4. Optimize if needed

**Run**: `valgrind --leak-check=full cargo test --test connection_pool_test`

---

#### 7. Performance Regression Detection

**Objective**: Compare current performance against v0.1.13 baseline.

**Procedure**:
1. Measure throughput and latency
2. Compare against baseline metrics
3. Detect any regressions > 5%
4. Generate performance report

**Acceptance Criteria**:
- ✅ Zero performance regressions > 5%

---

## Running the Tests

### Run All Load Tests (Day 1)

```bash
# Run individual tests
cargo test --test connection_pool_test
cargo test --test cache_load_test
cargo test --test batch_operations_test

# Or use the comprehensive test runner
./scripts/run_load_soak_tests.sh
```

### Run All Soak Tests (Day 2)

```bash
# Run rate limiter test
cargo test --test rate_limiter_test

# Run stability test (CI mode - 60s)
cargo test --test stability_test

# Run full 24-hour stability test
cargo test --test stability_test -- --ignored --features full-soak
```

### Run Analysis (Day 3)

```bash
# Memory leak analysis with Valgrind
valgrind --leak-check=full --show-leak-kinds=all \
    cargo test --test connection_pool_test --no-run

# Run comprehensive test suite with analysis
./scripts/run_load_soak_tests.sh
```

### Run Complete Suite

```bash
# Run all tests with automated reporting
./scripts/run_load_soak_tests.sh

# View generated reports
cat plans/test-reports-summary_*.csv
```

## Test Reports

The test runner generates comprehensive reports in `plans/test-reports/`:

- `summary_<timestamp>.csv`: CSV summary of all test results
- `test_<timestamp>_<test_name>.txt`: Detailed output for each test
- `valgrind_<timestamp>.txt`: Memory leak analysis results
- `performance_report_<timestamp>.md`: Performance comparison report

## Acceptance Criteria Summary

All tests must meet the following criteria:

| Test | Criteria |
|------|----------|
| Connection Pool | 1000 concurrent, 10K queries, 100% success |
| Cache Load | 100K episodes, >80% hit rate |
| Batch Operations | 200-300 ops/sec, zero data loss |
| 24-Hour Stability | No failures, <50% memory growth, <100% latency growth |
| Rate Limiter | >95% accuracy, >80% burst utilization, >80% recovery |

## Performance Targets

Based on v0.1.13 baseline:

| Operation | Target (P95) | v0.1.13 Actual | Notes |
|-----------|-------------|----------------|-------|
| Episode Creation | < 50ms | ~2.5 μs | 19,531x faster than target |
| Step Logging | < 20ms | ~1.1 μs | 17,699x faster than target |
| Episode Completion | < 500ms | ~3.8 μs | 130,890x faster than target |
| Pattern Extraction | < 1000ms | ~10.4 μs | 95,880x faster than target |
| Memory Retrieval | < 100ms | ~721 μs | 138x faster than target |

**Regression Threshold**: Any performance degradation > 5% from baseline is considered a regression.

## Troubleshooting

### Connection Pool Test Fails

- Check database file permissions
- Ensure sufficient system resources
- Increase connection timeout if needed

### Cache Hit Rate Below 80%

- Verify cache is properly enabled
- Check cache size configuration
- Review access patterns

### Batch Operations Too Slow

- Increase batch size
- Check disk I/O performance
- Verify database indexing

### Stability Test Memory Growth

- Check for memory leaks with Valgrind
- Review episode cleanup logic
- Verify cache eviction

### Rate Limiter Test Failures

- Verify rate limiter configuration
- Check time synchronization
- Review burst capacity settings

## Continuous Integration

These tests are designed to run in CI environments with modified durations:

- **CI Mode**: Short durations (60s-10min) for fast feedback
- **Full Mode**: Full durations for production validation

To configure CI mode, tests use conditional compilation flags:

```toml
[features]
full-soak = []
```

## Contributing

When modifying the memory system, ensure all load and soak tests pass:

1. Run Day 1 load tests (quick validation)
2. Run soak tests (stability validation)
3. Compare performance against baseline
4. Update baseline if improvements are made

## References

- Original task: Feature implementation - Load and Soak Tests
- Baseline performance: v0.1.13
- Test framework: Cargo test, Criterion benchmarks
