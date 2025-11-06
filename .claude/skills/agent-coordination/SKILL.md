---
name: agent-coordination
description: Coordinate multiple specialized agents through parallel, sequential, swarm, or hybrid execution strategies. Use this when orchestrating multi-agent workflows, managing dependencies between agents, or optimizing complex task execution with quality gates and validation checkpoints.
---

# Agent Coordination

Coordinate multiple specialized agents to solve complex multi-step tasks efficiently through strategic execution patterns.

## Coordination Strategies

### 1. Parallel Coordination

**Use When**: Independent tasks, no dependencies, maximize throughput

**Implementation**:
- Single message with multiple Task tool calls
- All agents start simultaneously
- Results collected and merged

**Example**:
```markdown
Task: "Review code and run tests"

├─ code-reviewer: Review code quality
└─ test-runner: Execute test suite

Execution: One message, both Task tools
```

### 2. Sequential Coordination

**Use When**: Strong dependencies, each task needs previous output

**Implementation**:
- Chain of messages, each waits for previous
- Explicit handoffs between agents
- Output of one is input to next

**Example**:
```markdown
Task: "Implement feature, test it, then review"

└─ feature-implementer: Build feature
   └─ test-runner: Test implementation
      └─ code-reviewer: Review code

Execution: Sequential messages with context transfer
```

### 3. Swarm Coordination

**Use When**: Complex problem needs multiple perspectives

**Implementation**:
- Phase 1: Parallel investigation (multiple agents)
- Phase 2: Synthesis (combine findings)
- Phase 3: Coordinated resolution

**Example**:
```markdown
Task: "Diagnose performance degradation"

Phase 1 [Parallel]:
├─ debugger: Profile runtime
├─ code-reviewer: Analyze code efficiency
└─ test-runner: Run benchmarks

Phase 2: Synthesize findings → identify root cause
Phase 3: Apply coordinated fix
```

### 4. Hybrid Coordination

**Use When**: Complex workflows with mixed dependencies

**Implementation**:
- Multiple phases with different strategies
- Parallel within phases, sequential between phases
- Validation gates between phases

**Example**:
```markdown
Task: "Refactor module, update tests, verify"

Phase 1 [Parallel]: Assessment
├─ code-reviewer: Assess current code
└─ test-runner: Run existing tests

Phase 2 [Sequential]: Implementation
└─ refactorer: Apply improvements

Phase 3 [Parallel]: Validation
├─ test-runner: Verify refactored code
└─ code-reviewer: Final quality check
```

## Available Agents

### Agent Capabilities

| Agent | Best For | Inputs | Outputs |
|-------|----------|--------|---------|
| **test-runner** | Testing, verification | Code to test | Test results, coverage |
| **code-reviewer** | Quality, standards | Code changes | Review report, issues |
| **feature-implementer** | New functionality | Requirements | Implementation, tests |
| **refactorer** | Code improvement | Code to refactor | Improved code |
| **debugger** | Issue diagnosis | Issue description | Root cause, fix |

## Coordination Workflow

### Phase 1: Strategy Selection

**Decision Matrix**:
- Independent tasks + Time-critical → **Parallel**
- Strong dependencies + Order matters → **Sequential**
- Complex problem + Multiple perspectives → **Swarm**
- Multi-phase + Mixed dependencies → **Hybrid**

### Phase 2: Agent Assignment

**Match Tasks to Agents**:
1. Capability matching (does agent have required skills?)
2. Workload balancing (distribute evenly)
3. Expertise routing (specialized tasks to expert agents)

### Phase 3: Execution Planning

```markdown
## Execution Plan

### Strategy: [Parallel/Sequential/Swarm/Hybrid]

### Phase 1: [Name]
**Mode**: [Parallel/Sequential]
**Agents**:
- Agent: [name] | Task: [description] | Deps: [dependencies]

**Quality Gate**:
- [Validation criteria]

### Overall Success Criteria:
- [Criterion 1]
- [Criterion 2]
```

### Phase 4: Execution & Monitoring

**Monitoring Checklist**:
- [ ] Agent has started
- [ ] Agent is making progress
- [ ] Agent output meets quality criteria
- [ ] No errors or failures
- [ ] Completion within expected time

### Phase 5: Quality Validation

**Validation Gates** (between phases):
1. Output validation (format, completeness, quality)
2. Success criteria check (phase goals met?)
3. Error handling (can errors be recovered?)

### Phase 6: Result Synthesis

```markdown
## Execution Summary

### Completed Tasks:
- [Task 1]: ✓ [Agent] - [Outcome]

### Deliverables:
- [Item 1]: [Location/Description]

### Quality Validation:
- [Criterion 1]: ✓ Met

### Performance Metrics:
- Total time: [duration]
- Success rate: [percentage]
```

## Communication Patterns

### Agent-to-Agent Handoff

**Context Transfer**:
```markdown
Agent A completes, produces output X

Message to Agent B:
"Previous agent (A) produced X. Use this as input.
Task: [specific instructions for B]
Success criteria: [how to validate]"
```

### Synchronization Points

**Parallel Convergence**:
1. Wait for all agents to complete
2. Collect outputs from each
3. Validate each output
4. If all valid → proceed, else handle errors

**Quality Gates**:
1. Validate all outputs from phase
2. Check success criteria
3. Decision: proceed / retry / adjust / abort

## Error Handling

### Recovery Strategies

**Retry**: For transient failures (max 2-3 attempts)
**Alternative Approach**: Different agent or technical approach
**Plan Adjustment**: Remove optional tasks, simplify requirements
**Graceful Degradation**: Partial completion with documentation

### Failure Scenarios

| Scenario | Response |
|----------|----------|
| Agent reports failure | Analyze reason, retry with adjusted params or find alternative |
| Quality gate fails | Stop dependent tasks, diagnose, fix, re-execute |
| Blocked dependency | Identify blocker, work around, reorder if possible |

## Best Practices

### DO:
✓ Choose appropriate strategy for task dependencies
✓ Match agents to tasks based on capabilities
✓ Validate at quality gates before proceeding
✓ Provide clear context in agent handoffs
✓ Monitor progress during execution
✓ Handle errors gracefully with recovery
✓ Aggregate results comprehensively

### DON'T:
✗ Force parallel execution when dependencies exist
✗ Assign tasks to inappropriate agents
✗ Skip quality validation
✗ Proceed after failed quality gates
✗ Provide insufficient context to agents

## Coordination Metrics

### Efficiency Metrics
- **Execution Time**: Track total time and per-agent time
- **Resource Utilization**: Agents active / available
- **Throughput**: Tasks completed / time

### Quality Metrics
- **Success Rate**: Tasks successful / total (should be >95%)
- **Quality Gate Passage**: All gates should pass (100%)
- **Rework Rate**: Tasks requiring retry (should be <10%)

## Integration with GOAP Agent

The GOAP agent uses this skill as its core coordination engine:

1. Decompose task (task-decomposition skill)
2. Select coordination strategy (this skill)
3. Assign agents to tasks (this skill)
4. Execute coordination (this skill + parallel-execution skill)
5. Validate and report (this skill)

## Summary

Effective agent coordination requires:
- **Right strategy** for the task structure
- **Right agents** for each sub-task
- **Clear communication** and context transfer
- **Quality validation** at key checkpoints
- **Graceful error handling** and recovery
- **Comprehensive result synthesis**

Use this skill to coordinate any multi-agent workflow effectively.
