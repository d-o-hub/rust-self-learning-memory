# Project Status — Self-Learning Memory System

**Last Updated**: 2026-03-15
**Released Version**: v0.1.21
**Next Version**: v0.1.22 (ADR-044 High-Impact Features)  
**Branch**: main  
**Edition**: Rust 2024

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Rust files | 870 | — | — |
| Rust LOC | ~208K | — | — |
| Total test functions | 2,898 | — | — |
| Ignored tests | 118 | ≤10 | 🔴 70 blocked by upstream libsql bug (ADR-027) |
| Production src files >500 LOC | 0 | 0 | ✅ |
| Duplicate dependency roots | 134 | <80 | 🟡 Architectural limit reached |
| Broken markdown links | 89 | 0 | 🟡 Mostly in archived files |
| `target/` directory | 19 GB | <2 GB | 🟡 Reduced from 74 GB |

## Completed Phases

All research/implementation phases are complete:

- ✅ **Phase 1 (PREMem)**: Quality assessment (89% accuracy)
- ✅ **Phase 2 (GENESIS)**: Capacity management (88–2307× above targets)
- ✅ **Phase 3 (Spatiotemporal)**: Retrieval accuracy (+150%, 4.4× target)
- ✅ **Phase 4 (Benchmarking)**: All research claims validated

## Completed Goals (v0.1.17–v0.1.18)

| Goal | Description | Status |
|------|-------------|--------|
| WG-001 | Docs integrity automation | ✅ |
| WG-002 | Release operations wrapper | ✅ |
| WG-003 | GOAP state index | ✅ |
| WG-004 | Architecture context contract | ✅ |
| WG-005 | PR #334 stabilization | ✅ |
| WG-006 | Dependabot PR merges | ✅ |
| WG-007 | rust-major breaking changes (redb 3.x, rand 0.10) | ✅ |
| WG-008 | Ignored test triage | ✅ Documented (ADR-027) |
| WG-009 | Batch MCP tool cleanup | ✅ PR #357 |
| WG-010 | Error handling analysis | ✅ Already follows best practices |
| WG-011 | Dependency deduplication | ✅ Architectural limit reached |

## Key Capabilities

- **Multi-provider embeddings**: 5 providers (OpenAI, Cohere, Ollama, Local, Custom)
- **MCP server**: Full tool registry with lazy loading (ADR-024)
- **Episode management**: Full lifecycle with relationships, tagging, patterns
- **Storage**: Turso/libSQL (persistent) + redb (cache) dual-layer
- **Security**: Wasmtime sandbox, path traversal protection, parameterized SQL
- **CI/CD**: 6 workflows all passing, cargo-nextest, mutation testing
- **Performance**: Exceeds all targets (17–2307×)

## Missing Implementation (Built but Not Wired)

These subsystems are fully implemented with tests but not connected to production paths:

| Gap | Location | Status |
|-----|----------|--------|
| **Transport compression not wired to Turso** | `memory-storage-turso/src/transport/wrapper.rs` | ✅ Documented: libsql handles transport internally; compression applied at data layer |

## Missing Implementation (Now Implemented)

| Feature | Evidence | Status |
|---------|----------|--------|
| **CLI `episode create --domain` / `episode search --domain` / `episode search --type`** | `tests/e2e/cli_workflows.rs` | ✅ Implemented (2026-03-12) |
| **AdaptiveCache wired with Cache trait adapter** | `memory-storage-redb/src/cache/adapter.rs` | ✅ Implemented (2026-03-12) |
| **Adaptive TTL wired to Turso storage** | `memory-storage-turso/src/lib_impls/helpers.rs` | ✅ Implemented (2026-03-12) |
| **CLI pattern discovery commands** | `memory-cli/src/commands/pattern/` | ✅ Implemented (2026-03-12) |

## Missing Implementation (Not Built)

| Feature | Location | Status |
|---------|----------|--------|
| **Changelog automation (git-cliff)** | ADR-034 Phase 4 | Not started |

> **Note**: MCP Completion, Elicitation, and Rate Limiting were previously listed here but are **fully implemented** (confirmed in ADR-040, 2026-03-13). Stale TODOs in `types.rs` were removed.

## Disabled Features

| Feature | Location | Reason |
|---------|----------|--------|
| **`execute_agent_code` MCP tool** | `handlers.rs:72-91` | Registered conditionally but returns error due to "WASM sandbox compilation issues" |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 118 | — | 70 Turso (upstream libsql bug), 29 slow integration, 9 WASM/sandbox, 10 other |
| `#[allow(dead_code)]` | 37 files | ≤20 files | — |
| Ignored-test ceiling | 125 | — | ✅ Enforced by `scripts/check-ignored-tests.sh` (ADR-041) |
| Broken markdown links | 89 | 0 | Mostly archived files |
| Duplicate dep roots | 134 | <80 | Architectural limit (wasmtime/libsql transitive deps) |

## WG-008 Follow-Up Progress

- ◼ Monitor CI and verify completion
- ✔ Fix timing-dependent tests (WG-008 Part 1)
- ✔ Fix flaky tests (WG-008 Part 2)
- ✔ Dead code cleanup (Phase B3)
- ✔ Implement CLI warm-start functionality

## Cross-References

- **Latest validation**: [VALIDATION_LATEST.md](VALIDATION_LATEST.md)
- **Latest codebase analysis**: [CODEBASE_ANALYSIS_LATEST.md](CODEBASE_ANALYSIS_LATEST.md)
- **Active roadmap**: [ROADMAPS/ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
- **ADRs**: [adr/](../adr/)
