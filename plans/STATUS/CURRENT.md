# Project Status — Self-Learning Memory System

**Last Updated**: 2026-06-28 (v0.1.33 release pending — 94 unreleased commits; issue #674)
**Released Version**: v0.1.32 (crates.io + GitHub Release, 2026-05-24)
**Workspace Version**: 0.1.33 (no tag — release drift)
**Active Sprint**: v0.1.33 — Release + CI Health + Quality
**Branch**: `main`
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
**Edition**: Rust 2024

## v0.1.33 Sprint — IN PROGRESS

| Phase | WGs | Strategy | Status |
|-------|-----|----------|--------|
| P1 — Release | WG-175 | Sequential | 🟡 Queued |
| P2 — CI Health | WG-176..WG-179 | Parallel | 🟡 Queued |
| P3 — Code Quality | WG-180..WG-181 | Parallel | 🟡 Queued |
| P4 — Architecture | WG-182 | Sequential | 🟡 Queued |
| P5 — DevX Backlog | WG-183..WG-184 | Parallel | 🟡 Queued |

**Blocking**: None (push CI is green; all phases are ready to execute)

**Plan**: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-28.md`

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.33 | — | 🔴 No tag/release (94 commits since v0.1.32) |
| Latest GitHub release | v0.1.32 | — | ✅ Published 2026-05-24 |
| Publishable workspace crates | 6 | — | ✅ All at `0.1.33` in workspace  |
| Commits since v0.1.32 | 94 | 0 | 🔴 Release drift (#674) |
| Clippy (default features) | Clean | Clean | ✅ |
| Clippy (--all-features) | 5 findings | 0 | 🔴 WG-180 |
| Production src files >500 LOC | 1 (537) | 0 | 🟡 WG-181 |
| Push CI (main) | Green | Green | ✅ |
| Scheduled Security | Failing | Green | 🔴 WG-176 |
| Nightly Full Tests | Failing | Green | 🔴 WG-177 |
| Mutation Testing | Cancelled (6h) | Completes | 🔴 WG-178 |
| Fuzzing | Green | Green | ✅ |
| Security audit | 2 allowed warnings | ≤2 allowed | ✅ (git2 transitive) |
| Open issues | 3 | 0 | 🟡 |
| Open PRs | 0 | 0 | ✅ |
| Fuzz harness | Present | Present | ✅ |
| Property test files | 17 | ≥13 | ✅ |
| MSRV | Rust 2024 / stable 1.95.0 | — | ✅ |

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

## Open Items (2026-06-28 Analysis)

### Open Issues

| # | Title | Labels | Status |
|---|-------|--------|--------|
| [#674](https://github.com/d-o-hub/rust-self-learning-memory/issues/674) | ⚠️ Release drift: 94 unreleased commits since v0.1.32 | release-drift | 🔴 Open — WG-175 |
| [#652](https://github.com/d-o-hub/rust-self-learning-memory/issues/652) | Automate llms.txt and full LLM context generation | — | 🔴 Open — WG-183 |
| [#653](https://github.com/d-o-hub/rust-self-learning-memory/issues/653) | Evaluate VERSION file adoption | — | 🔴 Open — WG-184 |

### Open PRs

None.

### CI Health

| Workflow | Status | Root Cause |
|----------|--------|------------|
| Push CI (main) | ✅ Green | — |
| Security (scheduled) | ❌ Failing | 3 gitleaks false positives (WG-176) |
| Nightly Full Tests | ❌ Failing | Disk exit-95 infra (WG-177) |
| Mutation Testing | ❌ Cancelled | 6h ceiling exceeded (WG-178) |
| Fuzzing | ✅ Green | — |

### Code Quality

| Finding | Location | WG |
|---------|----------|-----|
| 4× excessive_nesting + 1× unnecessary_wraps | `mistral/client.rs` | WG-180 |
| File >500 LOC (537) | `cache/wrapper.rs` | WG-181 |
| Non-CSM cascade returns empty silently | `cascade/mod.rs:207` | WG-182 |
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
