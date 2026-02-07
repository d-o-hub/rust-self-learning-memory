---
name: test-tdd-methodology
description: >
  Test-Driven Development methodology with 42 rules across 8 categories.
  Use when practicing TDD, writing tests first, or improving test quality.
  Covers TDD cycles, test design, isolation, performance, coverage, and quality gates.
  Complements quality-unit-testing and test-runner skills.
---

# TDD Methodology

42 rules for effective Test-Driven Development.

## The TDD Cycle

```
Red    → Write failing test (proves test works)
Green  → Write minimal code to pass
Refactor → Improve code while tests pass
Repeat
```

## Quick Reference

| Priority | Category | Rules | Focus |
|----------|----------|-------|-------|
| CRITICAL | [TDD Cycle](#1-tdd-cycle) | 6 | Red-Green-Refactor |
| CRITICAL | [Test Design](#2-test-design) | 6 | AAA pattern, assertions |
| CRITICAL | [Isolation](#3-test-isolation) | 6 | Mocks, state, dependencies |
| HIGH | [Test Performance](#4-test-performance) | 5 | Speed targets |
| HIGH | [Naming](#5-test-naming) | 5 | Descriptive names |
| MEDIUM | [Coverage](#6-coverage-strategy) | 5 | Meaningful targets |
| MEDIUM | [CI/CD](#7-ci-quality-gates) | 5 | Quality gates |
| LOW | [Organization](#8-test-organization) | 4 | Structure, location |

## 1. TDD Cycle (CRITICAL)

**The foundation of TDD**

- **[cycle-write-test-first](rules/cycle.md)** - Write test before implementation (40-90% defect reduction)
- **[cycle-watch-test-fail](rules/cycle.md)** - See test fail (proves test works)
- **[cycle-minimal-code](rules/cycle.md)** - Write minimal code to pass
- **[cycle-see-test-pass](rules/cycle.md)** - Confirm green before refactor
- **[cycle-refactor-clean](rules/cycle.md)** - Refactor with passing tests
- **[cycle-small-steps](rules/cycle.md)** - Small increments (2-10 min cycles)

## 2. Test Design (CRITICAL)

**Structure and clarity**

- **[design-aaa-pattern](rules/design.md)** - Arrange-Act-Assert structure
- **[design-behavior-over-impl](rules/design.md)** - Test what, not how (50-80% less brittle)
- **[design-one-assertion](rules/design.md)** - One logical assertion per test
- **[design-independent-tests](rules/design.md)** - No order dependency
- **[design-edge-cases](rules/design.md)** - Boundary conditions
- **[design-error-paths](rules/design.md)** - Test failures too

## 3. Test Isolation (CRITICAL)

**Prevent test interference**

- **[isolate-mock-external](rules/isolation.md)** - Mock databases, APIs, files
- **[isolate-no-shared-state](rules/isolation.md)** - No mutable shared state
- **[isolate-fresh-fixtures](rules/isolation.md)** - New instances per test
- **[isolate-deterministic](rules/isolation.md)** - Same input → same output
- **[isolate-cleanup](rules/isolation.md)** - Clean up after tests
- **[isolate-parallel-safe](rules/isolation.md)** - Tests run in parallel

## 4. Test Performance (HIGH)

**Fast feedback loop**

- **[perf-fast-unit-tests](rules/performance.md)** - Unit tests < 100ms
- **[perf-async-timeouts](rules/performance.md)** - Prevent hanging tests
- **[perf-caching](rules/performance.md)** - Cache expensive setups
- **[perf-parallel-execution](rules/performance.md)** - Run tests in parallel
- **[perf-skip-slow](rules/performance.md)** - Mark slow tests with #[ignore]

## 5. Test Naming (HIGH)

**Clear intent communication**

- **[name-descriptive](rules/naming.md)** - `test_withdraw_insufficient_funds_returns_error`
- **[name-scenario-focused](rules/naming.md)** - Describe scenario, not method
- **[name-expected-outcome](rules/naming.md)** - Include expected result
- **[name-consistent-style](rules/naming.md)** - Consistent naming convention
- **[name-no-test-prefix](rules/naming.md)** - `withdraw_insufficient_funds` not `test_withdraw`

## 6. Coverage Strategy (MEDIUM)

**Meaningful metrics**

- **[cover-meaningful-targets](rules/coverage.md)** - 80% minimum, 90% ideal
- **[cover-branch-not-line](rules/coverage.md)** - Branch coverage over line
- **[cover-critical-paths](rules/coverage.md)** - Focus on business logic
- **[cover-no-empty-tests](rules/coverage.md)** - Tests must assert
- **[cover-trend-monitoring](rules/coverage.md)** - Track over time

## 7. CI Quality Gates (MEDIUM)

**Automated quality enforcement**

- **[ci-test-before-merge](rules/ci.md)** - All tests must pass
- **[ci-coverage-gate](rules/ci.md)** - Coverage thresholds enforced
- **[ci-lint-gate](rules/ci.md)** - Clippy warnings as errors
- **[ci-format-gate](rules/ci.md)** - rustfmt check
- **[ci-fast-feedback](rules/ci.md)** - Fail fast on errors

## 8. Test Organization (LOW)

**Logical structure**

- **[org-unit-vs-integration](rules/organization.md)** - Clear separation
- **[org-describe-blocks](rules/organization.md)** - Group related tests
- **[org-setup-helper](rules/organization.md)** - Reusable fixtures
- **[org-test-utils](rules/organization.md)** - Shared utilities

## TDD Workflow Example

**Step 1: Write Failing Test (Red)**
```rust
#[test]
fn test_add_positive_numbers_returns_sum() {
    // Arrange
    let calc = Calculator::new();
    
    // Act
    let result = calc.add(2, 3);
    
    // Assert
    assert_eq!(result, 5);
}
```
Run: `cargo test` → **FAILS** (expected - no Calculator yet)

**Step 2: Minimal Implementation (Green)**
```rust
pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
```
Run: `cargo test` → **PASSES**

**Step 3: Refactor (while green)**
```rust
// Maybe extract common validation
// Maybe optimize implementation
// Tests still pass? Good!
```

**Step 4: Next Test**
```rust
#[test]
fn test_add_negative_numbers_returns_sum() {
    let calc = Calculator::new();
    assert_eq!(calc.add(-2, -3), -5);
}
```

## Success Metrics

✅ Deploy without manual testing  
✅ Refactoring doesn't break tests  
✅ Test failures pinpoint problems  
✅ Tests run in < 1 second per test  
✅ 80%+ branch coverage on critical paths  
✅ Zero flaky tests  

## Anti-Patterns to Avoid

❌ Writing tests after implementation  
❌ Testing implementation details  
❌ Shared mutable state between tests  
❌ Tests dependent on execution order  
❌ Mocking value types  
❌ Tests without assertions  

## Integration with Other Skills

- **quality-unit-testing**: Low-level test writing patterns
- **test-runner**: Test execution and troubleshooting
- **test-fix**: Debugging failing tests
- **test-optimization**: cargo-nextest, property testing
- **rust-idiomatic-patterns**: Idiomatic code to test

## When to Apply

| Situation | Approach |
|-----------|----------|
| New feature | Full TDD cycle |
| Bug fix | Write failing test first |
| Refactoring | Ensure tests exist first |
| Legacy code | Characterization tests first |
| Code review | Verify test coverage |
