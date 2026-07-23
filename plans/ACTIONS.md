# GOAP Actions Backlog

- **Last Updated**: 2026-07-23  
- **Active plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions (2026-07-23)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-302 | `./scripts/release-manager.sh ship --execute` for `v0.1.36` | R-A1 | ✅ Done |
| ACT-303 | Post-release workspace bump to 0.1.37 | R-A2 | ✅ #886 |
| ACT-315 | Plans progress truth (open PRs, post-ship) | R-G* | 🟡 This PR |
| ACT-316 | Optional: land #887 changelog hygiene | docs | 🟡 open |
| ACT-317 | Optional: review/merge #888 cosine perf | perf | 🟡 open |
| ACT-312 | Optional product/research spikes R-F* | R-F* | ⏸ DEFER |

All ACT-300…ACT-314 items (LOC, skills, journal, provenance, plans hygiene, evals, docs integrity, release) are **complete**.

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
