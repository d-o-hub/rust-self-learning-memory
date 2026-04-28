# Project Status — Self-Learning Memory System

**Last Updated**: 2026-04-28 (JWT security fix + CI optimization merged)
**Released Version**: v0.1.30 (crates.io + GitHub Release) + security patches
**Branch**: `main` (clean)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
**Edition**: Rust 2024

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.30 | — | ✅ |
| Latest GitHub release | v0.1.30 | — | ✅ Published 2026-04-16 |
| Publishable workspace crates | 6 | — | ✅ All at `0.1.30` |
| Total tests | 2,902 | — | 2,901 passing, 1 flaky (pre-existing) |
| Skipped/ignored tests | 123 | ≤125 ceiling | ✅ 70 blocked by upstream libsql bug (ADR-027) |
| Timed-out tests | 0 | 0 | ✅ |
| Failing doctests | 0 | 0 | ✅ |
| Production src files >500 LOC | 0 | 0 | ✅ Met |
| `#[allow(dead_code)]` (prod src) | 27 | ≤25 | ⚠️ Slightly over (API reserves/future features) |
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

## Open Items (2026-04-20 Validation)

### Open Issues
| # | Title | Status |
|---|-------|--------|
| — | No open issues | ✅ All closed |

### Open PRs (All Resolved)
| # | Title | Status |
|---|-------|--------|
| 453 | chore: bump version to 0.1.31 | ❌ CLOSED (not merged, deferred) |
| 450 | perf: parking_lot::RwLock for QueryCache | ✅ MERGED 2026-04-18 |
| 445 | ci(deps): bump actions/github-script 8→9 | ✅ MERGED 2026-04-18 |

### Recently Merged PRs
| # | Title | Status |
|---|-------|--------|
| 480 | 🔒 JWT signature verification | ✅ Merged 2026-04-28 (P0 security fix) |
| 491 | ⚡ CI optimization - paths-based benchmarks | ✅ Merged 2026-04-28 (perf) |
| 492 | docs(ci): CI optimization lessons | ✅ Merged 2026-04-28 (docs) |
| 493 | jsonwebtoken 9.3.1 → 10.3.0 | ✅ Merged 2026-04-28 (deps) |
| 494 | openssl 0.10.76 → 0.10.78 | ✅ Merged 2026-04-28 (deps) |
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

## Planned: CSM Cascading Retrieval (v0.1.31)

**Integration Method**: Crate dependency (`chaotic_semantic_memory = "0.3.2"`), not source code copy.

| Tier | Method | Source | API Calls | Status |
|------|--------|--------|-----------|--------|
| 1 | BM25 keyword index | `chaotic_semantic_memory` crate | 0 | ✅ WG-128 Complete |
| 2 | HDC 10,240-bit encoding | `chaotic_semantic_memory` crate | 0 | ✅ WG-129 Complete |
| 3 | ConceptGraph expansion | `chaotic_semantic_memory` crate | 0 | ✅ WG-130 Complete |
| 4 | API embedding (fallback) | OpenAI/Cohere/Ollama | 1 | Existing |
| Pipeline | Cascade orchestrator | New `CascadeRetriever` | 0-1 | 🔵 WG-131 Planned |

## Critical Issues for v0.1.22 Tag — ALL RESOLVED

| Issue | Priority | Status |
|-------|----------|--------|
| ~~2 failing doctests (attribution, playbook)~~ | P0 | ✅ Fixed |
| ~~1 test timeout (quality_gate_no_clippy_warnings)~~ | P0 | ✅ Fixed |
| ~~3 files >500 LOC~~ | P0 | ✅ Fixed |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 123 | ≤125 ceiling | 70 Turso (upstream libsql bug), rest by design |
| `#[allow(dead_code)]` (prod src) | 27 | ≤25 | ⚠️ Slightly over (API reserves/future features, verified 2026-04-22) |
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
- **Execution plan**: [GOAP_EXECUTION_PLAN_v0.1.22.md](../GOAP_EXECUTION_PLAN_v0.1.22.md)
- **Active roadmap**: [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
- **ADRs**: [ADR Directory](../adr/)
- **Comprehensive analysis**: [COMPREHENSIVE_ANALYSIS_2026-04-21.md](COMPREHENSIVE_ANALYSIS_2026-04-21.md)
- **CSM repo**: <https://github.com/d-o-hub/chaotic_semantic_memory>
