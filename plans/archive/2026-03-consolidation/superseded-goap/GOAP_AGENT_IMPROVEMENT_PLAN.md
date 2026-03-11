# GOAP Agent Improvement Plan

- **Last Updated**: 2026-03-06
- **Status**: Active
- **Related**: ADR-037, `plans/GOALS.md`, `plans/ACTIONS.md`

## Purpose

Track agent capability improvements and learning outcomes from GOAP execution cycles.

## Improvement Categories

### 1. Planning Accuracy

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Plan completion rate | ~85% | >95% | Monitoring |
| Estimation accuracy | Moderate | High | Improving |
| Blocker anticipation | Reactive | Proactive | In Progress |

### 2. Execution Efficiency

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Time to first action | Variable | <5 min | Monitoring |
| Context switches per task | 2-3 | 1-2 | Improving |
| Rework rate | ~10% | <5% | In Progress |

### 3. Quality Outcomes

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| First-pass CI success | ~70% | >90% | Improving |
| Test coverage delta | Stable | Increasing | Monitoring |
| Doc sync rate | High | Very High | Good |

## Active Improvements

### IMP-001: PR Remediation Sequencing

- **Issue**: Plans-only commits before CI check attachment cause empty rollup
- **Fix**: Add sequencing rule in GOAP docs
- **Status**: Complete
- **Learning**: Always verify CI check attachment before appending non-remediation commits

### IMP-002: Empty Rollup Detection

- **Issue**: Empty required-check rollup treated as implicit success
- **Fix**: Treat empty rollup as blocker, use GH CLI monitoring
- **Status**: Complete
- **Learning**: `gh pr view --json statusCheckRollup,mergeStateStatus` is required checkpoint

### IMP-003: Snapshot Test Baselines

- **Issue**: Missing `.snap` files cause test failures
- **Fix**: Commit baseline files alongside test code
- **Status**: Complete
- **Learning**: Snapshot tests require committed baselines

### IMP-004: Nightly Test Stability

- **Issue**: `--run-ignored all` runs timing-dependent tests that fail in CI
- **Fix**: Exclude known flaky tests from nightly workflow
- **Status**: Complete
- **Learning**: CI environment differs from local; exclude timing-sensitive tests

## Pending Improvements

### IMP-005: Dependency Drift Alerting

- **Goal**: Alert when duplicate deps exceed threshold
- **Action**: Integrate into quality-gates.sh (already present)
- **Status**: Monitoring

### IMP-006: Property Test Expansion

- **Goal**: Proptest in all 4 main crates
- **Action**: Add serialization roundtrip tests
- **Status**: Planned (ADR-033 Phase 5)

## Learning Capture Process

1. After each GOAP execution cycle, identify learnings
2. Document in `GOAP_STATE.md` under "Learning Delta"
3. Create IMP-XXX entry if actionable improvement
4. Link to relevant ADR or create new ADR if needed
5. Update `GOALS.md` and `ACTIONS.md` accordingly

## Success Metrics

- [ ] Plan completion rate >95%
- [ ] First-pass CI success >90%
- [ ] Rework rate <5%
- [ ] All IMP items resolved or documented