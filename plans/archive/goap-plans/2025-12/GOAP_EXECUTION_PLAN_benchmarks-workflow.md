# Execution Plan: Harden and Improve Benchmarks Workflow

## Objective
- Secure: Pin actions to SHAs; least-privilege permissions with per-job overrides.
- Reliable: Ensure correct git config for auto-push on main; explicit step timeouts.
- Consistent: Align artifact and cache actions with CI; standardize versions.
- Observable: Preserve step summary and artifacts with clear guidance.

## Proposed Changes
1) Security hardening
- Pin actions: checkout, rust-toolchain, install-action, cache, rust-cache, benchmark-action, upload/download-artifact, github-script, wait-on-check-action.
- Workflow-level permissions: contents: read.
- Job-level:
  - check-quick-check: contents: read.
  - benchmark: contents: write; actions: write (for cache save).
  - regression-check: contents: read; pull-requests: write.
- Checkout token/persistence tuned per job.

2) Reliability
- Git identity before auto-push; safe.directory set.
- Keep auto-push only on main; stash before benchmark store; unneeded after job ends.
- Add per-step timeouts for long operations.

3) Consistency with CI
- Use upload/download-artifact@v5 (pinned) and Swatinem/rust-cache@v2.8.2 (pinned).
- Keep dtolnay/rust-toolchain pinned; actions/cache@v5 pinned.

4) Observability
- Enhance step summary with artifact links; keep bench_results.txt & Criterion outputs in artifact.

## Concrete Edits
- Add top-level permissions: contents: read.
- benchmark job: set permissions, configure git identity, pin actions, step timeouts.
- regression-check job: pin actions, add timeouts.

## Validation Plan
- Lint: yamllint/actionlint; ensure only @<sha> is used.
- PR test: ensure no auto-push; verify summary and artifacts.
- Post-merge: verify auto-push on main and cache behavior.

## Risks & Mitigations
- Pin staleness → periodic refresh.
- Auto-push blocked by branch protection → switch to dedicated branch or enable write perms.

## Rollback
- Revert workflow; set auto-push: false to mitigate quickly.

## Implementation Checklist
- [ ] Add permissions blocks
- [ ] Configure git identity
- [ ] Pin and standardize actions
- [ ] Add step timeouts
- [ ] Validate via PR and schedule
