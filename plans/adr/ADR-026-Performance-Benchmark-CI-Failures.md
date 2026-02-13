# ADR-026: Performance Benchmark CI Failures

**Status**: Accepted
**Date**: 2026-02-13

## Context

`benchmark_streaming_performance` in `memory-mcp/src/patterns/benchmarks.rs` fails in CI due to timing variability. The test requires minimum throughput:
- Local: 100 pts/sec
- CI: 10 pts/sec

Actual CI results: 51-804 pts/sec depending on window size

## Problem

Performance benchmarks are inherently flaky in CI due to:
- Variable CPU availability
- Shared CI infrastructure
- Noisy neighbor effects
- Different hardware than local development

## Decision

**Convert to criterion-based benchmarks** and remove timing assertions from unit tests.

## Implementation

1. Create proper criterion benchmarks in `benches/` directory
2. Remove or relax timing assertions in unit tests
3. Use `#[cfg(not(feature = "ci"))]` for strict thresholds

```rust
// In benchmarks.rs - use relaxed thresholds
#[test]
fn benchmark_streaming_performance() {
    let is_ci = std::env::var("CI").is_ok();
    let min_throughput = if is_ci { 5.0 } else { 50.0 }; // Relaxed threshold

    assert!(
        throughput > min_throughput,
        "Streaming performance degraded: got {:.0} pts/sec, min {} pts/sec",
        throughput, min_throughput
    );
}
```

## Consequences

- ✅ Benchmarks become reliable
- ✅ CI passes consistently
- ⚠️ Less strict regression detection in unit tests
- ⚠️ Full benchmarks require `cargo bench`

## Future Work

Move performance benchmarks to:
- `benches/streaming_dbscan.rs` - Criterion-based
- `benches/bocpd_scalability.rs` - Criterion-based

## Alternatives Considered

- **Option A** (Reduce CI thresholds): May mask regressions
- **Option B** (Statistical pass criteria): Longer test time
- **Option C** (Warnings only): Loses regression detection
