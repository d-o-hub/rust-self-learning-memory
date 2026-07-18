# Release Cadence Manager - Methodology

## GOAP Planning Cycle

```
1. ANALYZE → Understand goals, constraints, resources
2. DECOMPOSE → Break into atomic tasks with dependencies
3. STRATEGIZE → Choose execution pattern
4. COORDINATE → Assign tasks to specialized agents
5. EXECUTE → Run with monitoring and quality gates
6. SYNTHESIZE → Aggregate results and validate success
```

## Phase 1: Task Analysis

### Initial Assessment Template

```markdown
## Task Analysis

**Primary Goal**: Automate release cadence management to eliminate manual intervention

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
```

### Context Gathering

1. **Codebase Understanding**: Analyze release-drift workflow and check-release-drift.sh
2. **Past Patterns**: Check similar tasks in memory
3. **Available Resources**: Identify agent capabilities
4. **Current State**: Understand starting conditions

### ADR Discovery

**CRITICAL**: Before planning, check for relevant ADRs in `plans/adr/`:

```
1. List all ADRs: ls plans/adr/ADR-*.md
2. Identify relevant ADRs by topic/domain
3. Read relevant ADRs for architectural constraints
4. Note decisions that affect current task
```

**ADR Integration Checklist**:
- [ ] Searched `plans/adr/` for topic-related ADRs
- [ ] Read relevant ADR(s) for architectural decisions
- [ ] Noted constraints from ADR decisions
- [ ] Verified ADR status (Accepted/Implemented vs Deprecated)
- [ ] Incorporated ADR guidance into task decomposition

**Common ADR Topics**:
- CI/CD: ADR-023, ADR-029
- GOAP: ADR-022
- Release: ADR-024

**See**: `plans/adr/` for complete ADR inventory

## Phase 2: Task Decomposition

Use the **goap-agent** skill DECOMPOSE phase to break down goals.

### Key Principles
- **Atomic**: Each task is indivisible and clear
- **Testable**: Can verify completion
- **Independent**: Minimize dependencies
- **Assigned**: Each task maps to agent capability

### Task Decomposition

```
Task: Automate release cadence management

Phase 0: ADR Discovery [MANDATORY]
├─ Read: plans/adr/ADR-*.md
├─ Identify: Relevant ADRs for release management
├─ Check: ADR-022 (GOAP), ADR-023 (CI/CD), etc.
└─ Note: Architectural constraints and decisions
Quality Gate: Relevant ADRs identified and reviewed

Phase 1: Create New Skill [Sequential]
├─ Create: .agents/skills/release-cadence-manager/SKILL.md
├─ Create: .agents/skills/release-cadence-manager/methodology.md
├─ Create: .agents/skills/release-cadence-manager/agents.md
├─ Create: .agents/skills/release-cadence-manager/patterns.md
└─ Create: .agents/skills/release-cadence-manager/integration.md
Quality Gate: Skill created with all supporting files

Phase 2: Implement Swarm Agents [Parallel]
├─ Agent 1: Drift Detector Agent
├─ Agent 2: Label Manager Agent
├─ Agent 3: Release Coordinator Agent
└─ Agent 4: Validation Agent
Quality Gate: All agents implemented and tested

Phase 3: Update Workflow [Sequential]
├─ Update: .github/workflows/release-drift.yml
├─ Add: Automated resolution path
├─ Integrate: New skill
└─ Maintain: Manual override option
Quality Gate: Workflow updated and tested

Phase 4: Update Documentation [Sequential]
├─ Update: AGENTS.md with new workflow
├─ Create: Skill documentation
└─ Document: Lessons learned and patterns discovered
Quality Gate: Documentation updated and reviewed

Phase 5: Test and Validate [Sequential]
├─ Test: Skill functionality
├─ Test: Swarm agents
├─ Test: Workflow integration
└─ Validate: End-to-end process
Quality Gate: All tests passing
```

## Phase 3: Strategy Selection

| Strategy | When to Use | Speed | Complexity |
|----------|-------------|-------|------------|
| **Parallel** | Independent tasks, time-critical | Nx | High |
| **Sequential** | Dependent tasks, order matters | 1x | Low |
| **Swarm** | Many similar tasks | ~Nx | Medium |
| **Hybrid** | Mixed requirements | 2-4x | Very High |
| **Iterative** | Progressive refinement | Varies | Medium |

### Decision Tree

```
Needs iterative refinement?
  ├─ Yes → ITERATIVE
  └─ No → Is time critical?
      ├─ Yes → Can tasks run in parallel?
      │   ├─ Yes → PARALLEL
      │   └─ No → SEQUENTIAL
      └─ No → Are tasks similar?
          ├─ Yes → SWARM
          ├─ No → HYBRID
          └─ Simple → SEQUENTIAL
```

**Selected Strategy**: **Hybrid** (combines parallel swarm agents with sequential validation)

## Phase 4: Agent Assignment

See **[agents.md](agents.md)** for capability matrix and **[patterns.md](patterns.md)** for common patterns.

## Phase 5: Execution Planning

### Template

```markdown
## Execution Plan: Release Cadence Manager

### Overview
- Strategy: Hybrid (Parallel Swarm + Sequential Validation)
- Total Tasks: 5 phases
- Quality Gates: 5 checkpoints

### Phase 0: ADR Discovery
**Tasks**:
- Read relevant ADRs (Agent: general)
- Note architectural constraints
**Quality Gate**: ADRs identified and reviewed

### Phase 1: Create New Skill
**Tasks**:
- Create SKILL.md (Agent: junior-coder)
- Create supporting files (Agent: junior-coder)
**Quality Gate**: Skill created with all files

### Phase 2: Implement Swarm Agents
**Tasks**:
- Drift Detector Agent (Agent: feature-implementer)
- Label Manager Agent (Agent: feature-implementer)
- Release Coordinator Agent (Agent: feature-implementer)
- Validation Agent (Agent: feature-implementer)
**Quality Gate**: All agents implemented

### Phase 3: Update Workflow
**Tasks**:
- Update release-drift.yml (Agent: github-action-editor)
- Add automated resolution (Agent: feature-implementer)
**Quality Gate**: Workflow updated and tested

### Phase 4: Update Documentation
**Tasks**:
- Update AGENTS.md (Agent: documentation)
- Create skill documentation (Agent: documentation)
**Quality Gate**: Documentation updated

### Overall Success Criteria
- [ ] All tasks complete
- [ ] Quality gates passed
- [ ] Tests passing
- [ ] Documentation updated

### Contingency Plans
- If Phase 0 fails → Use existing ADRs as reference
- If Phase 1 fails → Create minimal skill and iterate
- If Phase 2 fails → Implement agents incrementally
- If Phase 3 fails → Use existing workflow as-is
- If Phase 4 fails → Document manually
```

## Phase 6: Coordinated Execution

### Parallel Execution
Launch agents simultaneously, monitor independently, aggregate results.

### Sequential Execution
One agent completes before next starts. Quality gates between phases.

### Monitoring
- Track agent progress
- Monitor for failures
- Validate intermediate results

### Quality Gates
- **Gate 1**: ADRs identified and reviewed
- **Gate 2**: Skill created with all files
- **Gate 3**: All agents implemented
- **Gate 4**: Workflow updated and tested
- **Gate 5**: Documentation updated

## Phase 7: Synthesis and Validation

### Result Aggregation
- Combine results from all agents
- Validate overall success
- Document lessons learned

### Validation
- Run end-to-end tests
- Verify integration with existing workflows
- Ensure documentation is complete

### Documentation
- Update AGENTS.md
- Create skill documentation
- Document patterns and lessons learned
