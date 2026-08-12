# GOAP: CIT-A1/A2/A3 Workflow-Side Wave (2026-08-06)

> **HISTORICAL — completed slice.** Superseded by the closure PR from branch
> `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the
> controller after creation). Retained for history per ADR-039; do not treat as
> active backlog.

- **Status**: Implemented (workflow/validator side) — PR in review
- **Date**: 2026-08-06
- **Baseline**: `main` at `92db07bf` stacked on the CIT-A4/A5 branch (`ci/cit-a4-a5-plan-truth-2026-08-06`)
- **Decisions**: [ADR-079](adr/ADR-079-Fail-Closed-CI-Required-Check-Control-Plane.md) (Proposed; this wave implements the workflow/validator half only)
- **Related**: `plans/GATE_CONTRACT.md`, `scripts/validate-gate-contract.sh`, ADR-072
- **Orchestration**: GOAP skill — hybrid: parallel waiter edits → aggregate + validator → validation swarm → stacked PR

## Scope and promotion-gate boundaries

ADR-079 §5 stages the migration: (1) add the aggregate without changing
protection, (2) observe/fault-inject, (3) add `CI / Required` to the live
ruleset with **explicit maintainer approval**, (4) verify, (5) remove the echo
anchor and obsolete waiters. This wave performs **stages 1–2 workflow/validator
work only**. The ruleset mutation and echo-anchor removal remain gated on
maintainer acceptance of ADR-079 — not performed here.

## Changes

### CIT-A2 — waiters fail closed (stage 1/2)

All five cross-workflow waiters (`ci.yml`, `coverage.yml`, `security.yml`,
`benchmarks.yml`, `file-structure.yml`):

- `allowed-conclusions: success,skipped,cancelled` → `success`
- `fail-on-no-checks: false` → `true` (a missing check now fails, never passes)
- new second wait step for **`Commit Message Lint`** (commit-lint failure can no
  longer be masked by a green format/Clippy job — G-P0-13)
- waiter `if:` no longer excludes `dependabot[bot]` (Quick Check always runs on
  PRs; the waiter is cheap). Downstream substantive jobs keep their actor
  exclusion for now — full downstream actor parity is the remaining CIT-A2
  step, pending the ADR-079 acceptance/CI-cost decision.

### CIT-A1 — stable `CI / Required` aggregate (stage 1, no ruleset change)

`ci.yml` gains a `required` job (`name: CI / Required`) that `needs`
`[test, mcp-build, multi-platform, quality-gates]`, runs with `if: always()`,
and fails closed on `failure`/`cancelled`/unknown results. `skipped` is
accepted only when the dependency job's own in-workflow `if` classifier marked
it inapplicable. The context is stable and never path-filtered, so a
workflow-level path filter cannot make it disappear.

**Not done**: adding `CI / Required` to the live `main-protection` ruleset
(requires ADR-079 acceptance + explicit approval); removing `pr-check-anchor.yml`.

### CIT-A3 — semantic gate-contract validation (negative fixtures)

`scripts/validate-gate-contract.sh --ci-parity` now fails when:

- `ci.yml` lacks a `CI / Required` job using `if: always()`
- any workflow accepts `cancelled`/`skipped` in `allowed-conclusions`
- any `fail-on-no-checks: false` remains on a gate waiter
- `release.yml` re-exposes `workflow_dispatch`
- `publish-crates.yml` drops `--locked` or reintroduces `run: sleep 30`

`plans/GATE_CONTRACT.md` documents the new aggregate row, the fail-closed
waiter semantics, and marks W2.1b acceptance for the semantic fixtures.

## Validation performed

- `yamllint` clean on all changed workflows.
- `bash -n` clean on `validate-gate-contract.sh`.
- `validate-gate-contract.sh --ci-parity` passes on this state; **negative
  tests** (reintroducing `success,skipped,cancelled` and deleting the
  aggregate) both fail with the expected message.
- `./scripts/test-release-workflow.sh --publish-fixtures` passes (from the
  CIT-A4/A5 base branch).
- `validate-plans.sh` warnings unchanged (pre-existing).

## Exit criteria status (ADR-079)

| Acceptance item | Status |
|-----------------|--------|
| Cancelled or absent fast checks never authorize expensive work | ✅ waiters fail closed, commit lint included |
| Commit-lint failure fails the fast gate | ✅ new wait step (workflow side) |
| Stable `CI / Required` context emitted for every PR | ✅ aggregate job (same-run, always()) |
| Failed/cancelled applicable jobs fail the aggregate | ✅ aggregate evaluation |
| Live ruleset requires `CI / Required` | ⏸ maintainer approval + staged ruleset change |
| Dependabot runs the same substantive assertions | ⏸ downstream actor parity (CI-cost decision) |
| Gate-contract fixture fails when cancellation/Dependabot/ruleset drift returns | ✅ cancellation + aggregate + publish fixtures; Dependabot/ruleset fixtures follow |
| Echo anchor removed | ⏸ stage 5, after ruleset migration |

## Follow-ups

- Accept ADR-079 → add `CI / Required` to `main-protection` (stage 3),
  verify a deliberately failed aggregate blocks merge, remove the echo anchor
  and obsolete waiters (stage 5).
- Full Dependabot/fork downstream actor parity with read-only permissions.
- Add gate-contract fixtures for Dependabot exclusion and ruleset-context drift.
