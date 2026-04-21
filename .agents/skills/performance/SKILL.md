# Performance Skill

Benchmarking and performance optimization for the Rust self-learning memory system.

## Quick Reference

- **[Benchmarking](benchmarking.md)** - Criterion patterns and profiling
- **[Optimization](optimization.md)** - CPU/memory optimization strategies
- **[Profiling](profiling.md)** - perf, flamegraph, tokio-console

## When to Use

- Benchmarking hot paths before/after changes
- Profiling CPU/memory bottlenecks
- Validating performance improvements
- Regression detection in CI

## Key Metrics

| Operation | Target (P95) | Typical |
|-----------|-------------|---------|
| Episode Creation | < 50ms | ~2.5 µs |
| Step Logging | < 20ms | ~1.1 µs |
| Episode Completion | < 500ms | ~3.8 µs |
| Pattern Extraction | < 1000ms | ~10.4 µs |
| Memory Retrieval | < 100ms | ~721 µs |

## Benchmarking Workflow

### 1. Criterion Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench retrieval_quality

# Compare baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main  # Compare against saved
```

### 2. Profiling Commands

```bash
# CPU profiling
perf record -g cargo bench
perf report

# Flamegraph
cargo flamegraph --bench retrieval_quality

# Tokio console (async)
RUSTFLAGS="--cfg tokio_unstable" cargo build
tokio-console
```

## Performance Patterns

### Hot Path Optimization

1. **Avoid blocking in async**: Use `spawn_blocking` for CPU-heavy ops
2. **Lock contention**: Use `parking_lot::RwLock` instead of `std::sync::RwLock`
3. **Zero-copy**: Use references where possible, avoid cloning
4. **Batching**: Group operations to reduce syscall overhead

### Memory Optimization

1. **Arena allocation**: For frequent allocations/deallocations
2. **Cache-friendly data**: Keep related data contiguous
3. **String interning**: For repeated string values

## Validation Checklist

Before claiming performance improvement:
- [ ] Benchmark shows measurable improvement
- [ ] No regression in other benchmarks
- [ ] Memory usage not increased significantly
- [ ] Tested with realistic data sizes

## CI Integration

Quality gate `docs/QUALITY_GATES.md` includes performance regression check:
- Benchmarks must not degrade >10%
- `cargo bench` runs in nightly CI

## Cross-References

- **Testing**: `test-runner` skill
- **Debugging**: `debug-troubleshoot` skill
- **Quality Gates**: `docs/QUALITY_GATES.md`
- **Benchmark Location**: `benches/`