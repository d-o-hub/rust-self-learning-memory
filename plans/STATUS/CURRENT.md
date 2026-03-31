# Project Status — Self-Learning Memory System

**Last Updated**: 2026-03-31 (v0.1.24 stability sprint in progress)
**Released Version**: v0.1.23
**Branch**: `main` (PR #404 pending)
**PR**: [#404](https://github.com/d-o-hub/rust-self-learning-memory/pull/404) — v0.1.24 test stability
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED
**Edition**: Rust 2024

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.23 | — | — |
| Total test functions | 2,849/2,849 | — | ✅ All passing |
| Skipped/ignored tests | 115 | ≤125 ceiling | ✅ 70 blocked by upstream libsql bug (ADR-027) |
| Timed-out tests | 0 | 0 | ✅ |
| Failing doctests | 0 | 0 | ✅ Fixed (attribution clone, playbook sync) |
| Production src files >500 LOC | 0 | 0 | ✅ Split: generator→builder, management→tags, handlers→feature_handlers |
| `#[allow(dead_code)]` (production) | 31 | ≤40 | ✅ Target met |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 16 | ≥13 | ✅ Exceeds target |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) |
| Clippy | Clean | Clean | ✅ |
| Format | Clean | Clean | ✅ |

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
