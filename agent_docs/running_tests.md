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
```

### Unit Tests
```bash
# Library tests only
cargo test --lib --all-features --workspace

# Specific crate unit tests
cd memory-core && cargo test --lib
cd memory-mcp && cargo test --lib
```

### Integration Tests
```bash
# All integration tests
cargo test --test '*' --workspace

# Specific integration test
cargo test --test integration_test -- --nocapture

# Learning cycle tests
cargo test --test learning_cycle
```

### Single Test
```bash
# Run specific test by name
cargo test test_episode_creation

# With full output
cargo test test_name -- --nocapture
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
```

### Coverage Targets
- **Line coverage**: >90%
- **Branch coverage**: >85%
- All public APIs must be tested

### View Coverage
```bash
# Open HTML report
open coverage/html/index.html  # macOS
xdg-open coverage/html/index.html  # Linux
```

## Performance Testing

### Benchmarks
```bash
# Run all benchmarks
cd benches && cargo bench

# Specific benchmark
cd benches && cargo bench --bench episode_lifecycle

# Benchmark with criterion output
cd benches && cargo bench -- --output-format html
```

### Performance Targets
| Operation | Target | Measurement |
|-----------|--------|-------------|
| Episode Creation | <50ms | P95 latency |
| Step Logging | <20ms | Average |
| Episode Completion | <500ms | Average |
| Pattern Extraction | <1000ms | Average |
| Memory Retrieval (10K) | <100ms | P95 latency |

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
```

### Storage Testing
```rust
// Setup test storage
let (storage, _dir) = create_test_turso_storage().await;
```

## CI/CD Testing

The CI pipeline runs:
1. Format checks (`cargo fmt`)
2. Linting (`cargo clippy`)
3. Unit tests
4. Integration tests
5. Code coverage
6. Benchmarks (on PRs)

## Debugging Tests

### Test Hangs
- Check for deadlocks in async code
- Run with `--test-threads=1`
- Ensure proper `.await` usage

### Flaky Tests
- Check for race conditions
- Use proper test isolation
- Avoid shared mutable state

### Coverage Gaps
- Generate HTML report: `cargo llvm-cov --html`
- Review uncovered lines
- Add edge case tests

## Best Practices

1. **Test Independence**: Each test isolated
2. **Deterministic**: Same results every run
3. **Fast**: Unit tests in milliseconds
4. **Clear Names**: Descriptive test names
5. **One Assertion**: Single logical assertion
6. **Arrange-Act-Assert**: Clear structure