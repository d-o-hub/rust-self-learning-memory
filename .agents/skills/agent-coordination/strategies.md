# Agent Coordination Strategies

## Execution Patterns

### 1. Parallel Execution

Run multiple independent agents simultaneously for maximum throughput.

```
When to use:
- Tasks have no dependencies
- Results can be aggregated
- Agents are available for concurrent work

Example:
- Run code-reviewer AND test-runner in parallel
- Both complete independently, results merged
```

### 2. Sequential Execution

Execute agents one after another, each depending on the previous.

```
When to use:
- Output of one agent feeds into the next
- Order matters for correctness
- Resource constraints prevent parallelism

Example:
- codebase-locator -> codebase-analyzer -> feature-implementer
```

### 3. Swarm Execution

Multiple agents work on the same problem from different angles.

```
When to use:
- Complex analysis requiring multiple perspectives
- Quality-critical work needing consensus
- Avoiding single-perspective blind spots

Example:
- analysis-swarm (RYAN, FLASH, SOCRATES personas)
- Each analyzes independently, results synthesized
```

### 4. Hybrid Execution

Combine patterns for complex workflows.

```
When to use:
- Multi-phase work with mixed dependencies
- Some tasks parallel, some sequential

Example:
- Phase 1: Parallel research (web-search-researcher + codebase-analyzer)
- Phase 2: Sequential implementation (feature-implementer)
- Phase 3: Parallel validation (code-reviewer + test-runner)
```

### 5. Iterative Execution (Loop Agent)

Repeat execution until quality criteria are met.

```
When to use:
- Quality thresholds required
- Work may need refinement
- Convergence is the goal

Example:
- loop-agent: implement -> test -> review -> refine
- Continues until all tests pass AND review approves
```

## Decision Matrix

| Scenario | Strategy | Rationale |
|----------|----------|-----------|
| Independent fixes | Parallel | Max speed |
| Dependency chain | Sequential | Correctness |
| Critical quality | Swarm | Perspective diversity |
| Complex workflow | Hybrid | Flexibility |
| Quality gates | Iterative | Convergence |

## Quality Gates in Coordination

Every coordination strategy should include quality gates:

1. **Pre-execution**: Verify prerequisites
2. **During execution**: Monitor progress
3. **Post-execution**: Validate results
4. **Integration**: Check cross-agent consistency

## See Also

- [skills-agents.md](./skills-agents.md) - When to use Skills vs Agents
- [quality-gates.md](./quality-gates.md) - Validation checkpoints
- [examples.md](./examples.md) - Real-world coordination examples