# Active Development Roadmap

**Last Updated**: 2026-04-02 (v0.1.28 sprint planned)
**Released Version**: v0.1.26
**Branch**: `main` (all PRs merged)
**Epic**: [#373](https://github.com/d-o-hub/rust-self-learning-memory/issues/373) — ALL ISSUES CLOSED

---

## Current State

v0.1.26 released with crate renaming (`memory-*` → `do-memory-*`). All 4 crates published to crates.io. GitHub Release v0.1.26 created with multi-platform binaries.

v0.1.27 feature sprint complete on `main` branch. Features will be included in next release.

See [STATUS/CURRENT.md](../STATUS/CURRENT.md) for detailed metrics.

---

## Next Sprint — v0.1.28 (Planned)

### P1: Feature Enhancements

| Task | Description | Status | WG | Source |
|------|-------------|--------|----|--------|
| WG-089 | DyMoE routing-drift protection + affinity gating | 🔵 Planned | 89 | [#419](https://github.com/d-o-hub/rust-self-learning-memory/issues/419) |
| WG-090 | Dual reward scoring (stability + novelty signals) | 🔵 Planned | 90 | [#419](https://github.com/d-o-hub/rust-self-learning-memory/issues/419) |

### P1: Security

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-092 | Resolve open Dependabot alerts (rustls-webpki, lru, libsql-sqlite3-parser) | 🔵 Planned | 92 |
| WG-093 | Fix CodeQL cleartext logging alert in feedback CLI | 🔵 Planned | 93 |

### P2: CI/Infra

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-091 | Merge AI spam detector PR #406 | 🔵 Planned | 91 |

---

## Previous Sprint — v0.1.27 Feature Sprint (Complete ✅)

### P1: Feature Enhancements

| Task | Description | Status | WG |
|------|-------------|--------|----|
| WG-073 | Bayesian ranking with Wilson score from attribution data | ✅ Complete | 73 |
| WG-077 | MMR diversity retrieval for search results | ✅ Complete | 77 |
| WG-075 | Episode GC/TTL with retention policy | ✅ Complete | 75 |
| WG-078 | MCP Server Card at `.well-known/mcp.json` | ✅ Complete | 78 |

### P2: Code Quality

| Task | Description | Status |
|------|-------------|--------|
| WG-079 | Audit spawn_blocking usage for CPU-heavy async paths | ✅ Complete |

### P2: Infrastructure

| Task | Description | Status |
|------|-------------|--------|
| WG-084 | Restore GitHub Pages with mdBook | ✅ Complete |
| WG-085 | Add llms.txt for LLM context | ✅ Complete |

### P3: CI Optimization

| Task | Description | Status |
|------|-------------|--------|
| Semver timeout fix | Increase timeout + add baseline caching | ✅ Complete (PR #416) |

---

## Upcoming Sprint — v0.1.23 Remediation (Complete)

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
| WG-054 | Regenerate API reference, README, playbook/checkpoint docs, CLI command tables | ✅ Complete (2026-03-24) |
| WG-058 | Align AGENTS.md, agent_docs/, `.agents/skills/` with script-first workflow + disk/cov guidance | ✅ Complete (2026-03-24) |

### P1: Validation & Coverage Parity

| Task | Description | Status |
|------|-------------|--------|
| WG-055 | Expand PR-required CI workflows to cover integration + CLI/MCP suites, benchmark coverage | ✅ Complete (2026-03-24) |
| WG-056 | Enforce ≥90% coverage threshold in scripts/tests (update `scripts/check-coverage.sh`, `quality_gates.rs`) | ✅ Complete (2026-03-24) |

### P2: Disk & Developer Experience

| Task | Description | Status |
|------|-------------|--------|
| WG-057 | Reduce local disk footprint (target/node_modules) + automate cleanup | ✅ Complete (2026-03-24) |

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
| v0.1.27 | 2026-04 | Wilson score ranking, Episode GC/TTL, spawn_blocking audit, MCP Server Card, GitHub Pages, llms.txt, semver timeout fix |
| v0.1.24 | 2026-03 | Test stability (DBSCAN budget, quality gate timeout), dependency updates |
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
