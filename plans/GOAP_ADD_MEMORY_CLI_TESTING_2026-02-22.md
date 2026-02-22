# GOAP Plan: Add memory-cli Testing to GitHub Actions CI

**Date**: 2026-02-22
**Task**: Add memory-cli to CI test matrix
**Domain**: ci-cd, testing
**Complexity**: Medium

## Executive Summary

Add memory-cli package to GitHub Actions CI test jobs. Currently:
- ✅ Format/Clippy tests all targets (includes CLI)
- ❌ Test job only runs: memory-core, memory-storage-turso, memory-storage-redb
- ⚠️ Multi-platform job uses `--lib --all` but may not properly test CLI binaries

**Goal**: Ensure memory-cli tests run in both `test` and `multi-platform` jobs.

---

## Referenced ADRs

| ADR | Relevance |
|-----|-----------|
| **ADR-029** | GitHub Actions Modernization - CI workflow best practices |
| **ADR-030** | Test Optimization and CI Stability Patterns - nextest usage |
| **ADR-033** | Modern Testing Strategy (2026) - cargo-nextest standardization |

---

## Current CI State Analysis

### Test Job (ci.yml lines 57-85)
```yaml
# Currently tested packages:
cargo nextest run --package memory-core --lib
cargo nextest run --package memory-storage-turso --lib
cargo nextest run --package memory-storage-redb --lib
# MISSING: memory-cli
```

### Multi-Platform Job (ci.yml lines 123-161)
```yaml
# Current command:
cargo nextest run --lib --all
# Issue: --lib only tests library code, not binary tests
```

---

## Task Decomposition

### Phase 1: Analysis & Planning (Atomic)

| Step | Action | Dependencies | Success Criteria |
|------|--------|--------------|------------------|
| 1.1 | Analyze ci.yml test commands | None | List all test commands and packages tested |
| 1.2 | Identify test locations in memory-cli | None | Verify test file locations and types |
| 1.3 | Determine required changes | 1.1, 1.2 | Document exact changes needed |

### Phase 2: Implementation (Atomic)

| Step | Action | Dependencies | Success Criteria |
|------|--------|--------------|------------------|
| 2.1 | Add memory-cli to test job | 1.3 | Add `cargo nextest run -p memory-cli` to test job |
| 2.2 | Verify multi-platform covers CLI | 1.3 | Ensure --lib --all or explicit CLI test |

### Phase 3: Quality Gates (Atomic)

| Step | Action | Dependencies | Success Criteria |
|------|--------|--------------|------------------|
| 3.1 | Run cargo fmt | 2.1 | `cargo fmt --all -- --check` passes |
| 3.2 | Run cargo clippy | 2.1 | `cargo clippy --all -- -D warnings` passes |
| 3.3 | Run memory-cli tests locally | 2.1 | `cargo nextest run -p memory-cli` passes |
| 3.4 | Verify changes don't break CI syntax | 2.1 | YAML syntax valid |

---

## Execution Strategy

### Strategy: Sequential with Validation

**Rationale**: 
- Changes are straightforward (adding package names to existing commands)
- No complex dependencies between changes
- Quality gates provide safety net

### Workflow
```
[Analyze CI] → [Identify Changes] → [Implement] → [Verify Quality] → [Done]
```

---

## Detailed Implementation Plan

### Step 2.1: Modify Test Job

**File**: `.github/workflows/ci.yml`

**Current** (lines 76-85):
```yaml
- name: Run tests with timeout protection
  run: |
    # Run library tests for core packages (fast, isolated tests)
    echo "Running memory-core library tests..."
    cargo nextest run --package memory-core --lib
    echo "Running memory-storage-turso library tests..."
    cargo nextest run --package memory-storage-turso --lib
    echo "Running memory-storage-redb library tests..."
    cargo nextest run --package memory-storage-redb --lib
    echo "Library tests completed successfully"
```

**Change**: Add memory-cli tests after line 84:
```yaml
    echo "Running memory-cli tests..."
    cargo nextest run --package memory-cli
    echo "CLI tests completed successfully"
```

### Step 2.2: Verify Multi-Platform Job

**File**: `.github/workflows/ci.yml`

**Current** (lines 147-161):
```yaml
- name: Run tests on ${{ matrix.os }}
  run: |
    # Cross-platform timeout wrapper...
    if command -v timeout >/dev/null 2>&1; then
      timeout 900s cargo nextest run --lib --all
    # ... etc
```

**Issue**: `--lib` flag excludes binary targets (memory-cli is a binary)

**Fix**: Remove `--lib` flag to test all targets including binaries:
```yaml
    if command -v timeout >/dev/null 2>&1; then
      timeout 900s cargo nextest run --all
    # ... etc
```

**OR** add explicit memory-cli test (if we want to keep --lib for speed):
```yaml
    timeout 900s cargo nextest run --lib --all
    timeout 300s cargo nextest run -p memory-cli --bins  # Binary tests
```

**Recommendation**: Remove `--lib` flag since:
1. Multi-platform already has generous timeout (900s = 15min)
2. CLI has relatively few tests compared to libraries
3. Simpler than adding separate command

---

## Quality Gates

### Pre-Commit Validation
```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Clippy lint
cargo clippy --all -- -D warnings

# 3. Build verification
cargo build --all

# 4. Memory-cli tests
cargo nextest run -p memory-cli
```

### CI Verification
After PR merge, verify:
- [ ] `essential` job passes (format, clippy, doctest)
- [ ] `test` job passes (includes memory-cli)
- [ ] `multi-platform` job passes (includes CLI binaries)
- [ ] `quality-gates` job passes

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| CLI tests fail | Medium | Fix test failures before merge |
| CI timeout | Low | 15min timeout in multi-platform is generous |
| Disk space | Low | memory-cli tests are lightweight |

---

## Verification Checklist

- [ ] Analyze current ci.yml test commands
- [ ] Identify memory-cli test locations
- [ ] Add memory-cli to test job
- [ ] Verify multi-platform job tests CLI
- [ ] Run cargo fmt (pass)
- [ ] Run cargo clippy (pass)
- [ ] Run memory-cli tests locally (pass)
- [ ] Verify YAML syntax valid

---

## Estimated Effort

| Phase | Time | Notes |
|-------|------|-------|
| Analysis | 10 min | Already completed |
| Implementation | 15 min | Simple YAML changes |
| Quality Gates | 10 min | Local verification |
| **Total** | **~35 min** | |

---

## Related Plans

- `plans/GOAP_GITHUB_ACTIONS_2026-02-14.md` - Previous CI modernization
- `plans/GOAP_NIGHTLY_CI_FIXES_2026-02-16.md` - Nightly test fixes

---

## Next Steps

1. **Execute Phase 1**: Already analyzed above
2. **Execute Phase 2**: Modify ci.yml as specified
3. **Execute Phase 3**: Run quality gates locally
4. **Commit**: Create atomic commit with changes
