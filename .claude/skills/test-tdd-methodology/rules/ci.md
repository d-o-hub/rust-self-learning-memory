# CI Quality Gates

## ci-test-before-merge

All tests must pass.

```yaml
# .github/workflows/ci.yml
- name: Run tests
  run: cargo test --all
  
- name: Check test results
  if: failure()
  run: exit 1
```

## ci-coverage-gate

Coverage thresholds enforced.

```yaml
- name: Generate coverage
  run: cargo tarpaulin --out Xml

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    fail_ci_if_error: true
    minimum_coverage: 80
```

## ci-lint-gate

Clippy warnings as errors.

```yaml
- name: Run Clippy
  run: cargo clippy --all -- -D warnings
```

## ci-format-gate

rustfmt check.

```yaml
- name: Check formatting
  run: cargo fmt --all -- --check
```

## ci-fast-feedback

Fail fast on errors.

```yaml
jobs:
  check:
    steps:
      - uses: actions/checkout@v3
      
      - name: Format check (fast)
        run: cargo fmt -- --check
        
      - name: Clippy (medium)
        run: cargo clippy -- -D warnings
        
      - name: Tests (slow)
        run: cargo test --all
```
