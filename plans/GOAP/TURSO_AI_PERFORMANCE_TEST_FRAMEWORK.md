# Turso AI Performance Test Framework Design

**Date**: 2025-12-29
**Agent**: testing-qa
**Purpose**: Design for performance benchmarking and regression detection

## Overview

Performance testing framework for Turso AI enhancements must measure:
1. **Search Latency**: Vector search performance across dimensions
2. **Memory Usage**: Storage requirements for embeddings
3. **Scalability**: Performance as dataset size grows
4. **Regression Detection**: Automated detection of performance regressions

## Current Benchmark Suite

### Existing Benchmarks (`benches/turso_vector_performance.rs`)
1. `benchmark_384_dim_native_search`: Native vector search for 384-dim embeddings
2. `benchmark_1536_dim_brute_force_search`: Brute-force simulation for 1536-dim
3. `benchmark_memory_usage`: Memory calculation for different dimensions
4. `benchmark_json_query_performance`: JSON queries vs Rust deserialization
5. `benchmark_embedding_storage`: Embedding storage performance

### Gaps Identified
1. No native vector search benchmarks for 1536-dim, 1024-dim, 3072-dim
2. No hybrid search benchmarks (FTS5 + vector)
3. No extension performance benchmarks (JSON, Stats, Crypto)
4. No automated regression detection in CI

## Extended Benchmark Suite Design

### 1. Dimension-Specific Native Search Benchmarks

**New Benchmarks to Add**:
- `benchmark_1024_dim_native_search`: For 1024-dim embeddings
- `benchmark_1536_dim_native_search`: For 1536-dim embeddings (replaces brute-force)
- `benchmark_3072_dim_native_search`: For 3072-dim embeddings
- `benchmark_mixed_dimension_search`: Mix of dimensions in same database

**Implementation Requirements**:
- Update `setup_storage_with_data()` to use dimension-specific tables
- Ensure native vector search is enabled for each dimension
- Measure latency across different dataset sizes (100, 1K, 10K, 100K)

### 2. Hybrid Search Benchmarks

**New Benchmark File**: `benches/hybrid_search_performance.rs`

**Benchmarks**:
- `benchmark_fts5_keyword_search`: Pure keyword search performance
- `benchmark_hybrid_search_blending`: Combined vector + keyword search
- `benchmark_hybrid_search_scaling`: Performance with growing dataset
- `benchmark_hybrid_search_relevance`: Quality metrics (precision/recall)

**Implementation Details**:
- Create FTS5 virtual tables for episodes and patterns
- Implement hybrid ranking algorithm (alpha blending)
- Generate test queries with known relevance ground truth

### 3. Extension Performance Benchmarks

**New Benchmark File**: `benches/sqlite_extensions_performance.rs`

**Benchmarks**:
- `benchmark_json_functions`: `json_extract`, `json_group_array` vs Rust deserialization
- `benchmark_stats_functions`: `mean`, `median`, `stddev` vs Rust calculations
- `benchmark_crypto_functions`: `sha256`, `hmac` vs Rust crypto crates
- `benchmark_uuid_functions`: `uuid()` generation vs Rust uuid crate

**Implementation Details**:
- Feature-flag each extension for graceful fallback
- Compare performance and accuracy against Rust implementations
- Measure memory overhead of extension usage

### 4. Scalability Benchmarks

**New Benchmark File**: `benches/scalability_analysis.rs`

**Benchmarks**:
- `benchmark_scaling_embedding_count`: 1K to 1M embeddings
- `benchmark_scaling_concurrent_queries`: 1 to 100 concurrent searches
- `benchmark_scaling_mixed_workloads`: Reads/writes simultaneous
- `benchmark_memory_pressure`: Memory usage under load

**Implementation Details**:
- Use `tokio::spawn` for concurrent operations
- Measure throughput and latency percentiles (P50, P95, P99)
- Monitor system resource usage (memory, CPU)

## Regression Detection Framework

### Baseline Management

**Baseline Storage**: `benchmark_results/` directory with timestamped files
- `phase0_baseline.json`: Current performance before enhancements
- `phase1_baseline.json`: After multi-dimension implementation
- `phase2_baseline.json`: After index optimization
- etc.

**Baseline Format**:
```json
{
  "timestamp": "2025-12-29T12:00:00Z",
  "git_commit": "abc123",
  "benchmarks": {
    "384_dim_native_search/100": {
      "mean_ns": 5234000,
      "stddev_ns": 120000,
      "throughput": 191.2
    }
  }
}
```

### Automated Regression Detection

**Script**: `scripts/check_performance_regression.sh` (enhanced)

**Logic**:
1. Run benchmark suite with `criterion --save-baseline current`
2. Load appropriate baseline file based on phase
3. Compare mean execution times
4. Fail if any benchmark regresses >10%
5. Allow configurable thresholds per benchmark

**CI Integration**: GitHub Actions job `performance-regression`
- Runs on PRs to main/develop
- Stores artifacts with benchmark results
- Comments PR with performance comparison

### Statistical Validation

**Requirements**:
- Multiple runs (minimum 10 samples)
- Confidence intervals (95%)
- Outlier detection and removal
- Warm-up iterations excluded

**Implementation**:
- Use Criterion.rs built-in statistical analysis
- Configure sample size and measurement time
- Store raw measurements for later analysis

## Implementation Timeline

### Phase 0 (Preparation)
- [x] Analyze existing benchmark suite
- [ ] Design extended benchmark structure
- [ ] Create baseline measurement procedure

### Phase 1 (Multi-Dimension Support)
- [ ] Add dimension-specific native search benchmarks
- [ ] Update `setup_storage_with_data` for multi-dimension tables
- [ ] Establish Phase 1 baseline

### Phase 2 (Index Optimization)
- [ ] Add index configuration benchmarks
- [ ] Compare different DiskANN parameter settings
- [ ] Establish Phase 2 baseline

### Phase 3 (Hybrid Search)
- [ ] Implement hybrid search benchmarks
- [ ] Measure relevance quality metrics
- [ ] Establish Phase 3 baseline

### Phase 4 (Extensions)
- [ ] Implement extension performance benchmarks
- [ ] Compare against Rust implementations
- [ ] Establish Phase 4 baseline

## Quality Gates

### Performance Quality Gates
| Gate | Threshold | Measurement |
|------|-----------|-------------|
| Native search latency | <10ms (P95) | 10K embeddings |
| Hybrid search latency | <100ms (P95) | 10K episodes |
| Memory usage | <100MB | 100K embeddings |
| Regression detection | <10% degradation | All benchmarks |

### Implementation Quality Gates
- [ ] All benchmarks compile and run
- [ ] Baseline measurements stored
- [ ] Regression detection works in CI
- [ ] Statistical validation passes

## Integration with Existing Infrastructure

### Criterion.rs Configuration
Update `Cargo.toml` with optimized criterion configuration:
```toml
[dev-dependencies.criterion]
version = "0.5"
features = ["html_reports", "async_tokio"]
```

### GitHub Actions Workflow
Add new job to `.github/workflows/ci-enhanced.yml`:
```yaml
performance-regression:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run benchmarks
      run: cargo bench --bench turso_vector_performance -- --save-baseline current
    - name: Check regression
      run: ./scripts/check_performance_regression.sh current phase0_baseline
```

### Reporting
Generate HTML reports with:
- `criterion --output-format html`
- Store artifacts in GitHub Actions
- Generate trend graphs over time

## Risk Mitigation

### Technical Risks
1. **Benchmark Flakiness**: Use deterministic test data, isolate benchmarks
2. **Resource Constraints**: Limit benchmark sizes in CI, use dedicated runners
3. **Statistical Noise**: Increase sample size, use warm-up iterations

### Coordination Risks
1. **Baseline Management**: Version control baseline files, clear ownership
2. **Threshold Setting**: Realistic thresholds based on current performance
3. **False Positives**: Manual review process for regression failures

## Success Metrics

### Quantitative
- [ ] All new benchmarks implemented
- [ ] Regression detection catches >10% performance changes
- [ ] Benchmarks run within CI time budget (<30 minutes)
- [ ] Performance trends tracked over 10+ commits

### Qualitative
- [ ] Clear performance reports for developers
- [ ] Actionable insights from benchmark results
- [ ] Easy addition of new benchmarks
- [ ] Minimal false positives in regression detection

---

*Performance Test Framework Design v1.0*
*Created by testing-qa agent on 2025-12-29*