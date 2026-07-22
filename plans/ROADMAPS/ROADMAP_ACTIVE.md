# Active Development Roadmap

**Last Updated**: 2026-07-22  
**Released Version**: v0.1.36  
**Workspace Version**: 0.1.36  
**Active Sprint**: Release readiness + residual recommendations  
**Plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main`  
**Open PRs**: *(implementation PR for R-E2 may be open)*  
**Open issues**: none  

---

## Sprint 2026-07-22 — Release

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Ship v0.1.36 | `release-manager.sh ship --execute` when main green | 🟡 |
| 2 | Post-bump | Workspace → 0.1.37 immediately after tag | 🟡 |
| 3 | R-E2 skill evals | Medium-risk behavioral fixtures second wave | ✅ |
| 4 | Plans truth | CURRENT / GAP / GOALS / ACTIONS / GOAP_STATE | ✅ |

---

## Completed last sprint (2026-07-20…21)

| Priority | Item | Status |
|----------|------|--------|
| Plans hygiene (ADR-039 archive) | ✅ |
| Recommendations register R-A…R-G | ✅ |
| Implementation #878 (LOC, skills, journal CLI, docs) | ✅ |
| Release docs #880 | ✅ |
| rust-major deps #877 | ✅ |
| Tracker refresh #881 | ✅ |

---

## Follow-on backlog (P2)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P2 | Research | WG-108 version retention; WG-110 SIMD (bench-gated); WG-125 MoE; WG-135 federated HDC | ⏸ DEFER |
| P2 | Vision | Distributed sync, multi-tenancy/RBAC, OTel/Prometheus (see `ROADMAP_V030_VISION.md`) | Future |
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io | Future |
| P2 | Security | Transitive Dependabot advisories (upstream libsql/openssl chains) | Monitor |

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
