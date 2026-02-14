---
description: Diagnose and fix CI failures using patterns from CLAUDE_INSIGHTS_REPORT.md. Use when GitHub Actions CI fails, tests timeout, or linting issues occur.
subtask: true
---

# CI Fix Command

Quick CI diagnosis and repair using patterns from past CI issues.

## Quick Diagnosis

```bash
# Step 1: Check CI status
gh run list --limit 5 --json status,conclusion,name,headBranch

# Step 2: Get failed job details
gh run view <run_id> --log --job <job_name>

# Step 3: Categorize failure
# - lint: fmt/clippy warnings
# - test: test failures or timeouts
# - build: compilation errors
# - security: cargo audit/deny
# - coverage: threshold missed
# - deprecated: old action versions
```

## Common Fixes (From CLAUDE_INSIGHTS_REPORT.md)

### Deprecated Actions
```bash
# Check for deprecated actions
grep -r "actions/checkout@v1" .github/workflows/
grep -r "actions-rs" .github/workflows/

# Fix: Update to v2+ versions
```

### Optional Dependency Issues (libclang, wasmtime)
```bash
# Build without problematic crate
cargo build --workspace --exclude memory-mcp

# Or fix the dependency
cargo update -p libclang
```

### Clippy Lint Allow-List
```bash
# See new warnings
cargo clippy --all -- -D warnings 2>&1 | grep "warning:"

# Fix: Add #[allow(...)] to affected code
```

### Coverage Threshold
```bash
# Generate coverage
cargo tarpaulin --workspace

# Check threshold in .github/workflows/
```

### Benchmark Timeout
```bash
# Increase timeout-minutes in workflow YAML
# Check current: cat .github/workflows/*.yml | grep timeout
```

## Fix Commands

```bash
# Linting
cargo fmt --all
cargo clippy --workspace --fix --allow-dirty

# Testing
cargo test --workspace -- --nocapture
cargo test --workspace --test-threads=1

# Security
cargo audit
cargo deny check

# Build
cargo build --workspace
cargo build --workspace --exclude memory-mcp
```

## Success Criteria

- [ ] `gh run list` shows all jobs passing
- [ ] No new warnings introduced
- [ ] Changes committed if needed
- [ ] Branch CI passes before merge

## Example

```
$ ci-fix
[1/5] Get CI status...
Failed: clippy job

[2/5] Get clippy logs...
warning: new_clippy_lint_name

[3/5] Run clippy locally...
warning: new_clippy_lint_name

[4/5] Apply fix...
Added #[allow(new_clippy_lint_name)] to affected files

[5/5] Verify...
CI should pass on next push
```

## Related

- `quality-gates`: Run full quality check
- `github-workflows`: Workflow-specific issues
- `release-guard`: Verify CI before releases
