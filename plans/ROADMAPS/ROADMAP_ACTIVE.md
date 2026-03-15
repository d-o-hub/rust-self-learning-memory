# Active Development Roadmap

**Last Updated**: 2026-03-15
**Released Version**: v0.1.20
**Current Sprint**: v0.1.21 (publishing infrastructure)
**Branch**: main

---

## Current State

All research phases (1–4) and infrastructure work complete. CI/CD stable. v0.1.20 shipped with test coverage improvements and sprint fixes.

v0.1.21 sprint focuses on publishing infrastructure (ADR-045):
- crates.io publishing pipeline (P0)
- Supply chain security (P0)
- Cargo.toml metadata completion (P1)
- First dry-run publish (P1)

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Next Sprint: v0.1.21

### Publishing Infrastructure (ADR-045)

#### P0: crates.io Publishing Pipeline
- **Problem**: Project not published to crates.io — users can't use as dependency
- **Impact**: Enables ecosystem consumption, version management
- **Effort**: 3-4 days

#### P0: Supply Chain Security
- **Problem**: No provenance, SBOMs, or OIDC for publishing
- **Impact**: Supply chain attack prevention, trust establishment
- **Effort**: 2-3 days

#### P1: Cargo.toml Metadata Completion
- **Problem**: Incomplete metadata prevents discoverability
- **Impact**: Better crate visibility, docs.rs integration
- **Effort**: 1 day

### Completed in v0.1.20

- ✅ ACT-024: Sandbox timing tests with timeout wrappers
- ✅ ACT-029: Error handling tests (98 tests)
- ✅ ACT-031: Calculator property tests (27 tests)
- ✅ ACT-032: MCP JSON-RPC fuzz tests
- ✅ ACT-033: CLI integration tests
- ✅ ACT-034: MCP tool coverage tests
- ✅ ACT-035: Cache eviction tests
- ✅ ACT-037: Coverage monitoring script

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
| v0.1.20 | 2026-03 | Test coverage improvements, sprint fixes, coverage script |
| v0.1.19 | 2026-03 | MCP enhancements, gitleaks fixes |
| v0.1.18 | 2026-03 | AdaptiveCache, CLI filters, transport compression docs |
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
