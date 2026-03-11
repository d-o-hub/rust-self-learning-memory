# GOAP Agent Quality Gates

- **Last Updated**: 2026-03-06
- **Status**: Active
- **Related**: `scripts/quality-gates.sh`, ADR-037

## Purpose

Define quality gates specific to GOAP agent operations and planning cycles.

## GOAP-Specific Gates

### Gate 1: Plan Completeness

Before execution, verify:

- [ ] Objective clearly stated
- [ ] Success criteria defined
- [ ] Risks identified with mitigations
- [ ] Rollback plan documented
- [ ] ADR cross-references included

**Verification**: Template section check in `quality-gates.sh`

### Gate 2: State Consistency

Before committing:

- [ ] `GOAP_STATE.md` reflects current phase
- [ ] `GOALS.md` has relevant goal entries
- [ ] `ACTIONS.md` tracks active actions
- [ ] Blockers documented with status

**Verification**: Manual review during commit

### Gate 3: Execution Validation

After execution:

- [ ] All actions completed or documented
- [ ] CI checks passing (not just queued)
- [ ] Learning delta captured
- [ ] State files updated

**Verification**: `gh pr view --json statusCheckRollup,mergeStateStatus`

### Gate 4: Documentation Hygiene

- [ ] Plan files under 500 lines
- [ ] No non-permanent docs in root
- [ ] ADR references valid
- [ ] Cross-links functional

**Verification**: `scripts/check-docs-integrity.sh`

## Blocking vs Non-Blocking

| Gate | Default | Override |
|------|---------|----------|
| Plan Completeness | Non-blocking | — |
| State Consistency | Non-blocking | — |
| Execution Validation | Blocking | Manual override |
| Documentation Hygiene | Non-blocking | `QUALITY_GATE_SKIP_DOCS=true` |

## CI Integration

GOAP gates run as part of `scripts/quality-gates.sh`:

```bash
# Run all gates including GOAP checks
./scripts/quality-gates.sh

# Skip GOAP checks
QUALITY_GATE_SKIP_GOAP=true ./scripts/quality-gates.sh

# Skip docs integrity
QUALITY_GATE_SKIP_DOCS=true ./scripts/quality-gates.sh
```

## Gate Failure Response

1. **Plan Completeness**: Update plan with missing sections
2. **State Consistency**: Sync state files before commit
3. **Execution Validation**: Fix CI failures, verify check attachment
4. **Documentation Hygiene**: Move docs to `plans/`, fix links

## Metrics Tracking

| Metric | Collection | Target |
|--------|------------|--------|
| Gate pass rate | Per execution | >95% |
| Time to resolve | Per failure | <30 min |
| Repeat failures | Per week | 0 |

## Related Files

- `scripts/quality-gates.sh` - Gate runner
- `plans/GOAP_STATE.md` - Current state
- `plans/GOALS.md` - Goal index
- `plans/ACTIONS.md` - Action backlog