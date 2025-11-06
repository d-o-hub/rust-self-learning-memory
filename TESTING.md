# Testing Guide

This document describes the comprehensive testing infrastructure for the self-learning memory system.

## Overview

The testing infrastructure includes:
- **Unit tests**: Located within each module (`#[cfg(test)]`)
- **Integration tests**: Located in `tests/` directory
- **Benchmarks**: Located in `benches/` directory
- **Test utilities**: Shared helpers in `test-utils/` crate

## Running Tests

### Run all tests
```bash
cargo test --all
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
open coverage/index.html
```

### Coverage targets
- Line coverage: >90%
- Branch coverage: >85%
- All public APIs must be tested

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
        assert_eq!(result, expected_value);
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

criterion_group!(benches, benchmark_operation);
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
- [ ] Code coverage >90%
- [ ] No clippy warnings
- [ ] Benchmarks within targets
- [ ] Documentation updated
