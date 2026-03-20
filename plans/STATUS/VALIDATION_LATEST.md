# Codebase Validation — 2026-03-20

**Validated by**: Issue verification + codebase audit
**Branch**: `main` (PR #391 merged, commit `15bc3ab3`)
**Workspace Version**: `0.1.22` (bumped)
**Previous Validation**: 2026-03-11 (commit `c049ed3`, v0.1.17)

---

## Live Metrics (2026-03-11)

| Metric | Previous (2026-03-09) | Current (2026-03-11) | Delta | Target |
|--------|----------------------|---------------------|-------|--------|
| Workspace version | `0.1.16` | **`0.1.17`** | 🆙 Released | — |
| Rust files | 867 | **870** | +3 | — |
| Rust LOC | 207,679 | **208,683** | +1,004 | — |
| Test functions (sync) | ~1,560 | **1,676** | +116 | — |
| Test functions (async) | ~1,178 | **1,222** | +44 | — |
| Total test functions | ~2,738 | **2,898** | +160 | — |
| `#[ignore]` tests | 121 | **121** | 0 | ≤10 ⚠️ |
| `unwrap()` in src (broad) | 1,096 | **1,092** | −4 | ≤50 🔴 |
| `.expect()` in src (broad) | 118 | **131** | +13 | ≤20 🔴 |
| `#[allow(dead_code)]` | ~137 | **110** | −27 ✅ | 0 |
| Duplicate dep roots | 134 | **134** | 0 | <80 🔴 |
| Snapshot files (`.snap`) | 65 | **65** | 0 | — |
| Property test files | 7 | **7** | 0 | — |
| Broken markdown links | 204→118 (O3 fix) | **118** | 0 | 0 🟡 |
| `target/` directory size | 74 GB | **19 GB** | −55 GB ✅ | — |
| Prod src files >500 LOC | 0 | **0** ✅ | 0 | 0 ✅ |
| Oversize files (all) | 45 | **20** (test/bench/e2e only) | −25 ✅ | — |

### Key Changes Since v0.1.16 (128 commits)

1. **v0.1.17 released** — includes O1/O3/O5 implementations, G2/G9 cleanup, security fix
2. **Dead code reduced** — `#[allow(dead_code)]` dropped from ~137 → 110 (−27)
3. **`target/` cleaned** — 74 GB → 19 GB (−55 GB)
4. **Test count grew** — +160 new test functions (2,738 → 2,898)
5. **Security fix** — quinn-proto updated to 0.11.14 (RUSTSEC-2026-0037)
6. **All Dependabot PRs resolved** — PRs #344, #345, #346 merged (redb 3.x + rand 0.10)

---

## GOAP Goal Status

| Goal | Status | Notes |
|------|--------|-------|
| WG-001: Docs integrity automation | ✅ Complete | `scripts/check-docs-integrity.sh` integrated |
| WG-002: Release operations wrapper | ✅ Complete | `scripts/release-manager.sh` available |
| WG-003: GOAP state index | ✅ Complete | `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md` |
| WG-004: Architecture context contract | ✅ Complete | `docs/architecture/context.yaml` |
| WG-005: PR #334 stabilization | ✅ Complete | Merged 2026-03-06 |
| WG-006: Dependabot PRs | ✅ Complete | All PRs merged |
| WG-007: rust-major breaking changes | ✅ Complete | PR #345 merged with redb 3.x + rand 0.10 fixes |

## Opportunity Tracker (from GOAP 2026-03-09)

| ID | Opportunity | Priority | Status |
|----|-------------|----------|--------|
| O1 | MCP tool contract parity checker | P0 | ✅ Complete (v0.1.17) |
| O3 | Docs integrity ownership report | P1 | ✅ Complete — 204→118 links fixed |
| O5 | Runtime feature wiring verification suite | P0 | ✅ Complete — 19 tests added |
| O7 | Structured technical-debt registry | P1 | ⬜ Not started |
| O6 | CLI workflow parity generator | P1 | ⬜ Not started |
| O4 | Architecture drift linter | P2 | ⬜ Not started |
| O2 | Single-source MCP schema generation | P2 | ⬜ Not started |

---

## ADR Progress vs Code (Revalidated)

| ADR | Status | Reality (2026-03-11) |
|-----|--------|---------------------|
| ADR-024 MCP Lazy Tool Loading | ✅ Complete | Verified in code and tests |
| ADR-028 Feature Enhancement Roadmap | 🟡 Mixed | Some shipped, batch partially disabled, error handling incomplete |
| ADR-029 GitHub Actions Modernization | ✅ Mostly complete | All workflows updated |
| ADR-032 Disk Space Optimization | ✅ Improved | target/ down from 74→19 GB |
| ADR-033 Modern Testing Strategy | 🟡 Advanced | nextest, proptest (7 files), insta (65 snaps) present |
| ADR-034 Release Engineering | ✅ Active | v0.1.17 released via cargo-release |
| ADR-035 Rust 2024 Edition | ✅ Complete | Edition 2024 workspace-wide |
| ADR-036 Dependency Deduplication | 🔴 Not improving | 134 duplicate roots (target <80) |
| ADR-037 Selective Workflow Automation | ✅ Complete | Scripts, GOAP indices established |
| ADR-038 Local CI Parity | ✅ Active | quality-gates.sh in use |

---

## Remaining Technical Debt (Prioritized)

### P0 — Must Fix for v0.1.18

| Debt Item | Current | Target | Gap |
|-----------|---------|--------|-----|
| Ignored tests | 121 | ≤10 | −111 to go |
| Batch MCP handlers disabled at runtime | 3 tools | 0 | Remove schemas or re-enable |

### P1 — Should Fix

| Debt Item | Current | Target | Gap |
|-----------|---------|--------|-----|
| `unwrap()` in prod src | 1,092 | ≤50 | −1,042 to go |
| `.expect()` in prod src | 131 | ≤20 | −111 to go |
| `#[allow(dead_code)]` | 110 | 0 | −110 to go |
| Duplicate dep roots | 134 | <80 | −54 to go |
| Broken markdown links | 118 | 0 | −118 to go |

### P2 — Nice to Have

| Debt Item | Current | Status |
|-----------|---------|--------|
| Adaptive cache not default redb path | Documented by O5 | Integration decision pending |
| Transport compression not wired to Turso | Documented by O5 | Integration decision pending |
| CLI workflow parity (ignored e2e tests) | Several tests ignored | Needs command surface alignment |

---

## File Size Compliance

✅ **Production source: 0 files >500 LOC** — fully compliant.

Only test/bench/e2e files exceed 500 LOC (all acceptable):

| File | LOC | Type |
|------|-----|------|
| `tests/e2e/cli_workflows.rs` | 916 | E2E test |
| `tests/e2e/mcp_integration.rs` | 915 | E2E test |
| `tests/e2e/embeddings_mcp_test.rs` | 891 | E2E test |
| `memory-core/src/episode/relationship_manager_tests.rs` | 860 | Internal test module |
| `benches/soak_tests.rs` | 814 | Benchmark |
| `memory-mcp/src/patterns/statistical/bocpd_tests.rs` | 660 | Internal test module |

---

## Recommended Next Sprint (v0.1.18)

| Priority | Work Item | Effort | Impact |
|----------|-----------|--------|--------|
| P0 | Triage 121 ignored tests (70 Turso) — remove, fix, or document-with-tracking | 4-8h | Quality signal |
| P0 | Resolve batch MCP tool state (remove schemas or re-enable) | 2-4h | Contract correctness |
| P1 | Error handling: reduce unwrap/expect in memory-core (140+25 calls) | 6-10h | Robustness |
| P1 | Dependency deduplication pass (134→<100) | 3-5h | Build speed/size |
| P1 | O7: Structured debt registry | 2-3h | Maintainability |
| P2 | O6: CLI workflow parity generator | 4-6h | Test coverage |

---

*Generated: 2026-03-11 by Amp codebase analysis*
*Previous: plans/GOAP_CODEBASE_ANALYSIS_2026-03-09.md*
