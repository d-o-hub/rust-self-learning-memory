---
name: architecture-validation
description: Validate architecture compliance with planned design decisions, patterns, and system constraints for the Rust self-learning memory project
---

# Architecture Validation Skill

Systematically validate that the implemented architecture matches the planned architecture decisions, design patterns, and system constraints documented in the project plans.

## Purpose

Ensure the implementation adheres to:
- **Architectural decisions** from plans/02-plan.md
- **Component boundaries** and separation of concerns
- **Data flow patterns** (episode lifecycle, pattern extraction)
- **Storage architecture** (Hybrid Turso + redb)
- **Integration patterns** (MCP protocol, sandbox)
- **Performance targets** and resource constraints
- **Security architecture** and defense-in-depth

## Architecture Dimensions

### 1. System Architecture Overview
Planned: Self-Learning Memory System with Learning Core, MCP Integration, and Storage Layer (Turso + redb)

**Validation**:
- Crate boundaries match planned components
- Dependencies flow correctly (no cycles)
- Core business logic separated from infrastructure

### 2. Crate Architecture & Dependencies
Planned: `memory-core`, `memory-storage-turso`, `memory-storage-redb`, `memory-mcp`, `test-utils`

**Validation**:
- Core has NO dependency on storage implementations
- Storage implementations depend on core types
- No circular dependencies

### 3. Storage Layer Architecture
Planned: Hybrid Storage (Turso durable + redb cache)

**Validation**:
- Turso: episodes, patterns, heuristics tables
- redb: episodes, patterns, embeddings, metadata tables
- Sync mechanism exists
- Turso is source of truth

### 4. Learning Cycle Architecture
Planned: 5-Phase Cycle (Pre-Task, Execution, Post-Task, Learning, Retrieval)

**Validation**:
- start_episode, log_step, complete_episode, retrieve APIs exist
- Reward calculation, reflection generation, pattern extraction implemented

### 5. Pattern Extraction Architecture
Planned: 4 Pattern Types (ToolSequence, DecisionPoint, ErrorRecovery, ContextPattern)

**Validation**:
- Pattern enum has all 4 variants
- PatternExtractor trait exists
- Pattern storage in both Turso and redb

### 6. MCP Integration Architecture
Planned: MCP Server + Sandbox with security layers

**Validation**:
- MemoryMCPServer implemented
- Tool definitions (query_memory, execute_agent_code)
- CodeSandbox with security config
- Resource limits enforced

### 7. Data Model Architecture
Planned: Episode and ExecutionStep structures with specific fields

**Validation**:
- Episode has all planned fields (episode_id, task_type, steps, outcome, reward, reflection, patterns)
- ExecutionStep has all planned fields

### 8. Performance Architecture
Planned: Retrieval <100ms, Episode creation <50ms, etc.

**Validation**:
- Benchmarks exist for key operations
- Performance targets documented

### 9. Security Architecture
Planned: Defense-in-depth (parameterized SQL, input validation, sandbox isolation, resource limits, TLS)

**Validation**:
- All SQL uses parameterized queries
- Input validation exists
- Sandbox has security configuration
- No hardcoded secrets

### 10. Error Handling Architecture
Planned: Custom Error enum with thiserror, Result<T> for fallible operations

**Validation**:
- Custom Error enum exists
- Result<T> type alias defined
- Error variants cover all failure modes

## Validation Workflow

### Phase 1: Architecture Document Review
```bash
# Read architecture decisions
cat plans/02-plan.md
cat plans/03-execute.md
```

### Phase 2: Code Structure Analysis
```bash
# Analyze crate structure
tree -L 2 -I target

# Check dependencies
cargo tree --depth 1

# Verify module organization
find . -name "lib.rs" -o -name "mod.rs" | xargs cat
```

### Phase 3: Component Boundary Validation
```bash
# Check core doesn't depend on impl
! grep "memory-storage-turso\|memory-storage-redb" memory-core/Cargo.toml

# Verify storage depends on core
grep "memory-core" memory-storage-turso/Cargo.toml
grep "memory-core" memory-storage-redb/Cargo.toml
```

### Phase 4: Data Flow Validation
1. Trace episode lifecycle through code
2. Verify storage sync mechanism
3. Check pattern extraction pipeline
4. Validate retrieval query flow

### Phase 5: API Compliance Check
```bash
# Verify public API matches plan
rg "pub (async )?fn" memory-core/src/memory.rs

# Check type definitions
rg "pub struct|pub enum" memory-core/src/types.rs memory-core/src/episode.rs
```

### Phase 6: Performance Target Validation
```bash
# Check benchmarks
ls benches/
cat benches/*.rs | rg "target|criterion"

# Verify performance constants
rg "const.*TIMEOUT|const.*LIMIT|const.*MAX" memory-core/src/ memory-mcp/src/
```

### Phase 7: Security Architecture Validation
```bash
# Check security implementations
rg "SecurityConfig|validate|sanitize" memory-mcp/src/

# Verify SQL safety
rg "execute\(|query\(" memory-storage-turso/src/ -A 2
```

## Compliance Matrix

| Architecture Dimension | Planned | Status |
|------------------------|---------|--------|
| Crate Structure | 6 crates | Check |
| Dependency Flow | Core → Storage | Check |
| Storage Layer | Turso + redb | Check |
| Learning Cycle | 5 phases | Check |
| Pattern Types | 4 variants | Check |
| MCP Integration | Server + Sandbox | Check |
| Data Model | Episode + Step | Check |
| Performance Targets | 7 metrics | Check |
| Security Mitigations | 5 attack surfaces | Check |
| Error Handling | Custom Error enum | Check |

## Output Format

```markdown
# Architecture Validation Report
**Generated**: [Date]
**Project**: rust-self-learning-memory
**Validation Against**: plans/02-plan.md

## Executive Summary
- **Overall Compliance**: X% (Y/Z dimensions)
- **Critical Violations**: N
- **Architecture Drift**: M areas
- **Recommendations**: P action items

## Compliance Score by Dimension

### 1. System Architecture: ✅ COMPLIANT
- Crate boundaries match planned structure
- Component separation properly implemented
- No architectural violations detected

### 2. Storage Layer: ⚠️ PARTIAL
- ✅ Turso storage implemented
- ✅ redb cache implemented
- ❌ Sync mechanism incomplete
  - **Missing**: Two-phase commit
  - **Impact**: Data consistency risk
  - **Priority**: High

## Detailed Findings

### Critical Violations
[List any critical violations found]

### Architecture Drift
[List areas where implementation differs from plan]

### Partial Implementations
[List features partially implemented]

## Recommendations

### High Priority
1. **Issue**: [Description]
   - File: [file path]
   - Effort: [estimate]
   - Reference: [plan reference]

### Medium Priority
[List medium priority recommendations]

### Low Priority
[List low priority recommendations]

## Architecture Decision Compliance

| Decision | Status | Notes |
|----------|--------|-------|
| Hybrid Storage (Turso + redb) | ✅ | Fully implemented |
| Node.js + VM2 Sandbox | ✅ | Implemented |
| Rule-based Pattern Extraction | ✅ | Phase 1 complete |
| Circuit Breakers | ✅ | Implemented in Turso |
| Feature Flags | ❌ | Not implemented |
| Telemetry with tracing | ✅ | Basic implementation |

## Next Steps

1. Review and prioritize architecture drift items
2. Create tickets for missing implementations
3. Update architecture documentation if intentional changes made
4. Schedule architecture review session
```

## Validation Checklist

Quick checklist for architecture validation:

**Crate Structure**:
- [ ] All planned crates exist
- [ ] Dependencies flow correctly
- [ ] No circular dependencies

**Storage Layer**:
- [ ] Turso tables match schema
- [ ] redb tables defined
- [ ] Sync mechanism implemented

**Learning Cycle**:
- [ ] All 5 phases implemented
- [ ] Episode lifecycle complete
- [ ] Pattern extraction functional

**MCP Integration**:
- [ ] MCP server implemented
- [ ] Sandbox with security
- [ ] Resource limits enforced

**Data Model**:
- [ ] Episode structure complete
- [ ] ExecutionStep structure complete
- [ ] All required fields present

**Performance**:
- [ ] Benchmarks exist
- [ ] Targets documented
- [ ] Profiling available

**Security**:
- [ ] Parameterized SQL
- [ ] Input validation
- [ ] No hardcoded secrets
- [ ] Resource limits enforced

**Error Handling**:
- [ ] Custom Error enum
- [ ] Result<T> usage
- [ ] Error variants complete

## Integration with Plans

This skill validates implementation against:
- `plans/00-overview.md` - Project summary and metrics
- `plans/01-understand.md` - Requirements and components
- `plans/02-plan.md` - Architecture decisions and roadmap
- `plans/03-execute.md` - Implementation details
- `plans/04-review.md` - Quality requirements
- `plans/05-secure.md` - Security requirements
- `plans/06-feedback-loop.md` - Refinements

## Example Usage

When invoked, this skill will:
1. Read architecture decisions from plans/
2. Analyze codebase structure
3. Validate component boundaries
4. Check data flow patterns
5. Verify API compliance
6. Assess performance targets
7. Validate security architecture
8. Generate compliance report with recommendations
