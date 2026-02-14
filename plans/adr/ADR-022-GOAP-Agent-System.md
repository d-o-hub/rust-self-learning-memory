# ADR-022: GOAP Agent System for Multi-Agent Coordination

**Status**: Accepted and Implemented
**Date**: 2026-02-10
**Context**: Complex multi-step tasks require intelligent planning and coordination of multiple specialized agents. Current ad-hoc approaches lack systematic task decomposition, dependency management, and execution strategies.

**Decision**: Implement Goal-Oriented Action Planning (GOAP) system with multi-agent coordination capabilities

## Alternatives Considered

1. **Ad-hoc Task Assignment (Previous)**
   - Pros: Simple, immediate execution
   - Cons: No planning, poor coordination, quality issues, no learning from past executions
   - **REJECTED**: Unsustainable for complex tasks

2. **Simple Sequential Pipeline**
   - Pros: Predictable execution order
   - Cons: No parallelism, slow for independent tasks, rigid structure
   - **REJECTED**: Too limiting for real-world complexity

3. **Manual Task Decomposition**
   - Pros: Human control over planning
   - Cons: Time-consuming, inconsistent, doesn't scale
   - **REJECTED**: Not automatable

4. **GOAP with Multi-Agent Coordination (Chosen)**
   - Pros: Intelligent planning, multiple execution strategies, quality gates, learnable patterns
   - Cons: Initial complexity, requires skill/agent inventory
   - **ACCEPTED**: Best balance of automation and quality

## Decision

**Implement GOAP Agent Skill System** with:
- 6-phase planning cycle (ANALYZE → DECOMPOSE → STRATEGIZE → COORDINATE → EXECUTE → SYNTHESIZE)
- 5 execution strategies (Parallel, Sequential, Swarm, Hybrid, Iterative)
- Agent capability matrix for optimal assignment
- Quality gates between phases
- Integration with episodic memory system

## Rationale

- **Systematic Planning**: GOAP provides structured approach to complex tasks
- **Scalability**: Handles 5+ step tasks across multiple domains
- **Flexibility**: Multiple execution strategies for different task types
- **Quality Assurance**: Built-in quality gates and validation
- **Learning Integration**: Episodic memory tracks coordination patterns
- **Ecosystem Integration**: Works with existing Task Agents and Skills

## Tradeoffs

- **Complexity**: ~1000 LOC across skill files
- **Learning Curve**: Understanding when to use which strategy
- **Overhead**: Planning phase adds initial latency
- **Maintenance**: Agent capability matrix needs updates as new agents added

## Consequences

- **Positive**: Consistent handling of complex multi-step tasks
- **Positive**: 2-4x speedup through parallel execution where applicable
- **Positive**: Higher quality through systematic quality gates
- **Positive**: Reusable patterns from episodic memory
- **Negative**: Initial planning overhead (acceptable for complex tasks)
- **Negative**: Requires maintaining agent capability documentation
- **Negative**: Learning curve for optimal strategy selection

## Implementation Status

✅ **COMPLETED**

**Core Components**:
- `SKILL.md` - Entry point and quick reference
- `methodology.md` - 6-phase planning cycle
- `agents.md` - Agent capability matrix
- `skills.md` - Available skills reference
- `execution-strategies.md` - Parallel, Sequential, Swarm, Hybrid patterns
- `patterns.md` - Common GOAP execution patterns
- `examples.md` - Complete workflow examples

**Integration Points**:
- Works with `task-decomposition` skill for breaking down goals
- Integrates with `episode-start`, `episode-log-steps`, `episode-complete` for learning (documented capability, not yet demonstrated in GOAP executions)
- Uses `context-retrieval` skill for past coordination patterns
- Compatible with all existing Task Agents (feature-implementer, debugger, refactorer, etc.)

**Files Affected**:
- `.agents/skills/goap-agent/SKILL.md` (new, ~75 LOC)
- `.agents/skills/goap-agent/methodology.md` (new, ~190 LOC)
- `.agents/skills/goap-agent/agents.md` (new, ~95 LOC)
- `.agents/skills/goap-agent/skills.md` (new, ~95 LOC)
- `.agents/skills/goap-agent/execution-strategies.md` (new, ~270 LOC)
- `.agents/skills/goap-agent/patterns.md` (new, ~215 LOC)
- `.agents/skills/goap-agent/examples.md` (new, ~190 LOC)

**Update History**:
- 2026-02-14: Added ADR-driven planning and execution workflow to all skill files (+264 LOC total)
  - SKILL.md: Added ADR Integration Workflow section
  - methodology.md: Added ADR Discovery phase
  - patterns.md: Added Pattern 4 (ADR-Driven Planning) and best practices
  - execution-strategies.md: Added ADR Compliance section
  - examples.md: Added Example 5 (ADR-Driven CI/CD Remediation)

**Success Metrics**:
- Complex tasks (5+ steps) completed successfully
- Quality gates passing >90% of the time
- Execution time reduced through parallel strategies
- Patterns learned and reused from episodic memory

**Next Steps**: None (feature complete, ADR integration added 2026-02-14)

---

## Usage Example

```markdown
## Task: Refactor storage layer across 3 crates

### Phase 1: ANALYZE
- Goal: Extract storage interface, implement for Turso and redb
- Complexity: Complex (3 crates, dependencies)
- Quality: Tests must pass, coverage >90%

### Phase 2: DECOMPOSE
1. Design storage interface (architecture)
2. Implement for memory-core (base types)
3. Implement for memory-storage-turso
4. Implement for memory-storage-redb
5. Integration testing
6. Documentation

### Phase 3: STRATEGIZE
- Use HYBRID: Sequential design phase → Parallel implementation → Sequential integration

### Phase 4: COORDINATE
- Phase 1 (Sequential): goap-agent → Design
- Phase 2 (Parallel): 3 feature-implementer agents → Implementation
- Phase 3 (Sequential): test-runner + code-reviewer → Validation

### Phase 5: EXECUTE
- Run with quality gates between phases
- Monitor progress, handle failures

### Phase 6: SYNTHESIZE
- Aggregate results
- Validate all tests pass
- Complete episode for learning
```

---

**References**:
- `goap-agent/methodology.md` - Detailed planning methodology
- `goap-agent/execution-strategies.md` - Strategy selection guide
- `goap-agent/agents.md` - Agent capabilities reference
- `.opencode/skill/goap-agent/SKILL.md` - Main skill documentation
