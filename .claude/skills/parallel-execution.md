# Parallel Execution

Execute multiple independent tasks simultaneously using parallel agent coordination to maximize throughput and minimize total execution time.

## Purpose

Enable efficient parallel execution of independent tasks by launching multiple agents concurrently and managing their synchronized completion.

## When to Use

- Multiple independent tasks (no dependencies)
- Tasks that can benefit from concurrent execution
- Maximizing throughput is a priority
- Available agents for parallel work
- Results can be aggregated after completion

## Core Concepts

### Independence

**Tasks are independent when**:
- No data dependencies (one doesn't need other's output)
- No resource conflicts (different files, databases)
- No ordering requirements (either can complete first)
- Failures are isolated (one failing doesn't block others)

**Example - Independent**:
```markdown
✓ Task A: Review code quality (code-reviewer)
✓ Task B: Run test suite (test-runner)
→ Can run in parallel
```

**Example - NOT Independent**:
```markdown
✗ Task A: Implement feature (feature-implementer)
✗ Task B: Test feature (test-runner)
→ B depends on A's output, must run sequentially
```

### Concurrency

**Parallel Agent Execution**:
- Multiple agents work simultaneously
- Each agent has its own context and resources
- No shared state between agents
- Results collected independently

**Implementation**:
```markdown
Single message with multiple Task tool calls:

Message:
  - Task tool call #1 → Agent A
  - Task tool call #2 → Agent B
  - Task tool call #3 → Agent C

All three agents start and run concurrently.
```

### Synchronization

**Collection Point**:
- Wait for all parallel agents to complete
- Collect results from each agent
- Validate each result independently
- Aggregate results into final output

## Parallel Execution Process

### Step 1: Identify Independent Tasks

**From Decomposition**:
```markdown
Task List:
1. Review code quality
2. Run test suite
3. Check documentation
4. Run performance benchmarks

Dependency Analysis:
- All tasks are independent
- All read existing code/docs
- No writes that conflict
- Can all run simultaneously

Conclusion: All 4 tasks can execute in parallel
```

**Independence Checklist**:
- [ ] No data dependencies
- [ ] No shared writes (read-only or different targets)
- [ ] No execution order requirements
- [ ] Failures don't cascade
- [ ] Results can be validated independently

### Step 2: Agent Assignment

**Match Tasks to Agents**:
```markdown
Task 1: Review code quality
  → Agent: code-reviewer
  → Reason: Specialized in quality analysis

Task 2: Run test suite
  → Agent: test-runner
  → Reason: Specialized in test execution

Task 3: Check documentation
  → Agent: code-reviewer
  → Reason: Can check docs as part of review

Task 4: Run benchmarks
  → Agent: test-runner
  → Reason: Can run performance tests
```

**Agent Availability Check**:
- Ensure sufficient agents available
- Check for agent specialization overlap
- Consider agent workload balance

### Step 3: Prepare Execution Context

**For Each Agent**:
```markdown
Agent: [name]
Task: [clear description]
Context: [any needed context]
Success Criteria: [how to validate]
Expected Output: [what agent will produce]
Failure Handling: [what if task fails]
```

**Example**:
```markdown
Agent: code-reviewer
Task: Review all code in src/ for quality issues
Context: Focus on recent changes, check AGENTS.md compliance
Success Criteria: Report lists any issues found, or states "no issues"
Expected Output: Review report with findings
Failure Handling: Continue with other tasks, partial review acceptable
```

### Step 4: Launch Parallel Execution

**Critical**: Use single message with multiple Task tool calls

**Correct**:
```markdown
Send one message containing:
<Task tool call>
  agent: code-reviewer
  task: Review code quality
</Task tool call>
<Task tool call>
  agent: test-runner
  task: Run test suite
</Task tool call>
<Task tool call>
  agent: test-runner
  task: Run benchmarks
</Task tool call>

All agents start simultaneously.
```

**Incorrect**:
```markdown
Message 1: Task tool → code-reviewer
[wait]
Message 2: Task tool → test-runner
[wait]
Message 3: Task tool → test-runner

This is sequential, NOT parallel!
```

### Step 5: Monitor Execution

**Track Progress**:
- Monitor each agent independently
- Note completion as agents finish
- Collect outputs as available
- Track any errors or failures

**Monitoring Template**:
```markdown
Parallel Execution Status:

Agent 1 (code-reviewer): In Progress
  Task: Code quality review
  Status: Analyzing src/ directory

Agent 2 (test-runner): Completed ✓
  Task: Test suite
  Result: 45/45 tests passed

Agent 3 (test-runner): In Progress
  Task: Benchmarks
  Status: Running performance tests
```

### Step 6: Collect & Validate Results

**As Each Agent Completes**:
1. Collect output
2. Validate against success criteria
3. Check for errors
4. Mark as complete or failed

**Collection Template**:
```markdown
Agent: [name]
Task: [description]
Status: ✓ Completed / ✗ Failed
Output: [summary]
Quality: ✓ Meets criteria / ✗ Issues found
Issues: [any problems]
```

### Step 7: Aggregate Results

**Synthesize Final Output**:
```markdown
## Parallel Execution Results

### Completed Tasks:
1. ✓ Code quality review (code-reviewer)
   - Result: 3 minor issues found
   - Files: src/storage.rs:45, src/patterns.rs:78, src/lib.rs:12

2. ✓ Test suite (test-runner)
   - Result: All tests passing (45/45)
   - Coverage: 87%

3. ✓ Performance benchmarks (test-runner)
   - Result: All benchmarks within acceptable range
   - Episode retrieval: 45ms avg
   - Pattern extraction: 23ms avg

### Overall Status: ✓ Success (with minor issues)

### Summary:
Code is in good shape with 3 minor issues to address.
All tests passing, performance is acceptable.

### Recommendations:
- Fix 3 minor code quality issues
- Consider increasing test coverage to 90%
```

## Execution Patterns

### Pattern 1: Homogeneous Parallel

**All agents same type, different inputs**:
```markdown
Use Case: Test multiple modules in parallel

Execution:
├─ test-runner: Test memory-core
├─ test-runner: Test memory-storage-turso
└─ test-runner: Test memory-storage-redb

Single message, 3 Task tool calls to test-runner
Each with different module context
```

### Pattern 2: Heterogeneous Parallel

**Different agent types, related task**:
```markdown
Use Case: Comprehensive code check

Execution:
├─ code-reviewer: Quality analysis
├─ test-runner: Test execution
└─ debugger: Performance profiling

Single message, 3 Task tool calls to different agents
All analyzing same codebase from different angles
```

### Pattern 3: Parallel with Convergence

**Parallel execution → Single synthesis**:
```markdown
Phase 1: Parallel Investigation
├─ debugger: Profile performance
├─ code-reviewer: Analyze efficiency
└─ test-runner: Run benchmarks

[All complete]

Phase 2: Synthesis
└─ GOAP: Combine findings, identify root cause

Single parallel phase, then local synthesis
```

### Pattern 4: Nested Parallel

**Multiple rounds of parallel execution**:
```markdown
Round 1: Initial Analysis
├─ code-reviewer: Review structure
└─ test-runner: Test coverage check

[Both complete]

Round 2: Deep Dive (based on Round 1 findings)
├─ refactorer: Optimize identified hotspots
├─ feature-implementer: Add missing tests
└─ code-reviewer: Review new code

Each round is parallel within itself
Rounds are sequential
```

## Synchronization Strategies

### Wait for All (AND)

**Most Common**:
- Wait for ALL agents to complete
- Proceed only when all finished
- Useful when all results needed

```markdown
Agents: [A, B, C] all running in parallel

Synchronization:
- Wait: A complete AND B complete AND C complete
- Then: Aggregate all results
- Proceed: With complete picture
```

### Wait for Any (OR)

**Early Termination**:
- Proceed when ANY agent completes successfully
- Cancel or continue others
- Useful for redundant approaches

```markdown
Agents: [A, B] trying different approaches

Synchronization:
- Wait: A complete OR B complete
- Then: Use first successful result
- Optional: Let others finish or cancel
```

### Wait for Threshold

**Partial Completion**:
- Proceed when N out of M agents complete
- Useful for resilience

```markdown
Agents: [A, B, C, D, E] (5 agents)

Synchronization:
- Wait: At least 3 complete
- Then: Proceed with available results
- Handle: Missing results gracefully
```

## Resource Management

### Agent Pool Management

**Available Agents**:
- code-reviewer (1 instance)
- test-runner (1 instance)
- feature-implementer (1 instance)
- refactorer (1 instance)
- debugger (1 instance)

**Parallelization Limit**:
- Maximum 5 agents simultaneously (one of each type)
- Can reuse agent type if tasks are truly independent

### Workload Balancing

**Distribute Evenly**:
```markdown
Tasks: [T1, T2, T3, T4, T5, T6]
Agents: [A, B, C]

Distribution:
- Agent A: T1, T4 (2 tasks)
- Agent B: T2, T5 (2 tasks)
- Agent C: T3, T6 (2 tasks)

Balanced workload
```

**Avoid Imbalance**:
```markdown
❌ Poor Distribution:
- Agent A: T1, T2, T3, T4 (4 tasks)
- Agent B: T5 (1 task)
- Agent C: T6 (1 task)

Agent A is bottleneck
```

### Resource Conflicts

**Identify Conflicts**:
- File writes to same file → NOT parallel
- Database writes to same records → NOT parallel
- Shared mutable state → NOT parallel

**Resolution**:
- Serialize conflicting tasks
- Use different resources
- Coordinate writes through locks

## Error Handling

### Independent Failures

**Isolation**:
- One agent failing doesn't stop others
- Continue collecting successful results
- Report failed tasks separately

```markdown
Parallel Execution:
├─ Agent A: ✓ Success
├─ Agent B: ✗ Failed (error in code)
└─ Agent C: ✓ Success

Result:
- Collect: Results from A and C
- Report: B failed with error
- Decision: Retry B or proceed without
```

### Partial Success Handling

**Strategies**:

**1. Fail Fast**:
- If any agent fails, stop and report
- Don't wait for others
- Useful for critical tasks

**2. Best Effort**:
- Collect all successful results
- Report failures separately
- Proceed with partial results

**3. Retry Failed**:
- Let successful agents complete
- Retry only failed agents
- Aggregate all results

### Timeout Management

**Set Timeouts**:
```markdown
Each agent task should have timeout

Agent A: Timeout 5 minutes
Agent B: Timeout 10 minutes
Agent C: Timeout 3 minutes

Overall timeout: max(A, B, C) + buffer = 12 minutes
```

**Handle Timeouts**:
- Log timeout occurrence
- Decide: retry, skip, or abort
- Don't wait indefinitely

## Performance Optimization

### Speedup Calculation

**Ideal Speedup**:
```
Sequential time = T1 + T2 + T3 + ... + Tn
Parallel time = max(T1, T2, T3, ..., Tn)

Speedup = Sequential time / Parallel time
```

**Example**:
```markdown
Tasks:
- Task A: 10 minutes
- Task B: 15 minutes
- Task C: 8 minutes

Sequential: 10 + 15 + 8 = 33 minutes
Parallel: max(10, 15, 8) = 15 minutes

Speedup: 33 / 15 = 2.2x faster
```

### Optimal Parallelization

**Identify Bottlenecks**:
```markdown
Tasks:
- Quick tasks: A (1 min), B (2 min)
- Slow task: C (30 min)

Parallel execution time: 30 min (limited by C)

Optimization:
- Can C be decomposed further?
- Can C be optimized?
- Start C first, then parallelize A and B
```

### Diminishing Returns

**Amdahl's Law**:
```
Not all tasks can be parallelized
Serial portion limits maximum speedup

Example:
- 80% parallelizable, 20% serial
- Even with infinite agents, max speedup = 5x
```

## Best Practices

### DO:

✓ **Verify independence** before parallelizing
✓ **Use single message** with multiple Task tool calls
✓ **Balance workload** across agents
✓ **Set appropriate timeouts** for each task
✓ **Handle failures gracefully** (isolation)
✓ **Validate each result** independently
✓ **Aggregate comprehensively** at the end
✓ **Document parallel strategy** in plan

### DON'T:

✗ Parallelize dependent tasks
✗ Send sequential messages thinking they're parallel
✗ Overload single agent while others idle
✗ Skip validation because "it's parallel"
✗ Assume all will succeed
✗ Ignore agent failures
✗ Proceed without all results (unless by design)

## Examples

### Example 1: Simple Parallel Review

```markdown
Task: "Check code quality and run tests"

Analysis:
- Independent tasks (no dependencies)
- Different agents (code-reviewer, test-runner)
- Both read-only operations
- Can run simultaneously

Plan:
├─ code-reviewer: Review code quality
└─ test-runner: Run test suite

Execution:
[Single message with 2 Task tool calls]

Results:
- code-reviewer: 2 issues found ✓
- test-runner: 45/45 tests pass ✓

Speedup: 2x (both completed in 5 min vs 10 min sequential)
```

### Example 2: Multi-Module Testing

```markdown
Task: "Test all crates in workspace"

Analysis:
- 3 independent crates
- Same agent type, different inputs
- No shared state
- Fully parallel

Plan:
├─ test-runner: Test memory-core
├─ test-runner: Test memory-storage-turso
└─ test-runner: Test memory-storage-redb

Execution:
[Single message with 3 Task tool calls to test-runner]

Results:
- memory-core: 25/25 tests pass ✓
- memory-storage-turso: 15/15 tests pass ✓
- memory-storage-redb: 10/10 tests pass ✗ (1 failure)

Overall: 50/51 tests pass
Action: Investigate redb test failure

Speedup: 3x (3 crates tested in parallel)
```

### Example 3: Comprehensive Quality Check

```markdown
Task: "Pre-release quality validation"

Analysis:
- 4 independent validation tasks
- All read-only checks
- Different aspects of quality
- Maximum parallelization

Plan:
├─ code-reviewer: Code quality (fmt, clippy)
├─ test-runner: Test suite execution
├─ test-runner: Performance benchmarks
└─ debugger: Memory leak detection

Execution:
[Single message with 4 Task tool calls]

Results:
- Code quality: ✓ All checks pass
- Test suite: ✓ 45/45 pass
- Benchmarks: ✓ Performance acceptable
- Memory: ✓ No leaks detected

Overall: ✓ Ready for release

Speedup: 4x (all checks in 15 min vs 60 min sequential)
```

### Example 4: Parallel Investigation with Synthesis

```markdown
Task: "Diagnose performance issue"

Analysis:
- Investigation from multiple angles
- Swarm pattern
- Parallel investigation → Sequential synthesis

Plan:

Phase 1: Parallel Investigation
├─ debugger: Runtime profiling
├─ code-reviewer: Code efficiency analysis
└─ test-runner: Performance benchmarks

Phase 2: Synthesis (Sequential)
└─ Analyze all findings, identify root cause

Execution:
[Phase 1: Single message, 3 Task tool calls]

Phase 1 Results:
- Debugger: N+1 query pattern detected
- Code-reviewer: Missing database index
- Test-runner: 850ms average query time

Phase 2 Synthesis:
Root Cause: Unindexed database queries in hot path

Recommendation: Add index, batch queries

Speedup: 3x for investigation phase
```

## Integration with Agent Coordination

Parallel execution is one coordination strategy:

```markdown
Coordination Strategy Selection:
├─ Independent tasks → Use parallel-execution (this skill)
├─ Dependent tasks → Use sequential coordination
├─ Complex mix → Use hybrid coordination
└─ Multiple perspectives → Use swarm coordination (with parallel)
```

## Metrics & Monitoring

### Execution Metrics

**Time Metrics**:
- Total parallel execution time
- Per-agent execution time
- Speedup vs sequential
- Idle time (agent waiting)

**Throughput Metrics**:
- Tasks completed per time unit
- Agent utilization rate
- Parallel efficiency

**Quality Metrics**:
- Success rate (per agent)
- Failed task count
- Retry count
- Validation pass rate

### Performance Analysis

**After Execution**:
```markdown
## Parallel Execution Analysis

Agents: 3
Tasks Completed: 3
Total Time: 15 minutes
Sequential Time (estimated): 33 minutes
Speedup: 2.2x

Agent Utilization:
- code-reviewer: 100% (15/15 min)
- test-runner: 100% (15/15 min)
- test-runner: 53% (8/15 min)

Efficiency: 84% (average utilization)

Bottleneck: test-runner task B (15 min)

Optimization Opportunity:
- Task B could potentially be decomposed further
```

## Summary

Parallel execution maximizes efficiency for independent tasks by:

- **Concurrent agent execution** (single message, multiple tools)
- **Independent task validation** (no cross-dependencies)
- **Synchronized result collection** (wait for completion)
- **Comprehensive aggregation** (synthesize final output)

Key to success:
- Verify true independence
- Use correct tool invocation pattern
- Handle failures gracefully
- Validate and aggregate comprehensively

When done correctly, parallel execution provides significant speedup while maintaining quality and reliability.
