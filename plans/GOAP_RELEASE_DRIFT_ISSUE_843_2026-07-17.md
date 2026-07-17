# GOAP Plan: Permanent Release-Drift Remediation (#843)

- **Status**: Code complete
- **Date**: 2026-07-17
- **Issue**: [#843](https://github.com/d-o-hub/rust-self-learning-memory/issues/843)
- **Owner**: Release engineering
- **Related ADRs**: ADR-034, ADR-045, ADR-058

## Goal

Prevent release drift from silently growing beyond the cadence accepted in
ADR-058 while keeping one stable, actionable tracking issue. The automation
must distinguish expected development state from invalid release state and must
not prescribe a validation command that is guaranteed to fail before a tag
exists.

## Codebase Findings

1. `Cargo.toml` is at `0.1.35`, the latest release tag is `v0.1.34`, and the
   released-version documents correctly remain at `v0.1.34`. A one-patch-ahead
   workspace version is normal development state, not tag corruption.
2. `.github/workflows/release-drift.yml` closes and recreates the open
   `release-drift` issue on every qualifying push. This generated at least 20
   replacement issues between `v0.1.33` and issue #843, destroying stable issue
   history and creating notification noise.
3. The workflow alerts after more than five `feat`/`fix` commits, while ADR-058
   defines the release cadence as 20-30 total commits or 14 days, whichever
   comes first. The implementation and accepted policy disagree.
4. The generated remediation asks maintainers to run
   `release-manager.sh validate`. That command invokes
   `verify-release-state.sh --check-tag`, so it fails whenever the workspace has
   intentionally advanced to the next version before its tag exists.
5. The workflow only reports drift after pushes to `main`; it has no PR-visible
   warning before the 30-commit hard limit is crossed.

## State Model

| State | Meaning | Automation |
|---|---|---|
| `workspace == latest tag` and no commits after tag | Released/clean | Close the canonical drift issue |
| Workspace is exactly one valid SemVer step ahead | Normal development | Measure commit count and release age |
| 20-29 commits or release age >= 10 days | Release due soon | Upsert one canonical warning issue |
| >= 30 commits or release age >= 14 days | Hard cadence breach | Upsert issue and fail the drift gate |
| Workspace is behind/equal with unreleased commits, or skips versions | Invalid version state | Upsert issue and fail the drift gate |

Release age is based on the tagged commit timestamp, not tag creation sorting.
The latest release tag is selected with SemVer version sorting.

## Execution Plan

1. **Centralize drift calculation** in a testable shell script used by CI.
   Emit stable key/value outputs for workspace version, latest tag, commit
   counts, age, severity, and reason.
2. **Align thresholds with ADR-058**: warning at 20 total commits or 10 days;
   hard failure at 30 commits or 14 days; fail immediately for invalid version
   ordering.
3. **Replace issue churn with idempotent upsert**: edit the existing open issue
   in place, create only when none exists, and close it automatically after the
   drift is resolved.
4. **Add pull-request visibility** without granting issue write permissions to
   PR code. PR runs calculate and summarize state; main/scheduled runs own issue
   mutation.
5. **Correct remediation instructions** so development-state checks do not
   require the future tag. Keep strict tag verification as the final pre-tag
   release gate.
6. **Add regression tests** for clean, due, overdue, and invalid-version states,
   including stable output suitable for GitHub Actions.
7. **Validate** with shell syntax/tests and repository workflow linting where
   available.

## Success Criteria

- Repeated qualifying pushes update one issue rather than creating replacements.
- A normal `0.1.35` workspace after `v0.1.34` is described as unreleased
  development, not malformed tag drift.
- Drift cannot pass unnoticed beyond 30 commits.
- Biweekly drift is detected even with fewer conventional commits.
- The issue body gives commands that can succeed in the state being reported.
- Automated tests cover threshold boundaries and version-state failures.

## Non-Goals

- Cutting or tagging `v0.1.35` in this change.
- Bypassing the mandatory release workflow or branch protection.
- Introducing a third-party release bot before the existing release process is
  made internally consistent.

## Progress

- [x] Analyze issue history, current tags/releases, scripts, workflow, and ADRs.
- [x] Define the corrected drift state model and prevention policy.
- [x] Implement centralized drift calculation and tests.
- [x] Make workflow issue handling idempotent and PR-visible.
- [x] Correct release validation guidance and exact-tag safety.
- [x] Run targeted validation and record results.

## Validation Results

- `bash -n` — passed for all changed shell scripts.
- `shellcheck` — passed with zero findings.
- `scripts/test-release-drift.sh` — all state and boundary scenarios passed.
- Live repository calculation — `warning / commit_warning` at 25 commits and
  five days since `v0.1.34`, confirming the normal next-version state.
- `yamllint` — passed for `release-drift.yml`.
- `git diff --check` — passed.
- `actionlint` — not available in this environment (`go` is also unavailable),
  so GitHub Actions semantic linting remains for CI.
- `scripts/check-docs-integrity.sh` — still fails on the repository's existing
  archived/moved-document links; no reported failure points to a file changed by
  this remediation.
- Expert review blockers addressed: exact-tag push, clean synchronized `main`,
  PR-only hard failure, label-triggered rerun, least-privilege issue job,
  canonical-issue concurrency, and post-merge tagging instructions.
