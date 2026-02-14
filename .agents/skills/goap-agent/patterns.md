# GOAP Patterns

## Pattern 1: ADR → Research → Decide → Implement → Validate (Full Stack)

```
Task: Implement complex feature with architectural impact

Phase 0: ADR Discovery [MANDATORY]
├─ Read: ls plans/adr/ADR-*.md
├─ Identify: Relevant ADRs for task domain
├─ Check: ADR-022 (GOAP), ADR-023 (CI/CD), etc.
└─ Note: Architectural constraints and decisions
Quality Gate: Relevant ADRs identified and reviewed

Phase 1: Retrieve Context [Skills]
├─ Skill(command="context-retrieval")
│  Query: "similar feature implementations"
└─ Skill(command="episode-start")
   → Start tracking this coordination

Phase 2: Research [Parallel Skills + Agents]
├─ Skill(command="web-search-researcher")
├─ Skill(command="codebase-consolidation")
└─ Task(subagent_type="Explore")
Quality Gate: Architecture and requirements clear

Phase 3: Decision [Skill + ADR Check]
├─ Skill(command="analysis-swarm")
│  → Multi-perspective architectural decision
└─ Verify: Decision aligns with existing ADRs
Quality Gate: Architecture approved, ADR-compliant

Phase 4: Pre-Implementation Validation [Parallel Skills]
├─ Skill(command="architecture-validation")
├─ Skill(command="plan-gap-analysis")
└─ Skill(command="rust-code-quality")
Quality Gate: Design validated

Phase 5: Implementation [Parallel Agents]
├─ Task(subagent_type="feature-implementer") → Module A
├─ Task(subagent_type="feature-implementer") → Module B
└─ Task(subagent_type="feature-implementer") → Module C
Quality Gate: All modules implemented

Phase 6: Testing [Skills + Agents]
├─ Task(subagent_type="test-runner")
├─ Skill(command="test-fix")
└─ Skill(command="quality-unit-testing")
Quality Gate: All tests passing

Phase 7: Quality Validation [Parallel Skills + Agents]
├─ Skill(command="rust-code-quality")
├─ Skill(command="architecture-validation")
├─ Skill(command="plan-gap-analysis")
└─ Task(subagent_type="code-reviewer")
Quality Gate: Quality standards met

Phase 8: Build & CI [Skills]
├─ Skill(command="build-compile")
└─ Skill(command="github-workflows")
Quality Gate: Ready for merge

Phase 9: Update Plans & ADRs [Documentation]
├─ Update: plans/GOAP_EXECUTION_PLAN_*.md with results
├─ Create/Update: ADR if new architectural decisions made
└─ Document: Lessons learned and patterns discovered

Phase 10: Learning [Skills]
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

## Pattern 4: ADR-Driven Planning & Execution

**Use when**: Making architectural decisions or implementing features that impact system architecture

```
Task: Make architectural change or implement ADR-compliant feature

Phase 0: ADR Discovery & Analysis [CRITICAL]
├─ List: ls plans/adr/ADR-*.md
├─ Read: Relevant ADRs for the domain
├─ Check Status: Accepted/Implemented vs Deprecated
├─ Extract: Constraints, decisions, tradeoffs
└─ Decision: Create new ADR or follow existing?
Quality Gate: ADR context understood

Phase 1: Create/Update ADR [if needed]
├─ If new architectural decision required:
│  ├─ Create: plans/adr/ADR-XXX-[Short-Name].md
│  ├─ Follow template from ADR-022
│  ├─ Document: Context, Decision, Rationale, Consequences
│  └─ Status: Proposed → Accepted
└─ If following existing ADR:
   └─ Reference ADR in execution plan
Quality Gate: ADR created/updated and linked

Phase 2: Create Execution Plan in plans/
├─ Create: plans/GOAP_EXECUTION_PLAN_[DATE].md
├─ Link: Reference relevant ADRs
├─ Define: Tasks aligned with ADR constraints
└─ Set: Quality gates respecting ADR requirements
Quality Gate: Plan created with ADR references

Phase 3: Execute with ADR Compliance
├─ For each task:
│  ├─ Check: Does this align with ADR constraints?
│  ├─ Execute: Using appropriate agents/skills
│  └─ Validate: ADR compliance at each quality gate
└─ Monitor: Progress against ADR requirements
Quality Gate: All tasks complete, ADR-compliant

Phase 4: Update ADR & Plans
├─ Update: ADR with implementation details
├─ Update: Execution plan with completion status
├─ Document: Any deviations from original ADR
└─ Archive: Old plans if superseded
Quality Gate: Documentation current and accurate

Phase 5: Validate Architecture
├─ Skill(command="architecture-validation")
├─ Verify: Implementation matches ADR decisions
└─ Check: No architectural drift
Quality Gate: Architecture validated
```

### ADR-Driven Best Practices

**Before Planning**:
- ✓ Always check `plans/adr/` for relevant decisions
- ✓ Read ADRs that match your task domain
- ✓ Note which ADRs are Accepted vs Proposed
- ✓ Understand constraints and tradeoffs documented

**During Execution**:
- ✓ Reference ADRs in task descriptions
- ✓ Validate ADR compliance at quality gates
- ✓ Document any conflicts with existing ADRs
- ✓ Update ADRs if decisions change during implementation

**After Execution**:
- ✓ Update ADR with implementation status
- ✓ Link execution plan to ADR
- ✓ Document lessons learned
- ✓ Archive or mark plans as complete

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

## Architecture Reference

- **ADR-022**: [GOAP Agent System ADR](../../../plans/adr/ADR-022-GOAP-Agent-System.md) - Architecture decision and rationale
