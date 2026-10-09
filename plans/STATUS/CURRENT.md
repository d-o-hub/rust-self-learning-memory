# Project Status — Self-Learning Memory System

**Last Updated**: 2026-10-09
**Released Version**: v0.1.44 (latest tag)
**Workspace Version**: 0.1.45 (post-v0.1.44 bump)
**Edition**: Rust 2024
**Active plan**: **audit-backlog wave** (`GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`) — W1–W4 + Q02 merged; wave A in review (#1138, #1140, #1144–#1154, #1158, #1159); #1142 (#1065), #1143 (#1092 slice 1), #1148 (#1079) merged 2026-10-06…08. Remaining: chains #1067→#1068→#1089 and #1073→#1074→#1076, then release `v0.1.45` (#1137). Prior wave (2026-10-02/04): v0.1.44 shipped, checked completion receipts (#1080 → #1107), CLI drain-and-verify (#1081 → #1112), pattern-search input bounds (#1113), clippy 1.99 migration (#1108), architecture/status refresh (#1094 → #1110), coverage-floor reconciliation (#1090 → #1117), release pipeline C1–C7 (#1121/#1123/#1125).
**Branch**: main @ `5054c15a` (2026-10-08)

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | run `gh pr list --state open` (17 at 2026-10-09: wave A + plans #1141) |
| Open issues | run `gh issue list --state open` (16 at 2026-10-09) — per-issue verdicts, evidence and queue position live in `STATUS/GAP_ANALYSIS_LATEST.md` |

## Corrected claim (2026-10-04)

The "P0 plan gaps: **0 open code-side**" line once below this section described the **CI-trust / attribution** campaign (ADR-079,
CIT-A1…A5, PTA, RAT) and is still true for that campaign. It was never true of the repository as a whole: the
22-issue audit backlog was open the entire time and absent from the trackers, which is how "0 open gaps" got
restated at each refresh. Read any campaign claim as campaign-scoped; the repo-wide register is
`STATUS/GAP_ANALYSIS_LATEST.md`.

## Recent completed (2026-10-05 — audit wave W1–W4 + hygiene)

| Wave | Result |
|------|--------|
| W1 #1077 | ✅ #1131 (`874df209`) — ranking snapshot cloned; read guard dropped before every `.await`; contention test |
| W2 #1086 | ✅ #1135 (`75007b51`) — monotonic `Instant`; sub-second MCP latency no longer truncates to 0 ms |
| W3 #1088 | ✅ #1134 (`347b3296`) — Turso tag transactions roll back on every failure path |
| W4 #1066 | ✅ #1139 (`f1c31699`, `0485bb66`) — redb episode→session index ranks by `(timestamp, session_id)`; open-time repair heals stale write-order winners (no schema bump) |
| Hygiene | ✅ #1128 docs-integrity false positives, #1129 tracker reconciliation, #1133 local hook sensors, #1136 dependabot actions bump |

## In review (2026-10-09)

| Slice | PR | State |
|-------|-----|-------|
| W5 #1075 query-aware lexical fallback | #1138 | roast MAJORs fixed (raw-query tokens only, context double-count removed); CI re-running |
| W6 #1085 health probes + URL redaction (E1) | #1140 | roast MAJOR fixed (live retrieval-cache metrics); rebased; CI re-running |
| #1067 modification watermark | #1149 | 6 roast MAJORs fixed (txn atomicity, monotonic revisions, durable watermark, loud default, Turso backfill); rebased |
| #1064 scoped pool checkout | #1147 | roast BLOCKER fixed (brute-force fallback reuses the held connection); rebased |
| #1063 / #1070 / #1071 / #1073 / #1078 | #1146, #1150, #1145, #1151, #1154 | CI fixes pushed (assert_is_empty, doctest, LOC decomposition) |
| #1087 capability truth | #1144, #1152, #1158 | cleanup pair / procedural+relationships / inventory+checker, all in review |
| #1092 lint slice 2 | #1153 | crate-root allows 317 → 0 with expects; workspace clippy green locally |
| Nightly JSON flag fix | #1159 | isolated job never ran its tests; env var added |
| #1155 / #1157 (jules) | #1155, #1157 | perf (Cow lowercase) / security (query_range clamp); #1157 cleaned to one commit |

## Recent completed (2026-10-04 — release pipeline C5–C7)

| Wave | Result |
|------|--------|
| Release pipeline C5–C7 (#1109) | ✅ PR #1125 (`4fb0e20f`) — "Verify a release" book page + SECURITY.md signer-pinned attestation commands, `release.yml` PR trigger removed (it ran only skipped jobs), `v`-prefixed tag glob, publish-path `persist-credentials: false`, `host` OIDC blanked on SBOM build steps |
| Audit-backlog reconciliation | ✅ #1129 (`26b12fc8`) — 22 issues re-validated at `74a44a15`, registered in `GAP_ANALYSIS_LATEST.md`, wave slices W1–W6 queued (ACT-370…375) |

## Recent completed (2026-10-02 — v0.1.44 shipped)

| Wave | Result |
|------|--------|
| Pattern-search input bounds (#1113) | ✅ `search_patterns`/`recommend_patterns` clamp `limit`/`min_relevance` and truncate oversized `query`/`task_description`/`domain`/`tags` (CWE-770) |
| Architecture refresh (#1094 → #1110) | ✅ architecture/serialization/status evidence aligned with v0.1.44; bincode references corrected to postcard |
| Coverage floor (#1090 → #1117) | ✅ 70% is the blocking floor, 90% the aspirational target, with comparator unit tests |
| v0.1.44 release | ✅ tag `v0.1.44` on `4f4f4ba8`; GitHub Release with dist artifacts; drift issue auto-closed; workspace bumped to 0.1.45 |

## Recent completed (pointer — 2026-07…2026-10-01)

| Wave | Result |
|------|--------|
| Release-pipeline proposal (#1109) | ✅ code complete: C3/C4 (#1121 draft-first + attestations), C1/C2 (#1123 OIDC-only publish + dead-trigger fix), C5–C7 (#1125); manual crates.io steps tracked in ACT-368 |
| v0.1.43 / LOC ceiling | ✅ 2026-10-01 — tag `v0.1.43`; `check-loc.sh` now blocking in File Structure Validation (#1103) |
| MCP OAuth 2.1 + trusted rate-limit identity (#1082/#1084) | ✅ #1101 |
| redb fail-closed schema (#1069), embedding activation identity (#1072), MCP `tools/list` (#1083), pool ownership (#1060–#1062), MCP docs (#1099) | ✅ #1095–#1100 |
| Retrieval judgment (#1030 → #1041) + rerank (#1031 → #1042) + merge tooling (#1046) | ✅ 2026-09-24 |
| Feedback-to-ranking (ADR-082) + ADR registry | ✅ #952 (2026-08-13); lifecycle `Proposed` |
| CI trust + attribution closure (ADR-079/CIT/PTA/RAT) | ✅ #916/#927/#930/#938/#940/#947 (2026-08-06…12); stage 4 proof maintainer-external |
| v0.1.36…v0.1.42 ships + post-bumps + skills + R-F8/R-F9 | ✅ 2026-07…09 |

## Snapshot (current)

| Area | State |
|------|--------|
| First-party merge gate | ✅ **Live** — ruleset `9591004` requires `Codacy Static Code Analysis` + `CI / Required` (strict up-to-date policy); the aggregate is causally same-run and rejects `skipped`/`cancelled`/timed-out results |
| Repo-wide open gaps | **16 open issues**: audit backlog 14 remaining + #1109 (code-complete, manual) + #1137 (release drift) — register: `STATUS/GAP_ANALYSIS_LATEST.md` |
| Merged since 2026-10-06 | #1130 (#1091), #1142 (#1065), #1143 (#1092 slice 1), #1148 (#1079) |
| Production pattern query relevance | ⚠️ **pending #1074** (E2 — `semantic_service` has no writer until the live snapshot seam lands) |
| `episodes.created_at` watermark | ⚠️ **not a creation stamp** until #1067 lands (E3 — absent from the Turso INSERT column list) |
| Release | v0.1.44 tagged 2026-10-02; `v0.1.45` release follows the wave (#1137) |
| Code execution | Fail-closed; S1.1c Wasmtime/WASI **NO-GO** |
| Serialization | Postcard required |
| Skill evals / routes | 40/40 |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P1 | Finish the wave: merge #1138, #1140, #1130 with green CI | W5/W6/Q02 | 🔄 |
| P1 | Modification-watermark chain: #1067 → #1068 → #1089 (episode `updated_at` + Turso column + redb `SCHEMA_VERSION` bump, keyset pages, revision-aware merge, redb batch writer) | S08/S09/M08 | ⏳ next |
| P1 | Identity-scoped embedding chain: #1073 → #1074 (pattern sites) → #1076 | R02/R03/R05 | ⏳ next |
| P1 | `StorageBackend` capability truth (#1087, 4 PRs) · lint-suppression integrity (#1092, 4 PRs) | M06/Q03 | ⏳ |
| P1 | Remaining validated P1s: #1063, #1064, #1065, #1070, #1071, #1078, #1079 | S04/S05/S06/S11/S12/R07/R08 | ⏳ |
| P0 | Ship `v0.1.45` once the wave is green (drift issue #1137) | release | ⏳ |
| P0 | ADR-079 stage 4 live fault-injection proof; ADR-080/081 lifecycle; #1109 crates.io registrations + secret deletion | maintainer | ⏸ external |
| P2 | Research/product spikes (R-F1…R-F7, R-F10) | R-F* | ⏸ DEFER |

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Validation: `plans/STATUS/VALIDATION_LATEST.md`
- Archive: `plans/archive/2026-07-consolidation/`
