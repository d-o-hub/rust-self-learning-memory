# GOAP Execution Plan: Complete Implementation of Plans Folder Tasks

**Date**: 2025-11-08
**Status**: 92% Complete → Target: 100% Complete
**Strategy**: Hybrid (Parallel + Sequential with Quality Gates)

## Executive Summary

**Current State**: Codebase is 92% complete with excellent core functionality
**Target**: 100% complete and publication-ready
**Estimated Time**: 2-3 days
**Agents Required**: 5-7 specialized agents

---

## Phase 1: ANALYZE ✅ COMPLETE

### Analysis Results
- ✅ 192 tests (140 passed so far, more running)
- ✅ All core features implemented
- ⚠️ 16 files exceed 500 LOC limit
- ⚠️ Missing package metadata
- ⚠️ No CHANGELOG.md

---

## Phase 2: DECOMPOSE - Task Breakdown

### Main Goal
Implement all missing tasks from plans folder and prepare for crate publication

### Sub-Goals with Dependencies

```
G1: Package Metadata (P0) - No dependencies
├── T1.1: Create CHANGELOG.md
├── T1.2: Add Cargo.toml descriptions (5 crates)
├── T1.3: Add keywords and categories
└── T1.4: Verify with cargo publish --dry-run

G2: Code Quality - Refactoring (P1) - No dependencies
├── T2.1: Refactor reflection.rs (1,436 LOC → <500)
├── T2.2: Refactor memory.rs (1,326 LOC → <500)
├── T2.3: Refactor pattern.rs (809 LOC → <500)
├── T2.4: Refactor reward.rs (766 LOC → <500)
├── T2.5: Refactor extraction.rs (705 LOC → <500)
├── T2.6: Refactor server.rs (681 LOC → <500)
├── T2.7: Refactor sandbox.rs (670 LOC → <500)
└── T2.8: Refactor remaining 9 files

G3: Quality Assurance (P0) - Depends on G2
├── T3.1: Verify all tests pass
├── T3.2: Run cargo fmt --check
├── T3.3: Run cargo clippy -- -D warnings
├── T3.4: Run cargo audit
└── T3.5: Verify coverage >90%

G4: Documentation Updates (P1) - Depends on G2
├── T4.1: Update plan files with completion status
├── T4.2: Remove TODO comment
└── T4.3: Update README if needed

G5: Final Validation (P0) - Depends on G3, G4
├── T5.1: cargo publish --dry-run for all crates
├── T5.2: Verify docs.rs build
└── T5.3: Run full integration test suite
```

---

## Phase 3: STRATEGIZE - Execution Strategy

### Strategy: HYBRID

**Why Hybrid?**
- Package metadata tasks (G1) can run in parallel - independent
- Refactoring tasks (G2) must be sequential - order matters for compilation
- Quality checks (G3) must be sequential after refactoring
- Some tasks within groups can parallelize

### Execution Phases

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: PARALLEL - Package Metadata (30 min)              │
│   ├─ Agent A: CHANGELOG.md                                  │
│   ├─ Agent B: Cargo.toml metadata (crates 1-3)             │
│   └─ Agent C: Cargo.toml metadata (crates 4-5)             │
│                                                              │
│ Quality Gate 1: Metadata complete and valid                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: SEQUENTIAL - Large File Refactoring (2-3 days)    │
│   Step 1: reflection.rs (1,436 LOC) - CRITICAL             │
│   Step 2: memory.rs (1,326 LOC) - CRITICAL                 │
│   Step 3: pattern.rs (809 LOC)                             │
│   Step 4: reward.rs (766 LOC)                              │
│   Step 5: extraction.rs (705 LOC)                          │
│                                                              │
│ Quality Gate 2: All files <500 LOC, tests pass             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: PARALLEL - Remaining Refactoring (1 day)          │
│   ├─ Agent D: server.rs, sandbox.rs, queue.rs              │
│   ├─ Agent E: effectiveness.rs, validation.rs, storage.rs  │
│   └─ Agent F: Remaining 4 files                            │
│                                                              │
│ Quality Gate 3: All refactoring complete, tests pass       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 4: SEQUENTIAL - Quality Assurance (1 hour)           │
│   1. Run all tests (cargo test --all)                      │
│   2. Format check (cargo fmt --check)                      │
│   3. Lint check (cargo clippy -- -D warnings)              │
│   4. Security audit (cargo audit)                          │
│   5. Coverage check (cargo llvm-cov --fail-under-lines 90) │
│                                                              │
│ Quality Gate 4: All checks pass, ready for publication     │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 4: COORDINATE - Agent Assignment

### Agent Roles

| Agent ID | Type | Responsibilities | Files/Tasks |
|----------|------|------------------|-------------|
| **A1** | feature-implementer | CHANGELOG.md creation | 1 file |
| **A2** | feature-implementer | Cargo.toml metadata (3 crates) | 3 files |
| **A3** | feature-implementer | Cargo.toml metadata (2 crates) | 2 files |
| **R1** | refactorer | Critical refactoring (reflection.rs) | 1 large file |
| **R2** | refactorer | Critical refactoring (memory.rs) | 1 large file |
| **R3** | refactorer | Medium refactoring (pattern, reward, extraction) | 3 files |
| **R4** | refactorer | Small refactoring (server, sandbox, queue) | 3 files |
| **R5** | refactorer | Small refactoring (effectiveness, validation, storage) | 3 files |
| **R6** | refactorer | Remaining files | 4 files |
| **T1** | test-runner | Test execution and validation | All tests |
| **C1** | code-reviewer | Final quality review | Entire codebase |

### Parallel Capacity
- **Phase 1**: 3 agents in parallel (A1, A2, A3)
- **Phase 2**: 1 agent sequential (R1, then R2, then R3, etc.)
- **Phase 3**: 3 agents in parallel (R4, R5, R6)
- **Phase 4**: 1 agent sequential (T1, then C1)

---

## Phase 5: EXECUTE - Detailed Task Plans

### Phase 1: Package Metadata (PARALLEL)

#### Task 1.1: Create CHANGELOG.md
**Agent**: A1 (feature-implementer)
**Duration**: 15 minutes
**Dependencies**: None

**Steps**:
1. Create `/home/user/rust-self-learning-memory/CHANGELOG.md`
2. Follow Keep a Changelog format
3. Document v0.1.0 release with all features
4. Include:
   - Added: Core features, pattern extraction, MCP integration, security
   - Testing: 192 tests, >90% coverage
   - Security: 51 penetration tests, defense-in-depth

**Success Criteria**:
- File exists
- Follows standard format
- Includes all major features

---

#### Task 1.2-1.3: Cargo.toml Metadata
**Agents**: A2 (crates 1-3), A3 (crates 4-5)
**Duration**: 20 minutes per agent
**Dependencies**: None

**Crates to Update**:
1. `memory-core`
2. `memory-storage-turso`
3. `memory-storage-redb`
4. `memory-mcp`
5. `test-utils`

**Required Fields**:
```toml
[package]
name = "memory-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

# ADD THESE:
description = "..."  # 1-2 sentence description
keywords = ["ai", "memory", "learning", "episodic", "patterns"]
categories = ["database", "development-tools", "science"]
documentation = "https://docs.rs/memory-core"
readme = "README.md"
```

**Success Criteria**:
- All 5 Cargo.toml files updated
- `cargo publish --dry-run` succeeds for each crate
- No warnings about missing metadata

---

### Phase 2: Critical Refactoring (SEQUENTIAL)

#### Task 2.1: Refactor reflection.rs (1,436 LOC → <500)
**Agent**: R1 (refactorer)
**Duration**: 8-12 hours
**Dependencies**: None
**Priority**: CRITICAL

**File**: `/home/user/rust-self-learning-memory/memory-core/src/reflection.rs`

**Refactoring Plan**:
```
memory-core/src/reflection/
├── mod.rs (exports, main types, <100 LOC)
├── generator.rs (generate_reflection function, ~300 LOC)
├── analyzer.rs (analysis functions, ~300 LOC)
├── insights.rs (insight generation, ~300 LOC)
└── tests.rs (move tests here, ~400 LOC)
```

**Steps**:
1. Create `/memory-core/src/reflection/` directory
2. Split reflection.rs into 5 files
3. Update imports in `mod.rs`
4. Update `memory-core/src/lib.rs` to re-export from reflection::mod
5. Run `cargo test` to verify no breakage
6. Run `cargo fmt` and `cargo clippy`

**Success Criteria**:
- All files <500 LOC
- No functionality change
- All tests pass
- No clippy warnings

---

#### Task 2.2: Refactor memory.rs (1,326 LOC → <500)
**Agent**: R2 (refactorer)
**Duration**: 8-12 hours
**Dependencies**: Task 2.1 complete
**Priority**: CRITICAL

**File**: `/home/user/rust-self-learning-memory/memory-core/src/memory.rs`

**Refactoring Plan**:
```
memory-core/src/memory/
├── mod.rs (SelfLearningMemory struct, <200 LOC)
├── lifecycle.rs (start, log, complete episode, ~300 LOC)
├── retrieval.rs (retrieve functions, ~300 LOC)
├── stats.rs (statistics, health checks, ~200 LOC)
└── tests.rs (test helpers, ~300 LOC)
```

**Steps**:
1. Create `/memory-core/src/memory/` directory
2. Split memory.rs into 5 files
3. Keep main struct in `mod.rs`, move implementations to submodules
4. Update imports
5. Verify compilation and tests
6. Format and lint

**Success Criteria**:
- All files <500 LOC
- All tests pass
- No regressions

---

#### Task 2.3: Refactor pattern.rs (809 LOC → <500)
**Agent**: R3 (refactorer)
**Duration**: 4-6 hours
**Dependencies**: Task 2.2 complete
**Priority**: HIGH

**File**: `/memory-core/src/pattern.rs`

**Refactoring Plan**:
```
memory-core/src/pattern/
├── mod.rs (Pattern enum, <200 LOC)
├── types.rs (PatternId, Heuristic, <200 LOC)
├── operations.rs (similarity, merge, <200 LOC)
└── tests.rs (tests, <300 LOC)
```

---

#### Tasks 2.4-2.8: Remaining Refactoring
**Agent**: R3, then R4, R5, R6
**Duration**: 2-4 hours each
**Dependencies**: Sequential completion

**Files**:
- reward.rs (766 → split into reward/{mod, calculator, tests})
- extraction.rs (705 → split into extraction/{mod, extractors, utils})
- server.rs (681 → split into server/{mod, handlers, tools})
- sandbox.rs (670 → split into sandbox/{mod, executor, validators})
- queue.rs (642 → split into learning/{queue, workers, stats})
- effectiveness.rs, validation.rs, storage.rs (similar approach)

---

### Phase 3: Quality Assurance (SEQUENTIAL)

#### Task 3.1: Test Execution
**Agent**: T1 (test-runner)
**Duration**: 30 minutes
**Dependencies**: All refactoring complete

**Commands**:
```bash
cargo test --all --no-fail-fast
cargo test --all --release
```

**Success Criteria**:
- All 192+ tests pass
- No test failures or panics
- Test time <5 minutes

---

#### Task 3.2-3.5: Code Quality Checks
**Agent**: T1 (test-runner)
**Duration**: 20 minutes
**Dependencies**: Task 3.1 complete

**Commands**:
```bash
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo audit
cargo llvm-cov --all --fail-under-lines 90
```

**Success Criteria**:
- Format check passes
- Zero clippy warnings
- Zero security vulnerabilities
- Coverage ≥90%

---

### Phase 4: Final Validation (SEQUENTIAL)

#### Task 4.1: Publication Dry Run
**Agent**: C1 (code-reviewer)
**Duration**: 15 minutes
**Dependencies**: All previous phases complete

**Commands**:
```bash
cd memory-core && cargo publish --dry-run
cd memory-storage-turso && cargo publish --dry-run
cd memory-storage-redb && cargo publish --dry-run
cd memory-mcp && cargo publish --dry-run
cd test-utils && cargo publish --dry-run
```

**Success Criteria**:
- All crates validate successfully
- No warnings about missing files or metadata
- Package sizes reasonable (<1MB each)

---

#### Task 4.2: Final Code Review
**Agent**: C1 (code-reviewer)
**Duration**: 30 minutes
**Dependencies**: Task 4.1 complete

**Review Checklist**:
- ✅ All files <500 LOC
- ✅ No TODO comments in production code
- ✅ Documentation complete
- ✅ Tests comprehensive
- ✅ Security validated
- ✅ Performance benchmarks present

**Success Criteria**:
- Code review passes all checks
- Ready for publication

---

## Phase 6: SYNTHESIZE - Success Metrics

### Completion Criteria

**Must Have (Blocking)**:
- [ ] All 16 files refactored to <500 LOC
- [ ] CHANGELOG.md created
- [ ] All Cargo.toml files have complete metadata
- [ ] All 192+ tests passing
- [ ] cargo fmt, clippy, audit pass
- [ ] Coverage ≥90%
- [ ] cargo publish --dry-run succeeds for all crates

**Nice to Have (Non-Blocking)**:
- [ ] Connection pooling implemented
- [ ] Semantic search implemented
- [ ] Additional benchmarks

### Timeline

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Phase 1: Metadata | 30 min | T+0h | T+0.5h |
| Phase 2: Critical Refactoring | 24 hours | T+0.5h | T+24.5h |
| Phase 3: Remaining Refactoring | 8 hours | T+24.5h | T+32.5h |
| Phase 4: Quality Assurance | 1 hour | T+32.5h | T+33.5h |
| Phase 5: Final Validation | 1 hour | T+33.5h | T+34.5h |
| **Total** | **~35 hours (~2 days)** | | |

### Risk Mitigation

**Risk 1**: Refactoring breaks tests
- **Mitigation**: Run tests after each file refactoring
- **Recovery**: Revert and re-approach with smaller changes

**Risk 2**: Metadata incomplete
- **Mitigation**: Use cargo publish --dry-run validation
- **Recovery**: Add missing fields based on warnings

**Risk 3**: Time overruns
- **Mitigation**: Prioritize critical files first
- **Recovery**: Complete P0 tasks, defer P1 tasks

---

## Execution Command

```bash
# This plan will be executed by spawning specialized agents in the prescribed order:

# Phase 1 (PARALLEL):
Task(feature-implementer, "Create CHANGELOG.md", parallel=1)
Task(feature-implementer, "Update Cargo.toml 1-3", parallel=1)
Task(feature-implementer, "Update Cargo.toml 4-5", parallel=1)

# Phase 2 (SEQUENTIAL):
Task(refactorer, "Refactor reflection.rs", sequential=1)
Task(refactorer, "Refactor memory.rs", sequential=2)
Task(refactorer, "Refactor pattern.rs", sequential=3)
Task(refactorer, "Refactor reward.rs", sequential=4)
Task(refactorer, "Refactor extraction.rs", sequential=5)

# Phase 3 (PARALLEL):
Task(refactorer, "Refactor server.rs, sandbox.rs, queue.rs", parallel=2)
Task(refactorer, "Refactor effectiveness.rs, validation.rs, storage.rs", parallel=2)
Task(refactorer, "Refactor remaining 4 files", parallel=2)

# Phase 4 (SEQUENTIAL):
Task(test-runner, "Run all tests and quality checks", sequential=6)

# Phase 5 (SEQUENTIAL):
Task(code-reviewer, "Final validation and review", sequential=7)
```

---

## Status Tracking

**Current Phase**: Phase 1 Preparation
**Agents Active**: 0
**Tasks Complete**: 0/30
**Overall Progress**: 0%

**Last Updated**: 2025-11-08T19:00:00Z
**Next Checkpoint**: Phase 1 Quality Gate (30 min)
