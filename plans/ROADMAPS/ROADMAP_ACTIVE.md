# Active Development Roadmap

**Last Updated**: 2026-07-20  
**Released Version**: v0.1.35  
**Workspace Version**: 0.1.36 (unreleased development)  
**Active Sprint**: Recommendations backlog + release readiness  
**Plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main`  
**Open PR**: none  

---

## Sprint 2026-07-20 — Recommendations & plans consolidation

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Plans hygiene | Archive superseded GOAP/CI/research plans (ADR-039) | ✅ |
| 2 | Recommendations register | Single active backlog with R-A…R-G tracks | ✅ |
| 3 | Canonical status refresh | CURRENT, GOALS, ACTIONS, GOAP_STATE, GAP, VALIDATION | ✅ |
| 4 | Code / release | No code or tag in this sprint | ⛔ |

---

## Next sprint (proposed) — Release + invariants

| Priority | Item | Rec | Status |
|----------|------|-----|--------|
| 1 | Cut **v0.1.36** via release-manager + `release.yml` | R-A1 | 🟡 |
| 2 | Immediate workspace bump to **0.1.37** | R-A2 | 🟡 |
| 3 | Split `provider_config.rs` ≤500 LOC | R-B1 | 🟡 |
| 4 | ADR-025 / ADR-054 identifier aliases | R-B5 | 🟡 |
| 5 | Complete skill routes (34/34) + `ci-poll` evals + `.agents/SKILLS.md` | R-C1–C3 | 🟡 |
| 6 | F4 productization (provenance + journal operator surfaces) | R-B2 | 🟡 |
| 7 | Docs contract pass (README, AGENTS, HARNESS, TECH_DEBT) | R-D*, R-B3 | 🟡 |

---

## Follow-on backlog (P1/P2)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P1 | Skills depth | Medium-risk behavioral evals (R-E2); skill link lint (R-E4) | Backlog |
| P1 | Operator F4 | MCP provenance fields; journal CLI repair; digest e2e | Backlog |
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
