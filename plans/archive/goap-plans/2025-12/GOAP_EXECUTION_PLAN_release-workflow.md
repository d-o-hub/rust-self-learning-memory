# Execution Plan: Harden and Improve Release Workflow

## Objective
- Security: Pin actions to commit SHAs; apply least-privilege permissions per job; optionally add provenance.
- Consistency: Align artifact actions and cache keys; stabilize sccache/rust-cache keys.
- Reliability: Ensure Windows bash handling and maintain timeouts.

## Proposed Changes
1) Security hardening
- Workflow-level permissions: contents: read.
- build-release job permissions: contents: read; actions: write (for cache save if used).
- create-release job permissions: contents: write; actions: read; optionally id-token: write for provenance.
- Pin actions: checkout, rust-toolchain, install-action, cache, rust-cache, upload/download-artifact, softprops/action-gh-release.
- Optional: add step-security/harden-runner with egress-policy: audit.

2) Consistency
- Use same major for upload/download-artifact; pin to SHAs.
- Stabilize cache keys and include toolchain/rustc hash in sccache key; add restore-keys prefix.

3) Reliability
- Explicit shell: bash and set -euo pipefail in packaging steps.
- Keep timeouts as currently set; add retry for asset upload if needed.

4) Optional: Provenance and SBOM
- actions/attest-build-provenance with id-token: write and subject-path of built artifacts.
- anchore/syft-action to produce SBOM; attach to release.

## Validation Plan
- actionlint/yamllint; verify each pinned SHA corresponds to intended tag.
- Dry-run with workflow_dispatch using draft prerelease tag; confirm all assets uploaded.

## Risks & Mitigations
- Pinned SHAs can stale → quarterly refresh.
- Permissions too strict → adjust per job if failures occur; start with audit mode on harden-runner.

## Rollback
- Revert workflow; temporarily switch pins back to tags to unstick releases.

## Implementation Checklist
- [ ] Add minimal permissions per job
- [ ] Pin all actions to SHAs
- [ ] Stabilize cache keys
- [ ] Optional provenance/SBOM steps
- [ ] Validate and dry-run
