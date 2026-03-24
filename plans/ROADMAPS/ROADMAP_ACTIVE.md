# Active Development Roadmap

**Last Updated**: 2026-03-24 (v0.1.23 remediation in progress; WG-053 complete)
**Released Version**: v0.1.22
**Branch**: `main` (PR #391 merged)
**PR**: [#391](https://github.com/d-o-hub/rust-self-learning-memory/pull/391) ✅ Merged, all CI passing
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED

---

## Current State

v0.1.22 sprint COMPLETE. All 12 GitHub issues closed. All quality gates passing. PR #391 merged to main. All research phases (1–4), infrastructure work, and ADR-044 High-Impact Features (Playbooks, Attribution, Checkpoints, Feedback) are fully complete and polished.

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Upcoming Sprint — v0.1.23 Remediation (In Planning)

The 2026-03-24 audit reopened several items. The new sprint focuses on truth-source reset, ADR-044 durability, CI/test parity, and disk/DX hygiene. Full execution plan: [GOAP_EXECUTION_PLAN_v0.1.23.md](../GOAP_EXECUTION_PLAN_v0.1.23.md).

### P0: Feature Integrity (ADR-044)

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-051 | Persist recommendation sessions + feedback via storage traits and schema updates | ✅ Complete (2026-03-24) | 51 |
| WG-052 | Persist checkpoints/handoffs across Turso/redb read paths | ✅ Complete (2026-03-24) | 52 |
| WG-053 | Decide/implement batch MCP tool strategy + align status/docs | ✅ Complete (2026-03-24) | 53 |

### P1: Documentation & Contract Truth Sources

| Task | Description | Status |
|------|-------------|--------|
| WG-054 | Regenerate API reference, README, playbook/checkpoint docs, CLI command tables | 🚧 Planned |
| WG-058 | Align AGENTS.md, agent_docs/, `.agents/skills/` with script-first workflow + disk/cov guidance | 🚧 Planned |

### P1: Validation & Coverage Parity

| Task | Description | Status |
|------|-------------|--------|
| WG-055 | Expand PR-required CI workflows to cover integration + CLI/MCP suites, benchmark coverage | 🚧 Planned |
| WG-056 | Enforce ≥90% coverage threshold in scripts/tests (update `scripts/check-coverage.sh`, `quality_gates.rs`) | 🚧 Planned |

### P2: Disk & Developer Experience

| Task | Description | Status |
|------|-------------|--------|
| WG-057 | Reduce local disk footprint (target/node_modules) + automate cleanup | 🚧 Planned |

---

## Sprint v0.1.22 — COMPLETE ✅

### P0: Critical Fixes — ALL COMPLETE ✅

| Task | Description | Status | Issue |
|------|-------------|--------|-------|
| WG-040 | Fix 2 failing doctests (attribution, playbook) | ✅ Complete | #374 — CLOSED |
| WG-041 | Fix test timeout (quality_gate_no_clippy_warnings) | ✅ Complete | #375 — CLOSED |
| WG-042 | Split 3 production files >500 LOC | ✅ Complete | #376 — CLOSED |

### P1: Quality Polish — ALL COMPLETE ✅

| Task | Description | Final | Target | Status | Issue |
|------|-------------|-------|--------|--------|-------|
| WG-043 | Reduce `#[allow(dead_code)]` | 31 | ≤40 | ✅ Target met | #377 — CLOSED |
| WG-044 | Fix broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) | #378 — CLOSED |
| WG-045 | Add snapshot tests for new features | 80 | ≥80 | ✅ Complete | #379 — CLOSED |
| WG-046 | Add property tests for new features | 16 | ≥13 | ✅ Exceeds target | #380 — CLOSED |

### P2: Feature Enhancements — ALL COMPLETE ✅

| Task | Description | Status | Issue |
|------|-------------|--------|-------|
| WG-047 | MCP tool contract parity for new tools | ✅ Complete | #381 — CLOSED |
| WG-048 | Integration tests for attribution + checkpoint flows | ✅ Complete | #382 — CLOSED |
| WG-049 | Changelog automation (git-cliff) | ✅ Complete | #383 — CLOSED |
| WG-050 | Documentation for new features | ✅ Complete | #384 — CLOSED |

### P3: Infrastructure — ALL COMPLETE ✅

| Task | Description | Since | Status | Issue |
|------|-------------|-------|--------|-------|
| WG-051 | Nightly trend tracking artifact | v0.1.20 | ✅ Complete | #385 — CLOSED |
| WG-052 | libsql upstream version monitor | v0.1.20 | ✅ Complete | #386 — CLOSED |
| WG-053 | Structured tech-debt registry | v0.1.17 | ✅ Complete | #387 — CLOSED |

### Shipped in v0.1.21

- ✅ ADR-045: Publishing infrastructure (supply chain, OIDC, metadata)
- ✅ ADR-046: Claude Code configuration improvements (session analysis, tool enforcement)

### Shipped in v0.1.22 (Features — Pre-Tag)

- ✅ ADR-044: High-Impact Features (100% code complete)
  - ✅ Feature 1: Actionable Playbooks (26 tests)
  - ✅ Feature 2: Recommendation Attribution (8 tests)
  - ✅ Feature 3: Episode Checkpoints/Handoff (6 tests)
  - ✅ Feature 4: Recommendation Feedback (3 tests)

---

## Backlog

### Code Quality (from #373 epic — ALL RESOLVED)

| Item | Current | Target | Status | Notes |
|------|---------|--------|--------|-------|
| `#[allow(dead_code)]` annotations | 31 | ≤40 | ✅ Target met | Down from 70 |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) | |

### Testing

| Item | Current | Target | Status | Notes |
|------|---------|--------|--------|-------|
| Ignored tests | 124 | ≤125 ceiling | ✅ | 70 upstream libsql bug, rest by design |
| Property test expansion | 16 files | ≥13 | ✅ Exceeds target | |
| Snapshot test growth | 80 snaps | ≥80 | ✅ Target met | |

### Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Changelog automation (git-cliff) | ✅ Complete | `.github/workflows/changelog.yml` |
| Structured tech-debt registry | ✅ Complete | `docs/TECH_DEBT.md` |
| libsql version monitor | ✅ Complete | `scripts/check-libsql-version.sh` |
| Nightly trend tracking | ✅ Complete | Artifact added |
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
- **Execution plans**: [GOAP_EXECUTION_PLAN_v0.1.22.md](../GOAP_EXECUTION_PLAN_v0.1.22.md), [GOAP_EXECUTION_PLAN_v0.1.23.md](../GOAP_EXECUTION_PLAN_v0.1.23.md)
- **Long-term vision**: [ROADMAP_V030_VISION.md](ROADMAP_V030_VISION.md)
- **ADRs**: [adr/](../adr/)
