# Active Development Roadmap

**Last Updated**: 2026-03-16
**Released Version**: v0.1.22
**Current Sprint**: v0.1.23 (Next Generation Learning)
**Branch**: main

---

## Current State

All research phases (1–4) and infrastructure work complete. CI/CD stable. ADR-044 High-Impact Features (Playbooks, Attribution, Checkpoints, Feedback) are implemented and released in v0.1.22.

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Current Sprint: v0.1.23

### Next Generation Learning

- Integration with more external LLM providers
- Enhanced spatiotemporal indexing for faster large-scale retrieval
- Dynamic reward shaping based on feedback sessions

---

## Completed in v0.1.22 (ADR-044 High-Impact Features)

- ✅ Actionable Playbooks (synthesizes patterns and reflections)
- ✅ Recommendation Attribution (tracks feedback and adoption)
- ✅ Episode Checkpoints & Handoff Packs (multi-agent progress sharing)
- ✅ Storage Consistency Check (CLI tool for DB/cache sync)
- ✅ Test Health Improvements (doctest fixes, timeout optimizations)
- ✅ File Size Compliance (refactored generator and management)

### P2: Feature Enhancements

| Task | Description | Status |
|------|-------------|--------|
| WG-047 | MCP tool contract parity for new tools | ✅ |
| WG-048 | Integration tests for attribution + checkpoint flows | ✅ |
| WG-049 | Changelog automation (git-cliff) | ⏳ |
| WG-050 | Documentation for new features | ✅ |

### P3: Infrastructure (Carried Forward)

| Task | Description | Since | Status |
|------|-------------|-------|--------|
| WG-051 | Nightly trend tracking artifact | v0.1.20 | ⏳ |
| WG-052 | libsql upstream version monitor | v0.1.20 | ⏳ |
| WG-053 | Structured tech-debt registry | v0.1.17 | ⏳ |

### Shipped in v0.1.21

- ✅ ADR-045: Publishing infrastructure (supply chain, OIDC, metadata)
- ✅ ADR-046: Claude Code configuration improvements (session analysis, tool enforcement)

### Shipped in v0.1.22 (Features)

- ✅ ADR-044: High-Impact Features (100% complete)
  - ✅ Feature 1: Actionable Playbooks (26 tests)
  - ✅ Feature 2: Recommendation Attribution (8 tests)
  - ✅ Feature 3: Episode Checkpoints/Handoff (6 tests)
  - ✅ Feature 4: Recommendation Feedback (3 tests)

---

## Backlog

### Code Quality

| Item | Current | Target | Effort |
|------|---------|--------|--------|
| `#[allow(dead_code)]` annotations | 70 | ≤40 | 4–6h |
| Broken markdown links | 149 | ≤80 | 4–6h |

### Testing

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 113 | — | 70 upstream libsql bug, rest by design |
| Property test expansion | 10 files | ≥15 | ADR-033; new features need property tests |
| Snapshot test growth | 65 snaps | ≥80 | New MCP tools + CLI commands need snapshots |

### Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Changelog automation (git-cliff) | Not started | ADR-034 Phase 4 |
| Structured tech-debt registry | Not started | Opportunity O7 |
| CLI workflow parity generator | Not started | Opportunity O6 |

---

## Release History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.22 | 2026-03 | ADR-044 High-Impact Features (Playbooks, Attribution, Checkpoints, Feedback) |
| v0.1.21 | 2026-03 | Publishing infrastructure (ADR-045), supply chain security |
| v0.1.20 | 2026-03 | Test coverage improvements, sprint fixes, coverage script |
| v0.1.19 | 2026-03 | MCP enhancements, gitleaks fixes |
| v0.1.18 | 2026-03 | AdaptiveCache, CLI filters, transport compression docs |
| v0.1.17 | 2026-03 | MCP contract parity, dead code removal, doc fixes, G2/G9 |
| v0.1.16 | 2026-02 | Edition 2024, CI stabilization, quick wins |
| v0.1.15 | 2026-02 | MCP token optimization, GitHub Actions modernization |
| v0.1.14 | 2026-02 | Episode tagging, relationships, file compliance |
| v0.1.13 | 2026-01 | Semantic pattern search, recommendation engine |
| v0.1.12 | 2026-01 | Tasks utility, embedding config, contrastive learning |

---

## Cross-References

- **Current status**: [STATUS/CURRENT.md](../STATUS/CURRENT.md)
- **Gap analysis**: [STATUS/GAP_ANALYSIS_LATEST.md](../STATUS/GAP_ANALYSIS_LATEST.md)
- **Execution plan**: [GOAP_EXECUTION_PLAN_v0.1.22.md](../GOAP_EXECUTION_PLAN_v0.1.22.md)
- **Long-term vision**: [ROADMAP_V030_VISION.md](ROADMAP_V030_VISION.md)
- **ADRs**: [adr/](../adr/)
