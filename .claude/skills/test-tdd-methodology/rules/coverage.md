# Coverage Strategy Rules

## cover-meaningful-targets

80% minimum, 90% ideal.

**Rationale:**
- 80%: Catches most issues
- 90%: Comprehensive safety
- 100%: Often impractical, tests boilerplate

**Focus areas:**
- Business logic
- Edge cases
- Error handling

## cover-branch-not-line

Branch coverage over line coverage.

**Why:**
```rust
if condition {
    do_a();
} else {
    do_b();
}
```

- Line coverage: 100% (all lines "hit")
- Branch coverage: 50% if only one branch tested

**Tool:**
```bash
cargo tarpaulin --branch
```

## cover-critical-paths

Focus on business logic.

**Prioritize:**
1. Core business rules
2. Financial calculations
3. Security checks
4. Data validation

**Deprioritize:**
1. Getters/setters
2. Simple constructors
3. Debug formatting
4. UI code

## cover-no-empty-tests

Tests must assert.

**Bad:**
```rust
#[test]
fn test_something() {
    let result = do_something();
    // No assertion!
}
```

**Good:**
```rust
#[test]
fn test_something() {
    let result = do_something();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}
```

## cover-trend-monitoring

Track over time.

**Process:**
1. Measure baseline
2. Set minimum threshold
3. Fail CI if coverage drops
4. Review regularly

**Tools:**
- cargo-tarpaulin
- Codecov.io
- GitHub Actions
