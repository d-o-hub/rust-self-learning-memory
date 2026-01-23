---
name: code-quality
description: Maintain high code quality through formatting, linting, and static analysis. Use to ensure consistent code style, catch common mistakes, and verify adherence to coding standards.
---

# Code Quality

Systematically maintain and enforce code quality standards through automated formatting, linting, and static analysis.

## Purpose

Ensure code meets quality standards by:
- Consistent formatting across the codebase
- Zero warnings from static analysis
- No known security vulnerabilities
- Adherence to best practices and conventions
- Continuous improvement through regular quality checks

## Tools & Commands

### Rust Formatting (rustfmt)

**Check formatting without changes:**
```bash
cargo fmt -- --check
```

**Auto-format all code:**
```bash
cargo fmt
```

**Format specific crates:**
```bash
cargo fmt -p memory-core
cargo fmt -p memory-storage-turso
```

### Clippy Linting

**Basic clippy:**
```bash
cargo clippy
```

**Strict mode (zero warnings enforced):**
```bash
cargo clippy --all -- -D warnings
```

**Auto-fix common issues:**
```bash
cargo clippy --fix --allow-dirty --allow-staged
```

**Clippy for specific targets:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Security Audit

**Check for vulnerabilities:**
```bash
cargo audit
```

**Update vulnerability database first:**
```bash
cargo audit --fetch
```

**Check specific advisory:**
```bash
cargo audit --advisory <id>
```

### Combined Quality Check

**Run all checks (as in quality-gates.sh):**
```bash
cargo fmt -- --check
cargo clippy --all -- -D warnings
cargo audit
```

## Common Clippy Warnings & Solutions

### Dead Code
**Warning**: `warning: unused field: [...]`
**Solution**: Remove unused code or add `#[allow(dead_code)]` with justification comment

### Unnecessary Clones
**Warning**: `warning: using 'clone()' on a double-reference`
**Solution**: Use references or borrowing instead of cloning

### Inefficient Collections
**Warning**: `warning: calling 'push' within a loop`
**Solution**: Use `extend()` or pre-allocate with `Vec::with_capacity()`

### Complex Expressions
**Warning**: `warning: this expression has a cyclomatic complexity of...`
**Solution**: Extract complex logic into separate functions

### Manual Implementations
**Warning**: `warning: you are implementing 'From' manually`
**Solution**: Use derive macros or implement via blanket impl

## Formatting Standards

### rustfmt Configuration
The project uses standard rustfmt with these patterns:
- 100 character line width
- 4 space indentation
- Consistent struct literal formatting
- Chain formatting for long method calls

### Common Formatting Issues
- **Line too long**: Break at logical operators or method calls
- **Inconsistent spacing**: Run `cargo fmt` to auto-fix
- **Trailing whitespace**: Automatically removed by fmt
- **Inconsistent imports**: Sorted and grouped by fmt

## Security Best Practices

### Dependency Management
```bash
# Check for outdated dependencies
cargo outdated

# Update specific package
cargo update -p package_name

# Update everything
cargo update
```

### Vulnerability Response
1. Run `cargo audit` to identify vulnerabilities
2. Check CVE details and severity
3. Update to patched version using `cargo update`
4. Verify fix with `cargo audit`
5. If patch unavailable: Document mitigation in SECURITY.md

## Quality Gates

### Pre-Commit Checklist
- [ ] Code formatted with `cargo fmt`
- [ ] Zero clippy warnings (`cargo clippy --all -- -D warnings`)
- [ ] No security vulnerabilities (`cargo audit`)
- [ ] Tests passing (`cargo test --all`)

### CI/CD Integration
Quality gates should run in CI/CD:
1. Formatting check (fast, fails early)
2. Clippy (medium speed, catches bugs)
3. Security audit (medium speed, catches vulnerabilities)
4. Tests (slowest, validates functionality)

## Quality Metrics

Track these metrics:
- **Formatting Compliance**: Should be 100%
- **Clippy Warnings**: Target 0
- **Security Vulnerabilities**: Target 0
- **Code Coverage**: Target >90%
- **Duplicate Code**: Monitor with tools like `cargo-dup`

## When to Run Quality Checks

### Mandatory Runs
- Before committing code
- Before creating pull requests
- In CI/CD pipelines
- Before releases

### Recommended Runs
- After refactoring
- After adding dependencies
- Periodically (weekly/monthly)
- When onboarding new contributors

## Troubleshooting

### Formatting Issues
**Problem**: `cargo fmt` changes code unexpectedly
**Solution**:
1. Review changes with `git diff`
2. Check `rustfmt.toml` configuration
3. Run `cargo fmt -- --check` to preview changes

### Clippy False Positives
**Problem**: Clippy warns about code that's correct
**Solution**:
1. Verify code is actually correct
2. Add `#[allow(lint_name)]` with justification comment
3. Report false positive to Clippy team if warranted

### Audit Failures
**Problem**: `cargo audit` reports vulnerability but no patch available
**Solution**:
1. Check advisory severity (low/medium/high)
2. Look for workaround or mitigation
3. Document in SECURITY.md with planned resolution
4. Monitor for patch release

## Project-Specific Standards

For this Rust memory project, quality standards from AGENTS.md:
- **Clippy Warnings**: 0 (strictly enforced with `-D warnings`)
- **Code Formatting**: 100% rustfmt compliant
- **Security**: Zero known vulnerabilities
- **Test Coverage**: >90% (current: 92.5%)
- **File Size**: <500 LOC per module

### Quality Gates Script
The project includes `./scripts/quality-gates.sh` which runs all quality checks automatically.

## Best Practices

### DO:
✓ Run quality checks before every commit
✓ Fix clippy warnings immediately
✓ Keep dependencies up-to-date
✓ Document why warnings are suppressed
✓ Review automated suggestions before applying

### DON'T:
✗ Suppress warnings without justification
✗ Skip quality checks to save time
✗ Commit code that fails quality gates
✗ Ignore security vulnerabilities
✗ Use `#[allow(...)]` at file level (be specific)

## Integration with Other Skills

- **rust-code-quality**: For deeper analysis of code patterns and structure
- **clean-code-developer**: For refactoring to meet quality standards
- **testing-qa**: For ensuring quality through comprehensive testing

## Example Workflow

When working on a feature:
1. Write code and tests
2. Run `cargo fmt` to format
3. Run `cargo clippy --fix` to auto-fix issues
4. Manually review and fix remaining clippy warnings
5. Run `cargo audit` to check security
6. Run `cargo test --all` to validate functionality
7. Commit only if all checks pass
