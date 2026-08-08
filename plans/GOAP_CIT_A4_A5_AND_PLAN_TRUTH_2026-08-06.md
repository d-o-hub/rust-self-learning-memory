# GOAP: CIT-A4/A5 Implementation and Plan-Truth Refresh (2026-08-06)

- **Status**: Implemented (PR in review)
- **Date**: 2026-08-06
- **Baseline**: `main` at `92db07bf` · workspace `0.1.38` · tag `v0.1.37`
- **Decisions**: [ADR-079](adr/ADR-079-Fail-Closed-CI-Required-Check-Control-Plane.md) §6/§7 (Proposed; the workflow-side half implemented here, ruleset half still requires maintainer approval)
- **Related**: [ADR-072](adr/ADR-072-Authority-Evidence-Enforcement-Governance.md), [ADR-078](adr/ADR-078-Trusted-Publishing-OIDC.md), `agent_docs/LESSONS.md` LESSON-014, `scripts/test-release-workflow.sh`
- **Orchestration**: GOAP agent skill — hybrid strategy: parallel research swarm → parallel workflow implementation → sequential validation → PR

## Scope and non-scope

This wave implements the **actionable** remainder of the active plan
(`plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`) that does not need
maintainer ADR acceptance or live-ruleset mutation:

| Package | Status before | Disposition |
|---------|---------------|-------------|
| CIT-A4 release/publish trigger truth (ACT-338) | Planned | ✅ Implemented here |
| CIT-A5 durable fuzz/mutation evidence (ACT-339) | Planned | ✅ Implemented here (fuzz; mutants already durable) |
| R-F10 OIDC trusted publishing (ACT-325) | In progress (stale) | ✅ Already shipped — `publish-crates.yml` uses `id-token: write`; plans refreshed |
| R-F4 SIMD cosine (ACT-326) | In progress (stale) | ✅ Already shipped — `cosine_similarity_simd` + bench variants; plans refreshed |
| PTA-A1/A2/A3 | ✅ #916 | Already merged |
| ADR-080/081 attribution (RAT-A1…A7) | PR #927 open | Untouched; separate review queue |
| ADR-079 CIT-A1/A2/A3 (aggregate + ruleset) | Blocked | **Requires maintainer ADR acceptance + explicit ruleset approval** — not performed here |

## Changes

### CIT-A4 — truthful release/publish triggers (ADR-079 §6)

1. **`.github/workflows/release.yml`**: removed the `workflow_dispatch` trigger.
   Release creation is tag-only under ADR-072; the manual dispatch always failed
   its tag preflight, so it was a guaranteed-failure advertised trigger.
2. **`.github/workflows/publish-crates.yml`**:
   - `cargo publish` and `--dry-run` now use `--locked` (reproducible; matches
     `scripts/test-release-workflow.sh --publish-fixtures`, which now passes).
   - The three fixed `sleep 30` propagation waits were replaced with bounded
     crates.io API polling (20 × 15s = 5 min ceiling) per LESSON-014 and
     AGENTS.md publish-pipeline guidance.
   - New **dependency-closure verification** step on single-crate dispatch
     (`inputs.crate != ''`): a requested crate must publish, prove already
     present, or fail with an explicit reason. Workspace normal-dependency
     closure (dev-deps excluded) must already exist on crates.io; otherwise the
     job fails with `::error::` naming the missing dependency. Skipped
     prerequisites can no longer turn into a doomed or silent publish.
   - `cargo-semver-checks` findings are surfaced in `$GITHUB_STEP_SUMMARY`
     instead of a silent `continue-on-error` pass.

### CIT-A5 — durable informational evidence (ADR-079 §7)

- **`.github/workflows/fuzz.yml`**:
  - Crash/timeout/oom artifacts now upload with `if: always()` +
    `if-no-files-found: ignore` (previously `if: failure()` after
    `continue-on-error: true` steps — the job could never be "failure", so
    crash evidence was **never uploaded**).
  - New **Report Fuzz Status** step captures each target's exit status and
    artifact presence, emits `::error::` annotations, and fails the
    (non-merge-required, scheduled) job so a green conclusion is never the only
    durable signal. Startup failures and timeouts are now visible too.
- `mutants.yml` already uploads shard reports with `if: always()` — no change.

### Plan truth refresh

Stale trackers (R-F4/R-F10 marked "in progress", open-PR lists pointing at
merged #914/#915) were corrected across `GOALS.md`, `ACTIONS.md`,
`GOAP_STATE.md`, `ROADMAP_ACTIVE.md`, `STATUS/CURRENT.md`,
`STATUS/GAP_ANALYSIS_LATEST.md`, `STATUS/VALIDATION_LATEST.md`, and `README.md`.
ACT-325/ACT-326 are marked Done; ACT-338/ACT-339 are marked Done with this PR.

## Validation performed

- `yamllint` clean on all three changed workflows.
- Embedded bash/jq logic exercised against real `cargo metadata`
  (workspace-member name extraction for the `path+file://…#name@version`
  format, non-dev dependency filtering, `(.versions // [])` guard for
  crates.io error responses).
- `./scripts/test-release-workflow.sh --publish-fixtures` passes (asserts
  `--locked` present in `publish-crates.yml`).
- `./scripts/validate-gate-contract.sh` and `./scripts/validate-plans.sh`
  pass (see VALIDATION_LATEST slice).

## Exit criteria status (ADR-079)

| Acceptance item | Status |
|-----------------|--------|
| Release manual dispatch absent, tag release remains sole authority | ✅ workflow side |
| Publish selection/dependency fixtures cannot silently skip requested work | ✅ workflow side |
| Fuzz crashes upload evidence and produce a visible non-green signal | ✅ |
| Live ruleset requires `CI / Required` | ⏸ Requires ADR-079 acceptance + staged migration (maintainer) |
| Failed/cancelled/missing applicable jobs fail an aggregate | ⏸ CIT-A1, maintainer |
| Gate-contract semantic validation | ⏸ CIT-A3, after CIT-A1/A2 |

## Follow-ups

- Accept ADR-079, then implement the `CI / Required` aggregate (CIT-A1),
  fail-closed waiters + actor parity (CIT-A2), and semantic gate-contract
  validation (CIT-A3) with the live-ruleset change approved separately.
- Review/merge PR #927 (ADR-080/081 attribution) in the queue.
- Consider a later wave for publish-crates fixture tests (single-crate
  dispatch with missing dependency must fail with the named reason).
