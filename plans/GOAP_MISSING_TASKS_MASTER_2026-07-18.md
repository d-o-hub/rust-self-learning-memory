# GOAP Missing Tasks Master — 2026-07-18

**Status**: Wave 1–3 on PR #873; F4 remainder + spikes on PR #874 — **all missing implemented, no release**  
**Coordinator**: goap-agent + agent-coordination swarm  
**Workspace**: `0.1.36` (unreleased) · **Released tag**: `v0.1.35`  
**Backlog source**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`  
**PRs**: <https://github.com/d-o-hub/rust-self-learning-memory/pull/873> · <https://github.com/d-o-hub/rust-self-learning-memory/pull/874>

---

## Goal

Land the highest-value remaining packages from the 2026-07-14 improvements backlog after harness (#870), S1.7/K3.1b/W2.1b (#860), and release-cadence-manager (#872) merged to main. **Do not cut a release** in this campaign.

## Swarm split

| Phase | Agent | Ownership | Outcome |
|-------|-------|-----------|---------|
| 1 | A | `scripts/run-evals.sh` (do not conflict) | Skill-eval runner work |
| 1 | B | Plans D3.3/V5.1 + optional W2.5 nightly polish | Plan docs + nightly reorder |
| 2 | A | `scripts/run-feature-spike.sh` (+ configs) when ready | Spike producer script |
| 2 | C | Spike decisions + canonical plans | `plans/STATUS/spikes/*.json` + GOALS/ACTIONS/ROADMAP/CURRENT/GOAP_STATE/VALIDATION |

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
| F4.1–F4.4 | Feature pilots | Implemented on PR #874; spikes **GO** |
| S1.1c | Wasmtime/WASI spike | Decision artifact **NO-GO**; fail-closed retained |

## F4 remainder (PR #874)

| ID | Package | Status | Spike |
|----|---------|--------|-------|
| F4.1 | Provenanced retrieval API | ✅ Implemented | **GO** `plans/STATUS/spikes/F4.1.json` |
| F4.2 | Operation journal | ✅ Implemented | **GO** `plans/STATUS/spikes/F4.2.json` |
| S1.5b/F4.3 | Local model digests/size pins | ✅ Implemented | **GO** `plans/STATUS/spikes/F4.3.json` |
| F4.4 | Skill contract compiler | ✅ Implemented | **GO** `plans/STATUS/spikes/F4.4.json` |
| S1.1c | Wasmtime/WASI feasibility | ✅ Decided | **NO-GO** `plans/STATUS/spikes/S1.1c.json` |

**Phase 2 note**: Spike producer `scripts/run-feature-spike.sh` + validator `scripts/validate-feature-spike.sh` used with configs `plans/spikes/*.toml` (single-line arrays; `force_decision` GO for F4.1–4, NO-GO for S1.1c). Schema: `plans/STATUS/spikes/README.md`.

## Still deferred

None from the 2026-07-14 improvements backlog (F4 pilots implemented; S1.1c NO-GO recorded). Research backlog WG-108/110/125 remain optional product epics, not this plan's missing tasks. **Release intentionally deferred.**

## Prior waves (same day / series)

| Wave | Plan / PR | Status |
|------|-----------|--------|
| S1.7 + K3.1b + W2.1b | `GOAP_MISSING_TASKS_S17_K31B_W21B_2026-07-18.md` / #860 | ✅ Merged |
| Harness engineering | `GOAP_EXECUTION_PLAN_HARNESS_SPRINT_2026-07-18.md` / #870 | ✅ Merged |
| Release cadence manager | `GOAP_RELEASE_CADENCE_MANAGER.md` / #872 | ✅ Merged |
| Master wave 1–3 | PR #873 | 🟡 Open |
| F4 remainder + spikes | PR #874 | 🟡 Open |

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
| F4 spikes | `plans/STATUS/spikes/F4.{1,2,3,4}.json` decision=GO |
| S1.1c spike | `plans/STATUS/spikes/S1.1c.json` decision=NO-GO |
| Plans | Active set → this master + PR #874; deferred empty; **no release** |

## Validation pointer

`plans/STATUS/VALIDATION_LATEST.md`
