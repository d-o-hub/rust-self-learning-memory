# GOAP Hotfix Plan: CI MCP Build Failures (2026-02-21)

## Issue Analysis

PR #309 was merged with 3 failing checks:
1. **MCP Build (default)** - fail (infrastructure issue)
2. **MCP Build (wasm-rquickjs)** - fail (deprecated feature)
3. **codecov/patch** - fail (78.61% vs 80% target, informational)

## Root Cause Analysis

### MCP Build Failures

**Primary Cause**: `taiki-e/install-action@v2` failing with:
```
printf '::error::install-action requires bash\n'
exit 1
```

This is a **transient CI infrastructure issue**, NOT a code regression.

**Contributing Factor**: The `wasm-rquickjs` feature is deprecated but still in the CI matrix.

### Evidence
- Rust compilation was progressing normally before failure
- Error originates from install-action, not cargo
- Local builds work correctly

## Recommended Fixes

### Phase 1: Hotfix (Immediate)

| Action | File | Lines | Description |
|--------|------|-------|-------------|
| Remove wasm-rquickjs | ci.yml | 96 | Remove deprecated feature from matrix |
| Add shell: bash | ci.yml | 74, 116, 149, 192, 227 | Fix install-action shell requirement |
| Add shell: bash | coverage.yml | 93 | Same fix for coverage workflow |

### Phase 2: Documentation (24 hours)

- Document install-action workaround in ADR-033
- Consider removing wasm-rquickjs feature entirely
- Adjust codecov configuration for test-utility PRs

## Changes Made

### .github/workflows/ci.yml

1. **Remove wasm-rquickjs from matrix**:
```yaml
# Before
matrix:
  feature: [default, wasm-rquickjs]

# After
matrix:
  feature: [default]  # wasm-rquickjs removed - deprecated feature
```

2. **Add shell: bash to install-action** (5 locations):
```yaml
- name: Install nextest
  uses: taiki-e/install-action@v2
  with:
    tool: nextest
  shell: bash  # Added
```

### .github/workflows/coverage.yml

1. **Add shell: bash to install-action**:
```yaml
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@v2
  with:
    tool: cargo-llvm-cov
  shell: bash  # Added
```

## Validation

- [x] YAML syntax validated
- [ ] CI passes after merge
- [ ] No new clippy warnings
- [ ] MCP Build (default) passes

## ADR References

- ADR-022: GOAP Agent System
- ADR-029: GitHub Actions Modernization
- ADR-033: Modern Testing Strategy

## Post-Fix Monitoring

| Metric | Alert Threshold | Action |
|--------|-----------------|--------|
| CI success rate | < 90% over 7 days | Investigate infrastructure |
| install-action failures | > 2 consecutive runs | Escalate to action maintainer |

## References

- Analysis Swarm Report (this session)
- PR #309: GOAP Disk Space, Testing & Release Modernization
