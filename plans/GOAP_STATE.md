# GOAP State Snapshot

- **Last Updated**: 2026-10-30
- **Version**: workspace `0.1.48` (development bump restored in #1228) · latest tag `v0.1.47` (`854f6f8d`, GitHub Release published 2026-10-28T20:41:48Z)
- **Branch**: `main` (`e12c7101`) — no release branch; the only open PR is this tracker refresh (#1230)
- **Open PRs**: 1 (`gh pr list --state open`): #1230, this refresh. Merges are serial:
  `strict_required_status_checks_policy` puts every other branch `BEHIND` and re-queues their required runs
- **Open issues**: 0 (`gh issue list --state open`); the drift issue #1223 closed itself when 0.1.48 landed
- **Active plan**: **post-`v0.1.47` hardening** — released-CLI dogfood follow-ups (#1225, #1226), advisory #1224,
  generated-lockfile alert #1227, version restore #1228, toolchain pin #1222; ADR-079 stage 4 proven (#1229, closed)
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md) (per-issue verdicts)
- **Release**: `v0.1.47` shipped and **reached crates.io** (core, redb, turso, mcp). The released `do-memory-cli`
  was dogfooded against a persistent database (asset digest `76621912…ee58`, `--version` 0.1.47, episode
  `8c6c3261…236e`, 3 steps, 5 patterns), which is now a mandatory release-guard step.
- **Archive**: `plans/archive/2026-07-consolidation/`

---

## Wave status — audit backlog (2026-10-12)

| Slice | Issue | PR | Status |
|-------|-------|-----|--------|
| W0 | trackers + wave doc | #1129 (`26b12fc8`) | ✅ merged 2026-10-05 |
| W1 | #1077 ranking guard across await | #1131 (`874df209`) | ✅ merged 2026-10-05 |
| W2 | #1086 sub-second MCP latency | #1135 (`75007b51`) | ✅ merged 2026-10-05 |
| W3 | #1088 Turso tag-tx rollback | #1134 (`347b3296`) | ✅ merged 2026-10-05 |
| W4 | #1066 redb recommendation index order | #1139 (`f1c31699`, `0485bb66`) | ✅ merged 2026-10-05 |
| W5 | #1075 query-aware lexical fallback | #1138 | ✅ merged 2026-10-09 |
| W6 | #1085 real MCP health probes + URL redaction (E1) | #1140 | ✅ merged 2026-10-09 |
| Q02 | #1091 ignored-test inventory + isolation | #1130 (`8a363505`) | ✅ merged 2026-10-06 — 159-entry inventory, 14 tests un-ignored, isolated fail-visible nightly job; issue closed with evidence |

### Wave A (2nd batch) merged

| Issue | PR | Notes |
|-------|----|-------|
| #1065 recommendation precision | #1142 (`5054c15a`) | ✅ merged 2026-10-08 |
| #1079 execution-backed provenance | #1148 (`9fdbc765`) | ✅ merged 2026-10-07 |
| #1092 lint suppressions (slice 1) | #1143; slice 2 #1153 (`b6326d90`) | ✅ merged; crate-root allows 317 → 0 |
| #1071 strict row decode + query builder | #1145 | ✅ merged |
| #1070 capacity eviction | #1150 | ✅ merged |
| #1063 adaptive pool capacity | #1146 | ✅ merged |
| #1064 scoped pool checkout | #1147 | ✅ merged |
| #1067 modification watermark | #1149 | ✅ merged |
| #1073 identity-scoped embedding adapter | #1151 | ✅ merged |
| #1078 incremental ranking index | #1154 | ✅ merged |
| #1087 capability truth | #1144, #1152, #1158 | ✅ merged |
| #1074 live provider snapshot (pattern sites) | #1160 | ✅ merged |
| nightly JSON / fuzz toolchain fixes | #1159, #1161 | ✅ merged |

Adjacent merges: #1128 docs-integrity, #1133 local hook sensors, #1136 dependabot.

## Next queue

- **Serial merges only**: `strict_required_status_checks_policy` sends every other open branch `BEHIND`, so one
  rebase + CI cycle per merge. #1226 is the last PR in flight; it needs one rebase after #1228.
- **Next release**: `0.1.48` is a development bump, not a release. Cut it through
  `./scripts/release-manager.sh ship --execute` once `main` is green and the CHANGELOG has the release heading;
  the released-CLI dogfood is now a mandatory post-ship step (`.agents/skills/release-guard/SKILL.md`).
- **Dependabot (ACT-404)**: still upstream-blocked — `rustls-webpki 0.102.8` only via
  `libsql → hyper-rustls 0.25 → rustls 0.22.4`, `lru <0.16.3` via `augurs-changepoint`/`agentfs-sdk` (both newest),
  `libsql-sqlite3-parser` with no patch. The one fixable advisory (RUSTSEC-2026-0418, `libz-sys`) is cleared by #1224.
- **Watermark / embedding follow-ups**: #1068, #1089 and #1076 closed on 2026-10-19/20 with the release evidence.
- **#1109 external steps**: ✅ done (crates.io environment restricted, registry secret deleted, issue closed).

## Current truth flags

```text
release_current                   = true  (v0.1.47 tagged 2026-10-28 on 854f6f8d; workspace bumped to 0.1.48 in #1228)
crates_io_publish_truthful        = true  (0.1.46 and 0.1.47 both published for core/redb/turso/mcp; the #1179 guard fix held twice)
nightly_turso_green               = repaired in #1180 (that run: 86 + 20 = 106, 0 failures, nothing ignored); no fresh
                                    scheduled run was re-verified in this refresh — consult the latest nightly run
audit_backlog_registered          = true  (#1129 — every #1063–#1092 issue has a verdict in GAP_ANALYSIS_LATEST.md)
w1_ranking_guard_released         = true  (#1131)
w2_subsecond_latency_released     = true  (#1135)
w3_turso_tag_tx_rollback          = true  (#1134)
w4_redb_rec_index_ordering        = true  (#1139 — repair pass heals write-order winners without a schema bump)
w5_lexical_fallback               = true (#1138)
w6_mcp_health_truthful_and_redacted = true (#1140)
ignored_test_inventory            = true  (#1130; nightly job fail-visible, JSON flag follow-up in #1159)
recommendation_precision          = true  (#1142 — ms ordering + UUID tie-break + backfill)
retrieval_provenance_execution_backed = true (#1148)
lint_crate_root_allows_zero       = true (#1143/#1153)
production_pattern_query_relevance = true (#1160 — runtime provider snapshot across pattern paths)
watermark_source_of_truth         = true (#1149 — revision table + keyset pages + durable watermark)
storage_stats_pattern_count_truthful = true (#1225 — counts backends via get_all_patterns; 2 == 2 across processes)
context_subsignal_weights_preserved  = true (#1226 — scaled by the 0.4 budget before the cap; 5/5 Rust on top)
adr079_stage4_merge_block_proven  = true  (#1229 probe: CI / Required failed → mergeStateStatus BLOCKED, merge-pr exit 1)
required_ci_causal                = true  (ruleset 9591004 requires Codacy + `CI / Required`; same-run aggregate)
release_dispatch_truthful         = true  (#1179 guard + #1223 self-close + two consecutive published releases)
```

## Closed campaigns (pointer)

| Campaign | Result |
|----------|--------|
| Release-pipeline hardening #1109 C1–C7 | ✅ code-side merged (#1121, #1123, #1125); manual crates.io registrations + secret deletion remain maintainer-external |
| ADR-079/CIT/PTA/RAT CI-trust + attribution | ✅ code-side closed (#947, 2026-08-12); ADR-079 **stage 4 proven 2026-10-30** (probe #1229: failing `CI / Required` → `mergeStateStatus=BLOCKED`, `merge-pr.sh` exit 1); ADR-080/081/082 lifecycle = maintainer-external |
| Retrieval judgment + rerank | ✅ #1041, #1042 (2026-09-24) |
| Feedback-to-ranking adaptation (ADR-082) | ✅ #952 (2026-08-13); lifecycle `Proposed` |
| v0.1.44 ship + post-bump | ✅ 2026-10-02 |
| PR queue cleanup (GOAP swarm) | ✅ 2026-07-27 |
| v0.1.36 ship + post-bump | ✅ 2026-07-22…23 |

Details: `plans/archive/2026-07-consolidation/completed-sprints/`.
