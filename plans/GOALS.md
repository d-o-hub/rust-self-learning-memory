# GOAP Goals Index

- **Last Updated**: 2026-10-30
- **Status**: **post-`v0.1.47` hardening — closed.** `v0.1.47` shipped (tag `854f6f8d`) and the released CLI was
  dogfooded; the two defects it found are fixed (#1225 persisted pattern counts, #1226 same-domain retrieval
  ranking), alongside the fixable advisory (#1224), the generated-lockfile alert (#1227), the workspace bump
  restore (#1228) and the toolchain pin (#1222). ADR-079 stage 4 is proven (#1229, closed unmerged).
  The earlier audit-backlog wave is fully merged (see the tables below); ADR-080/081/082 lifecycle acceptance
  remains maintainer-external.
- **Workspace**: `0.1.48` · **Tag**: `v0.1.47` · **Plan**: post-`v0.1.47` hardening (`ACTIONS.md` ACT-413...ACT-419)
- **Queued chains**: none open — the watermark chain (#1067→#1068→#1089) and the embedding chain
  (#1073→#1074→#1076) are closed.
- **Archive**: `plans/archive/2026-07-consolidation/`

## Active goals — post-`v0.1.47` hardening (2026-10-30, closed)

| Goal | Rec | Priority | Status |
|------|-----|----------|--------|
| G-B1 patch the one fixable advisory on both dependency graphs | RUSTSEC-2026-0418 | P1 | ✅ #1224 (`0bc68aad`) — `libz-sys` 1.1.31 |
| G-B2 the shipped CLI must report the patterns it can list | dogfood D1 | P1 | ✅ #1225 (`0a461028`) — 0 vs 2 before, 2 == 2 after |
| G-B3 same-domain candidates must not tie on context relevance | dogfood D2 | P1 | ✅ #1226 — sub-signals scaled before the 40% cap |
| G-B4 no generated lockfile in the tree | Dependabot #35 | P2 | ✅ #1227 (`6b28c2a0`) — alert `fixed` |
| G-B5 the workspace version must be ahead of the latest tag | drift gate | P0 | ✅ #1228 (`7dd67587`) — `severity=clean`, #1223 self-closed |
| G-B6 one pinned toolchain for builds, lint and advisories | toolchain | P2 | ✅ #1222 (`16a31396`) — 1.99.1 |
| G-B7 live proof that a failing required check blocks the merge | ADR-079 stage 4 | P0 | ✅ #1229 (closed unmerged) |
| G-B8 released-CLI dogfood is mandatory, not optional | release process | P1 | ✅ `.agents/skills/release-guard/SKILL.md` + evals |

## Goals — audit-backlog wave (2026-10-04..., all merged)

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

## Wave A goals (2026-10-06...09, all merged)

| Goal | Rec | PR | Status |
|------|-----|----|--------|
| Recommendation precision (ms + UUID tie-break + backfill) | #1065 / S06 | #1142 | ✅ merged 2026-10-08 (`5054c15a`) |
| Execution-backed retrieval provenance | #1079 / R08 | #1148 | ✅ merged 2026-10-07 (`9fdbc765`) |
| Lint suppressions slice 1 (turso unsafe peel + ratchet) | #1092 / Q03 | #1143 | ✅ merged 2026-10-06 (`cdb70594`); slice 2 #1153 merged (`b6326d90`) |
| Strict row decode + allowlisted query builder | #1071 / S12 | #1145 | ✅ merged |
| Capacity eviction atomic-or-repairable | #1070 / S11 | #1150 | ✅ merged |
| Adaptive pool capacity model (E4) | #1063 / S04 | #1146 | ✅ merged |
| Scoped pool checkout | #1064 / S05 | #1147 | ✅ merged |
| Modification watermark + keyset pages | #1067 / S08 | #1149 | ✅ merged |
| Identity-scoped embedding adapter | #1073 / R02 | #1151 | ✅ merged |
| Incremental ranking index + benches | #1078 / R07 | #1154 | ✅ merged |
| Capability truth (cleanup / procedural+relationships / inventory) | #1087 / M06 | #1144, #1152, #1158 | ✅ merged |
| Nightly libtest-JSON flag fix (follow-up to #1130) | — | #1159 | ✅ merged |

## Queued goals (sequenced) — all closed as of 2026-10-30

| Goal | Rec | Outcome |
|------|-----|---------|
| Modification watermark + bounded keyset pages | #1067 / S08 | ✅ merged #1149 |
| Revision-aware merge of redb + Turso | #1068 / S09 | ✅ closed with the release evidence (2026-10-19) |
| Batch writes through redb + synchronizer | #1089 / M08 | ✅ merged #1171 (`16426d08`); issue closed |
| Identity-scoped embedding storage adapter | #1073 / R02 | ✅ merged #1151 |
| One runtime provider snapshot across memory paths (remaining pattern sites) | #1074 / R03 | ✅ merged #1160 |
| Reuse + batch pattern embeddings | #1076 / R05 | ✅ merged #1198; issue closed |
| Adaptive pool capacity model (cooldown + permits) | #1063 / S04 | ✅ merged #1146 |
| Scoped pool checkout (`with_connection`) | #1064 / S05 | ✅ merged #1147 |
| Capacity eviction atomic-or-repairable | #1070 / S11 | ✅ merged #1150 |
| Strict row decode + raw-query error surfacing | #1071 / S12 | ✅ merged #1145 |
| Incremental ranking index + benches | #1078 / R07 | ✅ merged #1154 |
| Execution-backed retrieval provenance | #1079 / R08 | ✅ merged #1148 |
| Capability truth for `StorageBackend` defaults | #1087 / M06 | ✅ merged #1144, #1152, #1158 |
| Lint-suppression integrity | #1092 / Q03 | ✅ merged #1143, #1153 |
| Ship a release to clear drift | #1137 | ✅ `v0.1.45` → `v0.1.46` → `v0.1.47` all shipped |
| crates.io trusted-publisher registrations + secret deletion | #1109 C1 | ✅ done; issue closed |

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
