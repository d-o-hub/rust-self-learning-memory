# GOAP Actions Backlog

- **Last Updated**: 2026-08-06
- **Active plan**: `plans/GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md`
- **Prior plan**: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions — ADR-081 attribution completion (2026-08-06)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-344 | Accept ADR-081 and freeze the completed attribution contract | RAT-B0 | Proposed |
| ACT-345 | Split `attribution/tracker.rs` (557 LOC) into mod/integrity/stats/tests | RAT-B1 | Blocked by ACT-344 |
| ACT-346 | Add `supports_recommendation_attribution` to `StorageBackend`; make `persist_session_checked` capability-aware | RAT-B2 | Blocked by ACT-344 |
| ACT-347 | **Resolve feedback sessions from storage before rejecting — fixes post-restart feedback regression** | RAT-B3 | Blocked by ACT-345/346 · **merge blocker** |
| ACT-348 | Validate episode existence; make MCP reject malformed `episode_id` like the CLI | RAT-B4 | Blocked by ACT-346 |
| ACT-349 | Add `try_retrieve_playbooks`; generation failure must create no session | RAT-B5 | Blocked by ACT-346 |
| ACT-350 | Add `persist_feedback_checked`; receipts on manual MCP/CLI session + feedback commands | RAT-B6 | Blocked by ACT-346/347 |
| ACT-351 | Declare `episode_id` in both MCP tool registries + registry-agreement test | RAT-B7 | Blocked by ACT-348 |
| ACT-352 | Deduplicate CLI rendering; replace `too_many_arguments` suppression with a request struct | RAT-B8 | Blocked by ACT-348/349 |
| ACT-353 | Restart-safety, receipt matrix, MCP snapshot, CLI e2e tests to ≥90% | RAT-B9 | Blocked by ACT-345…352 |
| ACT-354 | `API_REFERENCE` attribution docs, ADR-058 duplicate resolution, `.gitignore` hygiene | RAT-B10 | Blocked by ACT-353 |

Detailed per-file code changes for every ACT-344…354 item are in
`plans/GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md` §4.

## Active actions (2026-07-30)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-334 | Accept ADR-079 and freeze the `CI / Required` aggregate contract | CIT-A1 | Proposed |
| ACT-335 | Implement and fault-inject same-run required aggregation | CIT-A1 | Blocked by ADR acceptance |
| ACT-336 | Fail closed on cancellation/missing/commitlint and restore Dependabot/fork assertion parity | CIT-A2 | Blocked by ACT-335 |
| ACT-337 | Reconcile test/Clippy/quality scopes and add semantic gate-contract fixtures | CIT-A3 | Blocked by ACT-336 |
| ACT-338 | Remove broken release dispatch and make publish selection/dependency planning truthful | CIT-A4 | Planned |
| ACT-339 | Preserve fuzz/mutation evidence, then measure and remove duplicate CI work | CIT-A5 | Planned |
| ACT-340 | With approval, require the verified aggregate in ruleset `9591004` and validate blocking | CIT-A1/PTA-A9 | Blocked by ACT-335…337 and maintainer approval |
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
| ACT-341 | Make non-`csm` cascade retrieval capability-truthful | PTA-A1 | ✅ Implemented |
| ACT-342 | Make CLI storage metrics measured/estimated/unavailable explicitly | PTA-A2 | ✅ Implemented |
| ACT-343 | Hide unsupported `eval set-threshold` command | PTA-A3 | ✅ Implemented |
| ACT-328 | Accept ADR-080 and freeze attributed contracts | RAT-A1 | Proposed |
| ACT-329 | Add capability-aware checked attribution persistence | RAT-A2 | Blocked by ADR acceptance |
| ACT-330 | Add core attributed pattern/playbook operations | RAT-A3 | Blocked by ACT-329 |
| ACT-331 | Enforce session and feedback integrity | RAT-A4 | Blocked by ACT-330 |
| ACT-332 | Wire optional attribution through MCP and CLI | RAT-A5/A6 | Blocked by ACT-331 |
| ACT-333 | End-to-end validation, docs, and authority update | RAT-A7/PTA-A9 | Blocked by ACT-341…343 + RAT chain |

All ACT-300…ACT-324 items are complete. ACT-341…ACT-343 (PTA-A1/A2/A3) are
implemented 2026-08-01. ACT-325/326 (R-F10/R-F4) are in progress. Remaining open
items must not be marked complete without code, workflow, live-ruleset, and
validation evidence as applicable.

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
- Required status must be a causal same-run aggregate; an echo anchor is not a gate
- Dependabot/fork trust changes permissions and secret access, not test assertions
- Fail-closed `execute_agent_code` unless approved capability backend  
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+)  
- Docs integrity: do not re-check `plans/archive/**` link rot as a ship blocker  
- After tag `vX.Y.Z`, immediately bump workspace to next patch before more feat/fix commits  
- A `StorageBackend` no-op default `Ok(())` is not evidence of a write — gate durability claims on an advertised capability, never on "a backend is configured"
- Tightening a validation from warn-to-error requires the full resolution chain first; rejecting on the in-memory view alone converts a missing feature into a restart regression
- An optional identifier parsed with `.ok()` degrades silently — parse absent vs. malformed distinctly, and identically on every surface
- A new surface parameter is not shipped until it appears in the MCP tool schema (both registries)  
