# GOAP Goals Index

- **Last Updated**: 2026-03-05 (GH monitoring snapshot)
- **Source ADR**: ADR-037
- **Status**: Active

## Current Goals

1. **WG-001**: Add docs integrity automation
   - Priority: P1
   - Owner: code-quality + documentation
   - Target: add `scripts/check-docs-integrity.sh` and wire into quality gates
   - Status: Complete

2. **WG-002**: Add release operations wrapper
   - Priority: P1
   - Owner: github-release-best-practices + release-guard
   - Target: add `scripts/release-manager.sh`
   - Status: Complete

3. **WG-003**: Introduce lightweight GOAP state index
   - Priority: P2
   - Owner: goap-agent
   - Target: establish `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`
   - Status: Complete

4. **WG-004**: Add machine-readable architecture context contract
   - Priority: P2
   - Owner: codebase-analyzer + yaml-validator
   - Target: add `docs/architecture/context.yaml` and validation hook
   - Status: Complete

5. **WG-005**: Stabilize PR #334 checks after ADR-037 rollout
   - Priority: P1
   - Owner: github-workflows + code-quality
   - Target: clear format/yaml/quick-check chain failures on PR validation
   - Status: In Progress
