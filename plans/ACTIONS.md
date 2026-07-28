# GOAP Actions Backlog

- **Last Updated**: 2026-07-26  
- **Active plan**: `plans/GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md`  
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions (2026-07-25)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-302 | `./scripts/release-manager.sh ship --execute` for `v0.1.36` | R-A1 | ✅ Done |
| ACT-303 | Post-release workspace bump to 0.1.37 | R-A2 | ✅ #886 |
| ACT-315 | Plans progress truth (open PRs, post-ship) | R-G* | ✅ #889 |
| ACT-316 | Land #887 changelog hygiene | docs | ✅ #887 merged |
| ACT-317 | Review/merge #888 cosine perf | perf | ✅ #888 merged |
| ACT-318 | Mark ADR-074 as Accepted / Implemented | docs | ✅ Done (#891) |
| ACT-319 | Gap analysis tasks: pattern extract + ADR-074 docs | G-P1-12/ACT-317/318 | ✅ #891 merged |
| ACT-320 | R-F8 CLI relationship show polish | R-F8 | ✅ #893 merged |
| ACT-321 | R-F9 HNSW persistence + capacity eviction | R-F9 | ✅ #893 merged |
| ACT-322 | Add 6 domain skills (40 total, all routed) | skills | ✅ #894 |
| ACT-323 | ADR-077 A1-A5 runtime embedding activation | ADR-077 | ✅ main (`9ef4b742`, `e0f7f712`) |
| ACT-324 | ADR-077 A6 validate/document/gate (docs + concurrency + zero-unsafe redaction tests) | ADR-077 | ✅ #897 merged |
| ACT-312 | R-F* GO spike artifacts written + validated (2026-07-28) | R-F* | ✅ Done |
| ACT-325 | Implement R-F10 OIDC trusted publishing in publish-crates.yml | R-F10 | 🔄 In progress |
| ACT-326 | Implement R-F4 SIMD cosine acceleration + benchmark variants | R-F4 | 🔄 In progress |

All ACT-300…ACT-324 items (excluding ACT-325/326 in progress) are **complete**.

## Completed actions (summary)

All ACT-190…ACT-279 series and 2026-07 recommendation waves are **complete**.  
Full tables: `plans/archive/2026-07-consolidation/completed-sprints/`

### Prevention permanently (do not regress)

- Never `#[serde(tag=)]` on postcard types  
- StorageBackend new methods → all backends  
- CLI path flags → set `redb_path`  
- Cross-process storage features → e2e CLI test  
- No manual `gh release create`; use release-manager + `release.yml`  
- No soft-pass on cargo deny / required cancelled checks  
- Fail-closed `execute_agent_code` unless approved capability backend  
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+)  
- Docs integrity: do not re-check `plans/archive/**` link rot as a ship blocker  
- After tag `vX.Y.Z`, immediately bump workspace to next patch before more feat/fix commits  
