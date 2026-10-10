# GOAP Goals Index

- **Last Updated**: 2026-10-06
- **Status**: **audit-backlog wave — W1–W4 merged 2026-10-05, W5/W6 + Q02 in review.** 22 code issues (#1063–#1092) were filed
  at `9f50c607`, never registered in these trackers, and re-validation at `74a44a15` found **0 fixed** (20 OPEN, 2 PARTIAL);
  the wave closes them in dependency order. Earlier campaigns remain campaign-scoped: retrieval judgment (#1030 → #1041),
  rerank (#1031 → #1042), ADR-082 ranking adaptation (#952) merged; ADR-080/081/082 lifecycle acceptance and ADR-079
  stage 4 fault-inject proof remain maintainer-external
- **Workspace**: `0.1.45` · **Tag**: `v0.1.44` · **Plan**: `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`
- **Queued chains**: ACT-376…ACT-385 (watermark #1067→#1068→#1089; embedding identity #1073→#1074→#1076)
- **Archive**: `plans/archive/2026-07-consolidation/`

## Active goals — audit-backlog wave (2026-10-04…)

| Goal | Rec | Priority | Status |
|------|-----|----------|--------|
| G-A1 no lock held across `.await` in the recommendation path | #1077 / R06 | P1 | ✅ merged #1131 (`874df209`) |
| G-A2 sub-second MCP latency is real, not truncated to 0 | #1086 / M05 | P2 | ✅ merged #1135 (`75007b51`) |
| G-A3 a failed tag transaction cannot leave partial state | #1088 / M07 | P2 | ✅ merged #1134 (`347b3296`) |
| G-A4 the redb episode→session index follows recency, not write order | #1066 / S07 | P1 | ✅ merged #1139 (`f1c31699` + repair pass) |
| G-A5 pattern-search lexical fallback is query-aware (no constant `0.5`) | #1075 / R04 | P1 | 🔄 in review #1138 — roast MAJORs fixed (query-only tokens, context double-count), CI re-running |
| G-A6 MCP health reflects probed state and leaks nothing | #1085 / M04 | P1 | 🔄 in review #1140 — roast MAJOR fixed (live retrieval-cache metrics); includes escalation E1 (raw `TURSO_DATABASE_URL` in the response) |
| G-A7 trackers match `gh issue list` | governance | P0 | ✅ #1129 — `GAP_ANALYSIS_LATEST.md` is the repo-wide register |
| G-A8 ignored-test inventory + fail-visible isolated native job | #1091 / Q02 | P1 | ✅ merged #1130 (2026-10-06) — 159-entry inventory, 14 tests un-ignored, isolated job fail-visible (JSON flag follow-up #1159); issue closed with evidence |

## Wave A goals (2026-10-06…09, in review)

| Goal | Rec | PR | Status |
|------|-----|----|--------|
| Recommendation precision (ms + UUID tie-break + backfill) | #1065 / S06 | #1142 | ✅ merged 2026-10-08 (`5054c15a`) |
| Execution-backed retrieval provenance | #1079 / R08 | #1148 | ✅ merged 2026-10-07 (`9fdbc765`) |
| Lint suppressions slice 1 (turso unsafe peel + ratchet) | #1092 / Q03 | #1143 | ✅ merged 2026-10-06 (`cdb70594`); slice 2 #1153 in review |
| Strict row decode + allowlisted query builder | #1071 / S12 | #1145 | 🔄 in review (doctest fixed, rebased) |
| Capacity eviction atomic-or-repairable | #1070 / S11 | #1150 | 🔄 in review |
| Adaptive pool capacity model (E4) | #1063 / S04 | #1146 | 🔄 in review |
| Scoped pool checkout | #1064 / S05 | #1147 | 🔄 in review (roast BLOCKER fixed) |
| Modification watermark + keyset pages | #1067 / S08 | #1149 | 🔄 in review (6 roast MAJORs fixed) |
| Identity-scoped embedding adapter | #1073 / R02 | #1151 | 🔄 in review |
| Incremental ranking index + benches | #1078 / R07 | #1154 | 🔄 in review |
| Capability truth (cleanup / procedural+relationships / inventory) | #1087 / M06 | #1144, #1152, #1158 | 🔄 in review |
| Nightly libtest-JSON flag fix (follow-up to #1130) | — | #1159 | 🔄 in review |

## Queued goals (sequenced)

| Goal | Rec | Depends on |
|------|-----|------------|
| Modification watermark + bounded keyset pages | #1067 / S08 | — (E3, E5 constraints) |
| Revision-aware merge of redb + Turso | #1068 / S09 | #1067 |
| Batch writes through redb + synchronizer | #1089 / M08 | #1067 |
| Identity-scoped embedding storage adapter | #1073 / R02 | — |
| One runtime provider snapshot across memory paths (remaining pattern sites) | #1074 / R03 | #1073 (E2) |
| Reuse + batch pattern embeddings | #1076 / R05 | #1073, #1074 |
| Adaptive pool capacity model (cooldown + permits) | #1063 / S04 | — (E4: fix rewrites `adaptive_tests.rs:140-146`) |
| Scoped pool checkout (`with_connection`) | #1064 / S05 | — |
| Capacity eviction atomic-or-repairable | #1070 / S11 | — |
| Strict row decode + raw-query error surfacing | #1071 / S12 | — |
| Incremental ranking index + benches | #1078 / R07 | — |
| Execution-backed retrieval provenance | #1079 / R08 | — |
| Capability truth for `StorageBackend` defaults | #1087 / M06 | — (4 PRs) |
| Lint-suppression integrity | #1092 / Q03 | — (4 PRs) |
| Ship `v0.1.45` to clear release drift | #1137 | wave green |
| crates.io trusted-publisher registrations + secret deletion | #1109 C1 | maintainer-external |

### Why this wave and not the higher-effort chains

The wave criterion was: fixes a real defect, touches no `StorageBackend` trait signature (22 implementations — E5),
needs no schema migration, and is independently revertable. The watermark chain (#1067→#1068→#1089) and the
embedding-identity chain (#1073→#1074→#1076) are the highest-value structural fixes but each requires a migration plus
coordinated backend key changes; they are queued as ACT-376…ACT-381 with their rationale recorded.

### Escalations found by validation (changed priorities; no issue body states them)

| ID | Finding | Consequence |
|---|---|---|
| E1 | health output interpolates the raw `TURSO_DATABASE_URL` | #1085 is a security fix, not just observability (W6) |
| E2 | `semantic_service` is `None` on every production path, and both scoring fallbacks return the constant `0.5` | production pattern search ignores the query text; #1075 + #1074 are coupled and #1076 must not land first |
| E3 | `episodes.created_at` is absent from the Turso INSERT column list | it is not a creation stamp; constrains #1067 |
| E4 | adaptive pool cooldown compares against a fresh `Instant` (≈0 ns) and `adaptive_tests.rs:140-146` asserts the broken value | #1063's fix must change that test |
| E5 | `query_episodes_since` is required, not defaulted (22 impls) | #1067 adds a defaulted method instead of changing the signature |
| E6 | docs-integrity `<...>` handling inverted the AGENTS.md rule and aborted the ship path | fixed in #1128 (b5c87ec1) |

## Closed campaigns (pointer)

| Campaign / series | Outcome |
|---|---|
| v0.1.44 ship + release-pipeline C1–C7 (#1109) | ✅ 2026-10-02/04 — #1121 draft-first + attestations, #1123 OIDC-only publish, #1125 verify-a-release + slimming; manual crates.io steps tracked in ACT-368 |
| Retrieval judgment + rerank + merge tooling | ✅ #1041, #1042, #1046 (2026-09-24) |
| Feedback-to-ranking (ADR-082) + ADR registry | ✅ #952 (2026-08-13); lifecycle `Proposed` |
| ADR-079/CIT/PTA/RAT closure + capability truth | ✅ #916, #927, #930, #938, #940, #947 (2026-08-06…12); stage 4 proof maintainer-external |
| fuzz nightly toolchain + LTO-off | ✅ #934 (2026-08-09) |
| PR review & CI fix wave | ✅ 2026-08-07 |
| v0.1.36–v0.1.43 ships, post-bumps, R-E2/R-F8/R-F9/skills | ✅ #878…#897, #1107/#1112/#1113/#1108/#1110/#1117 (2026-07…10) |

Do not re-list completed WG tables here.
