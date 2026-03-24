# Test Code Review Checklist (Rust)

Use this checklist when reviewing test code in pull requests.

## 🎯 Overall Test Strategy

- [ ] Are the right things being tested?
- [ ] Is testing effort focused on high-risk areas?
- [ ] Are critical business paths covered?
- [ ] Is the test-to-code ratio reasonable? (Not testing trivial code)

## 📝 Test Naming

- [ ] Names follow `test_<function>_<scenario>_<expected>` pattern
- [ ] Names clearly communicate what behavior is tested
- [ ] Names indicate when the test should fail
- [ ] No vague names (`test1`, `test_method`, `test_function`)
- [ ] Names focus on behavior, not implementation

### Examples:
✅ `test_start_episode_valid_task_creates_episode`
✅ `test_calculate_discount_new_customer_returns_zero`
❌ `test_episode()`
❌ `test_calls_database_twice()`

## 🏗️ Test Structure (AAA Pattern)

- [ ] Clear **Arrange** section with setup
- [ ] Single **Act** section executing behavior
- [ ] **Assert** section verifying outcomes
- [ ] Sections are visually separated (for complex tests)
- [ ] No assertions in Arrange section
- [ ] No multiple Act sections (split into separate tests)

### Example:
```rust
#[test]
fn test_withdraw_valid_amount_decreases_balance() {
    // Arrange
    let mut account = Account::new(100);

    // Act
    let result = account.withdraw(30);

    // Assert
    assert!(result.is_ok());
    assert_eq!(account.balance(), 70);
}
```

## 🎯 Single Responsibility

- [ ] Each test verifies exactly ONE behavior
- [ ] One clear reason for test to fail
- [ ] If multiple assertions, they verify the same behavior
- [ ] Complex tests are split appropriately

## 🔌 Dependency Isolation

- [ ] External dependencies are properly mocked/faked
- [ ] No real database connections
- [ ] No network calls to external APIs
- [ ] No file system I/O (uses `TempDir` for temporary files)
- [ ] Time/date dependencies are mocked if needed
- [ ] Mocking strategy is appropriate

### Check For:
- [ ] Database connections → Use mocks or in-memory/test DB
- [ ] HTTP calls → Mock the client
- [ ] File operations → Use `TempDir`
- [ ] Random values → Inject controlled randomness
- [ ] Current time → Mock when necessary

## ⚡ Test Speed

- [ ] Tests execute in milliseconds
- [ ] No `thread::sleep()`, `tokio::time::sleep()` with long durations
- [ ] No waiting for external services
- [ ] Tests would run comfortably before every commit

### Red Flags:
❌ `thread::sleep(Duration::from_secs(1))` - timing assumption
❌ Real database connection - slow external dependency
❌ HTTP calls to real endpoints - network dependency

## ⚡ Async Testing (Rust-Specific)

- [ ] Async tests use `#[tokio::test]` attribute
- [ ] All async function calls have `.await`
- [ ] No `thread::sleep` in async code (uses `tokio::time::sleep`)
- [ ] Spawned tasks are awaited and errors checked
- [ ] Blocking operations use `tokio::task::spawn_blocking`
- [ ] No deadlocks or race conditions in concurrent code

### Common Issues:
```rust
// ❌ Bad
#[test]
async fn test_async() { ... }

// ✅ Good
#[tokio::test]
async fn test_async() { ... }

// ❌ Bad
#[tokio::test]
async fn test() {
    thread::sleep(Duration::from_secs(1));
}

// ✅ Good
#[tokio::test]
async fn test() {
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

## 📊 Test Data

- [ ] Test data is minimal and relevant
- [ ] Data clearly shows what matters for the test
- [ ] No unexplained magic numbers
- [ ] No production data or sensitive information
- [ ] Builders used for complex object construction
- [ ] Uses utilities from `test-utils` crate

## 🔄 Test Independence

- [ ] Tests don't depend on each other
- [ ] No execution order dependencies
- [ ] Tests clean up after themselves
- [ ] No shared mutable state between tests
- [ ] Each test can run in isolation

## 🎲 Reliability

- [ ] Tests are deterministic (same input = same output)
- [ ] No race conditions
- [ ] No flaky timing dependencies
- [ ] No reliance on external system state
- [ ] Tests pass consistently

### Flaky Test Indicators:
❌ Timing assumptions
❌ Shared state
❌ External dependencies
❌ Non-deterministic inputs (random, current time)

## 💎 Test Value

- [ ] Tests catch real bugs (not std library behavior)
- [ ] Tests provide deployment confidence
- [ ] Tests document expected behavior
- [ ] Tests are maintainable
- [ ] Testing effort matches risk

### Questions to Ask:
- If this test fails in production, is it a real problem?
- Does this test catch bugs that matter?
- Would you deploy based on this test passing?

## 🚫 Anti-Pattern Detection

Check for these common anti-patterns:

### ❌ Testing Standard Library
```rust
// Bad
#[test]
fn test_vec_push_increases_length() {
    let mut vec = Vec::new();
    vec.push(1);
    assert_eq!(vec.len(), 1);
}
```

### ❌ No Assertions
```rust
// Bad
#[tokio::test]
async fn test_start_episode() {
    memory.start_episode("Test", context).await.unwrap();
    // No assertions!
}
```

### ❌ Testing Implementation Details
```rust
// Bad - coupled to implementation
// Testing that internal method is called
```

### ❌ Excessive .unwrap() Chains
```rust
// Bad
let x = func().unwrap();
let y = func().unwrap();
let z = func().unwrap();

// Good
#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let x = func()?;
    let y = func()?;
    let z = func()?;
    Ok(())
}
```

## 📈 Coverage Context

- [ ] Coverage is meaningful, not just high percentage
- [ ] Critical paths are thoroughly tested
- [ ] Edge cases are covered appropriately
- [ ] Trivial code (getters/setters) not over-tested
- [ ] Coverage metrics don't drive poor test design

### Remember:
- 80% coverage with good tests > 100% coverage with bad tests
- Focus on business logic and risk areas
- Coverage is a tool, not a goal

## 🔍 Code Quality

- [ ] Test code follows same standards as production code
- [ ] No commented-out code
- [ ] No `TODO` comments (create issues instead)
- [ ] No `#[ignore]` without documented reason
- [ ] Proper error handling
- [ ] Clear variable names
- [ ] No code duplication (uses helpers/builders)
- [ ] Uses `test-utils` crate appropriately

## 📚 Documentation

- [ ] Complex test logic is explained
- [ ] Business rules are documented in test names
- [ ] Test serves as executable documentation
- [ ] Unclear behavior is clarified with comments

## 🎯 Final Questions

Before approving the PR, ask:

1. **Would I debug these tests at 3 AM?** (Are names clear enough?)
2. **Would I trust deployment with these tests?** (Do they catch real bugs?)
3. **Will these tests be easy to maintain?** (Are they well-structured?)
4. **Are we testing the right things?** (Appropriate focus and coverage?)
5. **Will these tests run quickly in CI?** (Fast enough for rapid feedback?)

## 💬 Giving Feedback

When providing feedback:

### ✅ Be Constructive
- Point out specific issues with examples
- Explain why something is problematic
- Suggest concrete improvements
- Acknowledge good patterns

### ✅ Provide Examples
```
❌ "This test is bad"
✅ "This test name doesn't follow our convention. Consider:
   test_process_payment_insufficient_funds_returns_error"
```

### ✅ Focus on Impact
```
❌ "Use AAA pattern"
✅ "Using AAA pattern here would make debugging easier when this fails in CI.
   Separate the setup, execution, and verification sections."
```

### ✅ Prioritize Issues
- 🔴 Critical: Flaky tests, no assertions, production dependencies, missing #[tokio::test]
- 🟡 Important: Poor naming, missing isolation, slow tests, excessive unwrap()
- 🟢 Nice-to-have: Minor style improvements, optimization opportunities

## 📊 Quality Scorecard

Rate the test code on these dimensions (1-10):

- **Naming Clarity**: Clear, self-documenting test names?
- **Structure Quality**: AAA pattern followed consistently?
- **Isolation**: Dependencies properly mocked?
- **Speed**: Tests run quickly?
- **Reliability**: Tests are deterministic and stable?
- **Value**: Tests catch meaningful bugs?
- **Async Correctness**: Proper async/await patterns?
- **Maintainability**: Easy to understand and modify?

**Overall Score**: Average of above

- **8-10**: Excellent - Approve immediately
- **6-7**: Good - Minor improvements suggested
- **4-5**: Acceptable - Requires changes before approval
- **1-3**: Needs significant work

## 🚀 Approval Criteria

Approve when:
- ✅ All critical issues addressed
- ✅ Tests provide real value
- ✅ Tests are maintainable
- ✅ Overall quality score ≥ 7
- ✅ Async patterns are correct (if applicable)
- ✅ `cargo test` passes
- ✅ `cargo clippy` has no warnings

Request changes when:
- ❌ Critical issues present (flaky, no assertions, wrong async attributes, etc.)
- ❌ Tests don't provide deployment confidence
- ❌ Tests will be maintenance burden
- ❌ Overall quality score < 6

## 🛠️ Automated Checks

Recommend using these tools:

```bash
# Run quality analysis
python .claude/skills/quality-unit-testing/scripts/analyze-test-quality.py path/to/file.rs

# Run tests
cargo test --all

# Run clippy
./scripts/code-quality.sh clippy --workspace

# Run fmt check
./scripts/code-quality.sh fmt
```

---

**Remember**: The goal is deployment confidence, not just code coverage. Quality tests enable fearless refactoring and rapid deployment.
