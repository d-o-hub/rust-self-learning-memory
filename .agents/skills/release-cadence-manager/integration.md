# Release Cadence Manager - Integration

## Integration with Existing Skills

### release-guard Skill

**Purpose**: Coordinate release process

**Integration Points**:
- Use `release-guard` for actual release execution
- `release-cadence-manager` handles drift detection and resolution
- `release-guard` handles version bumping and tagging

**Workflow**:
```
1. release-cadence-manager detects drift
2. release-cadence-manager coordinates resolution
3. release-guard executes release process
4. release-cadence-manager validates completion
```

### analysis-swarm Skill

**Purpose**: Multi-perspective strategy selection

**Integration Points**:
- Use `analysis-swarm` for complex drift resolution decisions
- Multiple personas analyze drift situation
- Consensus on resolution strategy

**Workflow**:
```
1. release-cadence-manager detects drift
2. analysis-swarm analyzes drift situation
3. Multiple personas propose resolution strategies
4. Consensus on best strategy
5. release-cadence-manager executes strategy
```

### goap-agent Skill

**Purpose**: Orchestrator pattern

**Integration Points**:
- Use `goap-agent` for complex multi-step tasks
- Decompose drift resolution into atomic tasks
- Coordinate multiple agents

**Workflow**:
```
1. goap-agent analyzes drift situation
2. goap-agent decomposes into atomic tasks
3. goap-agent coordinates multiple agents
4. goap-agent validates completion
```

### agent-coordination Skill

**Purpose**: Swarm agent management

**Integration Points**:
- Use `agent-coordination` for managing swarm agents
- Parallel execution of drift detection and resolution
- Quality gates between phases

**Workflow**:
```
1. agent-coordination initializes swarm agents
2. agent-coordination executes agents in parallel
3. agent-coordination synchronizes results
4. agent-coordination validates completion
```

## Integration with Workflows

### release-drift.yml Workflow

**Purpose**: Automated drift detection

**Integration Points**:
- `release-cadence-manager` provides drift detection logic
- Workflow triggers `release-cadence-manager` skill
- Workflow applies resolution strategies

**Current State**:
```yaml
- name: Enforce release cadence
  if: >-
    github.event_name == 'pull_request' &&
    steps.check.outputs.severity == 'critical' &&
    !contains(github.event.pull_request.labels.*.name, 'release-preparation')
  run: |
    echo "Release cadence exceeded or version state is invalid: ${{ steps.check.outputs.reason }}" >&2
    echo "A maintainer may label the release PR 'release-preparation' to break a cadence deadlock." >&2
    exit 1
```

**Enhanced State**:
```yaml
- name: Enforce release cadence
  if: >-
    github.event_name == 'pull_request' &&
    steps.check.outputs.severity == 'critical'
  run: |
    # Use release-cadence-manager skill for resolution
    ./scripts/release-cadence-manager.sh resolve \
      --severity "${{ steps.check.outputs.severity }}" \
      --reason "${{ steps.check.outputs.reason }}" \
      --pr "${{ github.event.pull_request.number }}"
```

### release.yml Workflow

**Purpose**: Release process

**Integration Points**:
- `release-cadence-manager` validates release state
- `release-cadence-manager` coordinates with `release-guard`
- `release-cadence-manager` ensures documentation is updated

### pr-readiness Workflow

**Purpose**: PR validation

**Integration Points**:
- `release-cadence-manager` checks drift status
- `release-cadence-manager` validates label usage
- `release-cadence-manager` ensures release coordination

## Integration with AGENTS.md

### Quick Reference Update

**Add**:
```markdown
- **Release Cadence**: `release-cadence-manager` | `./scripts/release-cadence-manager.sh`
```

### Skill + CLI Pattern Update

**Add**:
```markdown
| Release Cadence | `release-cadence-manager` | `./scripts/release-cadence-manager.sh` |
```

### Release Process Update

**Add**:
```markdown
## Release Cadence Management (MANDATORY when drift detected)

**Skill:** `.agents/skills/release-cadence-manager/SKILL.md`
**CLI:** `./scripts/release-cadence-manager.sh`

### When to Use
- Release drift detected (`version_not_advanced`, `commit_limit`, `age_limit`)
- PRs need `release-preparation` label
- Automated release coordination required

### Workflow
1. Detect drift: `./scripts/release-cadence-manager.sh detect`
2. Resolve drift: `./scripts/release-cadence-manager.sh resolve --pr {n}`
3. Validate: `./scripts/release-cadence-manager.sh validate`

### Integration
- Works with `release-guard` for release execution
- Works with `analysis-swarm` for strategy selection
- Works with `goap-agent` for orchestration
```

## Integration Scripts

### release-cadence-manager.sh

**Purpose**: CLI for release-cadence-manager skill

**Commands**:
```bash
# Detect drift
./scripts/release-cadence-manager.sh detect

# Resolve drift for PR
./scripts/release-cadence-manager.sh resolve --pr {n}

# Validate resolution
./scripts/release-cadence-manager.sh validate

# Show status
./scripts/release-cadence-manager.sh status
```

**Implementation**:
```bash
#!/usr/bin/env bash
# Release Cadence Manager CLI

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="${SCRIPT_DIR}/.."

case "${1:-help}" in
  detect)
    # Run drift detection
    "$PROJECT_ROOT/scripts/check-release-drift.sh"
    ;;
  resolve)
    # Resolve drift for PR
    PR_NUMBER="${2:?PR number required}"
    gh pr edit "$PR_NUMBER" --add-label "release-preparation"
    ;;
  validate)
    # Validate resolution
    "$PROJECT_ROOT/scripts/verify-release-state.sh" --check-unreleased
    ;;
  status)
    # Show status
    "$PROJECT_ROOT/scripts/release-manager.sh" status
    ;;
  help|*)
    echo "Usage: $0 {detect|resolve|validate|status}"
    echo ""
    echo "Commands:"
    echo "  detect    Detect release drift"
    echo "  resolve   Resolve drift for PR"
    echo "  validate  Validate resolution"
    echo "  status    Show release status"
    ;;
esac
```

## Testing Strategy

### Unit Tests

- Test each integration point independently
- Mock external dependencies
- Validate error handling

### Integration Tests

- Test skill integration
- Test workflow integration
- Test AGENTS.md integration

### End-to-End Tests

- Test complete release cadence management
- Validate with real PRs
- Check workflow integration

## Documentation

### Skill Documentation

- Update `release-cadence-manager/SKILL.md`
- Update supporting files (methodology.md, agents.md, patterns.md, integration.md)

### AGENTS.md Documentation

- Update Quick Reference
- Update Skill + CLI Pattern
- Update Release Process

### Workflow Documentation

- Update workflow comments
- Update workflow documentation
- Update troubleshooting guides

## Monitoring and Observability

### Metrics

- Drift detection accuracy
- Resolution success rate
- Agent execution time
- Quality gate pass rate

### Logging

- Log drift detection events
- Log resolution actions
- Log agent execution
- Log quality gate results

### Alerting

- Alert on drift detection failures
- Alert on resolution failures
- Alert on agent failures
- Alert on quality gate failures
