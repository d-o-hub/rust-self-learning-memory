# GOAP Patterns

## Pattern 1: Research → Decide → Implement → Validate (Full Stack)

```
Task: Implement complex feature with architectural impact

Phase 0: Retrieve Context [Skills]
├─ Skill(command="context-retrieval")
│  Query: "similar feature implementations"
└─ Skill(command="episode-start")
   → Start tracking this coordination

Phase 1: Research [Parallel Skills + Agents]
├─ Skill(command="web-search-researcher")
├─ Skill(command="codebase-consolidation")
└─ Task(subagent_type="Explore")
Quality Gate: Architecture and requirements clear

Phase 2: Decision [Skill]
└─ Skill(command="analysis-swarm")
   → Multi-perspective architectural decision
Quality Gate: Architecture approved

Phase 3: Pre-Implementation Validation [Parallel Skills]
├─ Skill(command="architecture-validation")
├─ Skill(command="plan-gap-analysis")
└─ Skill(command="rust-code-quality")
Quality Gate: Design validated

Phase 4: Implementation [Parallel Agents]
├─ Task(subagent_type="feature-implementer") → Module A
├─ Task(subagent_type="feature-implementer") → Module B
└─ Task(subagent_type="feature-implementer") → Module C
Quality Gate: All modules implemented

Phase 5: Testing [Skills + Agents]
├─ Task(subagent_type="test-runner")
├─ Skill(command="test-fix")
└─ Skill(command="quality-unit-testing")
Quality Gate: All tests passing

Phase 6: Quality Validation [Parallel Skills + Agents]
├─ Skill(command="rust-code-quality")
├─ Skill(command="architecture-validation")
├─ Skill(command="plan-gap-analysis")
└─ Task(subagent_type="code-reviewer")
Quality Gate: Quality standards met

Phase 7: Build & CI [Skills]
├─ Skill(command="build-compile")
└─ Skill(command="github-workflows")
Quality Gate: Ready for merge

Phase 8: Learning [Skills]
└─ Skill(command="episode-complete")
   Score: High
   Patterns: Document successful strategies
```

## Pattern 2: Investigate → Diagnose → Fix → Verify

```
Phase 1: Investigate
  - debugger → Reproduce issue
  - Quality Gate: Issue reproduced

Phase 2: Diagnose
  - debugger → Root cause analysis
  - Quality Gate: Root cause identified

Phase 3: Fix
  - refactorer → Apply fix
  - Quality Gate: Fix implemented

Phase 4: Verify
  - test-runner → Regression tests
  - Quality Gate: Tests pass
```

## Pattern 3: Audit → Improve → Validate

```
Phase 1: Audit
  - code-reviewer → Quality audit
  - Quality Gate: Issues identified

Phase 2 (Swarm): Improve
  - Multiple refactorer agents
  - Work queue: [issue list]
  - Quality Gate: All issues addressed

Phase 3: Validate
  - test-runner → Full test suite
  - code-reviewer → Final check
  - Quality Gate: Quality targets met
```

## Optimization Techniques

- **Critical path optimization**: Parallelize non-critical-path tasks
- **Resource pooling**: Share agents across similar tasks
- **Incremental delivery**: Complete and validate incrementally
- **Adaptive strategy**: Switch strategies based on progress

## Best Practices

### DO:
✓ Break tasks into atomic, testable units
✓ Define clear quality gates between phases
✓ Match agent capabilities to task requirements
✓ Monitor progress and validate incrementally
✓ Document decisions and rationale
✓ Learn from execution for future planning
✓ Use parallel execution where safe

### DON'T:
✗ Create monolithic tasks (break them down)
✗ Skip quality gates (leads to failures)
✗ Assume tasks are independent
✗ Ignore agent failures
✗ Over-complicate simple tasks
✗ Under-estimate coordination overhead
✗ Forget to aggregate results
