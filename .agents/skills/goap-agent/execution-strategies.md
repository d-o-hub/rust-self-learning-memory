# GOAP Execution Strategies

## Strategy Selection Guide

Choose the appropriate execution strategy based on task characteristics:

| Strategy | When to Use | Characteristics |
|----------|-------------|-----------------|
| **Parallel** | Independent tasks, speed critical | Maximum concurrency, complex coordination |
| **Sequential** | Dependent tasks, order matters | Simple, linear, easy to debug |
| **Swarm** | Many similar tasks | Distributed work, load balancing |
| **Hybrid** | Mixed requirements | Combines strategies, most flexible |

## ADR Compliance in All Strategies

**CRITICAL**: Every execution strategy must respect ADR constraints:

### Pre-Execution ADR Check
```markdown
Before choosing strategy:
1. List ADRs: ls plans/adr/ADR-*.md
2. Identify relevant: grep for topic/domain keywords
3. Read constraints: Note architectural boundaries
4. Validate strategy: Ensure chosen approach aligns with ADR
```

### ADR Validation During Execution
- **Quality Gate 1**: Strategy aligns with ADR decisions?
- **Quality Gate 2**: Implementation respects ADR constraints?
- **Quality Gate 3**: No architectural drift from ADR?

### ADR Documentation Post-Execution
- Update relevant ADRs with implementation details
- Reference ADRs in execution plans
- Document any ADR amendments or new decisions

## Parallel Execution Strategy

### When to Use
- Tasks are **independent** (no data dependencies)
- **Time-critical** delivery
- Resources available for concurrent execution
- Results can be aggregated

### Implementation Pattern
```markdown
**Parallel Execution:**
- Agent 1 → Task A (independent)
- Agent 2 → Task B (independent)
- Agent 3 → Task C (independent)
→ Aggregate results when all complete
```

### Example
```
Task: Implement multi-crate feature across memory-core, memory-storage-turso, memory-storage-redb

Parallel agents:
1. feature-implementer (Agent 1) → memory-core changes
2. feature-implementer (Agent 2) → memory-storage-turso changes
3. feature-implementer (Agent 3) → memory-storage-redb changes
4. test-runner (Agent 4) → Run tests after all complete

Benefits: 3x faster than sequential
```

### Considerations
- Higher coordination overhead
- Requires careful result aggregation
- Potential for merge conflicts
- Needs quality gates before integration

## Sequential Execution Strategy

### When to Use
- Tasks have **dependencies** (output of A feeds into B)
- **Linear workflow** (A → B → C)
- Quality gates between steps
- Simple coordination preferred

### Implementation Pattern
```markdown
**Sequential Execution:**
Agent 1 → Task A → Quality Gate
  ↓ (pass)
Agent 2 → Task B → Quality Gate
  ↓ (pass)
Agent 3 → Task C → Final Validation
```

### Example
```
Task: Refactor storage layer

Sequential phases:
1. debugger → Diagnose current architecture
   Quality Gate: Architecture documented
2. refactorer → Extract storage interface
   Quality Gate: Tests still pass
3. feature-implementer → Implement new backends
   Quality Gate: All tests pass, coverage >90%
4. code-reviewer → Final review
   Quality Gate: Approved for merge

Benefits: Clear progression, easier to debug
```

### Considerations
- Slower than parallel (linear time)
- Blocked if any step fails
- Simple to understand and monitor
- Good for critical path work

## Swarm Coordination Strategy

### When to Use
- **Many similar tasks** to process
- Tasks are homogeneous
- Load balancing important
- Results aggregated at end

### Implementation Pattern
```markdown
**Swarm Pattern:**
Controller Agent:
  - Distributes work queue to N workers
  - Monitors progress
  - Aggregates results

Worker Agents (1...N):
  - Process assigned work
  - Report completion
  - Request next task
```

### Example
```
Task: Refactor 50 large files (>500 LOC)

Swarm coordination:
1. goap-agent → Create work queue (50 files)
2. Spawn 5 refactorer agents (workers)
3. Each agent:
   - Takes next file from queue
   - Splits into modules
   - Runs tests
   - Reports completion
4. Controller monitors: 10/50 → 25/50 → 50/50 complete
5. code-reviewer validates all changes

Benefits: Scalable, fault-tolerant, efficient
```

### Considerations
- Requires work queue management
- Need progress tracking
- Results must be aggregated carefully
- Good for bulk operations

## Hybrid Execution Strategy

### When to Use
- Complex tasks with **mixed requirements**
- Some phases can be parallel, others must be sequential
- Optimization for both speed and dependencies
- Most real-world complex tasks

### Implementation Pattern
```markdown
**Hybrid Pattern:**
Phase 1 (Sequential): Research & Analysis
  ↓
Phase 2 (Parallel): Implementation across modules
  - Agent A → Module 1
  - Agent B → Module 2
  - Agent C → Module 3
  ↓ (all complete)
Phase 3 (Sequential): Integration & Testing
  ↓
Phase 4 (Swarm): Documentation across all modules
```

### Example
```
Task: Implement comprehensive error handling system

Hybrid execution:
1. [Sequential] Architecture design
   - goap-agent → Design error types
   - Quality Gate: Design approved

2. [Parallel] Implementation
   - Agent A → memory-core error types
   - Agent B → memory-storage-turso error handling
   - Agent C → memory-storage-redb error handling
   - Quality Gate: All implementations complete

3. [Sequential] Integration
   - feature-implementer → Wire error types together
   - test-runner → Integration tests
   - Quality Gate: All tests pass

4. [Swarm] Documentation
   - 3 agents → Document error handling in all modules
   - Quality Gate: Documentation complete

Benefits: Optimized for both speed and correctness
```

### Considerations
- Most complex to coordinate
- Requires careful phase boundaries
- Quality gates critical between phases
- Best balance of speed and quality

## Strategy Selection Decision Tree

```
Is time critical?
  ├─ Yes → Can tasks run in parallel?
  │   ├─ Yes → Use PARALLEL
  │   └─ No → Use SEQUENTIAL (but prioritize critical path)
  │
  └─ No → Are tasks similar/homogeneous?
      ├─ Yes (many similar) → Use SWARM
      ├─ No (mixed) → Use HYBRID
      └─ Simple linear → Use SEQUENTIAL
```

## Performance Comparison

| Strategy | Speed | Complexity | Best For |
|----------|-------|------------|----------|
| Sequential | 1x | Low | Simple, dependent tasks |
| Parallel | Nx | High | Independent tasks, N agents |
| Swarm | ~Nx | Medium | Many similar tasks |
| Hybrid | 2-4x | Very High | Complex real-world tasks |

## Common Pitfalls

### Parallel Execution
- ❌ Tasks actually have hidden dependencies
- ❌ No coordination between agents
- ❌ Results aggregated incorrectly
- ✅ Clear independence verification
- ✅ Explicit coordination points
- ✅ Careful result aggregation

### Sequential Execution
- ❌ Missing quality gates
- ❌ No failure recovery
- ❌ Blocking on slow tasks
- ✅ Quality gates between phases
- ✅ Timeout and retry logic
- ✅ Progress visibility

### Swarm Coordination
- ❌ Unbalanced work distribution
- ❌ No progress tracking
- ❌ Workers idle while some overloaded
- ✅ Dynamic work assignment
- ✅ Progress monitoring
- ✅ Load balancing

### Hybrid Execution
- ❌ Unclear phase boundaries
- ❌ Missing quality gates
- ❌ Over-complication
- ✅ Clear phase definitions
- ✅ Quality gates between phases
- ✅ Simplify where possible
