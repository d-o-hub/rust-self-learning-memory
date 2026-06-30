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
- Issue #674 tracks current drift; WG-175 is the release work group
- Use `gh release create` with auto-generated changelog from conventional commits
- Post-release, verify crates.io publish completes for all workspace members

## Implementation

### D1 — Gitleaks (Completed)

PR #675 (merged 2026-06-30) added the missing fingerprints to `.gitleaksignore`.
The scheduled security scan is now green.

### D2 — Release Drift (Pending)

WG-175 is queued to cut v0.1.33. The 100-commit drift exceeds the target cadence
by ~3-4× and should not recur. After release, the release-guard skill should be
consulted every 2 weeks.

## Consequences

### Positive
- Scheduled security scan is green for the first time in weeks
- Clear policy for release cadence prevents future drift accumulation
- Fingerprint maintenance is documented for future directory restructures

### Negative
- `.gitleaksignore` continues to grow (84 entries); may need periodic pruning of entries for deleted files
- Release cadence target is aspirational — no automated enforcement yet

### Neutral
- thiserror v1/v2 duplication remains (transitive, not actionable until upstream updates)
- The 100-commit drift will be resolved when WG-175 executes

## Follow-up

- [ ] WG-175: Cut v0.1.33 release (closes #674)
- [ ] Consider GitHub Action to warn when commit count since last tag exceeds 30
- [ ] Consider automated release-please or similar for cadence enforcement
