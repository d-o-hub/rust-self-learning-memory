# GOAP Actions Backlog

- **Last Updated**: 2026-03-05 (CI hardening + monitoring loop)
- **Related Plan**: `plans/GOAP_CSM_WORKFLOW_GAP_ADOPTION_2026-03-05.md`

## Active Actions

1. **ACT-001**
   - Goal: WG-001
   - Action: Implement and baseline-test `scripts/check-docs-integrity.sh`
   - Status: Complete

2. **ACT-002**
   - Goal: WG-002
   - Action: Implement `scripts/release-manager.sh` with dry-run default
   - Status: Complete

3. **ACT-003**
   - Goal: WG-003
   - Action: Create canonical GOAP index files under `plans/`
   - Status: Complete

4. **ACT-007**
   - Goal: WG-005
   - Action: Resolve `cargo fmt --check` drift in `tests/e2e/cli_workflows.rs`
   - Status: Complete

5. **ACT-008**
   - Goal: WG-005
   - Action: Fix `.github/workflows/changelog.yml` yamllint failures (truthy + newline)
   - Status: Complete

6. **ACT-009**
   - Goal: WG-005
   - Action: Re-run/monitor Quick Check and dependent Performance Benchmarks gate
   - Status: In Progress (checks now attaching; waiting full green on latest head)

7. **ACT-010**
   - Goal: WG-005
   - Action: Track `codecov/patch` failure and determine threshold vs upload issue
   - Status: In Progress (workspace coverage alignment shipped; validating in CI)

8. **ACT-011**
   - Goal: WG-005
   - Action: Diagnose workflow trigger/path conditions causing empty required-check rollup on plans-only head commits
   - Status: In Progress (anchor workflow added; validating behavior on follow-up heads)

10. **ACT-013**
   - Goal: WG-005
   - Action: Add `PR Check Anchor` workflow and align coverage jobs to workspace for PR diff mapping
   - Status: Complete

9. **ACT-012**
   - Goal: WG-005
   - Action: Add PR remediation sequencing rule in GOAP docs (avoid trailing plans-only commit before CI attaches)
   - Status: Complete

## Completed Actions

8. **ACT-004**
   - Goal: WG-001
   - Action: Integrate docs integrity check into `scripts/quality-gates.sh`
   - Status: Complete

9. **ACT-005**
   - Goal: WG-004
   - Action: Create `docs/architecture/context.yaml` and validation command
   - Status: Complete

10. **ACT-006**
   - Goal: WG-002
   - Action: Link release wrapper to ADR-034 release flow docs
   - Status: Complete
