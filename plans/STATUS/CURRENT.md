# Project Status — Self-Learning Memory System

**Last Updated**: 2026-04-01 (v0.1.26 released)
**Released Version**: v0.1.26
**Branch**: `main` (clean)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
**Edition**: Rust 2024

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.26 | — | ✅ Published to crates.io |
| Total test functions | 2,849/2,849 | — | ✅ All passing |
| Skipped/ignored tests | 115 | ≤125 ceiling | ✅ 70 blocked by upstream libsql bug (ADR-027) |
| Timed-out tests | 0 | 0 | ✅ |
| Failing doctests | 0 | 0 | ✅ |
| Production src files >500 LOC | 0 | 0 | ✅ |
| `#[allow(dead_code)]` (production) | 31 | ≤40 | ✅ Target met |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 16 | ≥13 | ✅ Exceeds target |
| Broken markdown links | 0 active | ≤80 | ✅ |
| Clippy | Clean | Clean | ✅ |
| Format | Clean | Clean | ✅ |

## v0.1.26 Release Highlights

- **Crate Renaming**: All crates renamed from `memory-*` to `do-memory-*` namespace
- **crates.io Publishing**: All 4 crates published successfully
- **Binary Names**: `do-memory-mcp-server`, `do-memory-cli`
- **GitHub Release**: v0.1.26 with multi-platform binaries

### Open Items (2026-04-02 Analysis)

#### Open Issues
| # | Title | Impact |
|---|-------|--------|
| #419 | DyMoE-inspired routing-drift protection, affinity gating & dual reward scoring | P1 Feature — significant pattern extraction improvement |
| #401 | Dispatch discoverability request | P3 Trivial — external spam, close or ignore |

#### Open PR
| # | Title | Status |
|---|-------|--------|
| #406 | AI spam detector workflows | Mergeable — closes #401 |

#### Security: Dependabot Alerts (Open)
| # | Dependency | Severity | Notes |
|---|-----------|----------|-------|
| 12 | rustls-webpki | Medium | CRL matching logic bug; fix available in 0.103.10 (transitive via libsql) |
| 2 | lru | Low | IterMut Stacked Borrows violation; transitive |
| 1 | libsql-sqlite3-parser | Low | Crash on invalid UTF-8; upstream libsql |

#### Security: Code Scanning (Open)
| # | Path | Rule | Tool |
|---|------|------|------|
| 60 | memory-cli/src/commands/feedback/core.rs | Cleartext logging of sensitive information | CodeQL |

#### Cargo Audit Warnings (Unmaintained — Transitive)
| Crate | Advisory | Source Chain |
|-------|----------|-------------|
| bincode 1.3.3 | RUSTSEC-2025-0141 | libsql → bincode; argmin → rv → bincode |
| instant 0.1.13 | RUSTSEC-2024-0384 | argmin → rv → changepoint → augurs-changepoint |
| paste 1.0.15 | RUSTSEC-2024-0436 | tokenizers, simba, rv, argmin |

### Post-v0.1.22 Audit Findings (2026-03-24)

- **ADR-044 durability** — Recommendation attribution (WG-051) and checkpoint/handoff metadata durability (WG-052) now persist through Turso + redb-backed round-trips, including restart-safe resume metadata.
- **MCP/CLI contract drift** — Batch tool contract truth remains aligned (WG-053), and core docs/CLI references were refreshed to runtime/parity truth source in WG-054.
- **CI/test coverage remediation** — Required CI test scope now runs workspace nextest slices instead of `--lib`-only gates, benchmark workflow surface expanded, and coverage enforcement now fails below configured threshold (default 90). (WG-055/WG-056 complete)
- **Disk hygiene remediation** — `scripts/clean-artifacts.sh` now supports practical cleanup modes, optional `--node-modules`, coverage artifact cleanup, and `CARGO_TARGET_DIR`-aware paths. (WG-057 complete)
- **Guidance parity remediation** — AGENTS.md, `agent_docs/`, and relevant `.agents/skills/` now reflect script-first workflow, coverage `>=90%` expectations, and non-mold default linker guidance. (WG-058 complete)

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
- **Security**: Wasmtime sandbox, path traversal protection, parameterized SQL
- **CI/CD**: 6 workflows all passing, cargo-nextest, mutation testing
- **Performance**: Exceeds all targets (17–2307×)

## Critical Issues for v0.1.22 Tag — ALL RESOLVED

| Issue | Priority | Status |
|-------|----------|--------|
| ~~2 failing doctests (attribution, playbook)~~ | P0 | ✅ Fixed |
| ~~1 test timeout (quality_gate_no_clippy_warnings)~~ | P0 | ✅ Fixed |
| ~~3 files >500 LOC~~ | P0 | ✅ Fixed |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 124 | ≤125 ceiling | 70 Turso (upstream libsql bug), rest by design |
| `#[allow(dead_code)]` (production) | 31 | ≤40 | ✅ Target met |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 16 | ≥13 | ✅ Exceeds target |

## Disabled Features

| Feature | Location | Reason |
|---------|----------|--------|
| `execute_agent_code` MCP tool | `handlers.rs:72-91` | WASM sandbox compilation issues |

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
