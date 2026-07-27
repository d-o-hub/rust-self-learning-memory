# Gap Analysis — 2026-07-26

**Generated**: 2026-07-26  
**Audit commit**: `e0f7f712` (`main`) + open PR `feat/adr077-a6-validate-document`
**Workspace**: `0.1.37` · **Tag**: `v0.1.36` (published 2026-07-22)  
**Full backlog**: [`../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`](../GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md)

## Method

- Live GitHub: open PRs **none**; open issues **none**  
- Release: `gh release view v0.1.36` published  
- Workspace version advanced post-tag (R-A2 / #886)  
- Prior gap register closed for all P0 ship items  
- R-F8 and R-F9 GO spikes implemented and merged (#893)  
- 6 new domain skills added (40 total, all routed)  

## Closed this wave

| Gap | Resolution |
|-----|------------|
| G-P0-1 v0.1.36 unreleased | ✅ Tag + GitHub Release 2026-07-22 |
| G-P0-4 / G-P0-5 release docs / rust-major | ✅ #880 / #877 |
| G-P1-7 medium-risk eval depth | ✅ R-E2 #883 |
| Docs integrity ship blocker | ✅ #885 |
| Post-tag version lag | ✅ workspace `0.1.37` #886 |
| G-P1-10 open hygiene/perf PRs | ✅ #887, #888, #889, #891, #893 all merged |
| R-F8 relationship show polish (GO spike) | ✅ #893 — box-drawing panel + unit tests |
| R-F9 HNSW persistence + eviction (GO spike) | ✅ #893 — file_dump/load + capacity eviction |
| Skill count 34, 6 domain skills untracked | ✅ 40 skills, all routed (#894) |
| ADR-077 runtime embedding activation A1-A5 | ✅ main (`9ef4b742`, `e0f7f712`) — exact-provider factory + atomic runtime seam + MCP end-to-end |
| ADR-077 A6 validate / document / gate | ✅ this PR — activation docs + concurrency + zero-unsafe credential-redaction regression tests |

## Open gaps (current)

### P0

*None.*

### P1

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P1-8 | Historical ADR number reuse on disk | Dual 025/054 filenames; aliases in `plans/adr/README.md` | residual docs |
| G-P1-9 | Transitive Dependabot advisories | Upstream chains (libsql/openssl/webpki) | security hygiene |

### P2 (product / research)

| ID | Gap | Notes | Track |
|----|-----|-------|--------|
| G-P2-1…7 | R-F1…R-F7, R-F10 epics | Spike-gated DEFER | R-F* |

## Explicit non-gaps

| Claim | Verdict |
|-------|---------|
| Working `execute_agent_code` backend | Intentional fail-closed |
| Batch MCP tools | Deferred product decision |
| Production LOC >500 | Closed |
| Medium-risk skill presence-only evals | Closed |
| Release lag / commit_limit on tag | Closed by v0.1.36 ship |
| R-F8 relationship show polish | ✅ #893 |
| R-F9 HNSW persistence | ✅ #893 |

## Exit criteria for this register

- G-P1-8 and G-P1-9 are monitor-only (no code action required)  
- P2 rows remain spikes until GO artifacts under `plans/STATUS/spikes/`  
