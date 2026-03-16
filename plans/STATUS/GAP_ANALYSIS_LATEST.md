# Gap Analysis — v0.1.22 Sprint

**Generated**: 2026-03-16
**Method**: Comprehensive codebase analysis (build, test, clippy, doctest, LOC, dead_code, links)
**Scope**: ADR-044 features shipped, quality polish, infrastructure backlog

---

## Executive Summary

| Area | Status | Gap Count | Priority |
|------|--------|-----------|----------|
| **ADR-044 Features** | ✅ Shipped | 2 doctest bugs | P0 |
| **File Size Compliance** | 🔴 3 violations | 3 files >500 LOC | P0 |
| **Test Health** | 🟡 1 timeout | 1 test + 2 doctests | P0 |
| **Dead Code** | 🟡 70 annotations | Target ≤40 | P1 |
| **Docs Integrity** | 🟡 149 broken links | Target ≤80 | P1 |
| **Snapshot Tests** | 🟡 65 snapshots | Target ≥80 | P1 |
| **Property Tests** | 🟡 10 files | Target ≥15 | P1 |
| **ADR-041 Remaining** | ⏳ 2 tasks | T5.2, T5.3 | P3 |

---

## Critical Gaps (P0)

### Gap 1: Failing Doctests (2 failures)

| File | Error | Fix |
|------|-------|-----|
| `memory-core/src/memory/attribution/mod.rs:21` | `use of moved value: session` — session moved into `record_session()` then accessed | Clone session before passing |
| `memory-core/src/memory/playbook/mod.rs:24` | `generate()` is sync but doctest `.await`s it; missing `context` field | Remove `.await`, add `context` field |

**Impact**: `cargo test --doc --all` fails → blocks release.

### Gap 2: Production Files >500 LOC (3 violations)

| File | Lines | Violation |
|------|-------|-----------|
| `memory-core/src/memory/playbook/generator.rs` | 631 | +131 over limit |
| `memory-mcp/src/bin/server_impl/tools/memory_handlers.rs` | 608 | +108 over limit |
| `memory-core/src/memory/management.rs` | 504 | +4 over limit |

**Impact**: Breaks project invariant "0 production source files >500 LOC".

### Gap 3: Test Timeout

| Test | Duration | Issue |
|------|----------|-------|
| `quality_gate_no_clippy_warnings` | >120s timeout | Runs full `cargo clippy` inside test; redundant with CI |

**Impact**: `cargo nextest run --all` reports 1 timeout → blocks release.

---

## Quality Gaps (P1)

### Gap 4: `#[allow(dead_code)]` Annotations

**Current**: 70 in production code (non-test)
**Target**: ≤40

| Hotspot | Count | Notes |
|---------|-------|-------|
| `embeddings/real_model/model.rs` | 8 | Model infrastructure for ONNX/candle; may need `#[cfg]` |
| `memory/types.rs` | 6 | Duplicate/stale types |
| `embeddings/openai/utils.rs` | 5 | Utility functions not called |
| `memory/core/struct_priv.rs` | 5 | Private fields on core struct |
| `embeddings/provider.rs` | 3 | Provider infrastructure |
| `monitoring/storage/mod.rs` | 3 | Monitoring structs not wired |
| Other files | 40 | Scattered |

### Gap 5: Broken Markdown Links

**Current**: 149 (up from 89 at v0.1.20)
**Increase cause**: New features added documentation with links to files that don't exist or have wrong paths.
**Target**: ≤80

| Category | Count |
|----------|-------|
| Archived files | ~90 (acceptable) |
| Active documentation | ~30 (fix) |
| New feature docs | ~29 (fix) |

### Gap 6: Missing Snapshot Tests for New Features

**Current**: 65 snapshots, **Target**: ≥80

New features added in v0.1.22 (playbook, attribution, checkpoint, feedback) have **0 snapshot tests** for:
- MCP tool responses (checkpoint_episode, get_handoff_pack, resume_from_handoff)
- MCP tool responses (record_recommendation_session, record_recommendation_feedback)
- MCP tool responses (recommend_playbook)
- CLI output (playbook recommend, episode checkpoint)

### Gap 7: Property Test Coverage

**Current**: 10 property test files, **Target**: ≥15

Missing property tests for:
- `PlaybookGenerator` — various input combinations
- `RecommendationTracker` — feedback scoring invariants
- `CheckpointManager` — checkpoint/handoff serialization
- `RecommendationSession/Feedback` types — serialization round-trips
- `HandoffPack` — serialization invariants

---

## Feature Completeness Assessment

### ADR-044 Features (v0.1.22)

| Feature | Core | MCP | CLI | Unit Tests | Integration Tests | Doctests | Snapshots |
|---------|------|-----|-----|------------|-------------------|---------|-----------|
| Playbooks | ✅ | ✅ | ✅ | 26 tests | ❌ Missing | 🔴 Broken | ❌ None |
| Attribution | ✅ | ✅ | ✅ | 8 tests | ❌ Missing | 🔴 Broken | ❌ None |
| Checkpoints | ✅ | ✅ | ✅ | 6 tests | ❌ Missing | ✅ OK | ❌ None |
| Feedback | ✅ | ✅ | ✅ | 3 tests | ❌ Missing | ✅ OK | ❌ None |

**Total**: 43 unit tests for new features. Missing integration tests and snapshots.

### MCP Tool Registry

All new tools are registered and dispatchable:
- ✅ `recommend_playbook`
- ✅ `record_recommendation_session`
- ✅ `record_recommendation_feedback`
- ✅ `checkpoint_episode`
- ✅ `get_handoff_pack`
- ✅ `resume_from_handoff`

### CLI Commands

All new commands are wired and dispatch correctly:
- ✅ `playbook recommend`
- ✅ `playbook explain`
- ✅ `feedback record`
- ✅ `feedback stats`
- ✅ `episode checkpoint`
- ✅ `episode handoff`

---

## Infrastructure Backlog (Carried Forward)

| Item | Since | Priority | Status |
|------|-------|----------|--------|
| Changelog automation (git-cliff) | v0.1.17 | P2 | Not started |
| Nightly trend tracking (T5.2) | v0.1.20 | P3 | Not started |
| libsql version monitor (T5.3) | v0.1.20 | P3 | Not started |
| Structured tech-debt registry | v0.1.17 | P3 | Not started |
| CLI workflow parity generator | v0.1.17 | P3 | Not started |

---

## Pre-Existing Issues (Unchanged)

| Issue | Status | Notes |
|-------|--------|-------|
| 113 ignored tests | 🟡 | 70 Turso upstream bug, rest by design/slow |
| `execute_agent_code` MCP tool disabled | 🟡 | WASM sandbox issues |
| 134 duplicate dep roots | 🟡 | Architectural limit (wasmtime/libsql) |

---

## Recommended Sprint Priorities

### Must-Fix for v0.1.22 Tag

1. **Fix 2 failing doctests** (ACT-053, ACT-054) — 30min
2. **Fix test timeout** (ACT-055) — 15min
3. **Split 3 >500 LOC files** (ACT-056, ACT-057, ACT-058) — 2-3h

### Should-Fix for v0.1.22

4. **Add snapshot tests for new features** (ACT-064, ACT-065) — 2h
5. **Fix active broken markdown links** (ACT-062, ACT-063) — 2h
6. **Add property tests** (ACT-066, ACT-067, ACT-068) — 2h

### Nice-to-Have

7. **Reduce dead_code annotations** (ACT-059–ACT-061) — 3h
8. **MCP tool contract parity for new tools** (ACT-069, ACT-070) — 1h
9. **Integration tests for new features** (ACT-071, ACT-072) — 3h

---

## Cross-References

- **Execution plan**: [GOAP_EXECUTION_PLAN_v0.1.22.md](../GOAP_EXECUTION_PLAN_v0.1.22.md)
- **Current status**: [CURRENT.md](CURRENT.md)
- **Active roadmap**: [ROADMAPS/ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
- **ADR-044**: [adr/ADR-044-High-Impact-Features-v0.1.20.md](../adr/ADR-044-High-Impact-Features-v0.1.20.md)
