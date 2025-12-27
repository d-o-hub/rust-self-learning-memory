# Quality Gates

Automated quality threshold enforcement for the rust-self-learning-memory project.

## Overview

Quality gates are automated tests that enforce minimum standards across the codebase. They run in CI/CD pipelines and can be executed locally to ensure code quality before committing.

## Quality Thresholds

| Gate | Threshold | Description |
|------|-----------|-------------|
| **Test Coverage** | > 90% | Line coverage across all crates |
| **Pattern Accuracy** | > 70% | Pattern recognition accuracy (aspirational, baseline: 25%) |
| **Code Complexity** | Avg < 10 | Average cyclomatic complexity |
| **Security** | 0 vulns | Zero critical/high/medium vulnerabilities |
| **Linting** | 0 warnings | Zero clippy warnings |
| **Formatting** | 100% | All code rustfmt compliant |
| **Performance** | < 10% regression | No performance degradation |

## Running Quality Gates Locally

### All Gates

```bash
cargo test --test quality_gates -- --nocapture
```

### Individual Gates

```bash
# Test Coverage
cargo test --test quality_gates quality_gate_test_coverage -- --nocapture

# Pattern Accuracy
cargo test --test quality_gates quality_gate_pattern_accuracy -- --nocapture

# Code Complexity
cargo test --test quality_gates quality_gate_code_complexity -- --nocapture

# Security
cargo test --test quality_gates quality_gate_no_security_vulns -- --nocapture

# Linting
cargo test --test quality_gates quality_gate_no_clippy_warnings -- --nocapture

# Formatting
cargo test --test quality_gates quality_gate_formatting -- --nocapture

# Performance
cargo test --test quality_gates quality_gate_performance_regression -- --nocapture

# Summary
cargo test --test quality_gates quality_gates_summary -- --nocapture
```

## Configuration

Quality gates can be configured via environment variables:

```bash
export QUALITY_GATE_COVERAGE_THRESHOLD=90          # Minimum coverage %
export QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD=70   # Minimum pattern accuracy %
export QUALITY_GATE_COMPLEXITY_THRESHOLD=10         # Maximum average complexity
export QUALITY_GATE_SECURITY_THRESHOLD=0            # Maximum vulnerabilities
export QUALITY_GATE_SKIP_OPTIONAL=false             # Skip optional gates
```

### Example: Lowering Thresholds for Development

```bash
# More lenient settings for local development
export QUALITY_GATE_COVERAGE_THRESHOLD=80
export QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD=50
export QUALITY_GATE_SKIP_OPTIONAL=true

cargo test --test quality_gates -- --nocapture
```

## Required Tools

Most gates require additional tools to be installed:

```bash
# Coverage analysis
cargo install cargo-llvm-cov

# Security auditing
cargo install cargo-audit

# Complexity analysis (optional)
cargo install cargo-cyclomatic

# Already required (should be installed):
# - rustfmt (cargo fmt)
# - clippy (cargo clippy)
```

## Interpreting Results

### Test Coverage Gate

**Example Output:**
```
=== Quality Gate: Test Coverage ===
Threshold: 90%
Running coverage analysis...
Current Coverage: 92.50%
Required: 90.00%
✅ Coverage gate PASSED: 92.50% >= 90.00%
```

**If Failed:**
- Identify uncovered code: `cargo llvm-cov --html`
- Open `target/llvm-cov/html/index.html` in browser
- Add tests for red/yellow sections

### Pattern Accuracy Gate

**Example Output:**
```
=== Quality Gate: Pattern Accuracy ===
Threshold: 70%
Running pattern accuracy tests...
Current Pattern Accuracy: 25.00%
Required: 70.00%
⚠️  Pattern accuracy below target: 25.00% < 70.00% (baseline: 25.00%)
Current implementation meets baseline. Target is aspirational.
```

**Note:** Pattern accuracy of 70% is an aspirational target. Current baseline is 25%.

**If Below Baseline:**
- Check pattern extraction logic in `memory-core/src/patterns/`
- Review test cases in `memory-core/tests/pattern_accuracy.rs`
- Ensure pattern matching is working correctly

### Code Complexity Gate

**Example Output:**
```
=== Quality Gate: Code Complexity ===
Threshold: Average complexity < 10
Checking codebase structure...
⚠️  Files exceeding 500 LOC guideline:
  - ./memory-core/src/lib.rs: 650 LOC
✅ Code complexity check completed
```

**If Files Exceed 500 LOC:**
- Split large files into smaller modules
- Extract related functionality into submodules
- Follow project guideline: keep files < 500 LOC

### Security Gate

**Example Output:**
```
=== Quality Gate: Security Vulnerabilities ===
Threshold: Max 0 critical/high/medium vulnerabilities
Running cargo audit...
Critical vulnerabilities: 0
High vulnerabilities: 0
Medium vulnerabilities: 0
Total: 0
✅ Security gate PASSED: 0 vulnerabilities <= 0 threshold
```

**If Failed:**
- Review `cargo audit` output for affected crates
- Update dependencies: `cargo update`
- Check for security advisories: `cargo audit --json`
- Consider alternative crates if needed

### Linting Gate

**Example Output:**
```
=== Quality Gate: Clippy Linting ===
Threshold: 0 warnings
Running cargo clippy...
✅ Clippy gate PASSED: No warnings
```

**If Failed:**
- Review clippy warnings in output
- Fix issues: `cargo clippy --fix --allow-dirty`
- For intentional violations, use `#[allow(clippy::...)]` with justification
- See [CLIPPY_FIX_PLAN.md](../plans/CLIPPY_FIX_PLAN.md) for examples of recent fixes

**Recent Updates (2025-12-26):**
All clippy warnings have been resolved using 2025 best practices:
- Modern format strings: `format!("{var}")` instead of `format!("{}", var)`
- Type-safe conversions: `From` trait instead of `as` casts
- Range checks: `(0.0..=1.0).contains(&value)` instead of explicit comparisons
- Documentation backticks for code elements
- Proper `#[allow]` attributes with justifications for dead code

### Formatting Gate

**Example Output:**
```
=== Quality Gate: Code Formatting ===
Threshold: 100% rustfmt compliant
Running cargo fmt --check...
✅ Formatting gate PASSED: All code properly formatted
```

**If Failed:**
- Run `cargo fmt --all` to auto-format code
- Verify with `cargo fmt --all -- --check`

### Performance Gate

**Example Output:**
```
=== Quality Gate: Performance Regression ===
Threshold: < 10% performance degradation
Running performance tests...
✅ Performance gate PASSED: All performance tests passed
```

**If Failed:**
- Performance regression detected in key operations
- Run benchmarks: `cargo bench`
- Profile with: `cargo flamegraph` (requires cargo-flamegraph)
- Review recent changes for inefficiencies

## CI Integration

Quality gates run automatically in CI on:
- Push to `main` or `develop` branches
- After Quick Check workflow succeeds

### CI Workflow

The `quality-gates` job:
1. Runs after: format, clippy, test, coverage, security-audit
2. Installs required tools
3. Executes all quality gates
4. Generates quality report artifact
5. Fails build if any gate fails

### Viewing CI Results

1. Go to GitHub Actions for your PR/branch
2. Find the "Quality Gates" job
3. Review output for each gate
4. Download quality-gates-report artifact for full details

## Improving Metrics

### Increasing Coverage

1. Identify gaps: `cargo llvm-cov --html`
2. Write unit tests for uncovered functions
3. Add integration tests for workflows
4. Test error paths and edge cases

### Improving Pattern Accuracy

1. Review ground truth patterns in `pattern_accuracy.rs`
2. Improve pattern extraction in `PatternExtractor`
3. Tune pattern matching heuristics
4. Add more training episodes

### Reducing Complexity

1. Break large functions into smaller ones
2. Extract repeated logic into helpers
3. Use early returns to reduce nesting
4. Split files > 500 LOC into modules

### Fixing Security Issues

1. Update dependencies regularly
2. Monitor security advisories
3. Use `cargo audit fix` for automatic fixes
4. Review dependency tree: `cargo tree`

## Troubleshooting

### Tools Not Installed

**Symptom:** Gate skipped with "not installed" message

**Solution:**
```bash
cargo install cargo-llvm-cov
cargo install cargo-audit
```

Or set `QUALITY_GATE_SKIP_OPTIONAL=true` to skip optional gates.

### Coverage Parsing Failed

**Symptom:** "Could not parse coverage percentage"

**Solution:**
- Ensure cargo-llvm-cov is up to date
- Run manually: `cargo llvm-cov --all-features --workspace --summary-only`
- Check for errors in test suite

### CI Timeout

**Symptom:** Quality gates job times out in CI

**Solution:**
- Some gates run full test suite (slow)
- Consider using `--ignored` flag for long tests
- Optimize slow tests or mark them `#[ignore]`

## Best Practices

1. **Run locally before committing:**
   ```bash
   cargo test --test quality_gates -- --nocapture
   ```

2. **Fix issues incrementally:**
   - Don't lower thresholds to pass gates
   - Fix root causes, not symptoms

3. **Monitor trends:**
   - Track metrics over time
   - Set goals for improvement

4. **Keep thresholds realistic:**
   - Start with achievable baselines
   - Gradually increase standards

5. **Document exceptions:**
   - Use `#[allow(...)]` with comments
   - Justify threshold adjustments

## FAQ

**Q: Can I disable a quality gate?**

A: Yes, use environment variables:
```bash
export QUALITY_GATE_SKIP_OPTIONAL=true
```

However, critical gates (formatting, linting) should not be disabled.

**Q: Why is pattern accuracy below 70%?**

A: 70% is an aspirational target. The current implementation baseline is 25%, which validates the framework is working. Future improvements will increase accuracy.

**Q: How do I add a new quality gate?**

A:
1. Add test function in `tests/quality_gates.rs`
2. Follow naming convention: `quality_gate_<name>`
3. Use environment variable for threshold configuration
4. Update this documentation

**Q: Can I run gates in parallel?**

A: Some gates modify state or run the full test suite, so serial execution is safer. For local development, run individual gates.

**Q: What if a gate fails in CI but passes locally?**

A:
- Check environment differences (OS, dependencies)
- Review CI logs for specifics
- Ensure same tool versions
- CI uses `--all-features --workspace`

## References

- [AGENTS.md](../AGENTS.md) - Project guidelines
- [TESTING.md](../TESTING.md) - Testing infrastructure
- [Cargo Book: Tests](https://doc.rust-lang.org/cargo/guide/tests.html)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-audit](https://github.com/rustsec/rustsec)
