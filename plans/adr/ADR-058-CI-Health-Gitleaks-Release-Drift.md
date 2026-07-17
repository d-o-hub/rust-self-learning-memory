# ADR-058: CI Health — Gitleaks False Positives & Release Drift Mitigation

- **Status**: 🟢 Accepted (Implemented)
- **Date**: 2026-06-14 (retroactive — originally documented in GOAP analysis but file never committed)
- **Implemented**: 2026-06-30 (PR #675 + PR #687)
- **Deciders**: Project maintainers
- **Related**: ADR-057 (CI Health: PR #616 / Nightly Timeout), ADR-034 (Release
  Engineering Modernization), Issue #674 (Release Drift),
  `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-14.md`

## Context

A comprehensive CI health sweep on 2026-06-14 identified two systemic issues:

### Problem 1: Gitleaks Scheduled Security Scan Failing

The full-history gitleaks scan (`security.yml`, scheduled) was reporting 3
findings, all **confirmed false positives**:

1. Historical placeholder API keys in documentation examples
2. Test fixture tokens in archived skill files
3. Example environment variable values in README/docs

These were not matched by `.gitleaksignore` because:
- The `.claude/` → `.agents/` directory rename changed commit SHAs
- Gitleaks fingerprints are commit-path-content dependent; moving files invalidates old entries
- The ignore list had grown to 81 entries but missed these 3 new fingerprint variants

### Problem 2: Release Drift

Issue #674 documented growing release drift: the workspace version was bumped to
`0.1.33` but no git tag or GitHub release existed. Commits accumulated without a
release checkpoint, creating audit/changelog difficulty and making it harder to
bisect regressions.

At the time of analysis: 94 commits since `v0.1.32`. By 2026-06-30: 100 commits.

## Decision

### D1: Gitleaks Fingerprint Maintenance

- Add the 3 missing fingerprints to `.gitleaksignore`
- Document the fingerprint maintenance procedure in CI guidance
- Accept that directory renames will invalidate fingerprints and require re-triage

### D2: Release Cadence Enforcement

- Target release every 20-30 commits or biweekly, whichever comes first
- Warn at 20 commits or 10 days; block ordinary PRs at 30 commits or 14 days
- Maintain one canonical drift issue by updating it in place
- Permit a trusted collaborator to label only the release-preparation PR to
  break a hard-limit deadlock
- Push one exact version tag only after the release PR merges and all required
  checks on `main` pass; the mandatory `release.yml` workflow creates the release
- Post-release, verify crates.io publish completes for all workspace members

## Implementation

### D1 — Gitleaks (Completed)

PR #675 (merged 2026-06-30) added the missing fingerprints to `.gitleaksignore`.
The scheduled security scan is now green.

### D2 — Release Drift (Implemented)

WG-175 cut v0.1.33, but issue #843 showed that alert-only automation still
allowed repeated drift and recreated the tracking issue on every push. The
2026-07-17 remediation centralizes the state calculation in
`scripts/check-release-drift.sh`, adds boundary regression tests, enforces the
cadence on pull requests, and gives issue mutation to a separate least-privilege
job. Tag pushes trigger immediate issue closure after a successful release.

## Consequences

### Positive
- Scheduled security scan is green for the first time in weeks
- Clear policy for release cadence prevents future drift accumulation
- Fingerprint maintenance is documented for future directory restructures

### Negative
- `.gitleaksignore` continues to grow (84 entries); may need periodic pruning of entries for deleted files
- The hard cadence gate requires a trusted-collaborator label for a release PR
  that starts after the deadline

### Neutral
- thiserror v1/v2 duplication remains (transitive, not actionable until upstream updates)
- Historical replacement drift issues remain closed as an audit trail; new
  workflow runs update one canonical issue

## Follow-up

- [x] WG-175: Cut v0.1.33 release (closes #674)
- [x] Add GitHub Action warning and enforcement for release cadence (#843)
- [x] Replace per-push issue recreation with a canonical issue upsert (#843)
- [ ] Evaluate release-please only if the enforced existing pipeline remains
  operationally expensive
