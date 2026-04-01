# ADR-043: Comprehensive Codebase Analysis & GOAP Sprint Plan (v0.1.20)

- **Status**: Proposed
- **Date**: 2026-03-15
- **Deciders**: Project maintainers
- **Related**: ADR-028 (Feature Enhancement Roadmap), ADR-040 (Gap Analysis v0.1.19), ADR-041 (Test Health), ADR-042 (Coverage)

## Context

Full codebase audit on 2026-03-15 against ADR-028 roadmap, ADR-042 coverage plan, and GOAP state. This ADR catalogs all findings across four dimensions: **new features**, **missing implementation**, **code coverage**, and **enhancements**.

### Current State Snapshot (2026-03-15)

| Metric | Value | Previous (v0.1.18) | Delta |
|--------|-------|---------------------|-------|
| Version | 0.1.19 | 0.1.18 | +1 |
| Rust LOC | 213,360 | ~208,000 | +5,360 |
| Source files (5 crates) | 672 | — | — |
| Total `#[test]` functions | 1,810 | 2,898 (reported) | Reconciled |
| Ignored tests | 112 | 118 | -6 ✅ |
| `#[allow(dead_code)]` (prod) | 69 | 79 | -10 ✅ |
| TODO/FIXME in prod | 0 | — | ✅ Clean |
| Clippy | ❌ 4 errors | ✅ | Regression |
| Build | ✅ Clean | ✅ | — |
| Branch | release/v0.1.19 | main | — |
| Code coverage target | 70% (project) | 90% | Corrected |

---

## GOAP Phase 1: ANALYZE

### A. Clippy Regression (P0 — Blocking CI)

4 clippy errors in test files introduced by ADR-042 coverage work:

| File | Line | Error | Fix |
|------|------|-------|-----|
| `do-memory-storage-redb/tests/persistence_coverage_tests.rs` | 29 | `unwrap()` on Result | Use `?` or `.expect("msg")` |
| `do-memory-storage-redb/tests/persistence_coverage_tests.rs` | 34 | `unwrap()` on Result | Use `?` or `.expect("msg")` |
| `do-memory-storage-redb/tests/persistence_coverage_tests.rs` | 44 | `unwrap()` on Result | Use `?` or `.expect("msg")` |
| `do-memory-mcp/tests/adr024_lazy_loading_tests.rs` | 43 | `expect()` on Result | `.clippy.toml` has `allow-expect-in-tests = true` but test is integration test — needs `#![allow(clippy::expect_used)]` at crate level |

**Root Cause**: `.clippy.toml` sets `allow-unwrap-in-tests = true` and `allow-expect-in-tests = true`, but these only apply to `#[cfg(test)]` modules, NOT to integration test files in `tests/` directories. Integration tests are separate crate roots and don't inherit clippy config for inline test modules.

### B. Missing Implementation Inventory (ADR-028 Roadmap)

| # | Feature | ADR-028 Status | Actual Status (2026-03-15) | Gap |
|---|---------|----------------|---------------------------|-----|
| 1 | MCP Token Optimization | ✅ Complete | ✅ Shipped v0.1.15 | None |
| 2 | Batch Module Rehabilitation | ✅ Complete | ✅ Re-enabled v0.1.15 | None |
| 3 | File Size Compliance | ✅ Complete | ✅ 0 prod files >500 LOC | None |
| 4 | Error Handling Improvement | ⬚ Not Started | 🟡 Prod uses `?` properly; 1,203 `unwrap()` + 139 `expect()` mostly in tests | **Partially addressed** — prod OK, tests noisy |
| 5 | Ignored Test Rehabilitation | 📋 Categorized | 🟡 112 ignored (70 libsql, 29 slow, 13 other) | **Blocked** upstream |
| **6** | **Adaptive TTL Phase 2** | ⬚ Not Started | ✅ **Implemented** | Feature-gated `adaptive-ttl` |
| **7** | **Embeddings Integration** | 🟡 Partial | ✅ **Implemented** | CLI + MCP both complete |
| **8** | **Transport Compression** | ⬚ Not Started | ✅ **Implemented** | Feature-gated `compression` |
| 9 | Distributed Sync (CRDT) | ⬚ Not Started | ❌ Not started | Long-term (Q3-Q4) |
| 10 | Observability Stack | ⬚ Not Started | 🟡 **Partial** | Prometheus metrics export exists; no OpenTelemetry |
| 11 | Multi-Tenancy & RBAC | ⬚ Not Started | ❌ Not started | Long-term |
| 12 | Real-Time Pattern Learning | ⬚ Not Started | ❌ Not started | Long-term |
| 13 | Custom Embedding Models | ⬚ Not Started | 🟡 **Partial** | ONNX download exists; no runtime loading |
| 14 | A/B Testing Framework | ⬚ Not Started | ❌ Not started | Long-term |

**Key Finding**: Features #6, #7, #8 are **fully implemented** since v0.1.19 but ADR-028 status table is stale. Feature #10 has partial implementation (Prometheus export).

### C. Code Coverage Gaps (ADR-042)

| Phase | Status | Completed Actions | Pending Actions |
|-------|--------|-------------------|-----------------|
| Phase 1: Critical Paths | ✅ Complete | ACT-026, ACT-027, ACT-028, ACT-029 | — |
| Phase 2: Property Tests | 🟡 Partial | ACT-030, ACT-031 | **ACT-032** (MCP JSON-RPC fuzz) |
| Phase 3: Integration | ⏳ Not Started | — | **ACT-033, ACT-034, ACT-035** |
| Config | ⏳ Pending | — | **ACT-036** (codecov.yml), **ACT-037** (coverage script) |

**Coverage Configuration**: `.codecov.yml` currently at 70% project / 80% patch — reasonable baseline but ADR-042 phased targets not yet applied.

### D. Dead Code & Quality Debt

| Category | Count | Target (WG-018) | Gap |
|----------|-------|-----------------|-----|
| `#[allow(dead_code)]` in prod src | 69 | ≤20 | -49 to remove |
| Hotspots | `do-memory-core/src/embeddings/` (12), `do-memory-core/src/memory/` (7), `do-memory-core/src/monitoring/` (3) | — | — |

### E. Pending Goals (Unfulfilled from GOALS.md)

| ID | Goal | Priority | Status | Notes |
|----|------|----------|--------|-------|
| WG-018 | Reduce dead_code attrs (79→≤20) | P1 | 🟡 Partial (69 remaining) | Need -49 more |
| WG-019 | Remove stale TODOs | P1 | ✅ Complete (0 found) | — |
| WG-020 | Fix stale #[ignore] on pattern CLI | P1 | ✅ Complete | — |
| WG-021 | Update ADR-039 "Not Built" table | P2 | ✅ Complete | — |
| WG-025 | Un-ignore fixable tests | P2 | 🟡 Partial (118→112) | 6 sandbox/WASM pending |

### F. Infrastructure & Enhancement Opportunities

| ID | Opportunity | Priority | Evidence | Impact |
|----|-------------|----------|----------|--------|
| E1 | **ADR-028 status table stale** | P1 | Features #6-#8 implemented but not reflected | Documentation accuracy |
| E2 | **ROADMAP_ACTIVE.md stale** | P1 | Lists v0.1.17 as released; v0.1.19 is current | Sprint planning accuracy |
| E3 | **Clippy regression in integration tests** | P0 | 4 errors in test files | CI blocked |
| E4 | **Changelog automation** | P2 | ADR-034 Phase 4, not started | Release efficiency |
| E5 | **OpenTelemetry integration** | P2 | ADR-028 #10, partial | Production observability |
| E6 | **ONNX runtime model loading** | P3 | ADR-028 #13, download exists, no runtime | Custom model support |
| E7 | **WASM sandbox fix** | P3 | ADR-040 G3.6, disabled since v0.1.14 | Agent code execution |
| E8 | **Coverage script (ACT-037)** | P2 | Not created | Coverage monitoring |
| E9 | **MCP JSON-RPC fuzz tests** | P2 | ACT-032 pending | Security/robustness |
| E10 | **libsql version monitor** | P3 | ADR-041 T5.3 pending | Un-ignore 70 tests when fixed |

---

## GOAP Phase 2: DECOMPOSE

### Sprint v0.1.20 — Prioritized Task Breakdown

#### Tier 1: Blockers (P0)

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| Fix clippy regression | T1.1 | Add `#![allow(clippy::unwrap_used)]` to integration test files or replace with `?`/`.expect()` | 15 min | None |
| Verify CI green | T1.2 | `cargo clippy --workspace --tests -- -D warnings` passes | 5 min | T1.1 |

#### Tier 2: Documentation Accuracy (P1)

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| Update ADR-028 status table | T2.1 | Mark features #6, #7, #8 as ✅ Complete; #10 as 🟡 Partial; update dates | 30 min | None |
| Update ROADMAP_ACTIVE.md | T2.2 | Update to v0.1.19 released, correct stale "next sprint" items | 30 min | None |
| Update STATUS/CURRENT.md | T2.3 | Update metrics to 2026-03-15 snapshot | 15 min | None |
| Update GOAP_STATE.md | T2.4 | Add v0.1.21 sprint section | 15 min | T1.1 |

#### Tier 3: Dead Code Reduction (P1) — WG-018

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| Remove dead code in `embeddings/openai/utils.rs` | T3.1 | 6 attrs — analyze if functions are genuinely unused, remove code or attrs | 1h | None |
| Remove dead code in `memory/types.rs` | T3.2 | 5 attrs — check for live usage paths | 30 min | None |
| Remove dead code in `memory/core/struct_priv.rs` | T3.3 | 5 attrs — check for live usage paths | 30 min | None |
| Remove dead code in `monitoring/storage/mod.rs` | T3.4 | 3 attrs — MonitoringStorage wrapper unused per ADR-040 | 15 min | None |
| Audit remaining 50 attrs | T3.5 | Triage each: remove code, remove attr, or document necessity | 2h | T3.1-T3.4 |
| Target: ≤20 `#[allow(dead_code)]` | T3.6 | Validate count | 5 min | T3.5 |

#### Tier 4: Code Coverage (P2) — ADR-042 Remaining

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| ACT-032: MCP JSON-RPC fuzz tests | T4.1 | Property tests for JSON-RPC parsing | 2h | T1.1 |
| ACT-033: CLI integration tests | T4.2 | E2E tests for episode/pattern/tag commands | 3h | T1.1 |
| ACT-034: MCP tool integration tests | T4.3 | Tests covering all MCP tools | 2.5h | T1.1 |
| ACT-035: Cache eviction tests | T4.4 | Cache pressure scenario tests | 1.5h | T1.1 |
| ACT-037: Coverage monitoring script | T4.5 | `scripts/check-coverage.sh` | 30 min | None |

#### Tier 5: Test Health (P2) — ADR-041 Remaining

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| Fix 4 sandbox timing tests (ACT-024) | T5.1 | Add `tokio::time::timeout` wrappers | 2h | T1.1 |
| Fix 2 WASM binary data tests | T5.2 | Use `from_utf8_lossy` or base64 | 1h | T1.1 |
| Target: 112→106 ignored tests | T5.3 | Validate count after T5.1+T5.2 | 5 min | T5.1, T5.2 |

#### Tier 6: New Features (P2-P3)

| Task | ID | Action | Effort | Dependencies |
|------|----|--------|--------|--------------|
| OpenTelemetry tracing integration | T6.1 | Add `tracing-opentelemetry` spans to MCP and storage ops | 8h | None |
| Changelog automation (git-cliff CI) | T6.2 | ADR-034 Phase 4 completion | 4h | None |
| WASM sandbox fix (Option A) | T6.3 | Fix probe to accept pre-compiled WASM only | 2h | None |
| ONNX runtime model loading | T6.4 | Use `ort` crate for custom model inference | 6h | None |

---

## GOAP Phase 3: STRATEGIZE

### Execution Strategy: 3-Phase Hybrid

```
Phase A (Parallel, 1h)           Phase B (Parallel, 4h)        Phase C (Sequential, 8h+)
┌─────────────────────┐          ┌────────────────────┐        ┌──────────────────────┐
│ T1.1-T1.2 Clippy    │          │ T3.1-T3.6 Dead code│        │ T6.1 OpenTelemetry   │
│ T2.1-T2.4 Docs      │──Gate──▶│ T4.1-T4.5 Coverage │──Gate─▶│ T6.2 Changelog auto  │
│                     │          │ T5.1-T5.3 Test fix │        │ T6.3 WASM fix        │
└─────────────────────┘          └────────────────────┘        └──────────────────────┘
```

**Gate A→B**: Clippy clean, docs updated  
**Gate B→C**: Dead code ≤20, coverage tests passing, ignored ≤106

### Agent Assignment

| Phase | Skill/Agent | Tasks |
|-------|-------------|-------|
| A | code-quality + docs | T1.1-T1.2, T2.1-T2.4 |
| B-parallel-1 | code-quality | T3.1-T3.6 (dead code audit) |
| B-parallel-2 | quality-unit-testing | T4.1-T4.5 (coverage tests) |
| B-parallel-3 | test-fix | T5.1-T5.3 (test health) |
| C-sequential | feature-implement | T6.1-T6.3 (new features) |

---

## GOAP Phase 4: World State Transitions

### World State (Before — 2026-03-15)

| Fact | Value |
|------|-------|
| `clippy_clean` | ❌ false (4 errors) |
| `docs_current` | ❌ false (ADR-028, ROADMAP stale) |
| `dead_code_attrs` | 69 |
| `ignored_tests` | 112 |
| `coverage_tests_pending` | 5 actions |
| `opentelemetry_integrated` | ❌ false |
| `changelog_automated` | ❌ false |
| `wasm_sandbox_working` | ❌ false |

### World State (After — Target v0.1.20)

| Fact | Value |
|------|-------|
| `clippy_clean` | ✅ true |
| `docs_current` | ✅ true |
| `dead_code_attrs` | ≤20 |
| `ignored_tests` | ≤106 |
| `coverage_tests_pending` | 0 |
| `opentelemetry_integrated` | ✅ true |
| `changelog_automated` | ✅ true |
| `wasm_sandbox_working` | 🟡 WASM-only mode |

---

## Summary: What's New, Missing, and Enhanced

### 🆕 New Features Since Last Analysis

1. **Adaptive TTL** fully wired to Turso storage (feature-gated)
2. **Adaptive Cache** wired to redb via `Cache` trait adapter
3. **CLI domain/type filters** (`--domain`, `--type` flags)
4. **CLI embedding commands** (test, config, list-providers, benchmark)
5. **Transport compression** wired (feature-gated)
6. **Property tests** for serialization round-trips and reward bounds
7. **Ignored-test ceiling guard** script in CI

### ❌ Missing Implementation

1. **Distributed Sync (CRDT)** — ADR-028 #9, not started (Q3-Q4 2026)
2. **Multi-Tenancy & RBAC** — ADR-028 #11, not started (Q3-Q4 2026)
3. **Real-Time Pattern Learning** — ADR-028 #12, not started (Q3-Q4 2026)
4. **A/B Testing Framework** — ADR-028 #14, not started (Q3-Q4 2026)
5. **OpenTelemetry tracing** — Prometheus export exists, no distributed tracing
6. **ONNX runtime model loading** — Download exists, no inference runtime
7. **Changelog automation** — ADR-034 Phase 4, not started
8. **WASM sandbox** — Disabled since v0.1.14

### 📊 Code Coverage Status

- **Estimated**: 55-65% (improving)
- **Target**: 70% (Phase 1) → 75% (Phase 2) → 80% (Phase 3)
- **Completed**: 77+ new tests across do-memory-core, redb, turso (ACT-026—031)
- **Pending**: 5 actions (ACT-032—037) for fuzz, integration, and monitoring

### 🔧 Enhancements Needed

1. **P0**: Fix clippy regression (4 errors in integration tests)
2. **P1**: Update stale documentation (ADR-028, ROADMAP, STATUS)
3. **P1**: Reduce dead_code attrs from 69 to ≤20
4. **P2**: Complete ADR-042 coverage actions (ACT-032—037)
5. **P2**: Fix 6 remaining fixable ignored tests (sandbox timing + WASM)
6. **P2**: Add OpenTelemetry integration
7. **P2**: Automate changelog generation
8. **P3**: Fix WASM sandbox (Option A: WASM-only mode)
9. **P3**: Add libsql version monitoring for auto-un-ignore

---

## Consequences

### Positive
- Complete inventory of gaps across all 4 dimensions
- ADR-028 roadmap features #6-#8 confirmed shipped (previously stale)
- Clear 3-phase execution plan with quality gates
- Coverage improvement trajectory established (70% → 80%)

### Negative
- 4 long-term features (#9, #11, #12, #14) remain unstarted — accepted for Q3-Q4
- 70 Turso tests remain ignored (upstream libsql bug)
- Clippy regression needs immediate fix

### Neutral
- WASM sandbox remains disabled — low user impact
- Documentation staleness is cosmetic but affects planning accuracy

---

## Quality Gates

- **Phase A Gate**: `cargo clippy --workspace --tests -- -D warnings` passes
- **Phase B Gate**: `grep -r '#\[allow(dead_code)\]' --include='*.rs' <crates> | grep -v test | wc -l` ≤ 20
- **Phase B Gate**: `grep -r '#\[ignore' --include='*.rs' | grep -v target/ | wc -l` ≤ 106
- **Phase C Gate**: `./scripts/quality-gates.sh` passes
- **Release Gate**: All CI workflows green on `main`

---

## Cross-References

- **Feature Roadmap**: [ADR-028](ADR-028-Feature-Enhancement-Roadmap.md)
- **Previous Gap Analysis**: [ADR-040](ADR-040-Gap-Analysis-And-GOAP-Sprint-v0.1.19.md)
- **Test Health**: [ADR-041](ADR-041-Test-Health-Remediation-v0.1.20.md)
- **Coverage Plan**: [ADR-042](ADR-042-Code-Coverage-Improvement.md)
- **GOAP State**: [GOAP_STATE.md](../GOAP_STATE.md)
- **Goals Index**: [GOALS.md](../GOALS.md)
- **Active Roadmap**: [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
