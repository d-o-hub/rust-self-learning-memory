# Release Cadence Manager - Agents

## Swarm Agent Capabilities

### Agent 1: Drift Detector Agent

**Purpose**: Monitor release cadence and detect drift

**Type**: `feature-implementer`

**Capabilities**:
- Run `check-release-drift.sh`
- Analyze drift severity
- Determine if resolution is needed
- Report drift status

**Triggers**:
- PR events (opened, synchronize, reopened, labeled, unlabeled)
- Push to main branch
- Scheduled checks (cron)
- Manual workflow dispatch

**Actions**:
```bash
# Run drift detection
./scripts/check-release-drift.sh

# Analyze output
# severity: clean | warning | critical
# reason: within_cadence | commit_warning | age_warning | version_not_advanced | etc.

# Determine action
if [[ "$severity" == "critical" ]]; then
  # Trigger resolution
  echo "Critical drift detected: $reason"
fi
```

**Quality Gate**: Drift severity determined accurately

---

### Agent 2: Label Manager Agent

**Purpose**: Manage the `release-preparation` label

**Type**: `feature-implementer`

**Capabilities**:
- Add label to valid release PRs
- Remove label from non-release PRs
- Validate label usage
- Check PR eligibility

**Triggers**:
- Drift detection (critical severity)
- PR creation
- Manual request

**Actions**:
```bash
# Check if PR is eligible for release-preparation label
gh pr view {n} --json labels,state,headRefOid

# Add label if eligible
gh pr edit {n} --add-label "release-preparation"

# Remove label if not eligible
gh pr edit {n} --remove-label "release-preparation"

# Validate label exists
gh label list --search "release-preparation"
```

**Quality Gate**: Label correctly applied to eligible PRs

---

### Agent 3: Release Coordinator Agent

**Purpose**: Coordinate the release process

**Type**: `feature-implementer`

**Capabilities**:
- Create release-preparation PR if needed
- Update CHANGELOG, ROADMAP, STATUS
- Coordinate with release-guard skill
- Validate release state

**Triggers**:
- Drift detection (critical severity)
- Label addition
- Manual request

**Actions**:
```bash
# Check release state
./scripts/release-manager.sh status

# Update documentation
# - CHANGELOG.md
# - plans/ROADMAPS/ROADMAP_ACTIVE.md
# - plans/STATUS/CURRENT.md

# Create release-preparation PR
# - Create branch: release/vX.Y.Z
# - Update version in Cargo.toml
# - Update documentation
# - Create PR with release-preparation label

# Coordinate with release-guard
# - Ensure main CI is green
# - Run release-manager.sh ship --execute
```

**Quality Gate**: Release process coordinated correctly

---

### Agent 4: Validation Agent

**Purpose**: Validate that all steps are completed correctly

**Type**: `feature-implementer`

**Capabilities**:
- Validate release state
- Check CI/CD status
- Ensure documentation is updated
- Verify integration with existing workflows

**Triggers**:
- After each phase
- Manual request

**Actions**:
```bash
# Validate release state
./scripts/verify-release-state.sh --check-unreleased

# Check CI/CD status
gh run list --branch main --commit $(git rev-parse origin/main)

# Verify documentation
# - CHANGELOG.md has version section
# - ROADMAP_ACTIVE.md has Released Version
# - STATUS/CURRENT.md has Released Version

# Check workflow integration
# - release-drift.yml runs correctly
# - Label management works
# - Release process completes
```

**Quality Gate**: All steps validated and successful

---

## Agent Coordination

### Parallel Execution

```
Drift Detector Agent ─┐
Label Manager Agent ──┼─→ Release Coordinator Agent → Validation Agent
Release Coordinator ──┘
```

### Sequential Execution

```
Drift Detector Agent → Label Manager Agent → Release Coordinator Agent → Validation Agent
```

### Quality Gates Between Agents

1. **After Drift Detector**: Drift severity determined
2. **After Label Manager**: Label correctly applied
3. **After Release Coordinator**: Release process coordinated
4. **After Validation**: All steps validated

## Agent Communication

### Messages

| From | To | Message |
|------|----|---------|
| Drift Detector | Label Manager | "Critical drift detected: version_not_advanced" |
| Label Manager | Release Coordinator | "Label added to PR #870" |
| Release Coordinator | Validation Agent | "Release process coordinated" |
| Validation Agent | All | "All steps validated successfully" |

### State Sharing

| Agent | State | Description |
|-------|-------|-------------|
| Drift Detector | `drift_status` | Current drift severity and reason |
| Label Manager | `label_status` | Label application status |
| Release Coordinator | `release_status` | Release process status |
| Validation Agent | `validation_status` | Validation results |

## Error Handling

### Agent Failures

| Agent | Failure | Recovery |
|-------|---------|----------|
| Drift Detector | Script fails | Manual drift check |
| Label Manager | Label not found | Create label |
| Release Coordinator | Release blocked | Manual intervention |
| Validation Agent | Validation fails | Re-run validation |

### Fallback Strategies

| Scenario | Strategy |
|----------|----------|
| All agents fail | Manual label addition |
| Partial failure | Continue with remaining agents |
| Validation fails | Re-run failed validation |
| Integration fails | Use existing workflow |

## Performance Considerations

### Execution Time

| Agent | Expected Time | Timeout |
|-------|---------------|---------|
| Drift Detector | ~10s | 30s |
| Label Manager | ~5s | 15s |
| Release Coordinator | ~30s | 60s |
| Validation Agent | ~20s | 45s |

### Resource Usage

| Agent | CPU | Memory | Network |
|-------|-----|--------|---------|
| Drift Detector | Low | Low | Low |
| Label Manager | Low | Low | Medium |
| Release Coordinator | Medium | Medium | High |
| Validation Agent | Medium | Medium | Medium |

## Testing

### Unit Tests

- Test each agent independently
- Mock external dependencies
- Validate error handling

### Integration Tests

- Test agent coordination
- Validate end-to-end workflow
- Check quality gates

### End-to-End Tests

- Test complete release cadence management
- Validate with real PRs
- Check workflow integration
