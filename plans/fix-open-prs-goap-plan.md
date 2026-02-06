# GOAP Plan: Fix All Open PRs

**Created**: 2026-02-06
**Goal**: Fix CI failures on all 7 open PRs and get them mergeable

## Analysis Phase

### PR Summary

| PR# | Title | Branch | CI Status | Critical Failures |
|-----|-------|--------|-----------|-------------------|
| 263 | Phase 4 Sprint 1 - Performance | feat/phase4-sprint1-performance | ❌ FAILING | Clippy, Tests, Doctest, CodeQL, MCP Build, Secret Scanning |
| 260 | bump wasmtime-wasi 40→41 | dependabot/cargo/wasmtime-wasi-41.0.1 | ⚠️ PARTIAL | Supply Chain Audit only |
| 258 | bump upload-artifact 4→6 | dependabot/github_actions/actions/upload-artifact-6 | ⚠️ PARTIAL | Supply Chain Audit only |
| 257 | bump setup-python 5→6 | dependabot/github_actions/actions/setup-python-6 | ⚠️ PARTIAL | Supply Chain Audit only |
| 256 | bump action-actionlint 1.69→1.70 | dependabot/github_actions/reviewdog/action-actionlint-1.70.0 | ⚠️ PARTIAL | Supply Chain Audit only |
| 250 | bump sysinfo 0.37→0.38 | dependabot/cargo/sysinfo-0.38.0 | ⚠️ PARTIAL | Supply Chain Audit only |
| 244 | bump wait-on-check-action 1.4→1.5 | dependabot/github_actions/lewagon/wait-on-check-action-1.5.0 | ⚠️ PARTIAL | Supply Chain Audit only |

### Problem Categories

1. **PR #263 (Critical)**: Multiple code failures - needs code fixes
2. **PRs #244, 250, 256, 257, 258, 260**: Supply Chain Audit failures only - likely `deny.toml` config issue

## Decomposition Phase

### Task Graph

```
T1: Diagnose Supply Chain Audit failures (affects 6 PRs)
├── T1.1: Check deny.toml configuration
├── T1.2: Run cargo deny locally
└── T1.3: Fix deny.toml if needed

T2: Fix PR #263 (main feature PR)
├── T2.1: Checkout the branch
├── T2.2: Run clippy and fix warnings
├── T2.3: Fix doctest failures  
├── T2.4: Fix MCP Build issues
├── T2.5: Run tests and fix failures
├── T2.6: Fix secret scanning issues
└── T2.7: Push fixes and verify CI

T3: Merge/Close ready PRs (after T1 completes)
├── T3.1: Verify Dependabot PRs pass after deny.toml fix
├── T3.2: Merge passing Dependabot PRs (with approval)
└── T3.3: Update PR #263 with main if needed
```

## Strategy Phase

**Execution Pattern**: Sequential with parallel sub-tasks

1. **First**: Fix Supply Chain Audit (T1) - affects 6 PRs at once
2. **Second**: Fix PR #263 (T2) - main feature work
3. **Third**: Merge ready PRs (T3) - cleanup

## Execution Plan

### Phase 1: Supply Chain Audit Fix (Parallel Impact)
- Diagnose `cargo deny check` failures locally
- Update `deny.toml` to allow new versions
- Commit to main branch → triggers re-check on all PRs

### Phase 2: Fix PR #263 (Sequential)
1. Checkout `feat/phase4-sprint1-performance`
2. Fix clippy warnings
3. Fix doctest failures
4. Fix MCP build issues
5. Run full test suite
6. Address secret scanning
7. Push and verify

### Phase 3: Merge Ready PRs
- Re-check CI status on Dependabot PRs
- Merge those that pass

## Success Criteria

- [ ] All 7 PRs have green CI (or only non-blocking failures)
- [ ] Supply Chain Audit passing on all PRs
- [ ] PR #263 ready for merge
- [ ] Dependabot PRs merged or clearly actionable

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Supply Chain failure is in GH Actions, not local | Check workflow logs for details |
| PR #263 has breaking changes | Run comprehensive tests locally first |
| Merge conflicts in Dependabot PRs | Rebase after main updates |
