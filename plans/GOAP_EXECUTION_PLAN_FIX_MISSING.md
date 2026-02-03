# GOAP Execution Plan: Fix All Missing Tasks

**Date**: 2026-02-02
**Status**: Analysis Complete → Execution Ready
**Coordinator**: GOAP Agent

---

## Executive Summary

Based on comprehensive analysis of the plans/ folder and codebase verification:

### ✅ Already Complete (No Action Needed)
- **MCP Episode Relationship Tools**: 100% complete (678 LOC)
- **CLI Relationship Commands**: 100% complete (1,247 LOC)
- **CLI Tag Commands**: 100% complete (1,142 LOC)
- **Integration Tests**: 100% complete (1,184 LOC)
- **Workspace Build**: ✅ Compiles successfully
- **Clippy Warnings**: ✅ Zero warnings

### 🔧 Actually Missing (Needs Implementation)

1. **Test Compilation Errors** (38 errors across 3 crates)
   - `memory-core`: 10 test errors (missing imports, types, functions)
   - `memory-storage-turso`: 12 test errors (missing types, imports)
   - `memory-mcp`: 16 test errors (missing types, functions, imports)

2. **Performance Features** (Already implemented, needs enablement)
   - Keep-alive pool: Behind feature flag
   - Adaptive pool: API access issue
   - Compression: Not integrated

3. **Security Features** (Needs verification/completion)
   - Rate limiting: Check integration
   - Audit logging: Check completeness

---

## Task Decomposition

### Category 1: Test Compilation Fixes (CRITICAL - Must Fix First)
**Impact**: Blocks test execution
**Effort**: 4-6 hours
**Agent Count**: 6 agents (parallel streams)

#### Stream A: memory-core Test Fixes (Agent 1)
```
A1. Fix embeddings/real_model/tests.rs - Missing validate_downloaded_file function
A2. Fix indexing/spatiotemporal.rs - Missing Duration import
A3. Fix indexing/mod.rs - Missing Episode type
A4. Fix patterns/changepoint/tests.rs - Unused variable warning
```

#### Stream B: memory-storage-turso Test Fixes (Agent 2)
```
B1. Fix prepared/tests.rs - Crate import issue
B2. Fix cache/integration.rs - Missing QueryType
B3. Fix transport/compression.rs - Missing CompressionAlgorithm
B4. Fix transport/wrapper.rs - Unused variable warning
```

#### Stream C: memory-mcp Test Fixes (Agent 3)
```
C1. Fix patterns/predictive/dbscan_tests.rs - Unused variable
C2. Fix patterns/benchmarks.rs - Unused variables
C3. Fix patterns/predictive/extraction.rs - Missing field access fix
```

#### Stream D: Code Quality Fixes (Agent 4)
```
D1. Fix all unused variable warnings (15 instances)
D2. Fix unused import warnings
D3. Verify cargo fmt compliance
```

#### Stream E: Security Verification (Agent 5)
```
E1. Check rate_limiter.rs integration with endpoints
E2. Check audit/ module completeness
E3. Document security status
```

#### Stream F: Performance Enablement (Agent 6)
```
F1. Verify keep-alive pool feature flag status
F2. Check adaptive pool API access issue
F3. Verify compression integration status
F4. Document enablement steps
```

---

## Agent Coordination Strategy

### Agent Pool (6 Specialized Agents)

| Agent | Type | Role | Assignment |
|-------|------|------|------------|
| **Agent 1** | `rust-specialist` | Fix memory-core tests | Stream A |
| **Agent 2** | `rust-specialist` | Fix memory-storage-turso tests | Stream B |
| **Agent 3** | `rust-specialist` | Fix memory-mcp tests | Stream C |
| **Agent 4** | `code-quality` | Fix warnings and format | Stream D |
| **Agent 5** | `security` | Verify security features | Stream E |
| **Agent 6** | `performance` | Verify performance features | Stream F |

### Execution Phases

#### Phase 1: Parallel Test Fixes (2-3 hours)
```
┌─────────────────┬─────────────────┬─────────────────┐
│ Agent 1         │ Agent 2         │ Agent 3         │
│ (rust-specialist)│ (rust-specialist)│ (rust-specialist)│
├─────────────────┼─────────────────┼─────────────────┤
│ Fix memory-core │ Fix memory-     │ Fix memory-mcp  │
│ test errors     │ storage-turso   │ test errors     │
│ (10 errors)     │ test errors     │ (16 errors)     │
│                 │ (12 errors)     │                 │
└─────────────────┴─────────────────┴─────────────────┘
        ↓                         ↓                 ↓
     Quality Gate 1: All test compilation errors fixed
```

#### Phase 2: Quality & Verification (1-2 hours)
```
┌─────────────────┬─────────────────┬─────────────────┐
│ Agent 4         │ Agent 5         │ Agent 6         │
│ (code-quality)  │ (security)      │ (performance)   │
├─────────────────┼─────────────────┼─────────────────┤
│ Fix warnings    │ Verify security │ Verify perf     │
│ Run fmt/clippy  │ features        │ features        │
└─────────────────┴─────────────────┴─────────────────┘
        ↓                         ↓                 ↓
     Quality Gate 2: All quality checks pass
```

#### Phase 3: Final Validation (30 minutes)
```
Agent 1-6: Run full test suite
     ↓
Quality Gate 3: cargo test --workspace passes
```

---

## Quality Gates

### Quality Gate 1: After Phase 1
- [ ] `cargo test --workspace --lib` compiles successfully
- [ ] Zero test compilation errors
- [ ] All 38 errors resolved

### Quality Gate 2: After Phase 2
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] All warnings resolved
- [ ] Security features verified
- [ ] Performance features documented

### Quality Gate 3: Final
- [ ] `cargo test --workspace` passes (or known failures documented)
- [ ] `cargo build --workspace` succeeds
- [ ] All quality checks pass
- [ ] Documentation updated

---

## Handoff Coordination Protocol

### Handoff Type 1: Fix → Validate
```
Agent A (fixer) → Implement fix → Commit → Notify GOAP
    ↓
GOAP → Spawn Agent B (validator) → Run checks → Report
    ↓
If pass: Continue to next task
If fail: Agent A → Fix issues → Re-validate
```

### Handoff Type 2: Parallel → Converge
```
Agent 1 ┐
Agent 2 ├─> Independent work ─> GOAP collects results
Agent 3 ┘
    ↓
Agent 4 (integrator) → Final validation
```

---

## Detailed Error List

### memory-core Errors (10)
1. `embeddings/real_model/tests.rs:85` - Missing `validate_downloaded_file`
2. `embeddings/real_model/tests.rs:93` - Missing `validate_downloaded_file`
3. `embeddings/real_model/tests.rs:101` - Missing `validate_downloaded_file`
4. `indexing/spatiotemporal.rs:839` - Missing `Duration` import
5. `indexing/spatiotemporal.rs:848` - Missing `Duration` import
6. `indexing/spatiotemporal.rs:953` - Missing `Duration` import
7. `indexing/spatiotemporal.rs:954` - Missing `Duration` import
8. `indexing/mod.rs:201` - Missing `Episode` type
9. `indexing/mod.rs:208` - Missing `Episode` type
10. `patterns/changepoint/tests.rs:213` - Unused variable warning

### memory-storage-turso Errors (12)
1. `prepared/tests.rs:6` - Crate import issue
2. `cache/integration.rs:287` - Missing `QueryType`
3. `transport/compression.rs:59` - Missing `CompressionAlgorithm`
4. `transport/compression.rs:139` - Missing `CompressionAlgorithm`
5. `transport/compression.rs:144` - Missing `CompressionAlgorithm`
6. `transport/wrapper.rs:615` - Unused variable warning
7-12. Additional type/import errors

### memory-mcp Errors (16)
1. `patterns/predictive/dbscan_tests.rs:454` - Unused variable `i`
2. `patterns/benchmarks.rs:111` - Unused variable `neighbors`
3. `patterns/benchmarks.rs:346` - Unused variable `i`
4. `patterns/predictive/extraction.rs:509` - Missing field access fix
5-16. Additional errors

---

## Success Metrics

### Completion Metrics
- [ ] 100% of test compilation errors fixed
- [ ] 100% of warnings resolved
- [ ] 100% of security features verified
- [ ] 100% of performance features documented

### Quality Metrics
- [ ] Zero compilation errors
- [ ] Zero clippy warnings
- [ ] All tests pass (or documented)
- [ ] 100% code formatted

### Timeline Metrics
- [ ] Phase 1 (Test Fixes): <3 hours
- [ ] Phase 2 (Quality): <2 hours
- [ ] Phase 3 (Validation): <30 minutes

---

## Next Steps

### Immediate (Next 30 minutes)
1. Start episode for tracking
2. Create git branch `feat/fix-missing-tasks`
3. Spawn Agents 1, 2, 3 for Phase 1 (parallel)
4. Monitor progress

### Today (Remaining 4-5 hours)
1. Complete Phase 1 (Test Fixes)
2. Complete Phase 2 (Quality & Verification)
3. Complete Phase 3 (Final Validation)
4. Create progress report
5. Update plans/ folder documentation

---

**Status**: ✅ PLAN COMPLETE
**Next Action**: Spawn Agents 1-3 for Phase 1 (parallel test fixes)
**Estimated Total Time**: 6-8 hours
**Critical Path**: Phase 1 → Phase 2 → Phase 3

---

**Created**: 2026-02-02
**Coordinator**: GOAP Agent
**Location**: `/workspaces/feat-phase3/plans/GOAP_EXECUTION_PLAN_FIX_MISSING.md`
