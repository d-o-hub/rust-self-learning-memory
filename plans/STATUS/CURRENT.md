# Project Status — Self-Learning Memory System

**Last Updated**: 2026-03-20
**Released Version**: v0.1.21
**Next Version**: v0.1.22 (Quality & Feature Polish)
**Branch**: `feature/v0.1.22-completion-14277757658140810815`
**PR**: [#391](https://github.com/d-o-hub/rust-self-learning-memory/pull/391) (supersedes #369)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373)
**Edition**: Rust 2024

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Rust LOC | ~208K | — | — |
| Total test functions | 2,971 | — | — |
| Tests passing | 2,847 | — | ✅ |
| Ignored tests | 124 | ≤125 ceiling | ✅ 70 blocked by upstream libsql bug (ADR-027) |
| Timed-out tests | 0 | 0 | ✅ |
| Failing doctests | 0 | 0 | ✅ Fixed (attribution clone, playbook sync) |
| Production src files >500 LOC | 0 | 0 | ✅ Split: generator→builder, management→tags, handlers→feature_handlers |
| `#[allow(dead_code)]` (production) | 46 | ≤40 | 🟡 Close (down from 70) |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 13 | ≥13 | ✅ Target met |
| Broken markdown links | ~130 | ≤80 | 🟡 ~20 fixed by PR #391 |
| Duplicate dependency roots | 134 | <80 | 🟡 Architectural limit reached |
| Clippy | Clean | Clean | ✅ |
| Format | Clean | Clean | ✅ |

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
- **Checkpoints**: Mid-task state snapshotting and agent handoff packs
- **Storage**: Turso/libSQL (persistent) + redb (cache) dual-layer
- **Security**: Wasmtime sandbox, path traversal protection, parameterized SQL
- **CI/CD**: 6 workflows all passing, cargo-nextest, mutation testing
- **Performance**: Exceeds all targets (17–2307×)

## Critical Issues for v0.1.22 Tag

| Issue | Priority | Status |
|-------|----------|--------|
| ~~2 failing doctests (attribution, playbook)~~ | P0 | ✅ Fixed |
| ~~1 test timeout (quality_gate_no_clippy_warnings)~~ | P0 | ✅ Fixed |
| ~~3 files >500 LOC~~ | P0 | ✅ Fixed |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 124 | ≤125 ceiling | 70 Turso (upstream libsql bug), rest by design |
| `#[allow(dead_code)]` (production) | 46 | ≤40 | Down from 70; 6 remaining in embeddings |
| Broken markdown links | ~130 | ≤80 | ~20 fixed; remaining mostly in archived docs |
| Duplicate dep roots | 134 | <80 | Architectural limit |
| Snapshot tests | 80 | ≥80 | ✅ Target met via PR #391 |
| Property test files | 13 | ≥13 | ✅ Target met (playbook, checkpoint, attribution) |

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
