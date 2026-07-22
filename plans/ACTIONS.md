# GOAP Actions Backlog

- **Last Updated**: 2026-07-22  
- **Active plan**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`  
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions (2026-07-22)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-300 | Archive superseded plans; write recommendations + refresh CURRENT/GAP/VALIDATION/README | R-G1–G3 | ✅ Done |
| ACT-300b | Generate `.agents/SKILLS.md` inventory (HARNESS link target) | R-C3 | ✅ Done |
| ACT-304 | Split provider configs ≤500 LOC + tests | R-B1 | ✅ Done |
| ACT-305 | ADR-025 / ADR-054 alias registry (`plans/adr/README.md`) | R-B5 | ✅ Done |
| ACT-306 | Expand `skill-rules.json` to all catalog skills | R-C1 | ✅ Done |
| ACT-307 | Add `ci-poll` evals + SKILLS.md sync | R-C2, R-C3 | ✅ Done |
| ACT-308 | F4 operator UX: `storage journal` CLI (`--pending`/`--repair`) | R-B2, R-C5 | ✅ Done |
| ACT-308b | MCP provenance fields on query responses | R-C4 | ✅ Already on main |
| ACT-309 | Align AGENTS skill table + TECH_DEBT + vision title | R-D*, R-B3 | ✅ Done |
| ACT-310 | Medium-risk skill behavioral evals (second wave) | R-E2 | ✅ Done |
| ACT-311 | `validate-plans.sh` warn on excess dated root files | R-G4 | ✅ Done |
| ACT-301 | Prepare CHANGELOG + release docs for v0.1.36 | R-A1 | ✅ #880 merged |
| ACT-313 | Land rust-major dependabot (sha2/lz4/cargo_metadata) | deps | ✅ #877 merged |
| ACT-314 | Refresh plans tracker truth | R-G* | ✅ Done |
| ACT-302 | `./scripts/release-manager.sh ship --execute` for `v0.1.36` | R-A1 | 🟡 After main green |
| ACT-303 | Post-release workspace bump to 0.1.37 | R-A2 | 🟡 After ACT-302 |
| ACT-312 | Optional product/research spikes R-F* | R-F* | ⏸ DEFER |

## Completed actions (summary)

All ACT-190…ACT-279 series (CLI UX, S1.*, W2.*, K3.*, F4, harness, release cadence) are **complete**.  
Full tables live under:

`plans/archive/2026-07-consolidation/completed-sprints/`

### Prevention permanently (do not regress)

- Never `#[serde(tag=)]` on postcard types  
- StorageBackend new methods → all backends  
- CLI path flags → set `redb_path`  
- Cross-process storage features → e2e CLI test  
- No manual `gh release create`; use release-manager + `release.yml`  
- No soft-pass on cargo deny / required cancelled checks  
- Fail-closed `execute_agent_code` unless approved capability backend  
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+)  
