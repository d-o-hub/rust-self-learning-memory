# Running Tests

## Test Categories

### All Tests
```bash
# Run entire test suite
cargo test --all

# With verbose output
cargo test --all -- --nocapture

# With debug logging
RUST_LOG=debug cargo test

# With test-threads=1 (for debugging race conditions)
cargo test --all -- --test-threads=1
```

### Unit Tests
```bash
# Library tests only
cargo test --lib --all-features --workspace

# Specific crate unit tests
cd memory-core && cargo test --lib
cd memory-storage-turso && cargo test --lib
cd memory-storage-redb && cargo test --lib
cd memory-mcp && cargo test --lib
cd memory-cli && cargo test --lib
```

### Integration Tests
```bash
# All integration tests
cargo test --test '*' --workspace

# Specific integration test
cargo test --test integration_test -- --nocapture

# Memory integration tests
cargo test --test learning_cycle
cargo test --test quality_gates

# Specific test by name
cargo test test_episode_creation
cargo test test_pattern_extraction
```

### Test Filtering
```bash
# Run tests matching pattern
cargo test pattern

# Run tests in specific module
cargo test memory::tests

# Run ignored tests
cargo test -- --ignored
```

## Coverage Testing

### Generate Coverage Report
```bash
# Install coverage tool (once)
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --html --output-dir coverage

# Generate multiple formats
cargo llvm-cov --html --lcov --json --output-dir coverage

# Coverage for specific crate
cd memory-core && cargo llvm-cov --html --output-dir coverage

# Coverage with all features
cargo llvm-cov --all-features --workspace --html --output-dir coverage
```

### Coverage Targets (Enforced)
- **Line coverage**: >90% target
- **Branch coverage**: >85%
- All public APIs must be tested
- Test pass rate: >99% target

### View Coverage
```bash
# Open HTML report
open coverage/html/index.html  # macOS
xdg-open coverage/html/index.html  # Linux

# View coverage summary
cargo llvm-cov --summary
```

### Quality Gates Script
```bash
# Run full quality gates (coverage, format, clippy)
./scripts/quality-gates.sh

# Customize thresholds
QUALITY_GATE_COVERAGE_THRESHOLD=95 ./scripts/quality-gates.sh

# Skip optional tools
QUALITY_GATE_SKIP_OPTIONAL=true ./scripts/quality-gates.sh

# GOAP checks run as non-blocking (cannot be skipped)
```

> **Note**: GOAP checks run as non-blocking checks and cannot be skipped. They check documentation hygiene and feedback loop markers.

## Performance Testing

### Benchmarks
```bash
# Run all benchmarks
cd benches && cargo bench

# Specific benchmark
cd benches && cargo bench --bench episode_lifecycle
cd benches && cargo bench --bench phase3_retrieval_accuracy
cd benches && cargo bench --bench spatiotemporal_benchmark

# Benchmark with criterion output
cd benches && cargo bench -- --output-format html

# Save benchmark results
cd benches && cargo bench -- --save-baseline main
```

### Performance Targets (Actual vs Target)
| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Episode Creation | <50ms | ~2.5 µs | ✅ 19,531x faster |
| Step Logging | <20ms | ~1.1 µs | ✅ 17,699x faster |
| Episode Completion | <500ms | ~3.8 µs | ✅ 130,890x faster |
| Pattern Extraction | <1000ms | ~10.4 µs | ✅ 95,880x faster |
| Memory Retrieval (10K) | <100ms | ~721 µs | ✅ 138x faster |

## Test Utilities

### Using Test Utils
```rust
use test_utils::*;

// Create test episode
let episode = create_test_episode("Task description");

// Create completed episode
let episode = create_completed_episode("Task", true);

// Create test context
let context = create_test_context("web-api", Some("rust"));

// Create test storage
let (storage, _dir) = create_test_turso_storage().await;
let (cache, _dir) = create_test_redb_storage().await;
```

### Async Test Helpers
```rust
#[tokio::test]
async fn test_async_operation() {
    // Use tokio::test for async tests
    let memory = create_test_memory().await;
    let result = memory.create_episode("test").await;
    assert!(result.is_ok());
}
```

## Test Categories by Module

### Memory Core Tests
- Episode lifecycle (creation, steps, completion)
- Pattern extraction and validation
- Reward scoring
- Spatiotemporal retrieval
- Semantic embeddings
- Reflection generation

### Storage Tests
- Turso database operations
- redb cache operations
- Connection pooling
- Circuit breaker behavior
- Synchronization between backends

### MCP Server Tests
- Tool registration and execution
- Sandbox isolation (Wasmtime)
- JSON-RPC protocol
- Security boundaries
- Pattern analysis tools

### CLI Tests
- Command execution
- Configuration management
- Output formatting
- Integration with storage backends

## CI/CD Testing

The CI pipeline runs:
1. **Format checks**: `cargo fmt -- --check`
2. **Linting**: `cargo clippy --all -- -D warnings`
3. **Unit tests**: `cargo test --lib`
4. **Integration tests**: `cargo test --test '*'`
5. **Code coverage**: `cargo llvm-cov --html` (threshold: >90%)
6. **Security audit**: `cargo audit`
7. **Benchmarks**: Run on PRs for performance regression detection

## Debugging Tests

### Test Hangs
```bash
# Run with single thread
cargo test -- --test-threads=1

# Run with timeout
timeout 60 cargo test test_hanging_function

# Enable backtrace
RUST_BACKTRACE=1 cargo test
```

### Flaky Tests
- Check for race conditions (use `--test-threads=1`)
- Ensure proper test isolation
- Avoid shared mutable state
- Use proper async/await patterns

### Coverage Gaps
```bash
# Generate HTML report
cargo llvm-cov --html --output-dir coverage

# View uncovered lines
open coverage/html/index.html

# Review specific module
cd memory-core && cargo llvm-cov --open -- src/memory/mod.rs
```

### Performance Regressions
```bash
# Compare with baseline
cd benches && cargo bench -- --baseline main

# Save new baseline
cd benches && cargo bench -- --save-baseline main
```

## Best Practices

1. **Test Independence**: Each test isolated
2. **Deterministic**: Same results every run
3. **Fast**: Unit tests in milliseconds
4. **Clear Names**: Descriptive test names
5. **One Assertion**: Single logical assertion
6. **Arrange-Act-Assert**: Clear structure
7. **Use test-utils**: Leverage shared test helpers
8. **Async tests**: Use `#[tokio::test]` for async code
9. **Error testing**: Test both success and failure cases
10. **Coverage**: Maintain >90% coverage

## Testing Commands Summary

```bash
# Quick test
cargo test --all

# Full test with coverage
cargo test --all && cargo llvm-cov --html --output-dir coverage

# Quality gates
./scripts/quality-gates.sh

# Debug test
RUST_LOG=debug cargo test test_name -- --nocapture

# Specific crate tests
cd memory-core && cargo test --lib

# Benchmarks
cd benches && cargo bench
```