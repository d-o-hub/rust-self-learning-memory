# ADR-038: Local CI Parity for Clippy Tests

- **Status**: Proposed
- **Date**: 2026-03-07
- **Deciders**: Project maintainers
- **Related**: ADR-022 (GOAP Agent System), ADR-033 (Modern Testing Strategy)

## Context

During PR #349 CI remediation, clippy errors were discovered in test files that were not caught locally before pushing:

1. **CI Quick Check workflow** runs:
   ```yaml
   cargo clippy --tests -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used
   ```

2. **Local `scripts/code-quality.sh clippy`** runs:
   ```bash
   cargo clippy --workspace -- -D warnings
   ```
   This only checks library code, NOT test code.

3. **Result**: Test-only clippy errors (unused variables, tautological expressions) slip through to CI.

## Problem Analysis

### Gap Identified

| Check | Local Script | CI Workflow | Gap |
|-------|--------------|-------------|-----|
| Clippy on lib | ✅ `--workspace` | ✅ `--lib` | None |
| Clippy on tests | ❌ Not run | ✅ `--tests` | **Missing** |
| Format check | ✅ `--all --check` | ✅ `--all --check` | None |

### Root Cause

The `scripts/code-quality.sh` script was designed for fast local feedback on library code, but CI enforces stricter checks including test code. This creates a "works locally, fails in CI" scenario.

### Specific Errors Missed

1. **Unused variable** in test: `let entry = ...` should be `let _entry = ...`
2. **Tautological boolean**: `a >= b || a < b` is always true
3. **field_reassign_with_default**: Should use struct update syntax

All three were in `tests/serialization_property_tests.rs` files.

## Decision

**Update local quality gates to match CI exactly:**

### Option A: Update `scripts/code-quality.sh` (Recommended)

Add `--tests` flag to clippy checks:

```bash
# Before (lib only)
cargo clippy --workspace -- -D warnings

# After (lib + tests)
cargo clippy --workspace --tests -- -D warnings \
  -A clippy::expect_used \
  -A clippy::uninlined_format_args \
  -A clippy::unwrap_used
```

### Option B: Add Pre-commit Hook

Create `.git/hooks/pre-commit` or `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: clippy-tests
        name: Clippy on tests
        entry: cargo clippy --tests -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used
        language: system
        pass_filenames: false
```

### Option C: Update `scripts/quality-gates.sh`

Add explicit test clippy check before running quality gates test.

## Implementation Plan

### Phase 1: Immediate Fix (P0)

1. Update `scripts/code-quality.sh` to include `--tests` flag
2. Add same clippy allow-list as CI (`-A clippy::expect_used`, etc.)
3. Document the change in AGENTS.md

### Phase 2: Pre-commit Hook (P1)

1. Create `.pre-commit-config.yaml` for developers who use pre-commit
2. Include both format and clippy checks with CI parity

### Phase 3: Documentation (P2)

1. Update AGENTS.md with "Run Before Push" checklist
2. Add to GOAP execution template as a quality gate step

## Rationale

1. **CI Parity**: Local checks should match CI exactly to prevent "works locally, fails in CI"
2. **Fast Feedback**: Catching errors locally is faster than waiting for CI
3. **Developer Experience**: Consistent expectations between local and CI environments
4. **GOAP Alignment**: ADR-022 emphasizes quality gates between phases

## Consequences

### Positive

- Earlier detection of test-code clippy errors
- Consistent behavior between local and CI
- Reduced CI failure rate

### Negative

- Slightly longer local check time (tests add ~10-20% more files to check)
- May require fixing existing test code to pass stricter checks

## Acceptance Criteria

1. `./scripts/code-quality.sh clippy` catches all errors that CI would catch
2. `./scripts/code-quality.sh check` includes test clippy checks
3. Documentation updated in AGENTS.md
4. GOAP_STATE.md updated with learning

## References

- `.github/workflows/quick-check.yml` - CI clippy configuration
- `scripts/code-quality.sh` - Local quality script
- `scripts/quality-gates.sh` - Quality gates runner
- ADR-022 - GOAP Agent System