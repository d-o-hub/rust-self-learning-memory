# Validation Latest — 2026-07-18 Missing-tasks master (PR #873)

**Orchestrator**: GOAP + agent-coordination swarm (Agent A: run-evals; Agent B: docs + W2.5 nightly polish)  
**Goal**: Land remaining improvements-backlog packages on branch `feat/goap-missing-tasks-s12-s14b-s11b-2026-07-18`  
**Workspace**: `0.1.36` unreleased · **Tag**: `v0.1.35` · **PR**: [#873](https://github.com/d-o-hub/rust-self-learning-memory/pull/873)

## Packages completed this sprint

| Package | Evidence | Status |
|---------|----------|--------|
| S1.2 remainder | CacheKey mode/provider/ranking/generation + RetrievalProvenance | ✅ |
| S1.4b | `pending_eviction_failures` / `reconcile_pending_evictions` + tests | ✅ |
| S1.1b | `sandbox-dev` feature; `scripts/check-source-reachability.sh` | ✅ |
| K3.2 | High-risk skill positive/negative eval fixtures | ✅ |
| K3.3 | Expanded `skill-rules.json` + `validate-skill-routes.sh` | ✅ partial |
| W2.2b / W2.4 | `test-workflow-guards.sh`, `test-release-workflow.sh` | ✅ |
| W2.3b | quality_gates refuse metrics from failed subprocesses | ✅ |
| W2.5a | benchmarks.yml no dummy soft-pass; `fail-on-alert: true` | ✅ |
| W2.5b nightly | upload **before** cleanup; `check-ignored-tests.sh` ratchet step | ✅ |
| Harness #862–#869 | Closed (code shipped in #870) | ✅ |
| D3.3 / V5.1 | GOALS / ACTIONS / ROADMAP / CURRENT / GOAP_STATE / this file | ✅ |

## Still deferred

| Item | Notes |
|------|-------|
| **F4 pilots only** | Feature pilots intentionally out of this PR |

## Open PR

| PR | Branch | Role | Status |
|----|--------|------|--------|
| #873 | `feat/goap-missing-tasks-s12-s14b-s11b-2026-07-18` | Missing tasks Wave 1–3 | 🟡 Open — CI/review |

## W2.5 nightly polish (Agent B)

| Check | Result |
|-------|--------|
| Upload before cleanup | ✅ Fixed: extract trends → upload results → upload trends → then `cargo clean` |
| Ignored-test ratchet in nightly | ✅ Added `./scripts/check-ignored-tests.sh` step before regular tests |
| Honor `run_slow_tests` dispatch input once / de-dupe suites | 🔵 Residual (not required for this polish; avoid larger nightly redesign) |

## Prior merged swarm (context)

| PR | Status |
|----|--------|
| #860 S1.7 + K3.1b/W2.1b | ✅ Merged |
| #870 Harness engineering | ✅ Merged |
| #872 release-cadence-manager | ✅ Merged |

## Merge gate (PR #873)

- [ ] `mergeable=MERGEABLE` and `mergeStateStatus=CLEAN`
- [ ] Required checks SUCCESS (no FAILURE/CANCELLED on non-skipped)
- [ ] Actionable PR comments addressed
- [ ] Do not touch `scripts/run-evals.sh` ownership (Agent A)

## Master execution record

`plans/GOAP_MISSING_TASKS_MASTER_2026-07-18.md`
