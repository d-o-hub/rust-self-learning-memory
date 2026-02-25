# GOAP Missing Tasks Execution Plan — 2026-02-25

**Status**: 🔄 Ready to Execute
**Branch**: `goap-missing-tasks-2026-02-25` (from `main`)
**Methodology**: GOAP (Analyze → Decompose → Strategize → Execute)

---

## Phase 1: ANALYZE — Current State Summary

### Verified Metrics (2026-02-25)

| Metric | Current | Target | Gap |
|-------|---------|--------|-----|
| Files >500 LOC (source) | ~28 | 0 | 28 files |
| unwrap/expect in prod | ~681 | ≤20 | 661 calls |
| dead_code annotations | 134 | ≤10 | 124 annotations |
| Batch module | Partially enabled (lib.rs ✓, tools/mod.rs ✗) | Fully functional | Integration needed |
| Embeddings CLI | Missing | Functional | Implementation needed |
| git-cliff | None | Automated | Setup needed |

### Key Findings

1. **Batch Module Status**:
   - `memory-mcp/src/lib.rs:102` → `pub mod batch;` ✅ ENABLED
   - `memory-mcp/src/server/tools/mod.rs:8` → `// pub mod batch;` ❌ DISABLED
   - Reason: "uses non-existent jsonrpsee and ServerState"

2. **dead_code Annotations**: 134 total (mix of legitimate and removable)

3. **File Size**: 28 source files >500 LOC need splitting

---

## Phase 2: DECOMPOSE — Task Breakdown

### Task Priority Matrix

| Priority | Task | Scope | Effort | Dependencies |
|----------|------|-------|--------|-------------|
| **P0** | D1: Batch Module | Re-enable batch in tools/mod.rs, fix jsonrpsee deps | 4-6h | None |
| **P0** | D2: Embeddings CLI | Add embed/embed-search commands to memory-cli | 4-6h | None |
| **P1** | C1: Test Triage | Triage ignored tests (fix/delete/document) | 4-6h | None |
| **P1** | E1: dead_code | Triage 134 annotations | 6-8h | None |
| **P2** | B: File Splits | Split 28 oversized files | 15-20h | None |
| **P2** | A: Error Handling | Convert unwrap/expect to Result | 20-30h | None |
| **P2** | E2: git-cliff | Set up changelog automation | 3-4h | None |

### Execution Order (Dependency-Free)

Since all tasks are largely independent, we can execute in parallel:

1. **Parallel Group 1** (P0 - Quick Wins):
   - D1: Batch Module rehabilitation
   - D2: Embeddings CLI implementation

2. **Parallel Group 2** (P1 - Quality Improvements):
   - C1: Test triage
   - E1: dead_code cleanup

3. **Sequential Group** (P2 - Long-term):
   - B: File splits (if time permits)
   - A: Error handling (if time permits)
   - E2: git-cliff setup

---

## Phase 3: STRATEGIZE — Execution Pattern

**Strategy**: Hybrid (Parallel execution for independent tasks, Sequential for dependencies)

```
Phase 1 (This Session):
├─ D1: Batch Module → Fix and re-enable
├─ D2: Embeddings CLI → Implement commands

Phase 2 (Next Session):
├─ C1: Test Triage → Categorize 58-62 ignored tests
├─ E1: dead_code → Triage and clean

Phase 3 (If Time Permits):
├─ B: File Splits → 28 files
├─ A: Error Handling → 681 calls
└─ E2: git-cliff → Setup
```

---

## Phase 4: Task Specifications

### D1: Batch Module Rehabilitation

**Objective**: Re-enable batch module in MCP tools and fix dependencies

**Scope**:
- Input: `memory-mcp/src/server/tools/mod.rs`
- Input: `memory-mcp/src/batch/`
- Output: Batch tool available via MCP

**Acceptance Criteria**:
- [ ] Uncomment `pub mod batch;` in tools/mod.rs
- [ ] Fix jsonrpsee dependency (use native JSON-RPC or remove)
- [ ] Update ServerState references if needed
- [ ] Build passes: `cargo build -p memory-mcp`
- [ ] Tests pass: `cargo test -p memory-mcp`

**Deliverables**:
- Modified `memory-mcp/src/server/tools/mod.rs`
- Fixed batch module dependencies
- Working batch tool

---

### D2: Embeddings CLI Implementation

**Objective**: Add embed and embed-search commands to memory-cli

**Scope**:
- Input: `memory-cli/src/commands/`
- Output: New CLI commands

**Acceptance Criteria**:
- [ ] Add `embed` command (generate embeddings for text)
- [ ] Add `embed-search` command (semantic search)
- [ ] Integrate with existing embeddings module
- [ ] Build passes: `cargo build -p memory-cli`
- [ ] Tests pass: `cargo test -p memory-cli`

**Deliverables**:
- New command modules in `memory-cli/src/commands/embed/`
- Updated CLI help documentation

---

### C1: Test Triage

**Objective**: Categorize and resolve ignored tests

**Scope**:
- All `#[ignore]` tests in codebase

**Acceptance Criteria**:
- [ ] List all ignored tests
- [ ] Categorize: fixable / delete / document
- [ ] Fix 3-5 tests
- [ ] Document rationale for remaining

**Deliverables**:
- Updated test files
- Documentation of ignored test rationale

---

### E1: dead_code Cleanup

**Objective**: Triage and remove unnecessary dead_code annotations

**Scope**:
- 134 `#[allow(dead_code)]` annotations

**Acceptance Criteria**:
- [ ] Categorize annotations by reason (error variants, feature-gated, test helpers, genuinely unused)
- [ ] Remove 20-30 clearly unnecessary annotations
- [ ] Document rationale for remaining

**Deliverables**:
- Cleaner codebase
- Documentation of remaining annotations

---

## Phase 5: Quality Gates

Each task must pass before commit:
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`

---

## Phase 6: Progress Tracking

### Session: D1+D2 (Batch + Embeddings CLI)

| Task | Status | Notes |
|------|--------|-------|
| D1: Batch Module | 🔄 Ready | lib.rs enabled, tools/mod.rs disabled |
| D2: Embeddings CLI | 🔄 Ready | Commands to implement |

---

*Generated: 2026-02-25 by GOAP Analysis*
*Next Action: Execute D1+D2 in parallel*
