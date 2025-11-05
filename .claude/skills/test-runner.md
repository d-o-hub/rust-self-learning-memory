# Test Runner

Execute and manage Rust tests for the self-learning memory project.

## Purpose
Run unit tests, integration tests, and doc tests to ensure code quality and correctness.

## Test Categories

### 1. Unit Tests
**Location**: Inline with source code in `src/`
**Scope**: Individual functions and modules

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib storage::turso

# Run with output
cargo test --lib -- --nocapture
```

### 2. Integration Tests
**Location**: `tests/` directory
**Scope**: End-to-end workflows, cross-module interactions

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test integration_memory

# With nocapture for debugging
cargo test --test integration_memory -- --nocapture
```

### 3. Doc Tests
**Location**: Documentation comments in source
**Scope**: Example code in documentation

```bash
# Run doc tests
cargo test --doc
```

### 4. All Tests
```bash
# Run everything
cargo test --all

# Run with release optimizations (faster for heavy tests)
cargo test --all --release
```

## Test Execution Strategy

### Step 1: Quick Check (Unit Tests)
```bash
# Fast feedback loop
cargo test --lib
```
- Should complete in < 30 seconds
- Run frequently during development
- Catch basic logic errors

### Step 2: Integration Tests
```bash
# After unit tests pass
cargo test --test '*'
```
- May take 1-3 minutes
- Tests real database interactions
- Requires Turso/redb setup

### Step 3: Full Suite
```bash
# Before committing
cargo test --all
```
- Complete validation
- All features enabled
- All workspaces tested

## Troubleshooting Failing Tests

### 1. Identify Failures
```bash
# Run with verbose output
RUST_LOG=debug cargo test -- --nocapture --test-threads=1
```

### 2. Isolate the Test
```bash
# Run single failing test
cargo test test_name -- --exact --nocapture
```

### 3. Common Failure Patterns

#### Async/Await Issues
**Symptoms**: Test hangs, runtime panics
**Solutions**:
- Ensure `#[tokio::test]` attribute
- Check `.await` on async calls
- Verify runtime is initialized

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await; // Don't forget .await
    assert!(result.is_ok());
}
```

#### Database Connection Issues
**Symptoms**: Connection refused, timeout
**Solutions**:
- Check environment variables (TURSO_URL, TURSO_TOKEN)
- Use test database, not production
- Clean up connections in teardown

```rust
#[tokio::test]
async fn test_with_db() {
    let db = create_test_db().await?;
    // ... test code ...
    cleanup_test_db(db).await?;
}
```

#### Race Conditions
**Symptoms**: Intermittent failures
**Solutions**:
- Run with `--test-threads=1`
- Add proper synchronization
- Use unique test data per test

```bash
# Force sequential execution
cargo test -- --test-threads=1
```

#### redb Lock Errors
**Symptoms**: "Database is locked", transaction errors
**Solutions**:
- Use separate DB file per test
- Close transactions promptly
- Don't hold read locks during writes

```rust
{
    let read_txn = db.begin_read()?;
    let value = read_txn.get(...)?;
} // Transaction dropped here

// Now safe to write
let write_txn = db.begin_write()?;
```

## Test Organization

### Test Structure
```
tests/
├── integration/
│   ├── memory_test.rs      # Core memory operations
│   ├── storage_test.rs     # Storage layer tests
│   ├── pattern_test.rs     # Pattern extraction tests
│   └── retrieval_test.rs   # Context retrieval tests
├── fixtures/
│   ├── test_episodes.json  # Sample episode data
│   └── test_patterns.json  # Sample pattern data
└── common/
    └── mod.rs              # Shared test utilities
```

### Test Utilities
```rust
// tests/common/mod.rs
use tempfile::TempDir;

pub fn create_test_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.redb");
    (dir, path)
}

pub async fn create_test_memory() -> SelfLearningMemory {
    let (dir, path) = create_test_db();
    SelfLearningMemory::new(
        "test_turso_url",
        "test_token",
        path,
    ).await.unwrap()
}
```

## Test Coverage

### Check Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/index.html
```

### Coverage Goals
- **Unit tests**: > 80% line coverage
- **Integration tests**: All critical paths
- **Edge cases**: Error conditions, boundary values

## Performance Testing

### Benchmark Tests
```bash
# Run benchmarks (if configured)
cargo bench

# Profile specific benchmark
cargo bench --bench pattern_extraction -- --profile-time=10
```

### Load Testing
```rust
#[tokio::test]
async fn test_concurrent_episodes() {
    let memory = create_test_memory().await;
    let mut handles = vec![];

    for i in 0..100 {
        let mem = memory.clone();
        handles.push(tokio::spawn(async move {
            mem.start_episode(format!("Task {}", i), context).await
        }));
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```

## CI Integration

### GitHub Actions
```yaml
- name: Run tests
  run: |
    cargo test --all --verbose

- name: Run clippy
  run: cargo clippy --all -- -D warnings
```

## Test Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Always clean up test data
3. **Determinism**: Tests should be reproducible
4. **Speed**: Keep unit tests fast (< 1s each)
5. **Clarity**: Test names should describe behavior
6. **Assertions**: Use descriptive assertion messages

## Example Test Patterns

### Basic Unit Test
```rust
#[test]
fn test_episode_creation() {
    let episode = Episode::new("test task");
    assert_eq!(episode.task_description, "test task");
    assert!(episode.steps.is_empty());
}
```

### Async Integration Test
```rust
#[tokio::test]
async fn test_episode_lifecycle() {
    let memory = create_test_memory().await;

    let id = memory.start_episode("test", context).await.unwrap();
    memory.log_step(id, step).await.unwrap();
    memory.complete_episode(id, outcome).await.unwrap();

    let retrieved = memory.get_episode(id).await.unwrap();
    assert_eq!(retrieved.verdict, Verdict::Success);
}
```

### Error Handling Test
```rust
#[tokio::test]
async fn test_invalid_episode_id() {
    let memory = create_test_memory().await;
    let result = memory.get_episode("invalid_id").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EpisodeNotFound(_)));
}
```

## Debugging Failed Tests

```bash
# Run with full logging
RUST_LOG=trace cargo test test_name -- --nocapture

# Run under debugger
rust-lldb target/debug/deps/test_binary -- test_name

# Get backtrace on panic
RUST_BACKTRACE=1 cargo test
```
