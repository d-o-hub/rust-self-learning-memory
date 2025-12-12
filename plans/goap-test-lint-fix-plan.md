# GOAP Execution Plan: Release 0.1.6 - WASM Optimization

**Date:** 2025-12-12
**Primary Goal:** Release 0.1.6 with updated dependencies and 50% WASM usage
**Strategy:** Sequential (Update → Build → Test → Lint → Commit → Release)

---

## Release 0.1.6 Changes Completed

### ✅ Sub-Goal 1: Update Dependencies (COMPLETED)
- **rquickjs**: 0.6 → 0.7 (WASM JavaScript engine)
- **wasmtime**: 19.0 → 20.0 (WASM runtime)
- **Status**: ✅ Dependencies updated and Cargo.lock refreshed
- **Build**: ✅ Successfully compiles

### ✅ Sub-Goal 2: Increase WASM Usage (COMPLETED)
- **wasm_ratio**: 0.1 → 0.5 (10% → 50% WASM usage)
- **File**: `memory-mcp/src/unified_sandbox.rs:77`
- **Status**: ✅ Default changed to 50% WASM

### ✅ Sub-Goal 3: Enhanced Error Handling (COMPLETED)
- **Retry Logic**: Added 3-attempt retry with exponential backoff
- **Warmup**: Added runtime pool warmup method
- **Health Status**: Added comprehensive health monitoring
- **File**: `memory-mcp/src/wasm_sandbox.rs`
- **Status**: ✅ All features implemented

### ✅ Sub-Goal 4: Code Quality (IN PROGRESS)
- **Formatting**: ✅ cargo fmt --all applied
- **Linting**: ⏳ cargo clippy running
- **Testing**: ⏳ cargo test --all running

---

## Next Steps

### ⏳ Sub-Goal 5: Git Operations (PENDING)
- **Status**: Ready to commit
- **Files Modified**:
  - `memory-mcp/Cargo.toml` (dependencies)
  - `memory-mcp/src/unified_sandbox.rs` (wasm_ratio)
  - `memory-mcp/src/wasm_sandbox.rs` (retry logic, warmup, health)
  - `memory-mcp/src/patterns/statistical.rs` (formatting)
- **Commit Message**: Prepared and ready

### ⏳ Sub-Goal 6: GitHub Actions (PENDING)
- **Status**: Awaiting commit and push
- **Checks**: Build, test, lint, security audit

### ⏳ Sub-Goal 7: Release Creation (PENDING)
- **Tag**: v0.1.6
- **Title**: "Release 0.1.6 - WASM Performance Optimization"
- **Status**: Ready after CI passes

### Complexity Level
**Complex**: Multiple failing tests across different modules, potential lint issues, requires:
- Root cause analysis for each test failure
- Coordinated fixes without breaking other tests
- Comprehensive validation

### Quality Requirements
- **Testing:** 100% tests passing (no skips, no ignores)
- **Linting:** cargo fmt --check, cargo clippy -- -D warnings both pass
- **Standards:** AGENTS.md compliance
- **Performance:** Tests complete in reasonable time (<5 minutes)

---

## Phase 2: Current State Assessment

### Known Test Failures

1. **memory-cli tests (6 tests)** - ✅ FIXED (tempfile isolation)
   - Status: Fixed with unique temp directories per test
   - Verification needed: Rerun to confirm

2. **memory-core timing test (1 test)** - ❌ NEEDS FIX
   - Test: `should_run_periodic_background_sync_automatically`
   - Issue: Flaky timing test - sync_count remains 0
   - Location: `memory-core/tests/storage_sync.rs:175`
   - Root cause: Background sync task not running in test timeframe

### Lint Status
- **cargo fmt:** ✅ Pass (last run successful)
- **cargo clippy:** ⚠️ 1 future-compat warning in dependency (not our code)
- **Build:** ✅ Pass (release build successful)

### Test Status Summary
- **Total Tests:** 26
- **Passing:** 25
- **Failing:** 1 (flaky timing test)
- **Target:** 26/26 passing

---

## Phase 3: Task Decomposition

### Main Goal Breakdown

#### Sub-Goal 1: Diagnose Flaky Timing Test (P0)
**Success Criteria:** Root cause identified and fix strategy determined
**Dependencies:** None
**Complexity:** Medium

**Atomic Tasks:**
- Task 1.1: Read and analyze `memory-core/tests/storage_sync.rs:160-179`
- Task 1.2: Understand background sync mechanism in StorageSynchronizer
- Task 1.3: Identify why sync_count remains 0 in test
- Task 1.4: Determine fix approach (increase timeout, fix sync trigger, etc.)

#### Sub-Goal 2: Fix Flaky Timing Test (P0)
**Success Criteria:** Test passes consistently (5/5 runs minimum)
**Dependencies:** Sub-Goal 1 complete
**Complexity:** Medium-High

**Atomic Tasks:**
- Task 2.1: Implement fix based on diagnosis
- Task 2.2: Verify fix doesn't break other tests
- Task 2.3: Run test 5 times to verify stability
- Task 2.4: Update test documentation if needed

#### Sub-Goal 3: Verify All CLI Tests Pass (P0)
**Success Criteria:** All 6 CLI tests pass with temp directory isolation
**Dependencies:** None (can run parallel with Sub-Goal 1)
**Complexity:** Low

**Atomic Tasks:**
- Task 3.1: Run `cargo test -p memory-cli`
- Task 3.2: Verify all 6 tests pass
- Task 3.3: Document any issues found

#### Sub-Goal 4: Full Test Suite Validation (P0)
**Success Criteria:** `cargo test --all` returns 0 failures
**Dependencies:** Sub-Goals 1, 2, 3 complete
**Complexity:** Low

**Atomic Tasks:**
- Task 4.1: Run full test suite `cargo test --all`
- Task 4.2: Verify 26/26 tests pass
- Task 4.3: Check for any warnings or issues
- Task 4.4: Run tests with `--test-threads=1` if parallelization issues

#### Sub-Goal 5: Lint Verification (P0)
**Success Criteria:** Both fmt and clippy pass with no warnings
**Dependencies:** None (can run parallel)
**Complexity:** Low

**Atomic Tasks:**
- Task 5.1: Run `cargo fmt --all --check`
- Task 5.2: Run `cargo clippy --all -- -D warnings`
- Task 5.3: Fix any lint issues found
- Task 5.4: Re-verify after fixes

### Dependency Graph

```
Phase 1 (Parallel):
  Task 1.1-1.4 (Diagnose timing test)
  Task 3.1-3.3 (Verify CLI tests)    ┐
  Task 5.1-5.2 (Initial lint check)  ┘ → All complete
                                          ↓
Phase 2 (Sequential):                     ↓
  Task 2.1-2.4 (Fix timing test) ←────────┘
  Task 5.3-5.4 (Fix lint issues if any)
                                          ↓
Phase 3 (Sequential):                     ↓
  Task 4.1-4.4 (Full validation) ←────────┘
```

---

## Phase 4: Strategy Selection

### Chosen Strategy: HYBRID

**Rationale:**
- **Phase 1:** Parallel (diagnosis + verification can run simultaneously)
- **Phase 2:** Sequential (fixes must be applied in order)
- **Phase 3:** Sequential (final validation after all fixes)

**Benefits:**
- Maximize efficiency with parallel diagnosis
- Ensure quality with sequential fixes
- Comprehensive validation at end

**Estimated Duration:**
- Phase 1: ~10 minutes (parallel)
- Phase 2: ~15 minutes (sequential fixes)
- Phase 3: ~10 minutes (validation)
- **Total:** ~35 minutes

---

## Phase 5: Agent Assignment

| Phase | Tasks | Agent | Rationale |
|-------|-------|-------|-----------|
| 1.1 | Diagnose timing test | debugger | Expert in runtime issues |
| 1.2 | Verify CLI tests | test-runner | Test execution specialist |
| 1.3 | Initial lint check | code-quality | Formatting/linting expert |
| 2.1 | Fix timing test | test-fix | Test repair specialist |
| 2.2 | Fix lint issues | code-quality | Code standards |
| 3.1 | Full validation | test-runner | Comprehensive testing |

---

## Phase 6: Execution Plan

### Phase 1: Parallel Diagnosis & Verification (10 minutes)

**Launch 3 agents in parallel:**

**Agent 1 - Debugger (timing test diagnosis):**
```
Task: Diagnose why should_run_periodic_background_sync_automatically fails
Files: memory-core/tests/storage_sync.rs, memory-core/src/sync/*
Objective: Identify root cause of sync_count staying at 0
Deliverable: Diagnosis report with fix recommendation
```

**Agent 2 - Test Runner (CLI test verification):**
```
Task: Run and verify all memory-cli tests pass
Command: cargo test -p memory-cli
Objective: Confirm CLI tests work with temp directory fixes
Deliverable: Test results and any issues found
```

**Agent 3 - Code Quality (lint check):**
```
Task: Run comprehensive lint checks
Commands:
  - cargo fmt --all --check
  - cargo clippy --all -- -D warnings
Objective: Identify any lint issues
Deliverable: List of lint issues (if any)
```

**Quality Gate 1:**
- ✅ Root cause of timing test identified
- ✅ CLI tests verified passing/failing
- ✅ Lint status known

### Phase 2: Sequential Fixes (15 minutes)

**Agent 4 - Test Fix (timing test fix):**
```
Task: Fix should_run_periodic_background_sync_automatically test
Approach: Based on diagnosis from Agent 1
Options:
  - Increase timeout for sync to occur
  - Mock/trigger sync manually in test
  - Fix sync task initialization
  - Add explicit sync triggers
File: memory-core/tests/storage_sync.rs
Validation: Run test 5 times, all must pass
```

**Agent 5 - Code Quality (lint fixes if needed):**
```
Task: Fix any lint issues identified by Agent 3
Actions: Apply fmt, fix clippy warnings
Validation: Re-run cargo fmt --check and cargo clippy
```

**Quality Gate 2:**
- ✅ Timing test passes consistently (5/5 runs)
- ✅ All lint checks pass
- ✅ No new test failures introduced

### Phase 3: Comprehensive Validation (10 minutes)

**Agent 6 - Test Runner (full suite):**
```
Task: Run complete test suite and validate
Commands:
  1. cargo test --all
  2. Verify 26/26 tests pass
  3. Check for warnings
  4. If failures, retry with --test-threads=1
  5. Document final status
Acceptance Criteria:
  - Zero test failures
  - Zero ignored tests
  - Zero skipped tests
  - Clean output (minimal warnings)
```

**Quality Gate 3 (FINAL):**
- ✅ cargo test --all: 26/26 passing
- ✅ cargo fmt --check: Pass
- ✅ cargo clippy -- -D warnings: Pass
- ✅ cargo build --release: Pass
- ✅ No skipped or ignored tests

---

## Phase 7: Success Criteria

### Must Achieve (Zero Tolerance):
- [ ] ALL 26 tests passing
- [ ] cargo fmt --check passes
- [ ] cargo clippy -- -D warnings passes
- [ ] Zero ignored tests
- [ ] Zero skipped tests
- [ ] Timing test stable (5/5 consecutive runs)

### Nice to Have:
- [ ] Test execution time < 5 minutes
- [ ] Comprehensive test documentation
- [ ] Test coverage report

---

## Contingency Plans

### If Timing Test Cannot Be Stabilized:
**Option 1:** Increase timeout significantly (e.g., 30s → 60s)
**Option 2:** Refactor test to manually trigger sync instead of waiting
**Option 3:** Mock the sync mechanism for deterministic testing
**Priority:** Option 2 (most reliable long-term)

### If CLI Tests Fail:
**Action:** Revert temp directory changes and implement alternative isolation
**Fallback:** Use unique database names with timestamp suffixes

### If Lint Issues Found:
**Action:** Auto-apply fmt, manually fix clippy warnings
**Escalation:** If clippy warnings are false positives, document with #[allow]

### If Full Suite Fails:
**Action:** Run with --test-threads=1 to isolate concurrency issues
**Diagnosis:** Identify which test(s) fail and re-apply fixes

---

## Monitoring & Reporting

### Progress Tracking:
- Each agent reports completion to central log
- Quality gates validated before proceeding
- Issues logged immediately

### Final Report Structure:
```markdown
## Test & Lint Fix Execution Report

### Status: [SUCCESS/FAILURE]

### Test Results:
- Total Tests: 26
- Passing: [N]
- Failing: [N]
- Ignored: [N]

### Lint Results:
- cargo fmt: [PASS/FAIL]
- cargo clippy: [PASS/FAIL]

### Issues Fixed:
1. [Issue description] → [Fix applied]

### Issues Remaining:
- [None if SUCCESS]

### Recommendations:
- [Future improvements]
```

---

## Execution Status

**Status:** READY TO EXECUTE
**Next Action:** Launch Phase 1 parallel agents
**Estimated Completion:** 35 minutes
**Risk Level:** LOW (well-defined fixes, clear validation)

---

**Plan Created:** 2025-12-12T08:20:00Z
**Plan Version:** 1.0
**Orchestrator:** GOAP Agent
