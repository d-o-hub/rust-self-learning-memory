---
name: goap-agent
description: Invoke for complex multi-step tasks requiring intelligent planning and multi-agent coordination. Use when tasks need decomposition, dependency mapping, parallel/sequential/swarm execution strategies, or coordination of multiple specialized agents with quality gates and dynamic optimization.
---

# GOAP Agent Skill: Goal-Oriented Action Planning

Enable intelligent planning and execution of complex multi-step tasks through systematic decomposition, dependency mapping, and coordinated multi-agent execution.

## When to Use This Skill

Use this skill when facing:

- **Complex Multi-Step Tasks**: Tasks requiring 5+ distinct steps or multiple specialized capabilities
- **Cross-Domain Problems**: Issues spanning multiple areas (storage, API, testing, documentation)
- **Optimization Opportunities**: Tasks that could benefit from parallel or hybrid execution
- **Quality-Critical Work**: Projects requiring validation checkpoints and quality gates
- **Resource-Intensive Operations**: Large refactors, migrations, or architectural changes
- **Ambiguous Requirements**: Tasks needing structured analysis before execution

## Core GOAP Methodology

### The GOAP Planning Cycle

```
1. ANALYZE → Understand goals, constraints, resources
2. DECOMPOSE → Break into atomic tasks with dependencies
3. STRATEGIZE → Choose execution pattern (parallel/sequential/swarm/hybrid)
4. COORDINATE → Assign tasks to specialized agents
5. EXECUTE → Run with monitoring and quality gates
6. SYNTHESIZE → Aggregate results and validate success
```

## Phase 1: Task Analysis

### Initial Assessment

Ask yourself these key questions:

```markdown
## Task Analysis

**What is the primary goal?**
- [Clear statement of what success looks like]

**What are the constraints?**
- Time: [Urgent / Normal / Flexible]
- Resources: [Available agents, tools, data]
- Dependencies: [External systems, prerequisites]

**What is the complexity level?**
- Simple: Single agent, <3 steps
- Medium: 2-3 agents, some dependencies
- Complex: 4+ agents, mixed execution modes
- Very Complex: Multiple phases, many dependencies

**What are the quality requirements?**
- Testing: [Unit / Integration / E2E]
- Standards: [AGENTS.md compliance, formatting, linting]
- Documentation: [API docs, examples, guides]
- Performance: [Speed, memory, scalability requirements]
```

### Context Gathering

Before planning, gather relevant context:

1. **Codebase Understanding**: Use Explore agent to understand relevant code
2. **Past Patterns**: Check if similar tasks have been done before
3. **Available Resources**: Identify available agents and their capabilities
4. **Current State**: Understand starting conditions and existing implementations

## Phase 2: Task Decomposition

Use the **task-decomposition** skill to break down the goal:

### Decomposition Template

```markdown
## Task Decomposition: [Task Name]

### Main Goal
[Clear statement of primary objective]

### Sub-Goals
1. [Component 1] - Priority: [P0/P1/P2]
   - Success Criteria: [How to verify completion]
   - Dependencies: [Prerequisites]
   - Complexity: [Low/Medium/High]

2. [Component 2] - Priority: [P0/P1/P2]
   - Success Criteria: [How to verify completion]
   - Dependencies: [Prerequisites]
   - Complexity: [Low/Medium/High]

### Atomic Tasks
**Component 1: [Name]**
- Task 1.1: [Action] (Agent: [type], Deps: [list])
- Task 1.2: [Action] (Agent: [type], Deps: [list])

**Component 2: [Name]**
- Task 2.1: [Action] (Agent: [type], Deps: [list])
- Task 2.2: [Action] (Agent: [type], Deps: [list])

### Dependency Graph
```
[Visual representation of task dependencies]
Task A ──┐
Task B ──┼──> Task D
Task C ──┘
```
```

### Key Decomposition Principles

✓ **Atomic Tasks**: Each task is a single, clear action
✓**Testable**: Completion can be verified objectively
✓ **Bounded**: Tasks have clear start and end points
✓ **Assignable**: Each task maps to an agent capability
✓ **Independent**: Minimize unnecessary dependencies

## Phase 3: Strategy Selection

Use the **agent-coordination** skill to choose the right strategy:

### Parallel Execution Strategy

**Use When**:
- Tasks have no dependencies
- All tasks can run simultaneously
- Goal is to maximize speed

**Implementation**:
```markdown
Phase: [Name] - Parallel Execution
├─ Agent: test-runner | Task: Run test suite
├─ Agent: code-reviewer | Task: Review code quality
└─ Agent: debugger | Task: Profile performance

Execution: Single message with 3 Task tool calls
```

### Sequential Execution Strategy

**Use When**:
- Strong dependencies between tasks
- Each task needs previous output
- Order matters for correctness

**Implementation**:
```markdown
Phase: [Name] - Sequential Chain
└─ Step 1: feature-implementer → Implement core logic
   └─ Step 2: test-runner → Test implementation
      └─ Step 3: code-reviewer → Review quality

Execution: Chain with validation gates
```

### Swarm Coordination Strategy

**Use When**:
- Complex problem needs multiple perspectives
- Investigation before solution
- Parallel exploration benefits the task

**Implementation**:
```markdown
Phase 1: Parallel Investigation
├─ Agent: debugger | Task: Profile performance
├─ Agent: code-reviewer | Task: Analyze code
└─ Agent: test-runner | Task: Run benchmarks

Phase 2: Synthesis
└─ GOAP: Combine findings, identify root cause

Phase 3: Coordinated Resolution
└─ Agent: refactorer | Task: Apply fix based on findings

Execution: Investigate → Synthesize → Fix
```

### Hybrid Execution Strategy

**Use When**:
- Mix of dependent and independent tasks
- Multi-phase work with varying parallelism
- Complex workflows with validation checkpoints

**Implementation**:
```markdown
Phase 1: Assessment [Parallel]
├─ Agent: code-reviewer | Task: Assess current state
└─ Agent: test-runner | Task: Baseline tests

Phase 2: Implementation [Sequential]
└─ Agent: feature-implementer | Task: Build feature

Phase 3: Validation [Parallel]
├─ Agent: test-runner | Task: Test new feature
└─ Agent: code-reviewer | Task: Final review

Execution: Parallel → Sequential → Parallel
```

## Phase 4: Agent Assignment

### Agent Capability Matrix

| Agent | Specialization | Best For | Outputs |
|-------|---------------|----------|---------|
| **test-runner** | Testing, debugging | Test execution, coverage, failure diagnosis | Test results, diagnostics |
| **code-reviewer** | Quality, standards | Code review, quality audit, compliance | Review reports, recommendations |
| **feature-implementer** | Feature development | New functionality, module creation | Code, tests, docs |
| **refactorer** | Code improvement | Performance, maintainability, cleanup | Refactored code, optimizations |
| **debugger** | Issue diagnosis | Runtime issues, performance, async bugs | Root cause, fixes |
| **Explore** | Code exploration | Understanding codebase, finding patterns | Code analysis, summaries |

### Assignment Principles

1. **Match Expertise**: Assign tasks to agents with relevant specialization
2. **Balance Load**: Distribute work evenly across available agents
3. **Minimize Handoffs**: Group related tasks with same agent when possible
4. **Consider Dependencies**: Ensure agents have needed inputs from previous tasks

## Phase 5: Execution Planning

### Create the Execution Plan

```markdown
## Execution Plan: [Task Name]

### Overview
- **Objective**: [Clear goal statement]
- **Strategy**: [Parallel/Sequential/Swarm/Hybrid]
- **Estimated Duration**: [Time estimate]
- **Key Risks**: [Potential blockers]

### Phase 1: [Phase Name]
**Mode**: [Parallel/Sequential]
**Quality Gate**: [Validation criteria before next phase]

| Agent | Task | Dependencies | Success Criteria |
|-------|------|--------------|------------------|
| [agent-1] | [task description] | [deps or "none"] | [how to verify] |
| [agent-2] | [task description] | [deps or "none"] | [how to verify] |

### Phase 2: [Phase Name]
[Repeat structure]

### Overall Success Criteria
- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]

### Contingency Plans
- **If [Risk 1] occurs**: [Recovery action]
- **If [Risk 2] occurs**: [Recovery action]
```

### Quality Gates

Define validation checkpoints between phases:

```markdown
## Quality Gate: Phase 1 → Phase 2

### Validation Criteria
- [ ] All Phase 1 tasks completed
- [ ] Output artifacts present and valid
- [ ] Success criteria met for each task
- [ ] No blocking errors or failures

### Decision
- ✓ Pass → Proceed to Phase 2
- ✗ Fail → [Recovery action]
```

## Phase 6: Coordinated Execution

### Parallel Execution

**How to Execute**:
```
Send single message with multiple Task tool calls:

Task 1: task-decomposition skill
Task 2: agent-coordination skill
Task 3: parallel-execution skill

All launch simultaneously, results collected
```

### Sequential Execution

**How to Execute**:
```
Message 1: Task tool → Agent A
[Wait for completion, validate output]

Message 2: Task tool → Agent B
[Pass Agent A results, wait for completion]

Message 3: Task tool → Agent C
[Pass Agent B results, wait for completion]
```

### Monitoring During Execution

Track for each agent:
- **Status**: pending / in_progress / completed / failed
- **Progress**: Percentage or current milestone
- **Blockers**: Any issues preventing completion
- **Quality**: Output meeting criteria?

## Phase 7: Result Synthesis

### Aggregate Results

```markdown
## Execution Summary: [Task Name]

### ✓ Completed Tasks
1. [Task 1] - Agent: [name] - Outcome: [description]
2. [Task 2] - Agent: [name] - Outcome: [description]

### 📦 Deliverables
- [Deliverable 1]: [location] - [description]
- [Deliverable 2]: [location] - [description]

### ✅ Quality Validation
- [Criterion 1]: ✓ Met
- [Criterion 2]: ✓ Met
- [Criterion 3]: ✓ Met

### 📊 Performance Metrics
- Total Duration: [time]
- Tasks Completed: [count]
- Quality Gate Pass Rate: [percentage]

### 💡 Recommendations
- [Future improvement 1]
- [Future improvement 2]

### 🎓 Lessons Learned
- [Pattern or insight discovered]
- [What worked well]
- [What to improve next time]
```

## Common GOAP Patterns

### Pattern 1: Research → Implement → Validate

**Use Case**: New feature development

```markdown
Phase 1: Research [Parallel]
├─ Explore: Understand existing code
└─ code-reviewer: Assess integration points

Phase 2: Implement [Sequential]
└─ feature-implementer: Build feature

Phase 3: Validate [Parallel]
├─ test-runner: Execute tests
└─ code-reviewer: Quality review
```

### Pattern 2: Investigate → Diagnose → Fix → Verify

**Use Case**: Bug fixing and debugging

```markdown
Phase 1: Investigate [Swarm]
├─ debugger: Profile and reproduce
├─ code-reviewer: Analyze related code
└─ test-runner: Test edge cases

Phase 2: Diagnose [Synthesis]
└─ GOAP: Combine findings, identify root cause

Phase 3: Fix [Sequential]
└─ refactorer: Apply fix

Phase 4: Verify [Parallel]
├─ test-runner: Verify fix works
└─ code-reviewer: Ensure quality maintained
```

### Pattern 3: Audit → Improve → Validate

**Use Case**: Code quality improvement

```markdown
Phase 1: Audit [Parallel]
├─ code-reviewer: Code quality analysis
├─ test-runner: Test coverage audit
└─ refactorer: Performance analysis

Phase 2: Improve [Prioritized Sequential]
└─ For each high-priority issue:
   refactorer: Apply improvement

Phase 3: Validate [Parallel]
├─ test-runner: Verify all tests pass
└─ code-reviewer: Confirm improvements
```

## Error Handling & Recovery

### Agent Failure Recovery

**If agent fails**:
1. Analyze failure message and reason
2. Determine if retry appropriate (transient errors)
3. Consider alternative agent or approach
4. Adjust plan if fundamental blocker
5. Log failure for learning

### Quality Gate Failure

**If quality gate fails**:
1. STOP execution of dependent tasks
2. Diagnose what didn't meet criteria
3. Determine fix approach
4. Re-execute failed phase
5. Re-validate quality gate

### Blocked Dependencies

**If dependency is blocked**:
1. Identify what's blocking
2. Check if can work around
3. Consider reordering tasks
4. Communicate to user if critical blocker

## Best Practices

### DO:
✓ Always analyze before planning
✓ Create explicit execution plans for complex tasks
✓ Use parallel execution for independent tasks
✓ Validate at quality gates before proceeding
✓ Provide clear context to agents
✓ Monitor progress during execution
✓ Handle errors gracefully with recovery plans
✓ Aggregate and synthesize results comprehensively
✓ Learn from execution patterns

### DON'T:
✗ Skip analysis and jump straight to execution
✗ Force parallel execution when dependencies exist
✗ Ignore quality validation checkpoints
✗ Continue with failed subtasks
✗ Assign tasks to wrong agents
✗ Provide insufficient context
✗ Skip result synthesis

## Integration with Other Skills

The GOAP agent skill orchestrates other skills:

- **task-decomposition**: For breaking down complex goals
- **agent-coordination**: For choosing execution strategies
- **parallel-execution**: For managing concurrent agents
- **episode-start/log-steps/complete**: For learning from coordination
- **storage-sync**: When memory consistency needed
- **context-retrieval**: To learn from past similar tasks

## Example GOAP Sessions

### Example 1: Implement Batch Pattern Update Feature

```markdown
## GOAP Plan: Batch Pattern Update Feature

### Phase 1: Analysis
**What**: Complex feature requiring storage layer changes
**Complexity**: High (multiple components, quality-critical)
**Strategy**: Hybrid (parallel research + sequential implementation)

### Phase 2: Decomposition
Main Goal: Enable efficient batch updates of patterns

Sub-Goals:
1. Design batch schema (P0)
2. Implement Turso batch operations (P0)
3. Implement redb batch caching (P0)
4. Add comprehensive tests (P0)
5. Write API documentation (P1)

### Phase 3: Execution Plan

**Phase 1: Research & Assessment** [Parallel]
- Explore: Analyze existing pattern storage code
- test-runner: Run baseline tests

**Phase 2: Design** [Sequential]
- feature-implementer: Design batch schema

**Phase 3: Implementation** [Parallel]
- feature-implementer: Implement Turso batch ops (depends on Phase 2)
- feature-implementer: Implement redb batch ops (depends on Phase 2)

**Phase 4: Validation** [Parallel]
- test-runner: Execute all tests
- code-reviewer: Quality review

### Phase 4: Execution
[Execute with monitoring...]

### Phase 5: Synthesis
✓ Batch pattern update implemented
✓ 15 new tests added, all passing
✓ Performance: 10x faster for bulk operations
✓ Code quality: passes fmt, clippy, standards
```

### Example 2: Debug Performance Issue

```markdown
## GOAP Plan: Diagnose Slow Episode Retrieval

### Phase 1: Analysis
**What**: Performance degradation in episode queries
**Complexity**: Medium (investigation + fix)
**Strategy**: Swarm investigation → Sequential fix

### Phase 2: Execution Plan

**Phase 1: Parallel Investigation** [Swarm]
- debugger: Profile runtime, identify bottlenecks
- code-reviewer: Review query patterns and indexes
- test-runner: Run performance benchmarks

**Phase 2: Synthesis**
- GOAP: Combine findings, identify root cause

**Phase 3: Resolution** [Sequential]
- refactorer: Apply optimizations

**Phase 4: Verification** [Parallel]
- test-runner: Verify performance improvement
- code-reviewer: Ensure quality maintained

### Phase 3: Execution
[Execute with findings aggregation...]

### Phase 4: Results
Root Cause: Missing database index on timestamp
Fix: Added index, implemented caching
Performance: 850ms → 45ms (95% improvement)
✓ Quality maintained
```

## Success Metrics

Track effectiveness of GOAP planning:

### Planning Quality
- **Accuracy**: Actual execution matches plan (target: >90%)
- **Completeness**: All requirements covered (target: 100%)
- **Efficiency**: Optimal use of parallelization

### Execution Quality
- **Success Rate**: Tasks completed successfully (target: >95%)
- **Quality Gate Pass**: All gates passed (target: 100%)
- **Time Efficiency**: Faster than sequential baseline

### Learning
- **Pattern Extraction**: Successful strategies logged
- **Heuristic Development**: Agent assignment rules refined
- **Continuous Improvement**: Each execution improves next

## Advanced Topics

### Dynamic Re-Planning

**When to re-plan**:
- Major blocker discovered
- Requirements change mid-execution
- Agent failure can't be recovered
- Better approach identified

**How to re-plan**:
1. Pause execution
2. Reassess remaining work
3. Update plan with new information
4. Communicate changes to user
5. Resume with adjusted plan

### Optimization Techniques

**Reduce Handoff Overhead**:
- Batch related tasks to same agent
- Minimize context transfer

**Maximize Parallelism**:
- Identify more independent tasks
- Consider speculative execution

**Critical Path Optimization**:
- Focus resources on longest dependency chain
- Parallelize non-critical tasks

## Summary

The GOAP Agent Skill enables intelligent, systematic planning and execution of complex multi-step tasks:

1. **Analyze** task complexity and requirements
2. **Decompose** into atomic, testable tasks
3. **Strategize** optimal execution pattern
4. **Coordinate** specialized agents
5. **Execute** with quality gates
6. **Synthesize** results and validate
7. **Learn** from execution patterns

Use this skill when complexity demands structured planning, when multiple agents provide value, or when quality-critical work requires validation checkpoints.

The GOAP methodology transforms overwhelming complexity into manageable, coordinated execution.
