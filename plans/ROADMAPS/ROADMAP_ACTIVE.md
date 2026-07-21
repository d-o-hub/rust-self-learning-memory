# Active Development Roadmap

**Last Updated**: 2026-07-21  
**Released Version**: v0.1.35  
**Workspace Version**: 0.1.36 (unreleased development)  
**Active Sprint**: Release v0.1.36 + open PR CI hygiene  
**Plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main`  
**Open PRs**: #880 (release docs), #877 (rust-major deps)  
**Open issues**: #879 (release drift)

---

## Sprint 2026-07-21 — Release + CI

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Release docs | PR #880 finalize CHANGELOG / ROADMAP / CURRENT for v0.1.36 | 🟡 CI |
| 2 | Ship v0.1.36 | `release-manager.sh ship --execute` after #880 + main green | 🟡 |
| 3 | Post-bump | Workspace → 0.1.37 immediately after tag | 🟡 |
| 4 | rust-major deps | PR #877 sha2 0.11 / lz4_flex 0.14 / cargo_metadata 0.23 | 🟡 CI |
| 5 | Plans truth | CURRENT / GAP / GOALS / ACTIONS / GOAP_STATE tracker refresh | 🟡 |

---

## Completed last sprint (2026-07-20)

| Priority | Item | Status |
|----------|------|--------|
| Plans hygiene (ADR-039 archive) | ✅ |
| Recommendations register R-A…R-G | ✅ |
| Implementation #878 (LOC, skills, journal CLI, docs) | ✅ |

---

## Follow-on backlog (P1/P2)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P1 | Skills depth | Medium-risk behavioral evals (R-E2) | Backlog |
| P1 | Operator F4 | Any remaining MCP provenance gaps (R-C4) | Backlog |
| P2 | Research | WG-108 version retention; WG-110 SIMD (bench-gated); WG-125 MoE; WG-135 federated HDC | Backlog |
| P2 | Vision | Distributed sync, multi-tenancy/RBAC, OTel/Prometheus (see `ROADMAP_V030_VISION.md`) | Future |
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io | Future |

---

## Standing product decisions (do not reopen casually)

| Topic | Decision |
|-------|----------|
| Agent code execution | **Fail-closed**; S1.1c Wasmtime/WASI **NO-GO** |
| Batch MCP tools | Explicitly deferred |
| Release creation | Automated only: `release-manager.sh ship` → tag → `release.yml` |
| Serialization | Postcard required |

---

## History pointer

Completed sprint tables (v0.1.28–v0.1.35, July GOAP waves) live under:

- `plans/archive/2026-07-consolidation/completed-sprints/`
- `plans/archive/2026-03-consolidation/`
- Older `plans/archive/2026-0{1,2}-completed/`

Do not re-expand completed WG tables into this file (ADR-039: roadmap is forward-only).
