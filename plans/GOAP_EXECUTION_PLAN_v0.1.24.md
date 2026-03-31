# GOAP Execution Plan — v0.1.24 Stability & Hygiene Sprint

- **Date**: 2026-03-30
- **Branch**: `main`
- **Scope**: Fix test failures, merge dependency updates, sync plans/ to reality
- **Strategy**: Sequential (Phase 1 first, then parallel Phases 2+3)
- **Primary ADR**: ADR-048

## Goals

1. Achieve 0 test failures and 0 timeouts in `cargo nextest run --all`
2. Merge open Dependabot PRs and validate CI with updated actions
3. Bring plans/ folder into full alignment with v0.1.23 codebase reality
4. Close all stale WGs and pending ACTs from prior sprints

## Quality Gates

- `./scripts/code-quality.sh fmt`
- `./scripts/code-quality.sh clippy --workspace`
- `cargo nextest run --all` → 0 failures, 0 timeouts
- `cargo test --doc` → 0 failures
- `./scripts/quality-gates.sh`
- `./scripts/check-ignored-tests.sh`
- `./scripts/check-docs-integrity.sh`

## Phase Plan

### Phase 1 — Test Stability (Sequential, P0)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-059 | Fix benchmark_memory_usage | DBSCAN test exceeds 60s CI budget (actual: 103s). Options: increase budget to 120s in CI, or reduce dataset from 100K→50K points. | test-fix |
| WG-060 | Fix quality_gate_performance_regression | Test runs `cargo test` subprocess (inherently >120s). Add `#[ignore = "runs full cargo test subprocess, covered by CI workflows"]` or refactor to check cached benchmark results. | test-fix |

**Validation**: `cargo nextest run --all` passes with 0 failures, 0 timeouts.

### Phase 2 — Dependency & CI Modernization (Parallel, P1)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-061 | Merge Rust patch-minor PR #403 | 9 crate bumps (patch/minor). Verify CI passes on Dependabot branch before merge. | deps |
| WG-062 | Merge GitHub Actions PR #402 | Major bumps: checkout v4→v6, codecov v5→v6, wait-on-check v1.5→v1.6. Review breaking changes (Node.js 24 requirement, credential persistence changes). | ci-engineer |
| WG-063 | Post-merge CI validation | After PR #402 merge, verify all 13 workflows pass on main. | ci-engineer |

**Validation**: All Dependabot PRs merged. `gh run list --limit 5` shows all-green on main.

### Phase 3 — Plans & Documentation Hygiene (Parallel with Phase 2, P2)

| WG | Task | Details | Owner |
|----|------|---------|-------|
| WG-064 | Sync STATUS/CURRENT.md | Update: released version → v0.1.23, workspace version → 0.1.23, ignored tests 124→114, total tests updated | docs |
| WG-065 | Close stale ACTIONS.md pending items | Mark ACT-038–046 as ✅ Complete, resolve ACT-017 | docs |
| WG-066 | Resolve stale GOALS.md WGs | Update WG-008 (✅ revised target), WG-018 (✅ 31≤40), WG-019 (assess TODOs), WG-020 (✅ done via ACT-021), WG-021 (assess) | docs |
| WG-067 | Update GOAP_STATE.md + ROADMAP_ACTIVE.md | Add v0.1.24 sprint section, link ADR-048 and this execution plan | docs |

**Validation**: `grep -c "Pending" plans/GOALS.md plans/ACTIONS.md` returns only legitimately pending items.

## Dependencies & Sequencing

1. Phase 1 must complete first — test stability is a prerequisite for validating dependency PRs.
2. Phases 2 and 3 can run in parallel — dependency merges don't affect plan docs.
3. WG-063 depends on WG-062 completing first.
4. WG-067 should be the last task (captures final sprint state).

## Contingencies

- If actions/checkout v6 breaks CI: pin to v5 temporarily; open issue to track runner compatibility.
- If DBSCAN budget increase masks real regression: add monitoring via nightly benchmark trend artifacts (already implemented in WG-051).

## Exit Criteria

- `cargo nextest run --all` = 0 failures, 0 timeouts
- All Dependabot PRs merged or explicitly closed with reason
- plans/ STATUS, GOALS, ACTIONS, GOAP_STATE, ROADMAP_ACTIVE all reflect v0.1.23/v0.1.24 reality
- Zero stale "Pending" items from sprints prior to v0.1.24
