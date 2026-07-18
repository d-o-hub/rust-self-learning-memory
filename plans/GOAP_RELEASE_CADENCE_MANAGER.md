# GOAP Release Cadence Manager

## Task Analysis

**Primary Goal**: Automate release cadence management using GOAP orchestrator with swarm agents to eliminate manual intervention when release drift occurs.

**Constraints**:
- Time: Normal (can be implemented incrementally)
- Resources: Existing skills, agents, and workflows
- Dependencies: Release-drift workflow, release-guard skill, GOAP agent system

**Complexity Level**: Complex (4+ agents, mixed execution modes)

**Quality Requirements**:
- Testing: Unit tests for new skills, integration tests for workflow
- Standards: AGENTS.md compliance, formatting, linting
- Documentation: Updated AGENTS.md, new skill documentation
- Performance: Minimal impact on CI/CD pipeline

## Problem Analysis

### Current State
1. **Release Drift Workflow**: `.github/workflows/release-drift.yml` detects when version in `Cargo.toml` hasn't been advanced
2. **Manual Workaround**: When `severity == 'critical'`, workflow fails unless PR has `release-preparation` label
3. **No Automation**: Currently requires manual label addition via `gh pr edit`

### Root Cause
- The workflow enforces release cadence rules but doesn't provide an automated path to resolve drift
- No integration between drift detection and release preparation processes
- Manual intervention required for critical drift situations

## Solution Design

### GOAP Orchestrator Pattern

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

## Implementation Plan

### Step 1: Create New Skill: `release-cadence-manager`

**Location**: `.agents/skills/release-cadence-manager/SKILL.md`

**Purpose**: Monitor release cadence, detect drift, and coordinate resolution

**Key Features**:
- Analyze current release state
- Detect drift severity
- Coordinate with other skills for resolution
- Provide automated resolution paths

### Step 2: Implement Swarm Agents

#### Agent 1: Drift Detector Agent
- **Purpose**: Monitor release cadence and detect drift
- **Triggers**: PR events, push to main, scheduled checks
- **Actions**:
  - Run `check-release-drift.sh`
  - Analyze drift severity
  - Determine if resolution is needed

#### Agent 2: Label Manager Agent
- **Purpose**: Manage the `release-preparation` label
- **Triggers**: Drift detection, PR creation
- **Actions**:
  - Add label to valid release PRs
  - Remove label from non-release PRs
  - Validate label usage

#### Agent 3: Release Coordinator Agent
- **Purpose**: Coordinate the release process
- **Triggers**: Drift detection, label addition
- **Actions**:
  - Create release-preparation PR if needed
  - Update CHANGELOG, ROADMAP, STATUS
  - Coordinate with release-guard skill

#### Agent 4: Validation Agent
- **Purpose**: Validate that all steps are completed correctly
- **Triggers**: After each phase
- **Actions**:
  - Validate release state
  - Check CI/CD status
  - Ensure documentation is updated

### Step 3: Update Workflow

**File**: `.github/workflows/release-drift.yml`

**Changes**:
1. Add automated resolution path
2. Integrate with new skill
3. Add swarm agent coordination
4. Maintain manual override option

### Step 4: Update AGENTS.md

**Additions**:
1. New skill documentation
2. Swarm agent workflow
3. Integration with existing release process
4. Troubleshooting guide

## Success Criteria

- [ ] Automated drift detection and resolution
- [ ] Reduced manual intervention
- [ ] Maintained release quality standards
- [ ] Integration with existing workflows
- [ ] Comprehensive documentation

## Contingency Plans

### If Swarm Agents Fail
- Fall back to manual label addition
- Notify maintainer for intervention
- Log failure for analysis

### If Integration Issues
- Use existing workflow as-is
- Document integration issues
- Plan incremental fixes

### If Performance Issues
- Optimize agent execution
- Reduce coordination overhead
- Simplify validation steps

## Related ADRs

- ADR-022: GOAP Agent System
- ADR-023: CI/CD Pipeline
- ADR-024: MCP Server Architecture

## Next Steps

1. Create `release-cadence-manager` skill
2. Implement swarm agents
3. Update workflow
4. Update AGENTS.md
5. Test and validate
6. Document lessons learned
