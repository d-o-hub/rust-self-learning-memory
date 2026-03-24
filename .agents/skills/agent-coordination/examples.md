# Agent Coordination Examples

## Example 1: Parallel Code Review

**Scenario**: Review a PR with multiple aspects in parallel.

```yaml
strategy: parallel
agents:
  - name: reviewer-style
    type: code-reviewer
    focus: code style, formatting
  - name: reviewer-security
    type: code-reviewer
    focus: security vulnerabilities
  - name: reviewer-performance
    type: code-reviewer
    focus: performance issues
synthesis:
  combine: all
  require: all_pass
```

**Execution**:
1. Launch all three reviewers in parallel
2. Each reviews independently
3. Combine results
4. All must pass for overall approval

---

## Example 2: Sequential Feature Implementation

**Scenario**: Implement a new feature with dependencies.

```yaml
strategy: sequential
steps:
  - name: research
    type: codebase-analyzer
    task: understand existing patterns
  - name: design
    type: system-architect
    task: design implementation approach
  - name: implement
    type: feature-implementer
    task: write code
    depends_on: [research, design]
  - name: test
    type: test-runner
    task: verify implementation
    depends_on: [implement]
  - name: review
    type: code-reviewer
    task: code quality check
    depends_on: [implement]
```

**Execution**:
1. Research existing codebase
2. Design approach based on research
3. Implement feature (after research + design)
4. Run tests and review in parallel (after implementation)

---

## Example 3: Swarm Analysis

**Scenario**: Complex architectural decision requiring multiple perspectives.

```yaml
strategy: swarm
agents:
  - name: RYAN
    type: analysis-swarm
    persona: methodical, detailed
    focus: edge cases, completeness
  - name: FLASH
    type: analysis-swarm
    persona: fast, practical
    focus: quick wins, efficiency
  - name: SOCRATES
    type: analysis-swarm
    persona: questioning, critical
    focus: assumptions, trade-offs
decision:
  method: consensus
  fallback: majority
```

**Execution**:
1. Each persona analyzes independently
2. Present findings
3. Synthesize into balanced recommendation
4. Require consensus for critical decisions

---

## Example 4: Iterative Quality Refinement

**Scenario**: Fix failing tests with quality gates.

```yaml
strategy: iterative
agent: loop-agent
max_iterations: 5
quality_gates:
  - cargo test
  - ./scripts/code-quality.sh clippy --workspace
  - ./scripts/code-quality.sh fmt
on_failure: refine_and_retry
on_success: complete
```

**Execution**:
1. Run tests
2. If fail: analyze, fix, retry (up to max iterations)
3. If pass: run clippy
4. If clippy warnings: fix, retry
5. If all gates pass: complete

---

## Example 5: Hybrid CI/CD Pipeline

**Scenario**: Full CI/CD workflow with mixed strategies.

```yaml
strategy: hybrid
phases:
  - name: validate
    strategy: parallel
    agents:
      - type: code-quality
        task: fmt + clippy
      - type: yaml-validator
        task: workflow validation
  - name: build
    strategy: sequential
    depends_on: [validate]
    agents:
      - type: build-compile
        task: build all targets
  - name: test
    strategy: parallel
    depends_on: [build]
    agents:
      - type: test-runner
        task: unit tests
      - type: test-runner
        task: integration tests
  - name: deploy
    strategy: sequential
    depends_on: [test]
    gates: [all_tests_pass]
    agents:
      - type: production-validator
        task: validate deployment readiness
```

**Execution**:
1. Parallel validation (format + yaml)
2. Sequential build (after validation)
3. Parallel tests (after build)
4. Sequential deploy (after tests pass)

---

## Example 6: Debugging Workflow

**Scenario**: Debug a failing test with systematic approach.

```yaml
strategy: sequential
steps:
  - name: reproduce
    type: test-runner
    task: isolate failing test
  - name: diagnose
    type: debugger
    task: identify root cause
    input: reproduce.output
  - name: fix
    type: test-fix
    task: implement fix
    input: diagnose.output
  - name: verify
    type: test-runner
    task: confirm fix works
    gates: [test_passes]
```

---

## Pattern Summary

| Pattern | Best For | Complexity |
|---------|----------|------------|
| Parallel | Independent tasks | Low |
| Sequential | Dependencies | Low |
| Swarm | Critical decisions | Medium |
| Iterative | Quality refinement | Medium |
| Hybrid | Complex pipelines | High |

## See Also

- [strategies.md](./strategies.md) - Execution patterns
- [skills-agents.md](./skills-agents.md) - Skills vs Agents
- [quality-gates.md](./quality-gates.md) - Validation checkpoints