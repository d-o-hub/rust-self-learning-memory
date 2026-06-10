# Project Status — Self-Learning Memory System

**Last Updated**: 2026-06-10 (PR #611 open with CI failures; CI fix session completed 2026-06-09)
**Released Version**: v0.1.31 (crates.io + GitHub Release)
**Current Workspace Version**: 0.1.32 (sprint in flight — v0.1.32 not yet released)
**Active Sprint**: v0.1.32 — see [GOAP_STATE.md](../GOAP_STATE.md), [GOAP plan](../GOAP_MISSING_IMPLEMENTATION_2026-05-21.md)
**Branch**: `feat/turso-local-mode-12832947082971821257` (PR #611 open, CI failing)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
**Edition**: Rust 2024

## v0.1.32 Sprint Snapshot (verified 2026-05-22)

| Phase | Done | Open |
|-------|------|------|
| P1 User contract | 5/6 | WG-154 (Mistral binary dequant) |
| P2 Telemetry | 1/5 | WG-156/157/158/160 (hard-coded placeholders) |
| P3 Internal debt | 2/4 | WG-161 cascade `analyze_query`, WG-162 `generate_simple_embedding` |
| P4 Validation/release | 0/6 | Blocked on remaining 6 functional WGs |

Re-audit command:
`rg -in 'not yet implemented|// *Placeholder' memory-core/src memory-mcp/src memory-cli/src memory-storage-redb/src memory-storage-turso/src`

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.32 | — | ✅ Prepared for release (2026-05-21) |
| Latest GitHub release | v0.1.31 | — | ✅ Published 2026-04-22 |
| Publishable workspace crates | 6 | — | ✅ All at `0.1.31` |
| Total tests | 3,282 | — | 3,282 passing (2026-05-16) |
| Skipped/ignored tests | 164 | ≤165 ceiling | ✅ 70 blocked by upstream libsql bug (ADR-027) |
| Timed-out tests | 0 | 0 | ✅ |
| Failing doctests | 0 | 0 | ✅ |
| Production src files >500 LOC | 0 | 0 | ✅ Met |
| `#[allow(dead_code)]` (prod src) | 0 | ≤25 | ✅ Met (all 38 eliminated; removed unused params, cfg-gated test-only utils, verified 2026-05-17) |
| CSM integration | Complete | BM25+HDC+ConceptGraph cascade | ✅ WG-128/129/130/131 via crate dependency |
| Stale analysis docs | 0 | 0 | ✅ Both refreshed 2026-04-22 |
| Skills count | 31 | ≤35 | ✅ Target met (consolidated in PR #460) |
| Skills LOC | ~3,500 | ≤4,000 | ✅ Compact high-frequency skills |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 17 | ≥13 | ✅ Exceeds target |
| Property test occurrences | 154 | — | New metric (2026-04-22) |
| Broken markdown links | 0 active | ≤80 | ✅ |
| Clippy | Clean | Clean | ✅ |
| Format | Clean | Clean | ✅ |

## v0.1.30 Sprint Highlights

- **MemoryEvent Broadcast**: `tokio::broadcast` channel for episode lifecycle events
- **Top-k Optimization**: O(n) `select_nth_unstable_by` for retrieval hot paths
- **Zero-copy Retrieval Caching**: Bolt optimization for episodic memory
- **Agent Skills**: Added `memory-context` and `learn` skills

## v0.1.31 Planning Focus

- **CSM integration**: Cascading retrieval pipeline (BM25 → HDC → ConceptGraph → API) to eliminate 50-70% of embedding API calls
- **CPU efficiency**: QueryCache contention, cached retrieval wiring, compression/cache thresholds
- **Token efficiency**: bounded context windows, hierarchical/gist reranking, compact high-frequency skills/docs
- **Housekeeping**: Create missing `performance` skill, prune skills 40→≤35, fix metric contradictions, refresh stale analysis docs
- **Release/package hygiene**: keep GitHub release, package versions, and planning docs aligned before the `0.1.31` bump

## v0.1.28 Release Highlights

- **DyMoE Routing-Drift Protection**: Affinity gating and dual reward scoring
- **CodeQL Fixed**: Cleartext logging alert resolved (WG-093)
- **Dependabot Analyzed**: All 3 alerts are transitive dependencies (accepted risk)

## v0.1.26 Release Highlights

- **Crate Renaming**: All crates renamed from `memory-*` to `do-memory-*` namespace
- **crates.io Publishing**: All 4 crates published successfully
- **Binary Names**: `do-memory-mcp-server`, `do-memory-cli`
- **GitHub Release**: v0.1.26 with multi-platform binaries

---

## Open Items (2026-05-16 Validation)

### Open Issues

| # | Title | Labels | Status |
|---|-------|--------|--------|
| [#610](https://github.com/d-o-hub/rust-self-learning-memory/issues/610) | feat(turso): expose local/offline mode as a first-class config path | jules | 🔴 Open — blocked on PR #611 CI |

### Open PRs

| # | Title | Branch | CI Status |
|---|-------|--------|-----------|
| [#611](https://github.com/d-o-hub/rust-self-learning-memory/pull/611) | Expose local/offline mode as a first-class config path | `feat/turso-local-mode` | 🔴 Failing (Tests, Multi-Platform ubuntu/macos, Coverage) |

**PR #611 CI failures** (last run 2026-06-08):
- `Tests` — FAILURE: SIGSEGV in turso relationship tests under `keepalive-pool` feature
- `Multi-Platform Test (ubuntu-latest)` — FAILURE: same root cause
- `Multi-Platform Test (macos-latest)` — FAILURE: same root cause
- `Code Coverage Analysis` — FAILURE: CLI `test_cli_help_output` snapshot mismatch

**CI fix plan**: See [`plans/GOAP_PR611_CI_FIX_2026-06-09.md`](../GOAP_PR611_CI_FIX_2026-06-09.md) for
root cause analysis and fix (local_config disables pooling+keepalive for SQLite; snapshot regen). Fix was
authored 2026-06-09 and needs to be pushed to the PR branch to trigger re-run.

### Recently Merged PRs

| # | Title | Status |
|---|-------|--------|
| 547 | chore(ci): resolve merge conflicts with main; use create-pull-request | ✅ Merged 2026-05-16 |
| 546 | fix(mcp): enforce input bounds clamping on all public tool parameters (CWE-770) | ✅ Merged 2026-05-16 |
| 548 | chore: YAML frontmatter validation + code quality | ✅ Merged 2026-05-16 |
| 545 | chore: YAML frontmatter validation and Dependabot fix plan | ✅ Merged 2026-05-16 |
| 544 | feat: CloudEvents EventEmitter, ConceptGraph, evaluations | ✅ Merged 2026-05-15 |
| 542 | code health: update coverage badge generation workflow | ✅ Merged 2026-05-14 |
| 454 | fix(persistence): SQL injection in metadata query | ✅ Merged 2026-04-18 (P0 security fix) |

### Security: Dependabot Alerts (Accepted Risk — Transitive)
| # | Dependency | Severity | Notes |
|---|-----------|----------|-------|
| 12 | rustls-webpki | Medium | CRL matching logic bug; transitive via libsql |
| 2 | lru | Low | IterMut Stacked Borrows violation; transitive |
| 1 | libsql-sqlite3-parser | Low | Crash on invalid UTF-8; transitive via libsql |

### Security: Code Scanning
| Status | Notes |
|--------|-------|
| ✅ No open alerts | CodeQL cleartext logging alert #60 resolved (WG-093) |

### Known Issues (P1)
| Issue | Status | Workaround |
|-------|--------|------------|
| CLI Turso segfault | Under investigation | Use redb-only or `turso dev` server |

## Completed Phases

All research/implementation phases are complete:

- ✅ **Phase 1 (PREMem)**: Quality assessment (89% accuracy)
- ✅ **Phase 2 (GENESIS)**: Capacity management (88–2307× above targets)
- ✅ **Phase 3 (Spatiotemporal)**: Retrieval accuracy (+150%, 4.4× target)
- ✅ **Phase 4 (Benchmarking)**: All research claims validated

## v0.1.22 Features (ADR-044 — Polished)

| Feature | Core | MCP | CLI | Tests | Doctests | Snapshots |
|---------|------|-----|-----|-------|---------|-----------|
| Actionable Playbooks | ✅ | ✅ | ✅ | 26 | ✅ Fixed | ✅ |
| Recommendation Attribution | ✅ | ✅ | ✅ | 8 | ✅ Fixed | ✅ |
| Episode Checkpoints/Handoff | ✅ | ✅ | ✅ | 6 | ✅ | ✅ |
| Recommendation Feedback | ✅ | ✅ | ✅ | 3 | ✅ | ✅ |

## Key Capabilities

- **Multi-provider embeddings**: 5 providers (OpenAI, Cohere, Ollama, Local, Custom)
- **MCP server**: Full tool registry with lazy loading (ADR-024)
- **Episode management**: Full lifecycle with relationships, tagging, patterns
- **Playbooks**: Template-driven actionable recommendations from patterns
- **Attribution**: Recommendation session tracking and feedback loops
- **Durable attribution storage**: Turso/redb persistence for sessions, feedback, and metrics (WG-051 validated via `tests/attribution_integration_test.rs`)
- **Durable checkpoint/handoff storage**: Turso episode checkpoint serialization + restart-safe handoff resume metadata persistence (WG-052 validated via `tests/checkpoint_integration_test.rs`)
- **Checkpoints**: Mid-task state snapshotting and agent handoff packs
- **Storage**: Turso/libSQL (persistent) + redb (cache) dual-layer
- **Security**: Path traversal protection, parameterized SQL (WASM removed in v0.1.29)
- **CI/CD**: 6 workflows all passing, cargo-nextest, mutation testing
- **Performance**: Exceeds all targets (17–2307×)

## CSM Cascading Retrieval (Completed ✅)

**Integration Method**: Crate dependency (`chaotic_semantic_memory = "0.3.2"`), not source code copy.
**Implementation**: 732 LOC, 20+ tests for full 4-tier cascade (`CascadeRetriever` behind `csm` feature flag).

| Tier | Method | Source | API Calls | Status |
|------|--------|--------|-----------|--------|
| 1 | BM25 keyword index | `chaotic_semantic_memory` crate | 0 | ✅ WG-128 Complete |
| 2 | HDC 10,240-bit encoding | `chaotic_semantic_memory` crate | 0 | ✅ WG-129 Complete |
| 3 | ConceptGraph expansion | `chaotic_semantic_memory` crate | 0 | ✅ WG-130 Complete |
| 4 | API embedding (fallback) | OpenAI/Cohere/Ollama | 1 | Existing |
| Pipeline | Cascade orchestrator | New `CascadeRetriever` | 0-1 | ✅ WG-131 Complete (732 LOC, 20+ tests) |

## Critical Issues for v0.1.22 Tag — ALL RESOLVED

| Issue | Priority | Status |
|-------|----------|--------|
| ~~2 failing doctests (attribution, playbook)~~ | P0 | ✅ Fixed |
| ~~1 test timeout (quality_gate_no_clippy_warnings)~~ | P0 | ✅ Fixed |
| ~~3 files >500 LOC~~ | P0 | ✅ Fixed |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 164 | ≤165 ceiling | 70 Turso (upstream libsql bug), rest by design |
| `#[allow(dead_code)]` (prod src) | 0 | ≤25 | ✅ Met (all 38 eliminated; removed unused params, cfg-gated test-only utils, verified 2026-05-17) |
| Skills count | 31 | ≤35 | ✅ Target met (5 skills merged/removed) |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 17 | ≥13 | ✅ Exceeds target |

## Removed Features

| Feature | Version Removed | Reason |
|---------|-----------------|--------|
| WASM sandbox (wasmtime) | v0.1.29 (WG-096) | Maintenance burden, security concerns, 1,899 LOC removed |

## Infrastructure (Completed via PR #391)

| Item | Since | Status |
|------|-------|--------|
| Changelog automation (git-cliff) | v0.1.17 | ✅ `.github/workflows/changelog.yml` |
| libsql version monitor (T5.3) | v0.1.20 | ✅ `scripts/check-libsql-version.sh` |
| Structured tech-debt registry | v0.1.17 | ✅ `docs/TECH_DEBT.md` |

## Infrastructure Backlog

| Item | Since | Priority |
|------|-------|----------|
| Nightly trend tracking (T5.2) | v0.1.20 | P3 |
| CLI workflow parity generator | v0.1.17 | P3 |

## Cross-References

- **Gap analysis**: [GAP_ANALYSIS_LATEST.md](GAP_ANALYSIS_LATEST.md)
- **Execution plan**: [GOAP Execution Plans](../GOAP_STATE.md)
- **Active roadmap**: [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
- **ADRs**: [ADR Directory](../adr/)
- **Comprehensive analysis**: [COMPREHENSIVE_ANALYSIS_2026-04-21.md](COMPREHENSIVE_ANALYSIS_2026-04-21.md)
- **CSM repo**: <https://github.com/d-o-hub/chaotic_semantic_memory>
