---
name: quality-unit-testing
description: Write high-quality Rust unit tests following best practices. Use when writing new tests, reviewing test code, or improving test quality. Emphasizes clear naming, AAA pattern, isolation, and deployment confidence.
---

# Quality Unit Testing for Rust

Expert guidance for writing unit tests that catch real bugs and provide deployment confidence.

## Core Philosophy

**Quality over coverage**: Tests should catch real bugs and enable fearless deployment, not just boost coverage percentages.

## When to Use This Skill

Use for:
- Writing new unit tests
- Reviewing test code quality
- Improving existing test suites
- Establishing testing standards
- Evaluating test effectiveness

## Quick Reference

### Test Naming: `test_<function>_<scenario>_<expected_behavior>`

Examples:
```rust
#[test]
fn test_process_payment_insufficient_funds_returns_error()

#[test]
fn test_calculate_discount_new_customer_returns_zero()

#[tokio::test]
async fn test_withdraw_valid_amount_decreases_balance()
```

### AAA Pattern: Arrange-Act-Assert

Always structure tests with clear sections:
```rust
#[test]
fn test_account_withdraw_valid_amount_decreases_balance() {
    // Arrange - Set up test context
    let mut account = Account::new(100);

    // Act - Execute behavior
    let result = account.withdraw(30);

    // Assert - Verify outcome
    assert!(result.is_ok());
    assert_eq!(account.balance(), 70);
}
```

### Isolation: Mock External Dependencies

- ✅ Mock: APIs, databases, file systems, time/date, external services
- ❌ Don't mock: Value types, pure functions, the code under test

### Single Responsibility

Each test verifies ONE behavior with ONE reason to fail.

### Speed Target

Milliseconds per test. Unit tests should run instantly.

## Detailed Guidance

For detailed information on specific topics, see:

- **Test Naming Patterns**: `reference/naming-conventions.md`
- **AAA Structure Details**: `reference/aaa-pattern.md`
- **Async Testing**: `reference/async-testing.md`
- **Test Builders**: `reference/test-builders.md`
- **Anti-Patterns to Avoid**: `reference/anti-patterns.md`

## Test Quality Analysis

To analyze test file quality:
```bash
python scripts/analyze-test-quality.py crates/memory-core/src/lib.rs
```

## Templates

Use pre-built templates for consistent test structure:
- Basic unit test: `templates/unit-test.md`
- Async test: `templates/async-test.md`
- Test builder pattern: `templates/test-builder.md`

## Quality Checklists

Before committing tests, verify against:
- Pre-commit checklist: `checklists/pre-commit.md`
- Code review checklist: `checklists/review.md`

## Key Success Metrics

You're succeeding when:
- ✅ You deploy without manual testing
- ✅ Test failures pinpoint exact problems
- ✅ Refactoring doesn't break unrelated tests
- ✅ Tests run in milliseconds
- ✅ Every failure is actionable

You need improvement when:
- ❌ Tests are skipped because they're slow
- ❌ Test failures require investigation
- ❌ High coverage but low deployment confidence
- ❌ Flaky tests train team to ignore failures

## Rust-Specific Best Practices

### 1. Use `#[tokio::test]` for Async Tests
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 2. Use Result<()> for Tests with Error Handling
```rust
#[test]
fn test_operation() -> anyhow::Result<()> {
    let result = fallible_operation()?;
    assert_eq!(result, expected);
    Ok(())
}
```

### 3. Use Test Builders for Complex Setup
```rust
let episode = TestEpisodeBuilder::new()
    .with_task("Test task")
    .with_context(context)
    .completed(true)
    .build();
```

### 4. Clean Up with RAII (Drop)
```rust
struct TestDb(TempDir);

impl Drop for TestDb {
    fn drop(&mut self) {
        // Cleanup happens automatically
    }
}
```

### 5. Use `tempfile` for File System Tests
```rust
#[test]
fn test_file_operation() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.db");
    // Test with path
    // Cleanup happens automatically when dir is dropped
}
```

## Project-Specific Patterns

### Testing Async Code with Tokio
```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode() {
    // Arrange
    let memory = create_test_memory().await;
    let context = TaskContext::default();

    // Act
    let episode_id = memory.start_episode("Test task", context).await?;

    // Assert
    assert!(!episode_id.is_empty());
    let episode = memory.get_episode(&episode_id).await?;
    assert_eq!(episode.task_description, "Test task");
}
```

### Testing Error Cases
```rust
#[tokio::test]
async fn test_get_episode_invalid_id_returns_error() {
    // Arrange
    let memory = create_test_memory().await;

    // Act
    let result = memory.get_episode("invalid_id").await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EpisodeNotFound(_)));
}
```

### Using Test Utilities
```rust
use test_utils::*;

#[test]
fn test_episode_completion() {
    // Arrange
    let episode = create_completed_episode("Test task", true);

    // Act & Assert
    assert!(episode.is_complete());
    assert_eq!(episode.verdict, Some(Verdict::Success));
}
```

## Workflow for Creating Tests

1. **Understand the code**: What behavior needs verification?
2. **Identify risks**: What could break in production?
3. **Write failing test first** (red-green-refactor)
4. **Apply AAA pattern** with clear naming
5. **Isolate dependencies** with proper mocking
6. **Verify test speed** (should be milliseconds)
7. **Check quality** against checklists
8. **Ensure value**: Does this catch real bugs?

## Workflow for Reviewing Tests

1. Run analysis script: `scripts/analyze-test-quality.py`
2. Check against review checklist: `checklists/review.md`
3. Verify naming follows conventions
4. Ensure proper isolation
5. Confirm single responsibility
6. Check for anti-patterns
7. Validate test value

## Remember

**The goal is deployment confidence, not coverage theater.**

Focus testing effort where failures hurt most:
- **High Priority**: Business logic, data integrity, async correctness
- **Medium Priority**: Integration points, error handling
- **Low Priority**: Simple getters/setters, trivial code

## Integration with Other Skills

- Use **test-runner** to execute tests: `cargo test`
- Use **test-fix** when tests fail and need diagnosis
- Use **code-quality** to ensure test code meets standards
- Use **rust-code-quality** for comprehensive review
