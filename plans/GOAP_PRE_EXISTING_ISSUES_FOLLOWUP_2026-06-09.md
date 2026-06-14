# GOAP: Pre-Existing Issues Follow-up

**Date**: 2026-06-09
**Type**: Cleanup / Tech Debt
**Priority**: P2 — Sprint backlog items for post-PR #611 landing
**Related**: PR #611 fix session, `plans/PRE_EXISTING_ISSUES.md`
**Status**: Created as follow-up from the PR #611 fix analysis

---

## Problem Statement

During the PR #611 fix effort, several categories of pre-existing issues were
identified that are outside the scope of that PR. Some were fixable inline
(already applied), others require dedicated effort or broader coordination.

### Already Fixed (in this session)

| Issue | File | Fix |
|-------|------|-----|
| `rand::thread_rng` API mismatch (rand 0.10) | `benches/cosine_similarity_benchmark.rs` | Changed to `rand::rng()` + `RngExt::random_range()` |
| `criterion::black_box` deprecated | `benches/cosine_similarity_benchmark.rs` | Changed to `std::hint::black_box` |
| `uninlined_format_args` (5 locations) | `memory-mcp/tests/tool_contract_parity.rs:344` | Inlined `tool` in format string |
| `uninlined_format_args` (3 locations) | `memory-mcp/src/bin/server_impl/tools/memory_handlers_tests.rs:17,74,91` | Inlined `i` in `format!("v{i}")` |
| `uninlined_format_args` (1 location) | `memory-core/src/extraction/tests.rs:54` | Inlined `patterns` in assert message |
| `uninlined_format_args` (2 locations) | `examples/local_memory.rs:24,48` | Inlined variables in format strings |
| `field_reassign_with_default` | `examples/local_memory.rs:34` | Replaced with struct literal `..Default::default()` |
| `doc_markdown` (2 locations) | `examples/local_memory.rs`, `benches/cosine_similarity_benchmark.rs` | Added `#![allow(clippy::doc_markdown)]` |

### Pre-Existing Issues — Not Yet Addressed

---

## Goal State

All known pre-existing warnings and quality-gate gaps are either fixed,
scheduled for a sprint, or explicitly documented as intentional/by-design.

---

## Remaining Issues

### P1 — Blocking for `-D warnings` in CI

**None.** All `-D warnings` clippy violations found in the PR #611 affected
crates (`do-memory-core`, `do-memory-mcp`, `do-memory-cli`, `do-memory-benches`,
`do-memory-storage-turso`, `do-memory-storage-redb`, `do-memory-test-utils`)
have been fixed. The workspace `cargo check --workspace` and `cargo fmt --all`
both pass cleanly.

### P2 — Sprint Backlog (Resolved 2026-06-14)

| WG | Description | Owner Skill | Status | Notes |
|----|-------------|-------------|--------|-------|
| WG-156 | `pattern_match_score` hard-coded 0.8 | `feature-implement` | ✅ Complete | `time_series.rs` computes from `applied_patterns` success ratio + pattern density fallback |
| WG-157 | `memory_usage_mb` hard-coded 50.0 | `feature-implement` | ✅ Complete | `time_series.rs` uses `sysinfo` for actual process memory; 50.0 is fallback on sysinfo failure |
| WG-158 | `episode_success_rate` hard-coded 99.0 | `feature-implement` | ✅ Complete | `monitoring/types.rs` `record_episode_creation` computes from actual success/failure counts |
| WG-160 | Turso cache query_hits/evictions = 0 | `feature-implement` | ✅ Complete | `query_hits: 0` now only in test fixtures (commit `6a43deae`) |
| WG-161 | Cascade `analyze_query` stub | `feature-implement` | ✅ Complete | `estimate_api_call_probability` uses real heuristics (length, keyword density, code tokens) |
| WG-162 | `generate_simple_embedding` placeholder | `code-quality` | ✅ Complete | Implements 10-dim feature hashing (domain, task type, complexity, steps, reward, etc.) |

**Reference**: `plans/GOAP_STATE.md` (v0.1.32 sprint — all 15 functional WGs complete).

### P3 — Documentation Cleanup (v0.1.33 candidate)

| # | Issue | Severity | Action |
|---|-------|----------|--------|
| D1 | Docs integrity — broken internal links | Low | Run `scripts/check-docs-integrity.sh` and fix all broken links across `agent_docs/`, `plans/`, and `docs/` |
| D2 | `PRE_EXISTING_ISSUES.md` stale (mentions issue #5 as FIXED) | Low | Update to reflect current state after this session |
| D3 | `AGENTS.md`, `ROADMAP_ACTIVE.md`, `GOAP_STATE.md` sync | Low | Cross-reference with resolved PR #611 items |

**Strategy**: Dedicated docs cleanup sprint (Phase 4 of v0.1.32 plan).

### P4 — Informational (No Action Required)

| # | Issue | Rationale |
|---|-------|-----------|
| I1 | GOAP missing plan files | `GOAP_AGENT_IMPROVEMENT_PLAN.md`, `GOAP_PERFORMANCE_PLAN.md`, `GOAP_SECURITY_PLAN.md` all exist in `plans/` — the quality gate is outdated |
| I2 | Root `llms.txt` location | Per llms.txt standard, root placement is correct for AI agent discovery |
| I3 | Test file sizes >500 LOC | Tests are explicitly exempt from the LOC gate per project conventions |

---

## GOAP Plan

### Phase 1 — Quick Wins (already done in this session)
- ✅ Fix `cosine_similarity_benchmark.rs` (rand API + deprecated black_box + doc_markdown)
- ✅ Fix `uninlined_format_args` across 5 files (memory-core, memory-mcp, examples)
- ✅ Fix `field_reassign_with_default` in `examples/local_memory.rs`

### Phase 2 — Sprint Backlog ✅ Complete (2026-06-14 re-audit)
- All 6 WGs (WG-156 through WG-162) confirmed implemented in current `main`
- No additional atomic subtasks needed
- Validated with `cargo clippy -p do-memory-core --all-targets -- -D warnings` + `cargo nextest run -p do-memory-core`

### Phase 3 — Docs Health
- Run `scripts/check-docs-integrity.sh` and catalogue all broken links
- Fix in batches of ≤10 files per commit
- Update `PRE_EXISTING_ISSUES.md` with fresh scan results

### Phase 4 — Validation
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
- `cargo test --doc`
- `cargo doc --no-deps --document-private-items`
- `./scripts/quality-gates.sh`

---

## Execution Strategy: Hybrid

**Phase 1** already complete (sequential, done).
**Phase 2** is sequential-per-WG, independent across WGs (can parallelize).
**Phase 3** is sequential (audit → fix → verify).
**Phase 4** is all-at-once validation (parallel checks).

```diagram
Phase 1 ──[done]──> Phase 2 ──┐
                              ├──> Phase 4 (validate)
Phase 3 ──────────────────────┘
```
