# GOAP Methodology

## Planning Cycle

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

**Primary Goal**: [Clear statement of what success looks like]

**Constraints**:
- Time: [Urgent / Normal / Flexible]
- Resources: [Available agents, tools, data]
- Dependencies: [External systems, prerequisites]

**Complexity Level**:
- Simple: Single agent, <3 steps
- Medium: 2-3 agents, some dependencies
- Complex: 4+ agents, mixed execution modes
- Very Complex: Multiple phases, many dependencies

**Quality Requirements**:
- Testing: [Unit / Integration / E2E]
- Standards: [AGENTS.md compliance, formatting, linting]
- Documentation: [API docs, examples, guides]
- Performance: [Speed, memory, scalability]
```

### Context Gathering

1. **Codebase Understanding**: Use Explore agent
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
- Performance: ADR-025, ADR-026
- MCP/Architecture: ADR-022, ADR-024
- Testing: ADR-025, ADR-027

**See**: `plans/adr/` for complete ADR inventory

## Phase 2: Task Decomposition

Use the **task-decomposition** skill to break down goals.

### Key Principles
- **Atomic**: Each task is indivisible and clear
- **Testable**: Can verify completion
- **Independent**: Minimize dependencies
- **Assigned**: Each task maps to agent capability

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

## Phase 4: Agent Assignment

See **[agents.md](agents.md)** for capability matrix and **[skills.md](skills.md)** for skill categories.

## Phase 5: Execution Planning

### Template

```markdown
## Execution Plan: [Task Name]

### Overview
- Strategy: [Parallel/Sequential/Swarm/Hybrid/Iterative]
- Total Tasks: [N]
- Quality Gates: [N checkpoints]

### Phase 1: [Phase Name]
**Tasks**:
- Task 1: [Description] (Agent: type)
- Task 2: [Description] (Agent: type)
**Quality Gate**: [Validation criteria]

### Overall Success Criteria
- [ ] All tasks complete
- [ ] Quality gates passed
- [ ] Tests passing

### Contingency Plans
- If Phase 1 fails → [Recovery plan]
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
- Adjust plan if needed

## Phase 7: Result Synthesis

```markdown
## Execution Summary: [Task Name]

### Completed Tasks
- [Task 1]: Success
- [Task 2]: Success

### Deliverables
- [File/Feature 1]
- [File/Feature 2]

### Quality Validation
- Tests: [Pass/Fail]
- Linting: [Pass/Fail]

### Performance Metrics
- Duration: [actual vs estimated]
- Efficiency: [parallel speedup]

### Lessons Learned
- [What worked well]
- [What to improve]
```

## Dynamic Re-Planning

If during execution:
- Dependencies change
- Requirements clarified
- Blockers discovered

Then:
1. Pause execution
2. Re-analyze with new information
3. Adjust plan
4. Resume with updated plan
