# Execution Plan: Improve Security Workflow

## Objective
- Upload Gitleaks results as SARIF; pin actions; minimize permissions; set policy for cargo-audit placement.

## Proposed Changes
1) SARIF upload for Gitleaks
- Generate SARIF via gitleaks-action and upload via codeql-action/upload-sarif.
- Job permissions: security-events: write (only for this job).

2) Pin actions to SHAs
- checkout, gitleaks-action, upload-sarif, dependency-review, rust-toolchain, rust-cache, upload-artifact.
- Add inline comments indicating source version for each pinned SHA.

3) Minimize permissions
- Workflow-level: contents: read.
- Job-level: grant only what is needed (security-events: write for SARIF job).

4) cargo-audit policy
- Keep cargo-audit in Security workflow (recommended), or move to CI as an alternative; document chosen policy.

5) Minor cleanups
- Drop unused envs (e.g., GITLEAKS_LICENSE if unused).
- Ensure timeouts and keep existing concurrency.

## Validation Plan
- actionlint/yamllint; open PR with a benign test secret to confirm SARIF appears in Security > Code scanning.

## Risks & Mitigations
- Incorrect SHA pin → verify against upstream releases; maintain refresh schedule.
- Insufficient permissions → ensure security-events: write for SARIF job.

## Rollback
- Revert workflow; comment out SARIF step and remove security-events permission if needed.

## Implementation Checklist
- [ ] Add SARIF upload steps
- [ ] Pin actions with comments
- [ ] Tighten permissions
- [ ] Validate via PR and schedule
