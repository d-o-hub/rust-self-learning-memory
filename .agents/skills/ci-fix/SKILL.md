---
name: ci-fix
description: "Diagnose and fix GitHub Actions CI failures for Rust projects. Use when CI fails, tests timeout, or linting issues occur. Captures common patterns from CLAUDE_INSIGHTS_REPORT.md."
---

# CI Fix Skill

Diagnose and fix GitHub Actions CI failures using patterns from past CI issues.

## Quick Diagnosis Pattern

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
- **security**: cargo audit/deny failures
- **coverage**: coverage threshold missed
- **deprecated**: old action versions (common!)

## Common Fixes (From History)

### Deprecated GitHub Actions
```bash
# Detect
grep -r "actions/checkout@v1" .github/workflows/
grep -r "actions-rs" .github/workflows/

# Fix: Update to v2+
```

### Optional Dependency Issues (libclang, wasmtime)
```bash
# Pattern: --all-features triggered optional dep issues
# Fix: Use workspace exclude
cargo build --workspace --exclude do-memory-mcp
```

### Clippy Lint Allow-List
```bash
# See new warnings
./scripts/code-quality.sh clippy --workspace 2>&1 | grep "warning:"

# Fix: Add #[allow(...)] comments
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
```

### Check Quick Check Status CANCELLED after 15m
```bash
# Symptom: YAML Lint (or CI) "Check Quick Check Status" cancelled; real jobs skipped
# Cause: wait-on-check timeout-minutes:15 < Quick Check wall time (~15–20m)
# Permanent fix:
#  1. Remove wait gate from cheap workflows (yaml-lint.yml)
#  2. Raise remaining wait jobs to timeout-minutes: 40
#  3. Raise Quick Check job to timeout-minutes: 25
#  4. running-workflow-name must match this workflow's name:
# See LESSON-021, agent_docs/github_actions_patterns.md
```

### Insta snapshot fails only on macOS
```bash
# Cause: f32/f64 {:?} Debug differs by platform
# Fix: format!("{:.4}", value) before assert_snapshot!
# See LESSON-022
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

## Codecov Patch Coverage Gap Fix

When Codecov reports "Patch coverage is X% with N lines missing coverage":

### Step 1: Parse the Report
Identify files with missing lines from the Codecov comment:
```
| File | Patch % | Missing Lines |
|------|---------|---------------|
| path/to/file.rs | 0.00% | 24 Missing |
```

### Step 2: Locate Uncovered Code
For each file, read the specific lines identified. They are typically:
- New functions without tests
- Extracted submodules (refactored code moved to new files)
- Changed branches not exercised by existing tests

### Step 3: Write Targeted Tests
Add tests to the existing `#[cfg(test)] mod tests` in each file, or to a sibling test file:
- For new public functions: unit test exercising the function directly
- For new structs: test Default, Debug, Clone impls + key methods
- For refactored code: test that the extracted module works identically to before
- For branch coverage: test each match arm / if-else branch

### Step 4: Verify
```bash
cargo nextest run -p <crate> -E 'test(<test_name_pattern>)'
cargo clippy --workspace -- -D warnings
```

### Step 5: Commit
Single atomic commit: `test(coverage): add tests for Codecov patch gaps in N files`

### Common Patterns
- **Extracted submodule** (e.g. `stats.rs` from `wrapper.rs`): test the new module's public API directly
- **New monitoring metric**: test the recording method + snapshot/getter
- **New heuristic function**: test boundary conditions (empty input, short/long input, edge cases)
- **New StorageType enum**: test Debug, Clone, and pattern matching
- Changes committed if needed
