# GOAP Execution Plan: v0.1.22 Quality & Feature Polish — COMPLETE ✅

- **Created**: 2026-03-16
- **Completed**: 2026-03-20
- **Version**: 0.1.22
- **Previous**: v0.1.21 (Publishing Infrastructure, ADR-045/ADR-046)
- **Strategy**: Sequential (Fix → Polish → Enhance → Document)
- **Branch**: `main` (PR #391 merged)
- **PR**: [#391](https://github.com/d-o-hub/rust-self-learning-memory/pull/391) ✅ Merged
- **Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
- **ADR**: [ADR-047](adr/ADR-047-v0.1.22-Quality-Feature-Polish.md)

---

## Analysis Summary

### Codebase Snapshot (2026-03-20 — FINAL VERIFIED)

| Metric | Value | Status |
|--------|-------|--------|
| Workspace version | 0.1.22 | — |
| PR #391 | ✅ Merged | All CI passing on main |
| Total tests | 2,841/2,841 | ✅ All passing |
| Skipped/ignored tests | 124 | ✅ ≤125 ceiling (70 upstream libsql) |
| Timed-out tests | 0 | ✅ |
| Failing doctests | 0 | ✅ |
| Files >500 LOC (production) | 0 | ✅ |
| `#[allow(dead_code)]` (production) | 31 | ✅ Target ≤40 met |
| Broken markdown links | 0 active | ✅ 101 archived-only (acceptable) |
| Snapshot tests | 80 | ✅ Target met |
| Property test files | 16 | ✅ Exceeds target (≥13) |
| Clippy | ✅ Clean | |
| Format | ✅ Clean | |

---

## Phase 1: Critical Bugs (P0) — WG-040–WG-042 ✅ COMPLETE

All P0 issues fixed and merged via PR #391.

### WG-040: Fix Failing Doctests ✅ DONE

**Priority**: P0
**Status**: ✅ Fixed (PR #391)
**Files**:
- `do-memory-core/src/memory/attribution/mod.rs` (line 21)
- `do-memory-core/src/memory/playbook/mod.rs` (line 24)

**Root Causes**:
1. Attribution doctest: `use of moved value: session` — `record_session(session)` moves, then `session.session_id` is accessed. Fix: clone session before passing.
2. Playbook doctest: `generate()` is sync but doctest `.await`s it; also missing `context` field in `PlaybookRequest`. Fix: remove `.await`, add missing field.

**Actions**:
- ACT-053: Fix attribution doctest (clone session before use)
- ACT-054: Fix playbook doctest (remove `.await`, add missing field)

### WG-041: Fix Test Timeout ✅ DONE

**Priority**: P0
**Status**: ✅ Fixed (PR #391)
**File**: `tests/e2e/quality_gates.rs` — `quality_gate_no_clippy_warnings`

**Root Cause**: This test runs `cargo clippy` internally and times out at 120s. The test is redundant with CI checks.

**Actions**:
- ACT-055: Add `#[ignore]` with reason "runs full clippy internally; covered by CI" or increase timeout

### WG-042: Fix >500 LOC Production Files ✅ DONE

**Priority**: P0 (project invariant)
**Status**: ✅ Fixed — all 3 files split (PR #391)
**Files**:
1. `do-memory-core/src/memory/playbook/generator.rs` — 631 LOC
2. `do-memory-mcp/src/bin/server_impl/tools/memory_handlers.rs` — 608 LOC
3. `do-memory-core/src/memory/management.rs` — 504 LOC

**Actions**:
- ACT-056: Split `generator.rs` into `generator.rs` + `templates.rs` (extract template functions)
- ACT-057: Split `memory_handlers.rs` into `memory_handlers.rs` + `feature_handlers.rs` (extract playbook/checkpoint/feedback handlers)
- ACT-058: Extract helper methods from `management.rs` into `management_helpers.rs`

---

## Phase 2: Quality Polish (P1) — WG-043–WG-046 ✅ COMPLETE

All P1 quality issues resolved via PR #391.

### WG-043: Reduce `#[allow(dead_code)]` in Production Code ✅ DONE

**Priority**: P1
**Final**: 31 annotations in production code (verified 2026-03-20)
**Target**: ≤40 — ✅ Target met

**Hotspots**:
- `do-memory-core/src/memory/core/struct_priv.rs` — 5 annotations
- `do-memory-core/src/memory/types.rs` — 6 annotations
- `do-memory-core/src/embeddings/real_model/model.rs` — 8 annotations
- `do-memory-core/src/embeddings/openai/utils.rs` — 5 annotations
- `do-memory-core/src/embeddings/provider.rs` — 3 annotations
- `do-memory-core/src/monitoring/storage/mod.rs` — 3 annotations

**Actions**:
- ACT-059: Audit dead_code in `types.rs` — remove or use suppressed fields
- ACT-060: Audit dead_code in `embeddings/` — remove unused model infrastructure or add `#[cfg]` guards
- ACT-061: Audit dead_code in `monitoring/storage/` — wire or remove

### WG-044: Reduce Broken Markdown Links ✅ DONE

**Priority**: P1
**Final**: 0 active broken links (verified 2026-03-20)
**Target**: ≤80 — ✅ Target met (101 archived-only, acceptable)

**Actions**:
- ACT-062: Fix broken links in active documentation (non-archived files)
- ACT-063: Validate links in newly added playbook/attribution/checkpoint docs

### WG-045: Expand Snapshot Tests ✅ COMPLETE

**Priority**: P1
**Current**: 80 snapshots (verified 2026-03-20)
**Target**: ≥80 — ✅ Target met

**Actions**:
- ACT-064: Add snapshot tests for new MCP tools (checkpoint, handoff, feedback, playbook)
- ACT-065: Add snapshot tests for new CLI commands (playbook recommend, episode checkpoint)

### WG-046: Expand Property Tests ✅ COMPLETE

**Priority**: P1
**Current**: 16 property test files (verified 2026-03-20)
**Target**: ≥13 — ✅ Exceeds target

**Actions**:
- ACT-066: Add property tests for PlaybookGenerator (various input combinations produce valid playbooks)
- ACT-067: Add property tests for RecommendationTracker (feedback scoring invariants)
- ACT-068: Add property tests for CheckpointManager (serialization round-trips)

---

## Phase 3: Feature Enhancements (P2) — WG-047–WG-050 ✅ COMPLETE

All P2 issues verified complete via PR #391.

### WG-047: MCP Tool Contract Parity for New Features

**Priority**: P2
**Issue**: New checkpoint/feedback/playbook MCP tools may not be fully covered by tool contract parity tests.

**Actions**:
- ACT-069: Verify all new tools in `tool_contract_parity.rs` test
- ACT-070: Add handler dispatch tests for checkpoint, feedback, playbook tools

### WG-048: Attribution Integration Test

**Priority**: P2
**Issue**: Attribution and checkpoint modules have unit tests but no integration tests that test the full flow.

**Actions**:
- ACT-071: Add integration test for: create episode → recommend patterns → record session → record feedback → verify stats
- ACT-072: Add integration test for: create episode → checkpoint → handoff pack → resume

### WG-049: Changelog Automation (git-cliff) ✅ DONE

**Priority**: P2
**Status**: ✅ Complete — `.github/workflows/changelog.yml` exists

**Actions**:
- ACT-073: Wire git-cliff into release workflow to auto-generate changelog entries

### WG-050: Documentation for New Features ✅ DONE

**Priority**: P2
**Status**: ✅ Complete (PR #391)

**Actions**:
- ACT-074: Add playbook usage examples to `docs/` or `README.md`
- ACT-075: Add checkpoint/handoff usage guide

---

## Phase 4: Infrastructure (P3) — WG-051–WG-053 ✅ COMPLETE

All P3 infrastructure items complete via PR #391.

### WG-051: Nightly Trend Tracking (ADR-041 T5.2) ✅ DONE

**Priority**: P3
**Status**: ✅ Complete (PR #391)

**Actions**:
- ACT-076: Add artifact upload for test results in nightly workflow

### WG-052: libsql Version Monitor (ADR-041 T5.3) ✅ DONE

**Priority**: P3
**Status**: ✅ Complete — `scripts/check-libsql-version.sh` exists

**Actions**:
- ACT-077: Create `scripts/check-libsql-version.sh` to track upstream fixes

### WG-053: Structured Tech-Debt Registry ✅ DONE

**Priority**: P3
**Status**: ✅ Complete — `docs/TECH_DEBT.md` exists

**Actions**:
- ACT-078: Create `docs/TECH_DEBT.md` with categorized entries and tracking

---

## Quality Gates for v0.1.22 — ALL PASSING ✅

- [x] `cargo fmt --all -- --check` passes ✅
- [x] `cargo clippy --workspace --tests -- -D warnings` passes ✅
- [x] `cargo build --all` succeeds ✅
- [x] `cargo nextest run --all` passes (0 timeouts, excluding ignored) ✅
- [x] `cargo test --doc --all` passes (0 failures) ✅
- [x] `./scripts/quality-gates.sh` passes ✅
- [x] No production source file >500 LOC ✅
- [x] Documentation updated (CURRENT.md, ROADMAP_ACTIVE.md, GOAP_STATE.md) ✅

---

## Execution Order — ALL PHASES COMPLETE ✅

```
Phase 1 (P0): WG-040 → WG-041 → WG-042    ✅ COMPLETE (PR #391 merged)
Phase 2 (P1): WG-043 → WG-044 → WG-045 → WG-046    ✅ COMPLETE (PR #391 merged)
Phase 3 (P2): WG-047 → WG-048 → WG-049 → WG-050    ✅ COMPLETE (PR #391 merged)
Phase 4 (P3): WG-051 → WG-052 → WG-053    ✅ COMPLETE (PR #391 merged)
```

Sprint complete. All 12 issues closed.

---

## Cross-References

- **Previous sprint**: [GOAP_EXECUTION_PLAN_v0.1.21.md](GOAP_EXECUTION_PLAN_v0.1.21.md)
- **Active roadmap**: [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md)
- **Current status**: [STATUS/CURRENT.md](STATUS/CURRENT.md)
- **ADR-044**: [adr/ADR-044-High-Impact-Features-v0.1.20.md](adr/ADR-044-High-Impact-Features-v0.1.20.md)
