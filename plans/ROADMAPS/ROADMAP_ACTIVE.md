# Active Development Roadmap

**Last Updated**: 2026-07-22  
**Released Version**: v0.1.36  
**Workspace Version**: 0.1.37  
**Active Sprint**: Post-v0.1.36 development  
**Plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
**Branch**: `main`  

---

## Sprint 2026-07-22 — Ship + post-bump

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Ship v0.1.36 | `release-manager.sh ship --execute` | ✅ |
| 2 | Post-bump | Workspace → 0.1.37 | ✅ |
| 3 | R-E2 skill evals | Medium-risk behavioral fixtures | ✅ #883 |
| 4 | Docs integrity | Unblock ship gate | ✅ #885 |

---

## Follow-on backlog (P2)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P2 | Research | WG-108 / WG-110 / WG-125 / WG-135 | ⏸ DEFER |
| P2 | Vision | Distributed sync, multi-tenancy, OTel | Future |
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io | Future |
| P2 | Security | Transitive Dependabot advisories | Monitor |

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

Completed sprint tables live under `plans/archive/2026-07-consolidation/` and older archives.  
Do not re-expand completed WG tables (ADR-039).
