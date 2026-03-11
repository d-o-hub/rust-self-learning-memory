# Active Development Roadmap

**Last Updated**: 2026-03-11  
**Released Version**: v0.1.17  
**Sprint Complete**: v0.1.18 (unreleased)  
**Branch**: main

---

## Current State

All research phases (1–4) and infrastructure work complete. CI/CD stable. v0.1.17 shipped with MCP tool contract parity, dead code removal, doc link fixes, and G2/G9 cleanup.

v0.1.18 sprint completed WG-008 through WG-011 (ignored test triage, batch tool cleanup, error handling analysis, dependency deduplication analysis).

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Next Sprint: v0.1.19

### Wire Built-but-Disconnected Subsystems

Three fully-implemented subsystems exist with tests but are not connected to production paths:

#### P1: Adaptive TTL → Turso Storage
- **Location**: `memory-storage-turso/src/cache/adaptive_ttl.rs` (435 LOC, tested)
- **Gap**: Not integrated into `TursoStorage` query paths
- **Config flag**: `TursoConfig.enable_transport_compression` exists but unused
- **Effort**: 6–8 hours
- **Impact**: ~20% cache hit rate improvement

#### P1: Transport Compression → Turso Client
- **Location**: `memory-storage-turso/src/transport/wrapper.rs` (tested)
- **Gap**: `CompressedTransport::new()` only called in tests/docs. Production constructors in `constructors_basic.rs` don't use it.
- **Effort**: 4–6 hours
- **Impact**: ~40% bandwidth reduction

#### P2: Adaptive Cache → redb Storage
- **Location**: `memory-storage-redb/src/cache/adaptive/` (327 LOC, tested)
- **Gap**: `RedbStorage::new_with_cache_config()` creates `LRUCache`, not `AdaptiveCache`. Different interfaces (LRU tracks metadata, Adaptive stores values). Needs a common `Cache` trait or adapter pattern.
- **Effort**: 10–14 hours (interface unification required)
- **Impact**: Dynamic TTL per access frequency

### New Features

#### P2: CLI Domain/Type Filters
- **Evidence**: `tests/e2e/cli_workflows.rs:678` — ignored test for `episode create --domain`, `episode search --domain`, `episode search --type`
- **Effort**: 6–8 hours
- **Impact**: Episode filtering UX

#### P2: CLI Pattern Discovery Commands
- **Evidence**: `tests/e2e/cli_workflows.rs:554` — ignored test referencing non-existent pattern CLI commands
- **Effort**: 4–6 hours
- **Impact**: Pattern exploration UX

#### P3: MCP Protocol Gaps
- MCP Completion support (`types.rs:81` TODO — returns empty placeholders)
- MCP Elicitation support (`types.rs:138` TODO — stub only)
- MCP Rate Limiting (`types.rs:332` TODO — `#[allow(dead_code)]`)
- **Effort**: 8–12 hours total
- **Impact**: MCP protocol completeness

#### P3: Fix `execute_agent_code` WASM tool
- **Location**: `handlers.rs:72-91` — registered conditionally, returns error
- **Issue**: "WASM sandbox compilation issues" — tool exists but always fails
- **Decision needed**: Fix WASM sandbox or remove the tool entirely
- **Effort**: 4–8 hours (fix) or 1–2 hours (remove)

---

## Backlog

### Code Quality

| Item | Current | Target | Effort |
|------|---------|--------|--------|
| `#[allow(dead_code)]` annotations | 110 | ≤50 | 6–8h |
| Broken markdown links | 89 | 0 | 4–6h |

### Testing

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 121 | — | 71 upstream libsql bug, 6 slow integration, 4 missing features |
| Property test expansion | 7 files | ≥15 | ADR-033; cover serialization invariants across crates |
| Snapshot test growth | 65 snaps | ≥80 | ADR-033; MCP responses and CLI output |

### Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Changelog automation (git-cliff) | Not started | ADR-034 Phase 4 |
| Structured tech-debt registry | Not started | Opportunity O7 from GOAP analysis |
| CLI workflow parity generator | Not started | Opportunity O6 from GOAP analysis |

---

## Release History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.17 | 2026-03 | MCP contract parity, dead code removal, doc fixes, G2/G9 |
| v0.1.16 | 2026-02-21 | Edition 2024, CI stabilization, quick wins |
| v0.1.15 | 2026-02-15 | MCP token optimization, GitHub Actions modernization |
| v0.1.14 | 2026-02-14 | Episode tagging, relationships, file compliance |
| v0.1.13 | 2026-01-12 | Semantic pattern search, recommendation engine |
| v0.1.12 | 2026-01-05 | Tasks utility, embedding config, contrastive learning |

---

## Cross-References

- **Current status**: [STATUS/CURRENT.md](../STATUS/CURRENT.md)
- **Latest validation**: [STATUS/VALIDATION_LATEST.md](../STATUS/VALIDATION_LATEST.md)
- **Latest analysis**: [STATUS/CODEBASE_ANALYSIS_LATEST.md](../STATUS/CODEBASE_ANALYSIS_LATEST.md)
- **Long-term vision**: [ROADMAP_V030_VISION.md](ROADMAP_V030_VISION.md)
- **ADRs**: [adr/](../adr/)
