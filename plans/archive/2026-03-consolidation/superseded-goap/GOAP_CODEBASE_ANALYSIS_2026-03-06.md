# GOAP Codebase Analysis — 2026-03-06

**Status**: 🔄 Quick wins executed — ADR-029 Phase 3/4 items shipped
**Methodology**: GOAP (Analyze → Decompose → Strategize → Coordinate → Execute → Synthesize) + ADR cross-reference
**Branch**: `main` (commit 09062b7)
**Last Updated**: 2026-03-06 (quick wins: Dependabot grouping, benchmark permissions, CI concurrency, matrix cleanup)

---

## Phase 1: ANALYZE — Live Codebase Metrics vs ADR Targets

### Codebase Overview

| Metric | Current (2026-03-06) | Previous (2026-02-25) | Change |
|--------|----------------------|-----------------------|--------|
| Total Rust files | **856** | 818 | +38 |
| Total LOC | **~204K** | ~205K | ~stable |
| Workspace version | v0.1.16 | v0.1.16 | — |
| Rust edition | 2024 | 2024 | ✅ |
| Test count (nextest) | ~2292 passed, ~69 skipped | 2289 passed, 73 skipped | +3 fixed |

### ADR Implementation Status Matrix

| ADR | Title | Status | Gap Summary |
|-----|-------|--------|-------------|
| **ADR-024** | MCP Lazy Tool Loading | ✅ Implemented | `tools/describe` + `describe_batch` endpoints still pending |
| **ADR-025** | Project Health Remediation | 🟡 Partial | Phase A ✅, Phase B 🟡, Phase C 🔴, Phase D 🔴 |
| **ADR-028** | Feature Enhancement Roadmap | 🔴 2/14 complete | 12 features pending across 3 horizons |
| **ADR-029** | GitHub Actions Modernization | 🟡 Mostly Done | Phases 1-3 done, Phase 4 mostly done (#15 deferred) |
| **ADR-032** | Disk Space Optimization | 🟡 Partial | Mold ✅, profiles ✅, CI isolation ✅, cleanup deferred |
| **ADR-033** | Modern Testing Strategy | 🟡 Partial | nextest ✅, mutants ✅, proptest 🟡, insta 🟡 |
| **ADR-034** | Release Engineering | 🟡 Partial | semver-checks ✅, release.toml ✅, cargo-dist ✅, git-cliff ✅ configured + verified |
| **ADR-035** | Rust 2024 Edition | ✅ Complete | All 9 crates on edition 2024 |
| **ADR-036** | Dependency Deduplication | 🟡 Monitoring | 129 duplicate roots (target: <80) |

---

## Phase 2: DECOMPOSE — Gap Analysis by Priority

### 🔴 P0: CRITICAL — Missing Implementation

#### G1: Error Handling (ADR-028 #4)
- **Current**: 816 `unwrap()` + 105 `.expect()` = **921 total** across 203 files (including test-adjacent code)
- **Target**: ≤20 unwrap/expect in production code
- **Effort**: 20-30 hours (crate-by-crate)
- **ADR**: ADR-028 #4, ADR-025 Phase C
- **Risk**: Public API breakage → mitigate with phased rollout

#### G2: Ignored Test Rehabilitation (ADR-028 #5, ADR-027)
- **Current**: **58 ignored tests**
- **Target**: ≤10
- **Breakdown**: 6 timing-dependent (fixable), ~30 slow (by design), 5 WASM, 3 missing features, 14 other
- **Effort**: 8-12 hours
- **ADR**: ADR-028 #5

### 🟠 P1: HIGH — Feature Completion & Enhancements

#### G3: Embeddings Integration Completion (ADR-028 #7)
- **Status**: CLI commands exist ✅, MCP tools pending
- **Missing MCP tools**: `generate_embedding`, `search_by_embedding`, `embedding_provider_status`
- **Effort**: 4-6 hours
- **ADR**: ADR-028 #7

#### G4: ADR-024 Integration Tests (MCP Lazy Loading)
- **Current**: Lazy loading implemented, no integration tests
- **Missing**: Tests for `lazy=true/false` parameter, `tools/describe` endpoint, `tools/describe_batch` endpoint
- **Effort**: 3-4 hours
- **ADR**: ADR-024

#### ~~G5: Changelog Automation (ADR-034 Phase 4)~~ ✅ DONE (2026-03-06)
- **Status**: `cliff.toml` configured, `git-cliff` installed and verified producing changelogs
- **Remaining**: Conventional commit CI enforcement (optional)
- **ADR**: ADR-034 Phase 4

#### ~~G6: GitHub Actions Security Hardening (ADR-029 Phase 3-4)~~ ✅ DONE (2026-03-06)
- **Completed**: Split benchmark permissions (PR=read, main=write), Dependabot grouping (cargo patch/minor + GH Actions), CI concurrency fix (ref instead of run_id), removed single-value matrix, yamllint already pinned, security.yml triggers already scoped
- **ADR**: ADR-029 Phase 3-4

#### G7: Property Test Expansion (ADR-033)
- **Current**: 2 proptest files, memory-core only
- **Target**: Proptest in all 4 main crates (serialization roundtrips, state machines)
- **Effort**: 6-8 hours
- **ADR**: ADR-033 Phase 5

#### G8: Snapshot Test Expansion (ADR-033)
- **Current**: 13 snapshot files
- **Target**: ≥25 (MCP tool response schemas, CLI output, error messages)
- **Effort**: 4-6 hours
- **ADR**: ADR-033 Phase 6

### 🟡 P2: MEDIUM — Code Quality & Optimization

#### G9: dead_code Cleanup
- **Current**: **140 `allow(dead_code)` annotations**
- **Target**: ≤10
- **Genuinely unused**: ~64 (rest are feature-gated/error variants/test helpers)
- **Effort**: 8-12 hours

#### G10: Dependency Deduplication (ADR-036)
- **Current**: 129 duplicate roots
- **Target**: <80
- **Key duplicates**: rand (3v), hashbrown (5v), getrandom (4v), itertools (3v)
- **Action**: Run cargo-machete/shear, prune unused features, track upstream unification
- **Effort**: 4-6 hours (Tier 1-2); ongoing for upstream tracking

#### G11: Adaptive TTL Phase 2 (ADR-028 #6)
- **Current**: Static TTL in redb cache
- **Target**: Access-frequency-based TTL adjustment
- **Effort**: 8-12 hours
- **ADR**: ADR-028 #6

#### G12: Transport Compression (ADR-028 #8)
- **Current**: No compression on Turso wire protocol
- **Target**: Optional Zstd compression
- **Effort**: 4-6 hours
- **ADR**: ADR-028 #8

### 🟢 P3: LOW — Long-Term Vision (Q3-Q4 2026)

| Feature | ADR-028 # | Status | Effort |
|---------|-----------|--------|--------|
| Distributed Memory Sync (CRDTs) | #9 | Not Started | Very High |
| Observability Stack (Prometheus + OTEL) | #10 | Not Started | High |
| Multi-Tenancy & RBAC | #11 | Not Started | Very High |
| Real-Time Pattern Learning | #12 | Not Started | High |
| Custom Embedding Models (ONNX) | #13 | Not Started | Medium |
| A/B Testing Framework | #14 | Not Started | High |

---

## Phase 3: STRATEGIZE — Execution Plan

### Strategy: Hybrid (Parallel + Sequential, 4-week sprint)

```
Week 1 (Mar 9-14) — Quick Wins + Foundation:
  PARALLEL:
    ├─ G5: Changelog automation (git-cliff config) ─── 3-4h
    ├─ G6: GH Actions security hardening ──────────── 2-3h
    ├─ G4: ADR-024 integration tests ──────────────── 3-4h
    └─ G10-T1: cargo-machete/shear audit ──────────── 2h

Week 2 (Mar 17-21) — Error Handling + Testing:
  SEQUENTIAL:
    └─ G1: Error handling — memory-core ─────────── 8-10h
  PARALLEL:
    ├─ G2: Test triage (fix 6 timing-dependent) ──── 4-6h
    └─ G7: Proptest expansion (storage crates) ───── 4-6h

Week 3 (Mar 24-28) — Error Handling + Features:
  PARALLEL:
    ├─ G1: Error handling — turso + redb ──────────── 8-12h
    ├─ G3: MCP embedding tools ────────────────────── 4-6h
    └─ G8: Snapshot test expansion ────────────────── 4-6h

Week 4 (Mar 31 - Apr 4) — Cleanup + Polish:
  PARALLEL:
    ├─ G1: Error handling — mcp + cli ─────────────── 8-12h
    ├─ G9: dead_code triage ───────────────────────── 8-12h
    └─ G10-T2: Feature pruning (unused features) ──── 3-4h
```

### Dependency Graph

```
G5 (git-cliff) ─────────────────────────────┐
G6 (GH Actions) ────────────────────────────┤
G4 (ADR-024 tests) ─────────────────────────┤
G10-T1 (dep audit) ─────────────────────────┤
                                             ├──→ G1 (error handling, 4 crates)
G2 (test triage) ────────────────────────────┤         │
G7 (proptest expansion) ─────────────────────┤         ├──→ G11 (adaptive TTL, after G1)
                                             │         └──→ G12 (compression, after G3)
G3 (MCP embedding tools) ───────────────────┤
G8 (snapshot expansion) ────────────────────┤
G9 (dead_code) ─────────────────────────────┤
G10-T2 (feature pruning) ───────────────────┘
```

---

## Phase 4: Summary — What's Missing, New, or Needs Enhancement

### Missing Implementation (Not Started)

| # | Item | ADR | Category | Effort |
|---|------|-----|----------|--------|
| 1 | Error handling audit (921 unwrap/expect) | ADR-028 #4 | Code Quality | 20-30h |
| 2 | `tools/describe` + `describe_batch` MCP endpoints | ADR-024 | Feature | 3-4h |
| 3 | MCP embedding tools (generate, search, status) | ADR-028 #7 | Feature | 4-6h |
| 4 | git-cliff integration + conventional commit CI | ADR-034 | Automation | 3-4h |
| 5 | GH Actions Phase 3 security hardening | ADR-029 | Security | 2-3h |
| 6 | Adaptive TTL Phase 2 (redb cache) | ADR-028 #6 | Feature | 8-12h |
| 7 | Transport compression (Zstd for Turso) | ADR-028 #8 | Performance | 4-6h |
| 8 | Distributed Memory Sync (CRDTs) | ADR-028 #9 | Long-term | Very High |
| 9 | Observability Stack (Prometheus + OTEL) | ADR-028 #10 | Long-term | High |
| 10 | Multi-Tenancy & RBAC | ADR-028 #11 | Long-term | Very High |
| 11 | Real-Time Pattern Learning | ADR-028 #12 | Long-term | High |
| 12 | Custom Embedding Models (ONNX) | ADR-028 #13 | Long-term | Medium |
| 13 | A/B Testing Framework | ADR-028 #14 | Long-term | High |

### Needs Enhancement (Partially Done)

| # | Item | Current State | Target | ADR |
|---|------|---------------|--------|-----|
| 1 | Ignored tests | 58 ignored | ≤10 | ADR-028 #5 |
| 2 | dead_code annotations | 140 | ≤10 | — |
| 3 | Property testing | 2 files (memory-core only) | 4+ crates | ADR-033 |
| 4 | Snapshot testing | 13 snapshots | ≥25 | ADR-033 |
| 5 | Dependency deduplication | 129 roots | <80 | ADR-036 |
| 6 | GH Actions optimization | Phase 1-2 done | Phase 4 done | ADR-029 |
| 7 | ADR-025 remediation | Phase A done | All phases | ADR-025 |

### New Feature Opportunities (Not in ADRs)

| # | Opportunity | Rationale | Effort |
|---|------------|-----------|--------|
| 1 | **Test file size compliance** | 24 test files >500 LOC — could benefit from splitting for maintainability | Medium |
| 2 | **CLI warm-start optimization** | Cross-subprocess episode persistence still has gaps (PR #297) | 4-6h |
| 3 | **Dependabot PR grouping** | Reduce PR noise from dep updates (ADR-029 Phase 4 #13) | 1h |
| 4 | **PR/Issue templates** | Missing (ADR-029 Phase 4 #16) | 1h |
| 5 | **`cargo public-api` in quality gates** | Track API surface changes automatically | 1h |

---

## Phase 5: ADR Traceability Matrix

| ADR | Affected Tasks | Status | Next Action |
|-----|---------------|--------|-------------|
| ADR-024 | G4 | 🟡 Tests needed | Write integration tests for lazy parameter |
| ADR-025 | G1, G2, G3 | 🟡 Phase A done | Complete Phases B-D |
| ADR-027 | G2 | 🟡 Categorized | Fix 6 timing-dependent tests |
| ADR-028 | G1-G12, P3 | 🔴 2/14 done | Error handling (P0), batch ✅, features |
| ADR-029 | G6 | 🟡 Phase 1-2 | Complete Phase 3-4 |
| ADR-030 | G2 | ✅ Patterns doc'd | Fix remaining flaky tests |
| ADR-032 | — | ✅ Core done | Monitor target/ size |
| ADR-033 | G7, G8 | 🟡 Foundation | Expand proptest + insta |
| ADR-034 | G5 | 🟡 Tooling ready | Configure git-cliff |
| ADR-035 | — | ✅ Complete | No action |
| ADR-036 | G10 | 🟡 Monitoring | Active cleanup when blocking |

---

## Risk Register

| Risk | Prob | Impact | Mitigation |
|------|------|--------|------------|
| Error handling changes break public API | Med | High | Crate-by-crate rollout, start with memory-core |
| Large unwrap audit causes merge conflicts | Med | Med | Short-lived feature branches, daily merges |
| Dependency dedup may break features | Low | Med | Use `cargo-machete --fix` with validation |
| git-cliff config may not match commit history | Low | Low | Rewrite history not needed; start fresh |
| Property tests may be flaky on CI | Med | Low | Use deterministic seeds, nextest retries |

---

## Success Criteria (v0.1.17 Exit Gate)

- [ ] unwrap()/expect() in prod ≤ 50 (phased from current ~921)
- [ ] Ignored tests ≤ 30 (fix 28 of 58)
- [ ] dead_code annotations ≤ 80 (remove 60 of 140)
- [ ] Property test crates ≥ 3 (from 1)
- [ ] Snapshot tests ≥ 20 (from 13)
- [ ] git-cliff configured and generating changelogs
- [ ] GH Actions Phase 3 security hardening complete
- [ ] ADR-024 integration tests passing
- [ ] Duplicate deps ≤ 120 (from 129)
- [ ] All CI workflows green

---

---

## Execution Log

### 2026-03-06: Quick Wins Executed

| Change | File | ADR | Status |
|--------|------|-----|--------|
| Dependabot grouping (cargo patch/minor + GH Actions) | `.github/dependabot.yml` | ADR-029 #13 | ✅ Done |
| Benchmark permissions split (PR=read, main=write) | `.github/workflows/benchmarks.yml` | ADR-029 #8 | ✅ Done |
| Benchmark store/stash guard for PR events | `.github/workflows/benchmarks.yml` | ADR-029 #8 | ✅ Done |
| CI concurrency fix (github.ref vs run_id) | `.github/workflows/ci.yml` | ADR-029 #12 | ✅ Done |
| Remove single-value matrix (rust: [stable]) | `.github/workflows/ci.yml` | ADR-029 #14 | ✅ Done |
| git-cliff verified working | `cliff.toml` (existing) | ADR-034 #4 | ✅ Verified |
| yamllint pinned | `yaml-lint.yml` (existing) | ADR-029 #11 | ✅ Already done |

**Files changed**: 3 (dependabot.yml, benchmarks.yml, ci.yml)
**Validation**: All YAML valid, `cargo check --all` passes

---

*Generated: 2026-03-06 by GOAP Analysis*
*Updated: 2026-03-06 after quick wins execution*
*Next Action: Start G1 (error handling), G2 (test triage), G4 (ADR-024 integration tests), G7 (proptest expansion)*
