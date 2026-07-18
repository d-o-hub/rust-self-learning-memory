# GOAP Missing Tasks Master — 2026-07-18

**Status**: Code + plans complete — PR #873 open  
**Coordinator**: goap-agent + agent-coordination swarm  
**Branch**: `feat/goap-missing-tasks-s12-s14b-s11b-2026-07-18`  
**Workspace**: `0.1.36` (unreleased) · **Released tag**: `v0.1.35`  
**Backlog source**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`  
**Open PR**: <https://github.com/d-o-hub/rust-self-learning-memory/pull/873>

---

## Goal

Land the highest-value remaining packages from the 2026-07-14 improvements backlog after harness (#870), S1.7/K3.1b/W2.1b (#860), and release-cadence-manager (#872) merged to main.

## Swarm split

| Agent | Ownership | Outcome |
|-------|-----------|---------|
| A | `scripts/run-evals.sh` (do not conflict) | Skill-eval runner work |
| B | Plans D3.3/V5.1 + optional W2.5 nightly polish | This record + plan docs + nightly reorder |

## Packages completed

| ID | Package | Notes |
|----|---------|-------|
| S1.2 | Retrieval cache identity remainder | mode, provider identity, ranking version, index generation; redacted provenance (ADR-074) |
| S1.4b | Eviction reconciliation | Typed partial failures; pending + reconcile APIs |
| S1.1b | Sandbox quarantine | Node sandbox behind optional `sandbox-dev`; reachability script |
| K3.2 | High-risk skill evals | Positive + negative fixtures for release-guard, pr-readiness, commit, ci-fix, code-quality, test-runner, goap-agent, web-doc-resolver |
| K3.3 | Skill routing (partial) | Expanded `skill-rules.json` + `validate-skill-routes.sh` |
| W2.2b | Cancelled-required guards | `scripts/test-workflow-guards.sh` |
| W2.3b | quality_gates honesty | Refuse metrics when subprocess fails; package name guards |
| W2.4 | Release publish fixtures | `scripts/test-release-workflow.sh` |
| W2.5 | Benchmark + nightly signal | No dummy soft-pass; `fail-on-alert: true`; nightly **upload before cleanup**; ignore-ceiling ratchet step |
| Harness | #862–#869 | Closed; implementation already in #870 |
| D3.3 / V5.1 | Plan hygiene | GOALS, ACTIONS, ROADMAP_ACTIVE, CURRENT, VALIDATION_LATEST, GOAP_STATE |

## Still deferred

| Item | Reason |
|------|--------|
| **F4 pilots only** | Optional feature pilots; not blocking correctness/gates |

## Prior waves (same day / series)

| Wave | Plan / PR | Status |
|------|-----------|--------|
| S1.7 + K3.1b + W2.1b | `GOAP_MISSING_TASKS_S17_K31B_W21B_2026-07-18.md` / #860 | ✅ Merged |
| Harness engineering | `GOAP_EXECUTION_PLAN_HARNESS_SPRINT_2026-07-18.md` / #870 | ✅ Merged |
| Release cadence manager | `GOAP_RELEASE_CADENCE_MANAGER.md` / #872 | ✅ Merged |
| This master wave | PR #873 | 🟡 Open |

## Acceptance snapshot

| Gate | Expected |
|------|----------|
| Core retrieval/eviction tests | `cargo nextest run -p do-memory-core retrieval::cache` / s13_s14 |
| Reachability | `./scripts/check-source-reachability.sh` |
| Workflow guards | `./scripts/test-workflow-guards.sh --cancelled-required` |
| Release fixtures | `./scripts/test-release-workflow.sh --publish-fixtures` |
| Benchmark fixtures | `./scripts/test-benchmark-workflow.sh --fixtures` |
| Ignored ratchet | `./scripts/check-ignored-tests.sh` / `--fixture ratchet` |
| Nightly order | Extract + upload artifacts **before** `cargo clean` |
| Plans | Active set points at this master + PR #873; deferred = F4 only |

## Validation pointer

`plans/STATUS/VALIDATION_LATEST.md`
