# Testing Guide

This document describes the comprehensive testing infrastructure for the self-learning memory system.

## Overview

The testing infrastructure includes:
- **Unit tests**: Located within each module (`#[cfg(test)]`)
- **Integration tests**: Located in `tests/` directory
- **Benchmarks**: Located in `benches/` directory
- **Test utilities**: Shared helpers in `test-utils/` crate

### Modern Testing Stack (2026)

In addition to the standard test infrastructure, the project adopts:
- **cargo-nextest**: Primary test runner (3x faster, per-test process isolation)
- **cargo-mutants**: Mutation testing to verify test effectiveness (nightly CI)
- **proptest**: Property-based testing for invariant verification
- **insta**: Snapshot testing for output regression detection
- **cargo-llvm-cov**: Code coverage with LCOV output

See [ADR-033](plans/adr/ADR-033-Modern-Testing-Strategy.md) for the full strategy.

## Running Tests

### Run all tests
```bash
# Preferred: cargo-nextest (faster, per-test isolation)
cargo nextest run --all

# Fallback: standard cargo test
cargo test --all

# Doctests (nextest doesn't support these yet)
cargo test --doc --all
```

### Run unit tests only
```bash
cargo test --lib --all-features --workspace
```

### Run integration tests only
```bash
cargo test --test '*' --workspace
```

### Run specific test
```bash
cargo test test_episode_creation
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests with logging
```bash
RUST_LOG=debug cargo test
```

## Code Coverage

### Generate coverage report
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage
```

### View coverage report
```bash
open coverage/html/index.html
```

### Coverage targets
- Line coverage: >90% (Current: 92.5%)
- Branch coverage: >85%
- All public APIs must be tested
- Test pass rate: >99% (Current: 99.3%)

### Configuration

**Note on Configuration:** Unlike `cargo-tarpaulin` which used `tarpaulin.toml`, `cargo-llvm-cov` is designed to work primarily through command-line flags. This approach offers:
- Better integration with CI/CD pipelines
- More flexible configuration per workflow
- No additional config files to maintain
- Explicit configuration in each command

Common options:
```bash
# Generate multiple output formats
cargo llvm-cov --html --lcov --json --output-dir coverage

# Coverage for all workspace crates
cargo llvm-cov --all-features --workspace

# Show coverage summary only
cargo llvm-cov --summary-only
```

## Benchmarks

### Run all benchmarks
```bash
cd benches
cargo bench
```

### Run specific benchmark
```bash
cd benches
cargo bench --bench episode_lifecycle
```

### View benchmark results
Results are saved to `target/criterion/` with HTML reports.

## Integration Tests

### Learning Cycle Tests (`tests/learning_cycle.rs`)
Tests the complete learning cycle: start → execute → score → learn → retrieve

### Storage Tests (`tests/storage_integration.rs`)
Tests storage backends:
- Episode storage and retrieval
- Query operations
- Concurrent writes
- Pattern storage

## Performance Targets

Based on 04-review.md requirements:

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Episode Creation | <50ms | P95 latency |
| Step Logging | <20ms | Average |
| Episode Completion | <500ms | Average |
| Pattern Extraction | <1000ms | Average |
| Memory Retrieval (10K) | <100ms | P95 latency |
| Concurrent Operations (1000) | <5000ms | Total time |

## Test Utilities

The `test-utils` crate provides:

### Episode Generators
```rust
use test_utils::*;

// Create simple test episode
let episode = create_test_episode("Task description");

// Create completed episode
let episode = create_completed_episode("Task", true); // success=true

// Create episode with context
let context = create_test_context("web-api", Some("rust"));
let episode = create_test_episode_with_context("Task", context, TaskType::Testing);
```

### Pattern Generators
```rust
// Create test pattern
let pattern = create_test_pattern("tool_sequence", 0.9); // 0.9 success rate

// Create test heuristic
let heuristic = create_test_heuristic("condition", "action");
```

### Storage Helpers
```rust
// Available in integration tests
let (storage, _dir) = create_test_turso_storage().await;
```

## CI/CD Integration

The enhanced CI workflow (`.github/workflows/ci-enhanced.yml`) runs:
1. Format checks (`cargo fmt`)
2. Linting (`cargo clippy`)
3. Unit tests
4. Integration tests
5. Doc tests
6. Code coverage
7. Benchmarks (on PRs)
8. Security audit

## Writing New Tests

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        let expected = "expected_value";
        assert_eq!(result, expected);
        println!("Test passed with result: {result}");
    }
}
```

### Async Test Template
```rust
#[tokio::test]
async fn test_async_feature() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Integration Test Template
```rust
use memory_core::*;
use test_utils::*;

#[tokio::test]
async fn test_integration_scenario() {
    // Setup
    let episode = create_test_episode("Test");
    
    // Execute
    // ... test logic
    
    // Verify
    assert!(episode.is_complete());
}
```

### Benchmark Template
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_operation(c: &mut Criterion) {
    c.bench_function("operation_name", |b| {
        b.iter(|| {
            operation(black_box(input))
        });
    });
}

fn benchmark_comparison(c: &mut Criterion) {
    c.bench_function("baseline", |b| {
        b.iter(|| baseline_operation(black_box(data)))
    });

    c.bench_function("optimized", |b| {
        b.iter(|| optimized_operation(black_box(data)))
    });
}

criterion_group!(benches, benchmark_operation, benchmark_comparison);
criterion_main!(benches);
```

## Troubleshooting

### Tests hang
- Check for deadlocks in async code
- Ensure proper use of `.await`
- Run with `--test-threads=1` to serialize tests

### Flaky tests
- Check for race conditions
- Use proper test isolation
- Avoid shared mutable state

### Coverage gaps
- Run `cargo llvm-cov --html --output-dir coverage`
- Review uncovered lines in HTML report
- Add tests for edge cases

### Clippy warnings
- Run `cargo clippy --all-targets --all-features`
- Apply fixes with `cargo clippy --fix --allow-dirty`
- For intentional violations, use `#[allow(clippy::...)]` with justification
- See [plans/CLIPPY_FIX_PLAN.md](plans/CLIPPY_FIX_PLAN.md) for recent fixes

## Best Practices

1. **Test Independence**: Each test should be isolated and not depend on others
2. **Deterministic**: Tests should produce same results on every run
3. **Fast**: Unit tests should run in milliseconds
4. **Clear Names**: Use descriptive test names that explain what is being tested
5. **One Assertion**: Prefer single logical assertion per test
6. **Arrange-Act-Assert**: Structure tests clearly
7. **Test Edge Cases**: Cover error paths and boundary conditions

## Quality Gates

Before merging:
- [ ] All tests passing
- [ ] Code coverage >90% (quality gate threshold)
- [ ] No clippy warnings (run `cargo clippy --all-targets --all-features`)
- [ ] All code formatted (`cargo fmt --all -- --check`)
- [ ] Benchmarks within targets
- [ ] Documentation updated with modern Rust patterns
- [ ] Format strings use variable capture: `format!("{var}")`
- [ ] Type conversions use `From` trait: `i64::from(value)`
- [ ] Documentation uses backticks for code elements

## Advanced Testing (2026)

### Mutation Testing

Mutation testing injects bugs into your code to verify tests catch them:

```bash
# Install cargo-mutants
cargo install --locked cargo-mutants

# Run on memory-core (recommended starting point)
cargo mutants -p memory-core --timeout 120 --jobs 4 -- --lib

# Acceptance criteria: <20% missed mutants in core business logic
```

### Property Testing

Property testing generates random inputs to verify invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn serialization_roundtrip(episode in any_episode_strategy()) {
        let bytes = postcard::to_allocvec(&episode).unwrap();
        let decoded: Episode = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(episode, decoded);
    }
}
```

### Snapshot Testing

Snapshot testing captures output for regression detection:

```rust
#[test]
fn test_mcp_tool_response() {
    let response = build_tool_response("search_patterns", &params);
    insta::assert_json_snapshot!(response);
}
```

Run `cargo insta review` to accept/reject snapshot changes.

### nextest Profiles

Configure in `.config/nextest.toml`:

```toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.ci]
retries = 2
slow-timeout = { period = "30s", terminate-after = 3 }
failure-output = "immediate-final"

[profile.nightly]
retries = 3
slow-timeout = { period = "120s", terminate-after = 2 }
```

See [ADR-033](plans/adr/ADR-033-Modern-Testing-Strategy.md) for complete details.

See [docs/QUALITY_GATES.md](docs/QUALITY_GATES.md) for complete quality gate definitions.
