# Execution Plan: Harden and Fix Quick Check Workflow

## Objective
- Security: Pin actions to commit SHAs, set least-privilege permissions, disable credential persistence in checkout.
- Performance: Retain rust-cache, optionally scope runs via path filters.
- Consistency: Standardize action versions across the repository.

## Scope
- Primary: .github/workflows/quick-check.yml
- Secondary (consistency sweep): ci.yml, security.yml, yaml-lint.yml

## Proposed Changes (diff-like bullets)
- Triggers and concurrency (optional scoping for PRs)
  - Add PR path filters to reduce noise:
    - on.pull_request.paths: ["**/*.rs", "Cargo.*", ".github/workflows/quick-check.yml"]
- Permissions (explicit, least-privilege)
  - Set top-level permissions:
    - permissions:
      contents: read
  - Avoid unnecessary write scopes.
- Checkout hardening
  - actions/checkout@<pinned-SHA-for-v4>
  - with: persist-credentials: false, fetch-depth: 1
- Pin and standardize action versions
  - dtolnay/rust-toolchain@<pinned-SHA> with: toolchain: stable; components: rustfmt, clippy
  - Swatinem/rust-cache@<pinned-SHA-for-v2.8.2>
- Keep clippy/fmt commands as-is
- Keep timeout-minutes as currently configured (or set 10–15 mins)

## Repository-wide consistency (follow-up/same PR separate commit)
- Pin all actions to SHAs across workflows: checkout, rust-toolchain, rust-cache, upload-artifact, codecov, reviewdog, setup-python, dependency-review, gitleaks, taiki-e/install-action.
- Explicit minimal permissions per workflow/job; elevate only where needed (e.g., security SARIF uploads).
- Optional: add concurrency blocks consistently.
- Optional: add PR path filters for faster feedback on lint-only changes.

## Notes on Pinned SHAs
- Use immutable commit SHAs with an inline comment of the upstream version tag for traceability.
- Verify pins via actionlint and a manual workflow_dispatch dry run.

## Validation Steps
- actionlint and yamllint locally or via existing workflows.
- Open a PR changing only quick-check.yml to validate triggers and pass.
- Optional: use act for a Linux dry-run.

## Risks & Mitigations
- Pins can go stale → schedule quarterly updates (Dependabot/Renovate for actions).
- Overly strict permissions → document and adjust per-job if failures indicate insufficiency.

## Rollback Plan
- Revert the workflow commit or temporarily unpin problematic actions back to version tags while investigating.

## Implementation Checklist
- [ ] Optional PR path filters
- [ ] Minimal permissions
- [ ] Checkout hardened
- [ ] Pin: checkout, rust-toolchain, rust-cache
- [ ] Cache save-if for main/develop
- [ ] Validate with actionlint/yamllint and PR run
