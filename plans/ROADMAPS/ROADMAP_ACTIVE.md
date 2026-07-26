# Active Development Roadmap

**Last Updated**: 2026-07-26  
**Released Version**: v0.1.36 (latest tag)  
**Workspace Version**: 0.1.37 (next release)  
**Active Sprint**: ADR-077 runtime embedding activation (A1-A6 complete)  
**Plan**: `plans/GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md`  
**Branch**: `main` @ `648e11ad`  
**Open PRs**: none  
**Open issues**: none  

---

## Completed sprint 2026-07-22…25 — Ship + post-bump + gap analysis + R-F8/R-F9 + skills

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Ship v0.1.36 | `release-manager.sh ship --execute` + release.yml | ✅ |
| 2 | Post-bump | Workspace → 0.1.37 (#886) | ✅ |
| 3 | R-E2 skill evals | Medium-risk behavioral fixtures (#883) | ✅ |
| 4 | Docs integrity | Unblock ship gate (#885) | ✅ |
| 5 | Plans truth (#889) | CURRENT / GOALS / ACTIONS / GOAP_STATE / GAP refresh | ✅ |
| 6 | Changelog hygiene (#887) | Update CHANGELOG.md for v0.1.36 | ✅ |
| 7 | Cosine unrolled (#888) | 8-way accumulator optimization | ✅ |
| 8 | Gap tasks (#891) | ADR-074 docs, pattern extract command (G-P1-12), coverage | ✅ |
| 9 | R-F8 + R-F9 (#893) | CLI relationship box-drawing panel + HNSW persistence/eviction | ✅ |
| 10 | 6 domain skills | checkpoint-handoff, embedding-ops, episode-relationships, episode-tags, playbook-ops, recommendation-feedback (40 total) | ✅ |
| 11 | ADR-077 A1-A5 | Runtime embedding activation: exact-provider factory, atomic runtime seam, MCP end-to-end (main `9ef4b742`, `e0f7f712`) | ✅ |
| 12 | ADR-077 A6 | Validate/document/gate: activation docs + concurrency + zero-unsafe credential-redaction regression tests | ✅ #897 merged |

---

## Follow-on backlog (P2 — spike-gated)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P2 | Research | WG-108 / WG-110 / WG-125 / WG-135 | ⏸ DEFER |
| P2 | Vision | Distributed sync, multi-tenancy, OTel | Future |
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io | Future |
| P2 | Security | Transitive Dependabot advisories | Monitor |
| P2 | CLI | ADR-076 §5 `pattern extract` error-arm coverage | ✅ Done (#891) |
| P2 | CLI | R-F8 relationship info box-drawing panel | ✅ Done (#893) |
| P2 | Embeddings | R-F9 HNSW persistence + capacity eviction | ✅ Done (#893) |

---

## Standing product decisions (do not reopen casually)

| Topic | Decision |
|-------|----------|
| Agent code execution | **Fail-closed**; S1.1c Wasmtime/WASI **NO-GO** |
| Batch MCP tools | Explicitly deferred |
| Release creation | Automated only: `release-manager.sh ship` → tag → `release.yml` |
| Serialization | Postcard required |
| ADR-075 durable complete | All-or-nothing; backend failures are hard errors |
| ADR-076 pattern list | Empty diagnostics in human mode; JSON/YAML machine-stable |

---

## History pointer

Completed sprint tables live under `plans/archive/2026-07-consolidation/` and older archives.  
Do not re-expand completed WG tables (ADR-039).
