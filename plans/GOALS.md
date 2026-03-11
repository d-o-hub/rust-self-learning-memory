# GOAP Goals Index

- **Last Updated**: 2026-03-11 (post-v0.1.17 revalidation)
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
   - Status: Complete (PR #334 merged 2026-03-06)

6. **WG-006**: Merge Dependabot dependency updates
   - Priority: P1
   - Owner: cicd-engineer
   - Target: merge PRs #344, #345, #346 once CI passes
   - Status: Partial (PR #344, #346 merged; #345 blocked by breaking changes)

7. **WG-007**: Fix rust-major breaking changes (PR #345)
   - Priority: P2
   - Owner: rust-expert
   - Target: resolve test/clippy failures from dependency update
   - Status: Complete (PR #345 merged with redb 3.x + rand 0.10 fixes)

## Next Goals (v0.1.18 Sprint)

8. **WG-008**: Triage 121 ignored tests
   - Priority: P0
   - Owner: test-runner + debug-troubleshoot
   - Target: reduce ignored tests to ≤30 (remove, fix, or document with tracking)
   - Status: Pending

9. **WG-009**: Resolve batch MCP tool state
   - Priority: P0
   - Owner: memory-mcp
   - Target: remove unreachable batch schemas or re-enable handlers
   - Status: ✅ Complete (PR #357 merged 2026-03-11)

10. **WG-010**: Error handling reduction (memory-core)
    - Priority: P1
    - Owner: code-quality
    - Target: reduce unwrap/expect in memory-core from 165 to ≤50
    - Status: Pending

11. **WG-011**: Dependency deduplication
    - Priority: P1
    - Owner: code-quality
    - Target: reduce duplicate dep roots from 134 to <100
    - Status: Pending
