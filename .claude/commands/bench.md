---
description: Run performance benchmarks and validate against baselines
---

# Performance Benchmarking

Run benchmarks for episodic memory operations with regression detection.

## Usage

```bash
# Run all benchmarks
/bench

# Run specific benchmark
/bench episode_creation

# Compare with baseline
/bench --compare main

# Generate HTML report
/bench --html
```

## Arguments

| Argument | Description |
|----------|-------------|
| `$ARGUMENTS` | Benchmark filter (e.g., "episode", "pattern", "storage") |
| `--compare` | Compare against specified baseline |
| `--html` | Generate HTML report |

## Implementation

```bash
# Build release
cargo build --release --benches

# Run benchmarks
cargo bench --bench $ARGUMENTS

# Compare with baseline
cargo bench --bench $ARGUMENTS -- --baseline main

# Fail if regression > 10%
if [ $? -eq 0 ]; then
    cargo bench --bench $ARGUMENTS -- --baseline main --test
fi
```

## Output

- Performance metrics (ops/sec, latency)
- Comparison with baseline
- Regression warnings (> 10% slower)
- HTML report in `target/criterion/`

## Examples

```bash
# Benchmark episode operations
/bench episode

# Benchmark pattern extraction
/bench pattern

# Benchmark storage operations
/bench storage

# Compare all against main
/bench --compare main
```

## Performance Targets

| Operation | Target (P95) | Current |
|-----------|-------------|---------|
| Episode Creation | < 50ms | ~2.5 us |
| Step Logging | < 20ms | ~1.1 us |
| Episode Completion | < 500ms | ~3.8 us |
| Pattern Extraction | < 1000ms | ~10.4 us |
| Memory Retrieval | < 100ms | ~721 us |
