# Active Development Roadmap

**Last Updated**: 2026-03-16
**Released Version**: v0.1.22
**Current Sprint**: v0.1.23 (ADR-041 Test Health completion)
**Branch**: main

---

## Current State

All research phases (1–4) and infrastructure work complete. CI/CD stable. v0.1.22 shipped with ADR-044 High-Impact Features.

v0.1.23 sprint focuses on completing test health remediation (ADR-041):
- Remaining test fixes (T4.2, T4.3, T5.2, T5.3)
- Documentation updates

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Next Sprint: v0.1.23

### Test Health Remediation (ADR-041)

#### P2: Remaining Test Fixes
- T4.2: Sandbox timing tests
- T4.3: WASM binary data tests
- T5.2: Nightly trend tracking
- T5.3: libsql version monitor

### Completed in v0.1.22

- ✅ ADR-044: High-Impact Features (100% complete)
- ✅ Feature 1: Actionable Playbooks (P0)
- ✅ Feature 2: Recommendation Attribution (P0)
- ✅ Feature 3: Episode Checkpoints/Handoff (P1)
- ✅ 26 playbook tests, 8 attribution tests, 3 checkpoint tests

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
| v0.1.22 | 2026-03 | ADR-044 High-Impact Features (Playbooks, Attribution, Checkpoints) |
| v0.1.21 | 2026-03 | Publishing infrastructure (ADR-045), supply chain security |
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
