# GOAP Execution Plan: Missing Tasks Implementation

**Date**: 2026-02-02
**Coordinator**: GOAP Agent
**Status**: Plan Created → Execution Ready
**Episode ID**: TBD

---

## Executive Summary

Based on comprehensive analysis of plans/ folder, 47 missing implementations identified across 3 priority levels. This plan orchestrates 9 specialized agents using hybrid parallel/sequential execution with handoff coordination.

**Critical Path**: Pre-existing fixes → Quick wins → P0 features → P1 features → Quality validation

---

## Task Decomposition

### Category 1: Pre-existing Issues (BLOCKER - Must Fix First)
**Impact**: Blocks all development
**Effort**: 2-4 hours

```
C1. Fix memory-storage-redb compilation errors
├─ Task 1.1: Add missing ttl_secs field to PersistedCacheEntry (30 min)
├─ Task 1.2: Fix HashMap import issue (15 min)
└─ Task 1.3: Run tests to verify (15 min)

C2. Fix clippy warnings
├─ Task 2.1: Fix excessive nesting (7 instances) (1 hour)
├─ Task 2.2: Fix doc markdown issues (6 instances) (30 min)
├─ Task 2.3: Fix must-use candidates (3 instances) (30 min)
├─ Task 2.4: Fix other clippy warnings (wildcard imports, etc.) (1 hour)
└─ Task 2.5: Verify zero warnings (15 min)

C3. Fix build warnings
├─ Task 3.1: Remove unused imports (5 instances) (15 min)
├─ Task 3.2: Fix unsafe code warnings (2 instances) (30 min)
└─ Task 3.3: Fix dead code warnings (5 instances) (30 min)
```

**Dependencies**: None (can run parallel)

**Success Criteria**:
- [ ] `cargo build --workspace` succeeds with zero errors
- [ ] `cargo test --workspace` compiles successfully
- [ ] `cargo clippy --all -- -D warnings` passes (zero warnings)
- [ ] All tests pass (or identify legitimate failures)

---

### Category 2: Quick Wins (LOW EFFORT - HIGH VALUE)
**Impact**: Remove technical debt quickly
**Effort**: 4 hours

```
Q1. Example file updates (15 min)
└─ Update ModelConfig → ProviderConfig in examples

Q2. Memory-mcp compilation fixes (15 min)
└─ Fix provider_config field access in execute.rs

Q3. Batch module TODO cleanup (30 min)
└─ Remove or fix broken batch module reference

Q4. ProviderConfig migration guide (2 hours)
└─ Document API changes and migration path
```

**Dependencies**: Category 1 complete

**Success Criteria**:
- [ ] All example files compile and run
- [ ] memory-mcp compiles without errors
- [ ] No TODO comments for broken modules
- [ ] Migration guide documented

---

### Category 3: P0 Critical Features (HIGH VALUE)
**Impact**: Enable episode relationships and tags via CLI/MCP
**Effort**: 76-106 hours (3-4 weeks)

```
P0-A. MCP Relationship Tools (41-56 hours)
├─ Phase 1: Core Tools (18-24h)
│  ├─ Task A.1: add_episode_relationship (6-8h)
│  ├─ Task A.2: remove_episode_relationship (4-6h)
│  └─ Task A.3: get_episode_relationships (4-6h)
├─ Phase 2: Query Tools (13-18h)
│  ├─ Task A.4: find_related_episodes (6-8h)
│  └─ Task A.5: check_relationship_exists (3-4h)
├─ Phase 3: Advanced Tools (18-24h)
│  ├─ Task A.6: get_dependency_graph (8-10h)
│  ├─ Task A.7: validate_no_cycles (4-6h)
│  └─ Task A.8: get_topological_order (6-8h)
└─ Phase 4: Integration & Testing (8-10h)

P0-B. CLI Relationship Commands (20-30 hours)
├─ Phase 1: Core Commands (10-12h)
│  ├─ Task B.1: relationship add (4-5h)
│  ├─ Task B.2: relationship remove (3-4h)
│  └─ Task B.3: relationship list (4-5h)
├─ Phase 2: Query Commands (7-9h)
│  ├─ Task B.4: relationship find (4-5h)
│  └─ Task B.5: relationship info (2-3h)
├─ Phase 3: Advanced Commands (9-12h)
│  ├─ Task B.6: relationship graph (6-8h)
│  └─ Task B.7: relationship validate (3-4h)
└─ Phase 4: Polish & Testing (4-6h)

P0-C. CLI Tag Commands (15-20 hours)
├─ Phase 1: Core Commands (8-10h)
│  ├─ Task C.1: tag add (3-4h)
│  ├─ Task C.2: tag remove (2-3h)
│  └─ Task C.3: tag list (3-4h)
├─ Phase 2: Search & Query (4-5h)
│  └─ Task C.4: tag search (4-5h)
├─ Phase 3: Advanced Features (5-7h)
│  ├─ Task C.5: tag rename (2-3h)
│  └─ Task C.6: tag stats (3-4h)
└─ Phase 4: Polish & Testing (3-4h)
```

**Dependencies**:
- Category 1 & 2 complete
- P0-A and P0-B can run in parallel
- P0-C independent of P0-A/B

**Success Criteria**:
- [ ] All 8 MCP tools implemented with tests
- [ ] All 7 CLI relationship commands working
- [ ] All 6 CLI tag commands working
- [ ] Integration tests pass
- [ ] Documentation complete

---

### Category 4: P1 High-Value Features
**Impact**: Performance and production readiness
**Effort**: 40-56 hours (2-3 weeks)

```
P1-A. Security Improvements (40-48 hours)
├─ Task A.1: Wire up rate limiting to all endpoints (20-24h)
├─ Task A.2: Complete audit logging integration (20-24h)
└─ Task A.3: Security testing (4-8h)

P1-B. Performance Features (Already implemented, need enablement)
├─ Task B.1: Enable keep-alive pool by default (4-6h)
├─ Task B.2: Fix adaptive pool connection exposure (8h)
└─ Task B.3: Wire up compression for large payloads (8-12h)
```

**Dependencies**: Category 3 complete

**Success Criteria**:
- [ ] Rate limiting active on all endpoints
- [ ] Audit logging for all security-sensitive operations
- [ ] Keep-alive pool enabled
- [ ] Compression integrated
- [ ] Performance benchmarks show improvement

---

## Agent Coordination Strategy

### Agent Pool (9 Specialized Agents)

| Agent | Role | Capabilities | Assignment |
|-------|------|--------------|------------|
| **Agent 1** | `code-quality` | Fix lint, format, clippy warnings | C2, C3 |
| **Agent 2** | `rust-specialist` | Fix Rust compilation errors | C1 |
| **Agent 3** | `test-runner` | Fix test failures, run test suites | C1.3, C2.5 |
| **Agent 4** | `junior-coder` | Quick wins, simple fixes | Q1, Q2, Q3 |
| **Agent 5** | `feature-implementer` | MCP relationship tools | P0-A |
| **Agent 6** | `feature-implementer` | CLI relationship commands | P0-B |
| **Agent 7** | `feature-implementer` | CLI tag commands | P0-C |
| **Agent 8** | `security` | Security improvements | P1-A |
| **Agent 9** | `performance` | Performance feature enablement | P1-B |

### Execution Phases

#### Phase 0: Pre-Execution (5 minutes)
- Start episode for tracking
- Create git branch `feat/missing-tasks-implementation`
- Set up progress tracking

#### Phase 1: Blocker Resolution (Parallel - 2-4 hours)
```
┌─────────────────┬─────────────────┬─────────────────┐
│ Agent 1         │ Agent 2         │ Agent 3         │
│ (code-quality)  │ (rust-specialist)│ (test-runner)  │
├─────────────────┼─────────────────┼─────────────────┤
│ Fix clippy      │ Fix redb comp.  │ Run tests       │
│ warnings        │ errors          │ and verify      │
│ (C2)            │ (C1)            │ (C1.3, C2.5)    │
└─────────────────┴─────────────────┴─────────────────┘
        ↓                         ↓                 ↓
    Quality Gate 1: All checks pass, zero warnings
```

**Handoff**: Agent 3 validates all fixes, reports to GOAP

#### Phase 2: Quick Wins (Sequential - 4 hours)
```
Agent 4 (junior-coder): Q1 → Q2 → Q3 → Q4
    ↓
Quality Gate 2: Examples compile, MCP compiles, TODOs clean
```

**Handoff**: Agent 4 commits each quick win atomically

#### Phase 3: P0 Features (Parallel Streams - 3-4 weeks)

**Stream A - MCP Tools** (Agent 5):
```
Phase 1: Core Tools → Agent 5 implements → Unit Tests
Phase 2: Query Tools → Agent 5 implements → Unit Tests
Phase 3: Advanced Tools → Agent 5 implements → Unit Tests
Phase 4: Integration → Agent 5 + Agent 3 coordinate → Integration Tests
```

**Stream B - CLI Relationships** (Agent 6):
```
Phase 1: Core Commands → Agent 6 implements → Unit Tests
Phase 2: Query Commands → Agent 6 implements → Unit Tests
Phase 3: Advanced Commands → Agent 6 implements → Unit Tests
Phase 4: Polish → Agent 6 + Agent 3 → Integration Tests
```

**Stream C - CLI Tags** (Agent 7):
```
Phase 1: Core Commands → Agent 7 implements → Unit Tests
Phase 2: Search → Agent 7 implements → Unit Tests
Phase 3: Advanced → Agent 7 implements → Unit Tests
Phase 4: Polish → Agent 7 + Agent 3 → Integration Tests
```

**Coordination**: All three streams run in parallel. Agent 3 (test-runner) coordinates testing phases.

#### Phase 4: P1 Features (Parallel - 2-3 weeks)
```
┌─────────────────┬─────────────────┐
│ Agent 8         │ Agent 9         │
│ (security)      │ (performance)   │
├─────────────────┼─────────────────┤
│ Rate limiting   │ Enable keep-    │
│ + audit logging │ alive pool      │
└─────────────────┴─────────────────┘
        ↓                 ↓
    Quality Gate 4: Security & performance validated
```

#### Phase 5: Final Validation (Sequential - 4 hours)
```
Agent 1 (code-reviewer): Full codebase review
Agent 3 (test-runner): Complete test suite
Agent 1 (code-quality): Final quality gates
    ↓
GOAP: Final report and documentation
```

---

## Handoff Coordination Protocol

### Handoff Type 1: Fix → Validate
```
Agent A (fixer) → Implement fix → Commit changes → Notify GOAP
    ↓
GOAP → Spawn Agent B (validator) → Run validation checks → Report results
    ↓
If pass: Continue to next task
If fail: Agent A (fixer) → Fix issues → Re-validate
```

### Handoff Type 2: Feature → Test
```
Agent A (implementer) → Implement feature → Unit tests → Notify GOAP
    ↓
GOAP → Spawn Agent B (test-runner) → Run integration tests → Report results
    ↓
If pass: Mark feature complete
If fail: Agent A (implementer) → Fix issues → Re-test
```

### Handoff Type 3: Parallel → Converge
```
Agent A ┐
         ├─> Independent work ─> GOAP collects results ─> Agent C (integrator)
Agent B ┘
```

---

## Quality Gates

### Quality Gate 1: After Phase 1
- [ ] cargo build --workspace succeeds
- [ ] cargo test --workspace compiles
- [ ] cargo clippy --all -- -D warnings passes (zero warnings)
- [ ] Zero compilation errors
- [ ] Zero unsafe code warnings (or documented)

### Quality Gate 2: After Phase 2
- [ ] All example files compile and run
- [ ] memory-mcp compiles successfully
- [ ] No broken TODO comments
- [ ] Migration guide created

### Quality Gate 3: After Phase 3 (Per Phase)
- [ ] All new tools/commands implemented
- [ ] Unit tests pass (>90% coverage)
- [ ] Integration tests pass
- [ ] Documentation complete
- [ ] Zero clippy warnings
- [ ] Code formatted

### Quality Gate 4: After Phase 4
- [ ] Rate limiting active
- [ ] Audit logging complete
- [ ] Performance features enabled
- [ ] Security tests pass
- [ ] Benchmarks show improvement

### Quality Gate 5: Final
- [ ] All tests pass (>90% coverage)
- [ ] Zero clippy warnings
- [ ] Zero unsafe code without documentation
- [ ] Documentation complete and up-to-date
- [ ] Release notes updated
- [ ] All plans/ files marked complete

---

## Progress Tracking

### Episode Structure
```json
{
  "episode_id": "UUID",
  "task_description": "Implement all missing tasks from plans folder",
  "context": {
    "language": "rust",
    "domain": "coordination",
    "tags": ["goap", "multi-agent", "missing-tasks"]
  },
  "steps": [
    {"phase": "1", "agent": "code-quality", "action": "Fix clippy warnings", "status": "completed"},
    {"phase": "1", "agent": "rust-specialist", "action": "Fix redb compilation", "status": "completed"},
    ...
  ]
}
```

### Git Commit Strategy
- **After each atomic task**: Create commit with descriptive message
- **Format**: `[module] description`
- **Examples**:
  - `fix(storage-redb): add missing ttl_secs field to PersistedCacheEntry`
  - `fix(clippy): resolve excessive nesting warnings in memory-core`
  - `feat(mcp-tools): implement add_episode_relationship tool`
  - `feat(cli): implement relationship add command`

### Progress Files
Create daily progress files in `plans/`:
- `PROGRESS_2026-02-02.md` - Day 1 progress
- `PROGRESS_2026-02-03.md` - Day 2 progress
- etc.

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent coordination overhead | Medium | Medium | Use clear handoff protocols, automate where possible |
| Feature interdependencies | Low | High | Identify dependencies upfront, test integration points |
| Test flakiness | Low | Medium | Run tests in isolation, use loop-agent for retries |
| Time estimation errors | Medium | Medium | 20% buffer built into estimates, prioritize critical path |
| Pre-existing issue complexity | Medium | High | Use perplexity-researcher for complex issues, iterate with loop-agent |

---

## Success Metrics

### Completion Metrics
- [ ] 100% of Category 1 (pre-existing) tasks complete
- [ ] 100% of Category 2 (quick wins) complete
- [ ] 100% of P0 features implemented and tested
- [ ] 100% of P1 features implemented and tested

### Quality Metrics
- [ ] Zero compilation errors
- [ ] Zero clippy warnings
- [ ] >90% test coverage maintained
- [ ] >99% test pass rate
- [ ] All code formatted

### Timeline Metrics
- [ ] Phase 1 (Blockers): <4 hours
- [ ] Phase 2 (Quick Wins): <4 hours
- [ ] Phase 3 (P0 Features): 3-4 weeks
- [ ] Phase 4 (P1 Features): 2-3 weeks
- [ ] Phase 5 (Validation): <4 hours

### Agent Performance Metrics
- [ ] Agent utilization rate >80%
- [ ] Handoff success rate >95%
- [ ] Iteration count <3 per task (loop-agent convergence)

---

## Next Steps

### Immediate (Next 30 minutes)
1. Start episode with TaskContext
2. Create feature branch
3. Spawn Agents 1, 2, 3 for Phase 1 (parallel execution)
4. Monitor progress and collect results

### Today (Remaining 4 hours)
1. Complete Phase 1 (Blocker Resolution)
2. Complete Phase 2 (Quick Wins)
3. Quality Gate validation
4. Commit all changes
5. Create progress report

### This Week (Days 2-5)
1. Start Phase 3 (P0 Features)
2. Spawn Agents 5, 6, 7 for parallel streams
3. Daily progress tracking
4. Weekly summary

---

## Communication Plan

### To User
- **Every phase completion**: Summary of what was done
- **Every blockage**: Clear explanation of what's blocking and how we're fixing it
- **Daily**: Progress update with metrics
- **Final**: Comprehensive completion report

### Between Agents
- **Handoff**: Clear summary of work done, artifacts created, next steps
- **Failure**: What failed, why, what's needed to fix
- **GOAP coordination**: Central coordination point for all agents

---

**Status**: ✅ PLAN COMPLETE
**Next Action**: Spawn Agent 1 (code-quality), Agent 2 (rust-specialist), Agent 3 (test-runner) for Phase 1
**Estimated Total Time**: 5-7 weeks (148-210 hours)
**Critical Path**: Phase 1 → Phase 2 → Phase 3 (P0-A) → Phase 4 → Phase 5

---

**Created**: 2026-02-02
**Coordinator**: GOAP Agent
**Location**: `/workspaces/feat-phase3/plans/GOAP_EXECUTION_PLAN_FEB_2026.md`
