# Pre-Commit Test Quality Checklist (Rust)

Before committing your tests, verify the following:

## ✅ Naming Convention
- [ ] Test names follow `test_<function>_<scenario>_<expected>` pattern
- [ ] Test names are descriptive and self-documenting
- [ ] No vague names like `test1`, `test_method`, etc.
- [ ] Names describe behavior, not implementation details

## ✅ Test Structure (AAA Pattern)
- [ ] Clear **Arrange** section with test setup
- [ ] Single **Act** section that executes the behavior
- [ ] **Assert** section that verifies outcomes
- [ ] Visual separation between AAA sections (comments for complex tests)

## ✅ Single Responsibility
- [ ] Each test verifies exactly ONE behavior
- [ ] One reason for test to fail
- [ ] If multiple assertions exist, they verify the same behavior
- [ ] Consider: Can this test be split further?

## ✅ Dependency Isolation
- [ ] External dependencies are mocked/faked
- [ ] No real database connections
- [ ] No network calls to external APIs
- [ ] No file system I/O (use `TempDir` for temporary files)
- [ ] Time/date dependencies are mocked if needed
- [ ] Random values are controlled

## ✅ Test Speed
- [ ] Test executes in milliseconds (unit tests)
- [ ] No `thread::sleep()` or timing assumptions
- [ ] No waiting for external services
- [ ] Would run this test before every commit? (If no, it's too slow)

## ✅ Async Testing (if applicable)
- [ ] Uses `#[tokio::test]` attribute for async tests
- [ ] All async calls have `.await`
- [ ] No `thread::sleep` in async tests (use `tokio::time::sleep`)
- [ ] Spawned tasks are awaited and checked for panics
- [ ] Sync operations in async code use `spawn_blocking`

## ✅ Error Handling
- [ ] Uses `Result<()>` return type with `?` operator instead of excessive `.unwrap()`
- [ ] Error cases are explicitly tested
- [ ] Panic tests use `#[should_panic(expected = "...")]`
- [ ] Errors provide clear context

## ✅ Test Data
- [ ] Test data is simple and relevant
- [ ] Data highlights what matters for the test
- [ ] No magic numbers without explanation
- [ ] Uses test builders for complex objects
- [ ] No production data or sensitive information

## ✅ Independence
- [ ] Test doesn't depend on other tests
- [ ] Test doesn't depend on execution order
- [ ] Test cleans up after itself (uses RAII/Drop)
- [ ] No shared mutable state between tests

## ✅ Reliability
- [ ] Test is deterministic (same input = same result)
- [ ] No race conditions or timing issues
- [ ] No reliance on external system state
- [ ] Test passes consistently on different machines

## ✅ Value
- [ ] Test catches real bugs (not just std library behavior)
- [ ] Test provides deployment confidence
- [ ] Test documents expected behavior
- [ ] Test is maintainable

## ✅ Code Quality
- [ ] No commented-out code
- [ ] No `TODO` comments (create issues instead)
- [ ] No `#[ignore]` without reason documented
- [ ] Test code follows same standards as production code
- [ ] Uses project's test utilities from `test-utils` crate

## ⚠️ Rust-Specific Anti-Patterns to Avoid

### ❌ Missing #[tokio::test] for Async
```rust
// Bad
#[test]
async fn test_async_operation() { ... }

// Good
#[tokio::test]
async fn test_async_operation() { ... }
```

### ❌ Blocking the Async Runtime
```rust
// Bad
#[tokio::test]
async fn test() {
    thread::sleep(Duration::from_secs(1));
}

// Good
#[tokio::test]
async fn test() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### ❌ Excessive .unwrap() Chains
```rust
// Bad
#[tokio::test]
async fn test() {
    let x = func1().await.unwrap();
    let y = func2().await.unwrap();
    let z = func3().await.unwrap();
}

// Good
#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let x = func1().await?;
    let y = func2().await?;
    let z = func3().await?;
    Ok(())
}
```

### ❌ Manual Resource Cleanup
```rust
// Bad
#[test]
fn test() {
    let path = create_temp_file();
    // ... test ...
    std::fs::remove_file(&path).unwrap(); // Might not run
}

// Good
#[test]
fn test() {
    let _temp_dir = TempDir::new().unwrap();
    let path = _temp_dir.path().join("test.db");
    // ... test ...
    // Cleanup automatic via Drop
}
```

## Quick Self-Review Questions

Before committing, ask yourself:

1. **Naming**: Can someone debug this test failure at 3 AM without reading the code?
2. **Structure**: Is it immediately obvious which section broke when the test fails?
3. **Focus**: Does this test have exactly one reason to fail?
4. **Speed**: Would I run this test 50 times per day locally?
5. **Reliability**: Will this test pass the same way on every machine, every time?
6. **Value**: Does this test catch real bugs that matter?
7. **Async**: If async, are all the async patterns correct?

If you answered "no" to any question, improve the test before committing.

## Running Quality Checks

Use the provided script to automate checks:

```bash
# Analyze test quality
python .claude/skills/quality-unit-testing/scripts/analyze-test-quality.py crates/memory-core/src/lib.rs

# Run tests
cargo test --lib

# Run with output
cargo test --lib -- --nocapture

# Check code quality
cargo clippy --all -- -D warnings
cargo fmt --all -- --check
```

## Success Criteria

Your tests are ready to commit when:

✅ All checklist items pass
✅ Quality score > 80 (from analyze script)
✅ You feel confident deploying based on these tests
✅ Tests run quickly and reliably
✅ Tests document expected behavior clearly
✅ `cargo test` passes
✅ `cargo clippy` has no warnings

Remember: **Quality over quantity. Deployment confidence over coverage metrics.**
