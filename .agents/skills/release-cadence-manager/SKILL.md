---
name: release-cadence-manager
description: "Monitor release cadence, detect drift, and coordinate resolution using GOAP orchestrator with swarm agents. Use when release drift is detected, PRs need release-preparation labels, or automated release coordination is required."
---

# Release Cadence Manager

Monitor release cadence, detect drift, and coordinate resolution using GOAP orchestrator with swarm agents.

## Quick Reference

- **[Methodology](methodology.md)** - Core GOAP planning cycle
- **[Agents](agents.md)** - Swarm agent capabilities
- **[Patterns](patterns.md)** - Common execution patterns
- **[Integration](integration.md)** - Workflow and skill integration

## When to Use

- Release drift detected (`version_not_advanced`, `commit_limit`, `age_limit`)
- PRs need `release-preparation` label
- Automated release coordination required
- Manual intervention needed for critical drift

## NOT Appropriate For

- Normal release process (use `release-guard`)
- CI/CD failures (use `ci-fix`)
- Code quality issues (use `code-quality`)

## Core Process

```
Phase 0: ADR Discovery [MANDATORY]
├─ Read: plans/adr/ADR-*.md
├─ Identify: Relevant ADRs for release management
├─ Check: ADR-022 (GOAP), ADR-023 (CI/CD), etc.
└─ Note: Architectural constraints and decisions

Phase 1: Drift Detection [Skill]
├─ Skill(command="release-cadence-manager")
│  Mode: detect
│  → Analyze current release state
└─ Quality Gate: Drift severity determined

Phase 2: Strategy Selection [Skill + Analysis Swarm]
├─ Skill(command="analysis-swarm")
│  → Multi-perspective decision on resolution strategy
├─ Options:
│  ├─ Auto-label PR (if valid release PR)
│  ├─ Create release-preparation PR
│  ├─ Notify maintainer for manual intervention
│  └─ Skip (if false positive)
└─ Quality Gate: Strategy approved

Phase 3: Execution [Swarm Agents]
├─ Agent 1: Drift Detector Agent
│  └─ Monitors release cadence and detects drift
├─ Agent 2: Label Manager Agent
│  └─ Manages the `release-preparation` label
├─ Agent 3: Release Coordinator Agent
│  └─ Coordinates the release process
└─ Agent 4: Validation Agent
   └─ Validates that all steps are completed correctly

Phase 4: Validation [Skills]
├─ Skill(command="code-quality")
├─ Skill(command="architecture-validation")
└─ Skill(command="release-guard")
└─ Quality Gate: Release process validated

Phase 5: Update Plans & Documentation [Documentation]
├─ Update: AGENTS.md with new workflow
├─ Create/Update: Skill documentation
└─ Document: Lessons learned and patterns discovered
```

## Drift Detection

### Severity Levels

| Severity | Condition | Action |
|----------|-----------|--------|
| `clean` | Within cadence | No action |
| `warning` | Approaching limits | Notify maintainer |
| `critical` | Exceeded limits | Automated resolution |

### Critical Conditions

- `version_not_advanced`: Version in Cargo.toml matches latest tag
- `tag_not_ancestor`: Latest tag is not an ancestor of HEAD
- `invalid_next_version`: Version doesn't follow semver rules
- `commit_limit`: Unreleased commits >= 30
- `age_limit`: Release age >= 14 days
- `no_release_tag`: No release tags found

## Swarm Agents

### Agent 1: Drift Detector Agent
- **Purpose**: Monitor release cadence and detect drift
- **Triggers**: PR events, push to main, scheduled checks
- **Actions**:
  - Run `check-release-drift.sh`
  - Analyze drift severity
  - Determine if resolution is needed

### Agent 2: Label Manager Agent
- **Purpose**: Manage the `release-preparation` label
- **Triggers**: Drift detection, PR creation
- **Actions**:
  - Add label to valid release PRs
  - Remove label from non-release PRs
  - Validate label usage

### Agent 3: Release Coordinator Agent
- **Purpose**: Coordinate the release process
- **Triggers**: Drift detection, label addition
- **Actions**:
  - Create release-preparation PR if needed
  - Update CHANGELOG, ROADMAP, STATUS
  - Coordinate with release-guard skill

### Agent 4: Validation Agent
- **Purpose**: Validate that all steps are completed correctly
- **Triggers**: After each phase
- **Actions**:
  - Validate release state
  - Check CI/CD status
  - Ensure documentation is updated

## Integration

### With Existing Skills

| Skill | Integration |
|-------|-------------|
| `release-guard` | Coordinate release process |
| `analysis-swarm` | Multi-perspective strategy selection |
| `goap-agent` | Orchestrator pattern |
| `agent-coordination` | Swarm agent management |

### With Workflows

| Workflow | Integration |
|----------|-------------|
| `release-drift.yml` | Automated drift detection |
| `release.yml` | Release process |
| `pr-readiness` | PR validation |

## Commands Reference

```bash
# Primary CLI (this skill)
./scripts/release-cadence-manager.sh detect
./scripts/release-cadence-manager.sh resolve --pr {n}
./scripts/release-cadence-manager.sh validate
./scripts/release-cadence-manager.sh status
./scripts/release-cadence-manager.sh help

# Low-level drift check (used by CI)
./scripts/check-release-drift.sh

# Add / remove escape-hatch label
gh pr edit {n} --add-label "release-preparation"
gh pr edit {n} --remove-label "release-preparation"

# Check PR status
gh pr view {n} --json labels,state,headRefName

# Ship (use release-guard skill / release-manager)
./scripts/release-manager.sh status
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| False positive drift | Check if version was recently bumped |
| Label not adding | Verify permissions and label exists |
| PR not detecting label | Check workflow triggers |
| Release process blocked | Use `release-guard` skill |

### Debug Commands

```bash
# Check drift status
./scripts/check-release-drift.sh

# Check PR labels
gh pr view {n} --json labels

# Check workflow runs
gh run list --workflow=release-drift.yml

# Check release status
./scripts/release-manager.sh status
```

## Progressive Disclosure

- CI reference: [ci-reference.md](../release-guard/ci-reference.md)
- Workflow definition: `.github/workflows/release-drift.yml`
- Drift detection: `./scripts/check-release-drift.sh`
- Release process: `.agents/skills/release-guard/SKILL.md`
