# Active Development Roadmap

**Last Updated**: 2026-03-15
**Released Version**: v0.1.20
**Current Sprint**: v0.1.21 (high-impact features)
**Branch**: main

---

## Current State

All research phases (1–4) and infrastructure work complete. CI/CD stable. v0.1.20 shipped with coverage monitoring script, documentation improvements, and dead code analysis.

v0.1.21 sprint focuses on high-impact features (ADR-044):
- Actionable Recommendation Playbooks (P0)
- Recommendation Attribution & Online Effectiveness (P0)
- Episode Checkpoints & Handoff Packs (P1)

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Next Sprint: v0.1.21

### High-Impact Features (ADR-044)

#### P0: Actionable Recommendation Playbooks
- **Problem**: Memory returns raw episodes/patterns — agents must infer what to do
- **Impact**: Closes biggest product gap for agent adoption
- **Effort**: 3-5 days

#### P0: Recommendation Attribution & Online Effectiveness
- **Problem**: Recommendations not tracked — can't learn which helped
- **Impact**: Closes feedback loop from recommendation → usage → outcome
- **Effort**: 3-4 days

#### P1: Episode Checkpoints & Handoff Packs
- **Problem**: Learning only at episode completion — multi-agent workflows blocked
- **Impact**: Unlocks multi-agent adoption
- **Effort**: 4-6 days

### Completed in v0.1.20

- ✅ Coverage monitoring script (scripts/check-coverage.sh)
- ✅ Dead code analysis and documentation
- ✅ Documentation improvements (similarity.rs)
- ✅ Version bumped to 0.1.20

### Completed in v0.1.19

- ✅ Adaptive TTL → Turso Storage wired (feature-gated)
- ✅ Transport Compression wired (feature-gated)
- ✅ Adaptive Cache → redb Storage via Cache trait adapter
- ✅ CLI Domain/Type Filters implemented
- ✅ CLI Pattern Discovery Commands implemented
- ✅ MCP Completion/Elicitation/Rate Limiting implemented

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
| v0.1.20 | 2026-03-15 | Coverage monitoring, dead code analysis, doc improvements |
| v0.1.19 | 2026-03 | AdaptiveCache wiring, CLI filters, pattern discovery, MCP enhancements |
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
