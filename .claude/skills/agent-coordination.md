# Agent Coordination

Coordinate multiple specialized agents through various execution strategies to solve complex multi-step tasks efficiently.

## Purpose

Enable effective multi-agent collaboration by selecting appropriate coordination strategies, managing agent communication, and ensuring quality outcomes.

## When to Use

- Tasks requiring multiple specialized agents
- Complex workflows with dependencies
- Need to optimize execution time through parallelization
- Coordinating sequential handoffs between agents
- Managing quality gates and validation checkpoints

## Coordination Strategies

### 1. Parallel Coordination

**Use When**:
- Tasks are independent (no dependencies)
- Maximizing throughput is priority
- Results can be aggregated afterward
- Resources (agents) are available

**Characteristics**:
- All agents start simultaneously
- No inter-agent dependencies
- Results collected and merged
- Fastest overall execution time

**Example**:
```markdown
Task: "Review code and run tests"

Parallel Strategy:
├─ code-reviewer: Review code quality
└─ test-runner: Execute test suite

Execution: Single message with both Task tool calls
Result: Both complete independently, results aggregated
```

**Implementation**:
```markdown
Send single message with multiple Task tool uses:

1. Task tool → code-reviewer agent (review task)
2. Task tool → test-runner agent (test task)

Wait for both to complete, then aggregate results.
```

### 2. Sequential Coordination

**Use When**:
- Strong dependencies between tasks
- Each task needs previous task's output
- Order matters for correctness
- Building up complex state

**Characteristics**:
- One agent at a time
- Explicit handoffs between agents
- Output of one is input to next
- Guarantees ordering

**Example**:
```markdown
Task: "Implement feature, test it, then review"

Sequential Strategy:
└─ feature-implementer: Build feature
   └─ test-runner: Test implementation
      └─ code-reviewer: Review code

Execution: Chain of messages, each waits for previous
Result: Each agent builds on previous work
```

**Implementation**:
```markdown
Message 1: Task tool → feature-implementer
[Wait for completion, review output]

Message 2: Task tool → test-runner
  Context: Feature from feature-implementer
[Wait for completion, review output]

Message 3: Task tool → code-reviewer
  Context: Implementation + test results
[Wait for completion]
```

### 3. Swarm Coordination

**Use When**:
- Complex problem needs multiple perspectives
- No single agent has complete solution
- Benefit from collective intelligence
- Need comprehensive coverage

**Characteristics**:
- Multiple agents work on related aspects
- Findings are synthesized
- Collective intelligence applied
- Comprehensive solution generated

**Example**:
```markdown
Task: "Diagnose performance degradation"

Swarm Strategy:
├─ debugger: Profile runtime
├─ code-reviewer: Analyze code efficiency
└─ test-runner: Run performance benchmarks

Synthesis: Combine findings → identify root cause
Resolution: Coordinated fix based on collective insights
```

**Implementation**:
```markdown
Phase 1: Parallel Investigation
- Task tool → debugger (profiling)
- Task tool → code-reviewer (code analysis)
- Task tool → test-runner (benchmarking)

Phase 2: Synthesis
- Collect all outputs
- Analyze collective findings
- Identify root cause
- Formulate solution

Phase 3: Resolution
- Task tool → refactorer (apply fix)
  Context: Synthesized findings
```

### 4. Hybrid Coordination

**Use When**:
- Complex workflows with mixed dependencies
- Some tasks parallel, some sequential
- Multi-phase execution needed
- Optimizing for both speed and correctness

**Characteristics**:
- Phases with different strategies
- Parallel within phases
- Sequential between phases
- Validation gates between phases

**Example**:
```markdown
Task: "Refactor module, update tests, verify"

Hybrid Strategy:

Phase 1 [Parallel]:
├─ code-reviewer: Assess current code
└─ test-runner: Run existing tests

Phase 2 [Sequential]:
└─ refactorer: Apply improvements
   Context: code-reviewer findings

Phase 3 [Parallel]:
├─ test-runner: Verify refactored code
└─ code-reviewer: Final quality check
```

**Implementation**:
```markdown
Phase 1: Parallel assessment
[Single message, multiple Task tools]

Phase 2: Sequential refactoring
[One message, wait for completion]

Phase 3: Parallel validation
[Single message, multiple Task tools]

Each phase has validation gate before next phase
```

## Agent Profiles

### Available Agents & Specializations

#### test-runner
**Capabilities**:
- Run unit, integration, doc tests
- Debug test failures
- Analyze async/await issues
- Verify test coverage
- Fix flaky tests

**Best For**:
- Testing and verification
- Quality assurance
- Test debugging
- Coverage analysis

**Inputs**: Code to test, test selection criteria
**Outputs**: Test results, failure diagnostics, coverage reports

#### code-reviewer
**Capabilities**:
- Code quality analysis
- Standards compliance checking
- Architecture review
- Performance analysis
- Security review

**Best For**:
- Code review
- Quality validation
- Pre-commit checks
- Architecture assessment

**Inputs**: Code changes, review scope
**Outputs**: Review report, issues, recommendations

#### feature-implementer
**Capabilities**:
- Feature development
- Module creation
- API design
- Test writing
- Documentation

**Best For**:
- New functionality
- Feature additions
- System extensions
- API development

**Inputs**: Feature requirements, design constraints
**Outputs**: Implementation, tests, documentation

#### refactorer
**Capabilities**:
- Code improvement
- Performance optimization
- Structure reorganization
- Technical debt reduction
- Complexity reduction

**Best For**:
- Code cleanup
- Performance tuning
- Maintainability improvements
- Architecture improvements

**Inputs**: Code to refactor, goals
**Outputs**: Refactored code, performance improvements

#### debugger
**Capabilities**:
- Runtime issue diagnosis
- Performance profiling
- Deadlock detection
- Memory leak analysis
- Root cause analysis

**Best For**:
- Production issues
- Performance problems
- Debugging complex issues
- System analysis

**Inputs**: Issue description, reproduction steps
**Outputs**: Root cause, fix, verification

## Coordination Workflow

### Phase 1: Strategy Selection

**Analyze Task**:
```markdown
1. Identify sub-tasks from decomposition
2. Map dependencies between sub-tasks
3. Identify available agents and capabilities
4. Assess resource constraints
5. Choose coordination strategy
```

**Decision Matrix**:
```
Independent tasks + Time-critical → Parallel
Strong dependencies + Order matters → Sequential
Complex problem + Multiple perspectives → Swarm
Multi-phase + Mixed dependencies → Hybrid
```

### Phase 2: Agent Assignment

**Match Tasks to Agents**:

1. **Capability Matching**:
   - Does agent have required skills?
   - Has agent succeeded on similar tasks?
   - Is agent's focus aligned with task?

2. **Workload Balancing**:
   - Distribute tasks evenly
   - Avoid overloading single agent
   - Consider task complexity

3. **Expertise Routing**:
   - Testing → test-runner
   - Quality → code-reviewer
   - Implementation → feature-implementer
   - Optimization → refactorer
   - Debugging → debugger

**Assignment Template**:
```markdown
Task: [Task description]
Agent: [Selected agent]
Reason: [Why this agent]
Inputs: [What agent needs]
Expected Output: [What agent will produce]
Success Criteria: [How to validate]
```

### Phase 3: Execution Planning

**Create Execution Plan**:

```markdown
## Execution Plan: [Task Name]

### Strategy: [Parallel/Sequential/Swarm/Hybrid]

### Phase Breakdown:

#### Phase 1: [Name]
**Mode**: [Parallel/Sequential]
**Agents**:
- Agent: [name] | Task: [description] | Deps: [dependencies]
- Agent: [name] | Task: [description] | Deps: [dependencies]

**Quality Gate**:
- Criterion 1: [validation]
- Criterion 2: [validation]

#### Phase 2: [Name]
[Same structure...]

### Overall Success Criteria:
- [Criterion 1]
- [Criterion 2]
- [Criterion 3]
```

### Phase 4: Execution & Monitoring

**Execute Coordination**:

**For Parallel**:
```markdown
1. Send single message with multiple Task tool calls
2. Monitor all agents simultaneously
3. Collect results as they complete
4. Validate each output
5. Aggregate results
```

**For Sequential**:
```markdown
1. Send message with first Task tool call
2. Wait for completion
3. Validate output
4. Prepare context for next agent
5. Send message with next Task tool call
6. Repeat until chain complete
```

**For Swarm**:
```markdown
1. Launch all agents in parallel (Phase 1)
2. Collect all findings
3. Synthesize collective insights (Phase 2)
4. Coordinate unified action (Phase 3)
5. Validate overall outcome
```

**For Hybrid**:
```markdown
1. Execute Phase 1 per its strategy
2. Validate Phase 1 quality gate
3. Execute Phase 2 per its strategy
4. Validate Phase 2 quality gate
5. Continue through all phases
6. Validate overall success criteria
```

**Monitoring Checklist**:
- [ ] Agent has started
- [ ] Agent is making progress
- [ ] Agent output meets quality criteria
- [ ] No errors or failures
- [ ] Completion within expected time

### Phase 5: Quality Validation

**Validation Gates**:

Between each phase or agent handoff:

1. **Output Validation**:
   - Does output match expected format?
   - Are all required elements present?
   - Is quality acceptable?

2. **Success Criteria Check**:
   - Are phase success criteria met?
   - Are there any blocking issues?
   - Is it safe to proceed?

3. **Error Handling**:
   - Did agent report errors?
   - Can errors be recovered?
   - Should execution stop or continue?

**Validation Template**:
```markdown
## Phase [N] Validation

### Outputs Received:
- Agent: [name] | Output: [description] | Status: ✓/✗

### Quality Checks:
- [Criterion 1]: ✓/✗
- [Criterion 2]: ✓/✗

### Decision:
- [ ] Proceed to next phase
- [ ] Retry current phase
- [ ] Adjust plan
- [ ] Abort execution

### Issues Found:
[List any issues and resolution approach]
```

### Phase 6: Result Synthesis

**Aggregate Outputs**:

1. **Collect Results**:
   - Gather outputs from all agents
   - Organize by phase/task
   - Identify relationships

2. **Validate Completeness**:
   - All tasks completed?
   - All deliverables produced?
   - All quality criteria met?

3. **Synthesize Report**:
   ```markdown
   ## Execution Summary

   ### Completed Tasks:
   - [Task 1]: ✓ [Agent] - [Outcome]
   - [Task 2]: ✓ [Agent] - [Outcome]

   ### Deliverables:
   - [Item 1]: [Location/Description]
   - [Item 2]: [Location/Description]

   ### Quality Validation:
   - [Criterion 1]: ✓ Met
   - [Criterion 2]: ✓ Met

   ### Performance Metrics:
   - Total time: [duration]
   - Tasks completed: [count]
   - Success rate: [percentage]

   ### Recommendations:
   - [Future improvement 1]
   - [Future improvement 2]
   ```

## Communication Patterns

### Agent-to-Agent Handoff

**Context Transfer**:
```markdown
Agent A completes, produces output X

Message to Agent B:
- Task for Agent B
- Context: "Previous agent (A) produced X. Use this as input."
- Specific instructions for B
- Success criteria for B
```

**Example**:
```markdown
feature-implementer produces new module at src/batch.rs

Message to test-runner:
"Test the batch processing feature implemented in src/batch.rs.
The feature-implementer has created:
- Batch update functionality
- Transaction handling
- Error recovery

Please run all tests and verify the implementation works correctly."
```

### Synchronization Points

**Parallel Convergence**:
```markdown
Agents A, B, C all complete in parallel

Synchronization:
1. Wait for all three to complete
2. Collect outputs from A, B, C
3. Validate each output
4. If all valid → proceed
5. If any invalid → handle error
```

**Quality Gates**:
```markdown
After each phase:
1. Validate all outputs from phase
2. Check success criteria
3. Decision: proceed / retry / adjust / abort
4. If proceed → start next phase
```

## Error Handling & Recovery

### Agent Failure Scenarios

**Scenario 1: Agent Reports Failure**:
```markdown
Response:
1. Analyze failure reason
2. Determine if retryable
3. If yes → retry with adjusted parameters
4. If no → find alternative approach
5. Update plan accordingly
```

**Scenario 2: Quality Gate Fails**:
```markdown
Response:
1. Identify which criterion failed
2. Stop dependent tasks
3. Diagnose root cause
4. Apply fix or adjustment
5. Re-execute failed phase
6. Re-validate
```

**Scenario 3: Blocked Dependency**:
```markdown
Response:
1. Identify blocking task
2. Check if can work around
3. Consider reordering if possible
4. Communicate blockage
5. Adjust plan to minimize impact
```

### Recovery Strategies

**Retry**:
- Appropriate for transient failures
- Limit retries (max 2-3)
- Adjust parameters if repeating

**Alternative Approach**:
- Different agent for same task
- Different technical approach
- Simplified version of task

**Plan Adjustment**:
- Remove optional tasks
- Simplify requirements
- Change coordination strategy

**Graceful Degradation**:
- Partial completion acceptable
- Document what was achieved
- Plan for completing remainder later

## Coordination Metrics

### Efficiency Metrics

**Execution Time**:
- Parallel: Time = max(agent times)
- Sequential: Time = sum(agent times)
- Hybrid: Time = sum(sequential phases) where each phase time = max(parallel agents)

**Resource Utilization**:
- Agents active / Total agents available
- Minimize idle time
- Maximize parallelization

**Throughput**:
- Tasks completed / Total time
- Higher with parallel execution

### Quality Metrics

**Success Rate**:
- Tasks completed successfully / Total tasks
- Should be >95%

**Quality Gate Passage**:
- Quality gates passed / Total gates
- Should be 100%

**Rework Rate**:
- Tasks requiring retry / Total tasks
- Should be <10%

### Coordination Effectiveness

**Communication Efficiency**:
- Clear agent instructions
- Minimal clarification needed
- Smooth handoffs

**Plan Accuracy**:
- Actual execution matches plan
- Few adjustments needed
- Predictable outcomes

## Best Practices

### DO:

✓ **Choose appropriate strategy** for task dependencies
✓ **Match agents to tasks** based on capabilities
✓ **Validate at quality gates** before proceeding
✓ **Provide clear context** in agent handoffs
✓ **Monitor progress** during execution
✓ **Handle errors gracefully** with recovery
✓ **Aggregate results** comprehensively
✓ **Learn from execution** for future improvements

### DON'T:

✗ Force parallel execution when dependencies exist
✗ Assign tasks to inappropriate agents
✗ Skip quality validation
✗ Proceed after failed quality gates
✗ Provide insufficient context to agents
✗ Ignore agent failures
✗ Report incomplete results

## Examples

### Example 1: Parallel Code Review & Testing

```markdown
Task: "Verify code quality and correctness before release"

Strategy: Parallel (independent checks)

Plan:
├─ code-reviewer: Review code quality
└─ test-runner: Execute full test suite

Execution:
[Single message with two Task tool calls]

Quality Gate:
- Code review: No critical issues
- Tests: All passing

Result:
✓ Code quality approved (clippy, fmt pass)
✓ All tests passing (45/45)
→ Ready for release
```

### Example 2: Sequential Feature Development

```markdown
Task: "Add new caching feature with tests and docs"

Strategy: Sequential (dependencies)

Plan:
1. feature-implementer: Build caching feature
   Success: Implementation complete, compiles

2. test-runner: Test caching functionality
   Context: Implementation from step 1
   Success: All tests pass

3. code-reviewer: Review implementation
   Context: Implementation + test results
   Success: Quality standards met

Execution:
[Three sequential messages, each waits for previous]

Quality Gates:
- After step 1: Code compiles, basic functionality works
- After step 2: Tests pass, coverage adequate
- After step 3: Quality approved

Result:
✓ Feature implemented (src/cache.rs)
✓ Tests passing (8 new tests)
✓ Quality approved
→ Ready to merge
```

### Example 3: Swarm Debugging

```markdown
Task: "System slow - diagnose and fix"

Strategy: Swarm (multiple perspectives)

Plan:

Phase 1: Parallel Investigation
├─ debugger: Profile runtime performance
├─ code-reviewer: Analyze code efficiency
└─ test-runner: Run performance benchmarks

Phase 2: Synthesis
- Analyze collective findings
- Identify root cause

Phase 3: Resolution
└─ refactorer: Apply fix based on findings

Execution:
[Phase 1: Single message, 3 Task tools]
[Wait for all, synthesize]
[Phase 2: Analysis]
[Phase 3: Single message, 1 Task tool]

Findings:
- Debugger: N+1 query pattern
- Code-reviewer: Missing database index
- Test-runner: 850ms average query time

Root Cause: Missing index + inefficient queries

Resolution:
- Added database index
- Batched queries
- Performance: 850ms → 45ms

Result: ✓ Performance issue resolved
```

### Example 4: Hybrid Multi-Phase Project

```markdown
Task: "Implement, test, optimize, and document new feature"

Strategy: Hybrid (multi-phase, mixed execution)

Plan:

Phase 1: Analysis [Parallel]
├─ code-reviewer: Analyze existing architecture
└─ test-runner: Run baseline benchmarks

Phase 2: Implementation [Sequential]
└─ feature-implementer: Build feature
   Context: Architecture analysis

Phase 3: Testing [Sequential]
└─ test-runner: Test implementation
   Context: New feature code

Phase 4: Optimization [Conditional]
└─ refactorer: Optimize if benchmarks show issues
   Context: Performance test results

Phase 5: Final Validation [Parallel]
├─ test-runner: Full test suite
└─ code-reviewer: Final quality check

Execution:
[Phase 1: Parallel]
Gate: Analysis complete, approach validated

[Phase 2: Sequential]
Gate: Feature implemented, compiles

[Phase 3: Sequential]
Gate: Tests pass, coverage good

[Phase 4: Conditional - SKIPPED (performance good)]

[Phase 5: Parallel]
Gate: All quality criteria met

Result:
✓ Feature complete
✓ Tests passing
✓ Performance acceptable
✓ Quality approved
→ Ready for production
```

## Integration with GOAP Agent

The GOAP agent uses this skill as its core coordination engine:

1. **Decompose task** (task-decomposition skill)
2. **Select coordination strategy** (this skill)
3. **Assign agents to tasks** (this skill)
4. **Execute coordination** (this skill + parallel-execution skill)
5. **Validate and report** (this skill)

## Summary

Effective agent coordination requires:

- **Right strategy** for the task structure
- **Right agents** for each sub-task
- **Clear communication** and context transfer
- **Quality validation** at key checkpoints
- **Graceful error handling** and recovery
- **Comprehensive result synthesis**

By applying these coordination patterns, complex multi-agent workflows achieve:
- Maximum efficiency (optimal parallelization)
- High quality (validation gates)
- Reliability (error handling)
- Transparency (clear reporting)

Use this skill to coordinate any multi-agent workflow effectively.
