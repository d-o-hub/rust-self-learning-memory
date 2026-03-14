# ADR-041: Test Health Remediation Plan (v0.1.20)

- **Status**: Accepted (Partially Implemented 2026-03-14)
- **Date**: 2026-03-14
- **Deciders**: Project maintainers
- **Related**: ADR-027 (Ignored Tests Strategy), ADR-033 (Modern Testing Strategy), ADR-040 (Gap Analysis v0.1.19)

## Context

A comprehensive audit on 2026-03-14 reveals three layers of test health issues:

1. **Build-blocking compilation errors** in `memory-storage-redb` preventing `cargo build --all` and all downstream test execution.
2. **119 ignored tests** across the workspace with mixed legitimacy.
3. **Stale nightly workflow exclusion filters** that still reference tests by name instead of a maintainable pattern.

The project cannot ship v0.1.20 or run CI reliably until these are resolved.

---

## Problem Statement

### P0: Build Is Broken — `memory-storage-redb` Does Not Compile

The crate `memory-storage-redb` (lib) fails with **4 compilation errors and 3 warnings**:

| Error | Location | Root Cause |
|-------|----------|------------|
| `E0425: cannot find type CacheConfig` | `lib.rs:178` | `CacheConfig` is defined in `cache::types` and re-exported from `cache::mod.rs`, but `lib.rs` uses it without importing |
| `E0433: use of undeclared type LRUCache` | `lib.rs:189` | `LRUCache` is re-exported from `cache::mod.rs` but not imported in `lib.rs` |
| `E0425: cannot find type CacheMetrics` | `lib.rs:376` | Same pattern — type defined in `cache::types`, not imported |
| `E0425: cannot find type CacheConfig in super::super` | `cache/adaptive/mod.rs:336` | `From<AdaptiveCacheConfig>` impl uses `super::super::CacheConfig` but types module isn't a direct child of `lib.rs` scope — it's in `cache::types` |

**Warnings (will become CI errors under `-D warnings`):**

| Warning | Location | Root Cause |
|---------|----------|------------|
| `unused import: ReadableTable` | `lib.rs:27` | Import no longer used after refactoring |
| `unused import: lru::LRUCache` | `cache/mod.rs:27` | Re-export exists but nothing in the `cache` module itself uses it |
| `unused import: CacheConfig` | `cache/mod.rs:29` | Re-export exists but nothing in the `cache` module itself uses it |

**Impact**: All downstream crates (`memory-cli`, `test-utils`, `tests`, `examples`, `benches`) fail to compile. **Zero tests can run.**

**Root Cause Analysis**: Commit `32f2dad` ("feat(redb): add AdaptiveCache with Cache trait adapter") introduced the `Cache` trait, `AdaptiveCacheAdapter`, and `new_with_adaptive_config()` constructor. It re-exported the new types from `cache::mod.rs` and `lib.rs` but did not add the corresponding `use` imports for `CacheConfig`, `LRUCache`, and `CacheMetrics` that `lib.rs` functions reference directly. The `super::super::CacheConfig` path in `adaptive/mod.rs` also broke because `CacheConfig` lives in `cache::types`, not at the `lib.rs` module root.

### P1: 119 Ignored Tests — Categorized Breakdown

| Category | Count | Root Cause | Fixability |
|----------|-------|------------|------------|
| **Turso libsql memory corruption** | 70 | Upstream `malloc_consolidate()` bug in libsql native library | ❌ Blocked — requires upstream fix |
| **Slow integration tests** | 29 | Tests intentionally marked slow (`memory-core`: learning_cycle, heuristic_learning, tag_operations, input_validation, performance) | ⚠️ By design — run in nightly |
| **WASM/WASI sandbox** | 9 | Sandbox timing issues, binary data handling, WASI implementation gaps | 🟡 Partial — some fixable with timeout tuning |
| **Flaky CI sandbox** | 5 | Timing-dependent sandbox tests in `memory-mcp/src/sandbox/tests.rs` | 🟡 Fixable — add retries or increase tolerances |
| **E2E / process-based** | 3 | Pattern CLI validation, MCP process test, soak test | 🟢 2 fixable (stale ignore reasons) |
| **Requires real backends** | 2 | `compliance.rs` requires MCP server, `relationship_integration.rs` requires storage | ⚠️ By design |
| **Local embeddings** | 1 | ONNX `Send` trait issue with `ort` crate | ❌ Blocked — upstream |

**Detailed file-by-file inventory:**

#### memory-storage-turso/tests/ (70 tests — all libsql blocked)

| File | Ignored Count |
|------|---------------|
| `security_tests.rs` | 15 |
| `sql_injection_tests.rs` | 11 |
| `capacity_enforcement_test.rs` | 8 |
| `multi_dimension_routing.rs` | 7 |
| `phase1_optimization_test.rs` | 6 |
| `keepalive_pool_integration_test.rs` | 5 |
| `compression_integration_test.rs` | 4 |
| `compression_integration.rs` | 5 |
| `cache_integration_test.rs` | 4 |
| `vector_search_test.rs` | 3 |
| `integration_test.rs` | 4 |
| `prepared_cache_integration_test.rs` | 1 |

All carry identical reason: `"Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"`.

#### memory-core/tests/ (37 tests — mostly slow integration)

| File | Ignored Count | Category |
|------|---------------|----------|
| `tag_operations_test.rs` | 9 | Slow integration |
| `heuristic_learning.rs` | 8 | Slow integration |
| `performance.rs` | 7 | Slow/long-running |
| `learning_cycle.rs` | 5 | Slow integration |
| `input_validation.rs` | 2 | Slow integration |
| `regression.rs` | 2 | Non-deterministic + long-running |
| `compliance.rs` | 2 | Requires MCP server |
| `relationship_integration.rs` | 1 | Requires real storage |
| `embeddings/local.rs` | 1 | ort crate Send issue |

#### memory-mcp/src/ (9 tests — WASM/sandbox)

| File | Ignored Count | Category |
|------|---------------|----------|
| `sandbox/tests.rs` | 4 | Flaky CI timing |
| `unified_sandbox/tests.rs` | 2 | WASM binary data handling |
| `wasmtime_sandbox/tests.rs` | 2 | WASI implementation |
| `wasm_sandbox/tests.rs` | 1 | WASM timeout flaky |

#### tests/ (3 tests — E2E)

| File | Ignored Count | Category |
|------|---------------|----------|
| `e2e/cli_workflows.rs` | 1 | Stale — pattern CLI IS implemented |
| `e2e/mcp_integration.rs` | 1 | Process-based, manual |
| `soak/stability_test.rs` | 1 | Intentionally slow soak |

### P1: Nightly Workflow Exclusion Filter Maintenance Burden

The nightly workflow (`nightly-tests.yml:154-155`) uses a 7-line exclusion filter with 18 test names and 3 binary names. This is:
- **Fragile**: Adding/renaming a test requires updating the filter
- **Incomplete**: Does not exclude all Turso integration tests (70 ignored tests still run via `--run-ignored all` and some panic when env vars are missing)
- **Redundant**: Many excluded tests already have `#[ignore]` — the filter only matters for tests run via `--run-ignored`

### P2: Placeholder Issue URLs

5 Turso test files reference `https://github.com/tursodatabase/libsql/issues/XXX` — the actual upstream issue number was never filled in.

| File | Line |
|------|------|
| `compression_integration_test.rs` | 7 |
| `integration_test.rs` | 5 |
| `capacity_enforcement_test.rs` | 5 |
| `multi_dimension_routing.rs` | 7 |
| `cache_integration_test.rs` | 5 |

---

## Decision

### Phase 1: Fix Build (P0 — Blocking Everything)

**Goal**: `cargo build --all` compiles cleanly with zero warnings.

| Task | ID | File | Action |
|------|----|------|--------|
| Import missing cache types in lib.rs | T1.1 | `memory-storage-redb/src/lib.rs` | Add `use crate::cache::{CacheConfig, CacheMetrics, LRUCache};` |
| Fix `super::super` path in adaptive | T1.2 | `memory-storage-redb/src/cache/adaptive/mod.rs:336` | Change `super::super::CacheConfig` to `crate::cache::CacheConfig` (add `use` import) |
| Remove unused `ReadableTable` import | T1.3 | `memory-storage-redb/src/lib.rs:27` | Remove from `use redb::{...}` statement |
| Verify clean build | T1.4 | — | `cargo build --all` must exit 0 with no warnings |
| Run clippy | T1.5 | — | `cargo clippy --workspace --tests -- -D warnings` must pass |

**Effort**: 30 minutes
**Risk**: None — purely additive imports and dead import removal

### Phase 2: Fix Stale Ignore Reasons (P1 — Quick Wins)

**Goal**: Every `#[ignore]` has an accurate, current reason.

| Task | ID | File | Action |
|------|----|------|--------|
| Update pattern CLI test ignore | T2.1 | `tests/e2e/cli_workflows.rs:559` | Either remove `#[ignore]` (pattern CLI is implemented) or update reason to describe actual remaining gap |
| Fill placeholder issue URLs | T2.2 | 5 Turso test files | Replace `issues/XXX` with actual libsql issue number or a generic reference to ADR-027 |
| Verify ignore reason consistency | T2.3 | All `#[ignore]` annotations | Audit that every ignore reason matches current reality |

**Effort**: 1 hour

### Phase 3: Nightly Workflow Exclusion Refactor (P1 — Maintainability)

**Goal**: Replace brittle per-test exclusion filter with a category-based approach.

**Current problem**: The nightly workflow runs `--run-ignored all` then manually excludes ~18 tests by name. This doesn't scale and misses new tests.

| Task | ID | Action |
|------|----|--------|
| Restructure nightly exclusion by binary/crate | T3.1 | Exclude entire `memory-storage-turso` integration test binaries from `--run-ignored` (all 70 tests are blocked by libsql) |
| Add tiered nightly strategy | T3.2 | Tier 1: run `--run-ignored` on `memory-core` only (29 slow tests). Tier 2: skip sandbox/WASM tests unless `WASM_TESTS=true` env var |
| Remove per-test-name exclusions | T3.3 | Replace individual `test(name)` exclusions with `binary(name)` or `package(name)` filters |
| Document exclusion policy | T3.4 | Add comments in workflow explaining why each exclusion exists and when to remove it |

**Proposed nightly filter (replaces current 7-line filter):**

```yaml
# Tier 1: Run slow memory-core integration tests
cargo nextest run -p memory-core --run-ignored only \
  -E 'not test(test_pattern_accuracy_measurement)'

# Tier 2: Run non-Turso integration tests (skip all Turso — blocked by libsql bug per ADR-027)
cargo nextest run --all --run-ignored only \
  -E 'not (package(memory-storage-turso) | test(quality_gate_) | test(test_update_command_) | binary(soak))'
```

**Effort**: 2 hours
**Risk**: Low — only affects nightly schedule, not regular CI

### Phase 4: Reduce Fixable Ignored Tests (P2 — Test Health)

**Goal**: Un-ignore tests where the underlying issue has been resolved.

| Task | ID | Category | Tests | Action |
|------|----|----------|-------|--------|
| Un-ignore pattern CLI e2e | T4.1 | Stale | 1 | Pattern CLI is implemented — remove `#[ignore]` or update test |
| Fix sandbox timing tests | T4.2 | Flaky | 4 | Add `tokio::time::timeout` wrappers with generous CI bounds (5-10s) |
| Fix WASM binary data tests | T4.3 | WASM | 2 | Use `from_utf8_lossy` or base64 encoding instead of `String::from_utf8` |
| Document remaining unfixable | T4.4 | All | — | Update ADR-027 inventory with current counts and exit criteria |

**Effort**: 4-6 hours
**Net reduction**: 119 → ~112 (7 tests un-ignored)

### Phase 5: Test Infrastructure Improvements (P2 — Long-term)

**Goal**: Prevent future test health regressions.

| Task | ID | Action |
|------|----|--------|
| Add ignored-test count CI check | T5.1 | Script that counts `#[ignore]` annotations and fails if count exceeds threshold (current: 119, ceiling: 125) |
| Add nightly success rate tracking | T5.2 | Persist nightly test results as artifact; track pass rate trend |
| libsql version monitor | T5.3 | Dependabot or manual check for libsql releases that fix `malloc_consolidate` — when fixed, bulk un-ignore 70 Turso tests |

**Effort**: 3-4 hours

---

## GOAP Execution Plan

### World State (Before)

| Fact | Value |
|------|-------|
| `build_compiles` | ❌ false |
| `all_tests_runnable` | ❌ false (build broken) |
| `ignored_test_count` | 119 |
| `stale_ignore_reasons` | ≥ 2 |
| `nightly_exclusion_maintainable` | ❌ false (18 per-name exclusions) |
| `placeholder_issue_urls` | 5 |

### World State (After — Target)

| Fact | Value |
|------|-------|
| `build_compiles` | ✅ true |
| `all_tests_runnable` | ✅ true |
| `ignored_test_count` | ≤ 112 |
| `stale_ignore_reasons` | 0 |
| `nightly_exclusion_maintainable` | ✅ true (category-based) |
| `placeholder_issue_urls` | 0 |

### Execution Strategy: Sequential (Phase 1 blocks all others)

```
Phase 1 (P0, 30min) ──► Phase 2 (P1, 1h) ──► Phase 3 (P1, 2h)
                                                      │
                                                      ▼
                                               Phase 4 (P2, 4-6h)
                                                      │
                                                      ▼
                                               Phase 5 (P2, 3-4h)
```

| Phase | Priority | Parallel? | Gate |
|-------|----------|-----------|------|
| 1: Fix Build | P0 | No — blocks everything | `cargo build --all` exits 0 |
| 2: Fix Stale Ignores | P1 | Yes (after Phase 1) | All `#[ignore]` reasons accurate |
| 3: Nightly Refactor | P1 | Yes (after Phase 1) | Nightly workflow uses category-based filters |
| 4: Un-ignore Fixable | P2 | After Phase 1 | ≤ 112 ignored tests |
| 5: CI Guards | P2 | After Phase 4 | Ignored-test ceiling enforced |

### Quality Gates

- **After Phase 1**: `cargo build --all && cargo clippy --workspace --tests -- -D warnings`
- **After Phase 2**: `grep -r '#\[ignore' --include='*.rs' | grep -v target/ | grep 'XXX'` returns 0 results
- **After Phase 3**: Nightly workflow uses ≤ 3 exclusion rules (crate/binary level, not test-name level)
- **After Phase 4**: `grep -r '#\[ignore' --include='*.rs' | grep -v target/ | wc -l` ≤ 112
- **After Phase 5**: `./scripts/quality-gates.sh` passes

---

## Consequences

### Positive

- Build restored — unblocks all development and CI
- Every `#[ignore]` annotation has a truthful, current reason
- Nightly workflow becomes maintainable (category filters vs per-test names)
- 7 tests recovered from ignored state
- Future regressions caught by ignored-test ceiling

### Negative

- 112 ignored tests remain (70 Turso upstream, 29 slow-by-design, 13 other blocked)
- libsql upstream fix timeline unknown

### Neutral

- No API changes — purely internal quality work
- No new features or user-facing changes
- Phase 4-5 can be deferred to future sprint without blocking v0.1.20

---

## Alternatives Considered

### A: Remove all Turso ignored tests entirely

**Rejected**: Losing 70 test specifications means they'd need to be rewritten when libsql is fixed. The tests are valuable documentation of expected behavior.

### B: Mock libsql for Turso tests

**Rejected**: The tests specifically validate real database behavior (SQL injection protection, capacity enforcement, vector search). Mocking would defeat their purpose.

### C: Pin libsql to older version without the bug

**Rejected**: Older versions have other issues and security patches. The current version (0.9.29) is the latest compatible version.

### D: Ignore the build failure and work around it

**Rejected**: The build failure is in a core crate that blocks compilation of 5 downstream crates. No workaround exists.

---

## Implementation Status (Updated 2026-03-14)

Concurrent implementation by other agents completed most tasks before this ADR was formally accepted.

### Verified Completion State

| Phase | Task | Status | Evidence |
|-------|------|--------|----------|
| **1: Fix Build** | T1.1-T1.5 | ✅ Complete | `cargo build --all` succeeds; `cargo clippy --workspace --tests` clean |
| **2: Fix Stale Ignores** | T2.1 cli_workflows stale ignore | ✅ Complete | `#[ignore]` removed from `cli_workflows.rs` (commit `bf7abab`) |
| **2: Fix Stale Ignores** | T2.2 placeholder issue URLs | ✅ Complete | 0 `issues/XXX` references remain in Turso tests (commit `bf7abab`) |
| **3: Nightly Refactor** | T3.1-T3.4 | ✅ Complete | Nightly uses `package(memory-storage-turso)` filter (commit `c70db69`) |
| **4: Un-ignore Fixable** | T4.1 pattern CLI e2e | ✅ Complete | `#[ignore]` removed from `cli_workflows.rs` |
| **4: Un-ignore Fixable** | T4.2 sandbox timing | ⏳ Pending | 4 flaky sandbox tests in `sandbox/tests.rs` still ignored |
| **4: Un-ignore Fixable** | T4.3 WASM binary data | ⏳ Pending | 2 tests in `unified_sandbox/tests.rs` still ignored |
| **5: CI Guards** | T5.1 ceiling script | ✅ Complete | `scripts/check-ignored-tests.sh` exists (ceiling=125, commit `e66f4e0`) |
| **5: CI Guards** | T5.2 nightly tracking | ⏳ Pending | No artifact-based trend tracking yet |
| **5: CI Guards** | T5.3 libsql monitor | ⏳ Pending | No automated upstream version check |

### Updated Metrics

| Metric | ADR-041 Baseline | Current (2026-03-14) | Target |
|--------|------------------|----------------------|--------|
| Build compiles | ❌ (4 errors) | ✅ clean | ✅ |
| Clippy | ❌ (warnings) | ✅ clean | ✅ |
| Ignored tests | 119 | 118 | ≤ 112 |
| Stale ignore reasons | ≥ 2 | 0 | 0 |
| Placeholder `issues/XXX` | 5 | 0 | 0 |
| Nightly per-name exclusions | 18 | 0 (category-based) | 0 |
| Ceiling guard script | ❌ missing | ✅ 125 ceiling | ✅ |

### Remaining Work

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| T4.2 Fix sandbox timing tests | P2 | 2h | Add `tokio::time::timeout` to 4 tests |
| T4.3 Fix WASM binary data tests | P2 | 2h | Use `from_utf8_lossy` or base64 |
| T5.2 Nightly trend tracking | P3 | 1h | Persist pass-rate artifact |
| T5.3 libsql version monitor | P3 | 1h | Check for upstream fix |

---

## Cross-References

- **Build fix**: `memory-storage-redb/src/lib.rs`, `memory-storage-redb/src/cache/adaptive/mod.rs`
- **Ignored test policy**: [ADR-027](ADR-027-Ignored-Tests-Strategy.md)
- **Testing strategy**: [ADR-033](ADR-033-Modern-Testing-Strategy.md)
- **Previous gap analysis**: [ADR-040](ADR-040-Gap-Analysis-And-GOAP-Sprint-v0.1.19.md)
- **Nightly workflow**: `.github/workflows/nightly-tests.yml`
- **Ceiling guard**: `scripts/check-ignored-tests.sh`
- **GOAP state**: `plans/GOAP_STATE.md`
- **Goals index**: `plans/GOALS.md`
