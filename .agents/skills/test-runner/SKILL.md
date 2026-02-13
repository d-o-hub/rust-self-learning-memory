---
name: test-runner
description: Execute and manage Rust tests including unit tests, integration tests, and doc tests. Use when running tests to ensure code quality and correctness.
---

# Test Runner

Execute and manage Rust tests for the self-learning memory project.

## Test Categories

| Category | Command | Scope |
|----------|---------|-------|
| Unit | `cargo test --lib` | Individual functions |
| Integration | `cargo test --test '*'` | End-to-end workflows |
| Doc | `cargo test --doc` | Documentation examples |
| All | `cargo test --all` | Complete validation |

## Execution Strategy

### Step 1: Quick Check (Unit Tests)
```bash
cargo test --lib
```
- Fast feedback (< 30s)
- Catch basic logic errors

### Step 2: Integration Tests
```bash
cargo test --test '*'
```
- Tests database interactions
- Requires Turso/redb setup

### Step 3: Full Suite
```bash
cargo test --all
```
- Complete validation before commit

## Troubleshooting

### Async/Await Issues
**Symptom**: Test hangs
```rust
#[tokio::test]
async fn test_async() {
    let result = async_fn().await;  // Don't forget .await
}
```

### Database Connection
**Symptom**: Connection refused
- Check TURSO_URL, TURSO_TOKEN
- Use test database

### Race Conditions
**Symptom**: Intermittent failures
```bash
cargo test -- --test-threads=1
```

### redb Lock Errors
**Symptom**: "Database is locked"
- Use separate DB per test
- Close transactions promptly

## Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage
```

## Best Practices

- Isolation: Each test independent
- Cleanup: Remove test data
- Speed: < 1s per unit test
- Naming: Describe behavior
