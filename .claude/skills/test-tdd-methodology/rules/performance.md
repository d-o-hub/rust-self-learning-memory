# Test Performance Rules

## perf-fast-unit-tests

Unit tests < 100ms.

**Targets:**
- Unit tests: < 10ms
- Integration tests: < 100ms
- End-to-end: < 1s

**Techniques:**
- Mock external dependencies
- Use in-memory databases
- Avoid network calls
- Parallel execution

## perf-async-timeouts

Prevent hanging tests.

```rust
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        async_operation()
    ).await;
    
    assert!(result.is_ok());
}
```

## perf-caching

Cache expensive setups.

```rust
lazy_static! {
    static ref SHARED_DB: TestDb = {
        TestDb::new().unwrap()
    };
}

#[test]
fn test_with_cached_db() {
    let db = &*SHARED_DB;
    // Use shared db
}
```

## perf-parallel-execution

Run tests in parallel.

```bash
# Default: parallel
cargo test

# Force sequential if needed
cargo test -- --test-threads=1
```

**Requirements:**
- No shared state
- No port conflicts
- Independent resources

## perf-skip-slow

Mark slow tests with #[ignore].

```rust
#[test]
#[ignore = "slow integration test"]
fn test_database_migration() {
    // Takes 30 seconds
}

// Run ignored tests:
// cargo test -- --ignored
```
