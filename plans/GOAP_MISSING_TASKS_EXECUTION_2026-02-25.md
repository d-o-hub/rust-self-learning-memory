# GOAP Missing Tasks Execution Plan — 2026-02-25

**Status**: 🔄 In Progress
**Branch**: `goap-week1-w1m4-ci-loop-2026-02-24` (from `main`)
**Methodology**: GOAP (Analyze → Decompose → Strategize → Execute)
**Latest Commit**: 87aa8a1 - feat(mcp): re-enable batch module in tools

---

## Phase 1: ANALYZE — Current State Summary

### Verified Metrics (2026-02-25)

| Metric | Current | Target | Gap | Status |
|-------|---------|--------|-----|--------|
| Files >500 LOC (source) | ~28 | 0 | 28 files | ⏸️ Pending |
| unwrap/expect in prod | ~681 | ≤20 | 661 calls | ⏸️ Pending |
| dead_code annotations | 134 | ≤10 | 124 annotations | ⏸️ Pending |
| Batch module | ✅ Enabled | Fully functional | - | ✅ DONE |
| Embeddings CLI | ✅ Exists | Functional | - | ✅ DONE |
| git-cliff | None | Automated | Setup needed | ⏸️ Pending |
| Ignored tests | 58 | ≤10 | 48 tests | 📋 Categorized |

### Key Findings

1. **Batch Module Status** (D1):
   - `memory-mcp/src/lib.rs:102` → `pub mod batch;` ✅ ENABLED
   - `memory-mcp/src/server/tools/mod.rs:8` → `pub mod batch;` ✅ NOW ENABLED
   - Created `batch.rs` to re-export batch types for tool integration
   - **Status**: ✅ COMPLETE

2. **Embeddings CLI Status** (D2):
   - Already implemented in `memory-cli/src/commands/embedding.rs`
   - Available commands: test, config, list-providers, benchmark, enable, disable
   - Semantic search available via `episode list --semantic-search <query>`
   - **Status**: ✅ ALREADY DONE

3. **dead_code Annotations**: 134 total
   - Test helpers: ~10 (keep)
   - Feature-gated: ~20 (keep)
   - Error variants: ~10 (keep)
   - Future use: ~20 (keep)
   - Compiler limitations: ~10 (keep)
   - Genuinely unused: ~64 (triage needed)

4. **Ignored Tests**: 58 total categorized:
   - Timing-dependent (CI flaky): 6
   - Slow integration tests: ~30
   - Flaky sandbox/WASM: 5
   - Missing features: 3
   - Infrastructure required: 3

---

## Phase 2: DECOMPOSE — Task Breakdown

### Task Priority Matrix

| Priority | Task | Scope | Effort | Dependencies | Status |
|----------|------|-------|--------|-------------|--------|
| **P0** | D1: Batch Module | Re-enable batch in tools/mod.rs | 4-6h | None | ✅ DONE |
| **P0** | D2: Embeddings CLI | Commands exist, verify | 1h | None | ✅ DONE |
| **P1** | C1: Test Triage | Triage ignored tests | 4-6h | None | 📋 Categorized |
| **P1** | E1: dead_code | Triage 134 annotations | 6-8h | None | ⏸️ Pending |
| **P2** | B: File Splits | Split 28 oversized files | 15-20h | None | ⏸️ Pending |
| **P2** | A: Error Handling | Convert unwrap/expect to Result | 20-30h | None | ⏸️ Pending |
| **P2** | E2: git-cliff | Set up changelog automation | 3-4h | None | ⏸️ Pending |

---

## Phase 3: STRATEGIZE — Execution Pattern

**Strategy**: Hybrid (Parallel execution for independent tasks, Sequential for dependencies)

```
Phase 1 (COMPLETED):
├─ D1: Batch Module → ✅ Re-enabled and working
├─ D2: Embeddings CLI → ✅ Already implemented

Phase 2 (Next Session):
├─ C1: Test Triage → 📋 Categorized (6 fixable, 30 slow, 22 keep)
├─ E1: dead_code → ⏸️ Pending (64 genuinely unused to triage)

Phase 3 (If Time Permits):
├─ B: File Splits → 28 files
├─ A: Error Handling → 681 calls
└─ E2: git-cliff → Setup
```

---

## Phase 4: Task Specifications

### D1: Batch Module Rehabilitation ✅ COMPLETE

**Status**: Completed in commit 87aa8a1

**Changes**:
- Uncommented `pub mod batch;` in `memory-mcp/src/server/tools/mod.rs`
- Created `memory-mcp/src/server/tools/batch.rs` to re-export batch types
- All batch tests pass (11 tests)

**Verification**:
- [x] Build passes: `cargo build -p memory-mcp`
- [x] Tests pass: `cargo test -p memory-mcp` (2292 passed)
- [x] Clippy passes: zero warnings

---

### D2: Embeddings CLI ✅ ALREADY DONE

**Status**: Already implemented

**Commands Available**:
```
memory-cli embedding test            # Test embedding provider
memory-cli embedding config          # Show configuration
memory-cli embedding list-providers  # List providers
memory-cli embedding benchmark      # Benchmark performance
memory-cli embedding enable          # Enable embeddings
memory-cli embedding disable         # Disable embeddings
```

**Semantic Search**:
- Available via: `memory-cli episode list --semantic-search <query>`

---

### C1: Test Triage 📋 CATEGORIZED

**Status**: Categorized, pending fix phase

**Ignored Tests Breakdown** (58 total):

| Category | Count | Action |
|----------|-------|--------|
| Timing-dependent (CI flaky) | 6 | Could fix with better async handling |
| Slow integration tests | ~30 | Keep (by design) |
| Flaky sandbox/WASM | 5 | Document, keep ignored |
| Missing features | 3 | Keep (feature not implemented) |
| Infrastructure required | 3 | Keep (needs real backends) |

**Recommendation**: Focus on fixing the 6 timing-dependent tests in future sessions

---

### E1: dead_code Cleanup ⏸️ PENDING

**Status**: Annotated, pending triage

**Breakdown**:
| Category | Count | Recommendation |
|----------|-------|----------------|
| Test helpers | ~10 | Keep |
| Feature-gated (oauth, etc) | ~20 | Keep |
| Error variants | ~10 | Keep |
| Future use ("available for...") | ~20 | Keep |
| Compiler false positives | ~10 | Keep |
| Genuinely unused | ~64 | Triage - may remove |

---

## Phase 5: Quality Gates

All gates passed ✅:
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all -- -D warnings`
- [x] `cargo build --all`
- [x] `cargo nextest run --all` (2292 passed, 69 skipped)
- [x] CI workflows: All passing

---

## Phase 6: Progress Tracking

### Session Summary (2026-02-25)

| Task | Status | Notes |
|------|--------|-------|
| D1: Batch Module | ✅ DONE | Re-enabled in tools/mod.rs |
| D2: Embeddings CLI | ✅ DONE | Already exists |
| C1: Test Triage | 📋 Categorized | 58 tests categorized |
| E1: dead_code | ⏸️ Pending | 134 annotated, 64 need triage |
| B: File Splits | ⏸️ Pending | 28 files remain |
| A: Error Handling | ⏸️ Pending | 681 calls remain |
| E2: git-cliff | ⏸️ Pending | Not started |

---

## CI Status

**Latest Run**: All workflows passing ✅
- Quick Check: ✅
- Coverage: ✅
- Security: ✅
- File Structure: ✅
- Release: ✅

---

*Generated: 2026-02-25 by GOAP Analysis*
*Updated: 2026-02-25 after D1+D2 completion*
*Next Action: Continue with E1 (dead_code triage) or C1 (test fixes)*
