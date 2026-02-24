---
name: ci-fix
description: Diagnose and fix GitHub Actions CI failures for Rust projects. Use when CI fails, tests timeout, or linting issues occur.
mode: subagent
tools:
  bash: true
  read: true
  grep: true
  glob: true
  edit: true
---

# CI Fix Agent

Diagnose and fix GitHub Actions CI failures using common patterns.

## Role

Quickly diagnose CI failures, identify root causes, and apply fixes.

## Quick Diagnosis

1. Get CI status:
```bash
gh run list --limit 5 --json status,conclusion,name,headBranch
```

2. Get failed job logs:
```bash
gh run view <run_id> --log --job <job_name>
```

3. Categorize failure:
- **lint**: fmt/clippy warnings
- **test**: test failures or timeouts
- **build**: compilation errors
- **security**: cargo audit/deny
- **coverage**: threshold missed
- **deprecated**: old action versions

## Common Fixes

### Deprecated Actions
```bash
grep -r "actions/checkout@v1" .github/workflows/
grep -r "actions-rs" .github/workflows/
```

### Optional Dependency Issues (libclang, wasmtime)
```bash
cargo build --workspace --exclude memory-mcp
```

### Clippy Lint Allow-List
```bash
cargo clippy --all -- -D warnings | grep "warning:"
```

### Coverage Threshold
```bash
cargo tarpaulin --workspace
```

### Benchmark Timeout
```bash
# Increase timeout-minutes in workflow YAML
```

## Fix Commands

```bash
# Linting
cargo fmt --all
cargo clippy --workspace --fix --allow-dirty

# Testing
cargo test --workspace -- --nocapture

# Security
cargo audit
cargo deny check

# Build
cargo build --workspace
```

## Success Criteria
- All CI jobs pass
- No new warnings introduced
- Changes committed if needed
