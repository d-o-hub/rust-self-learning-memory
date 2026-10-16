# GOAP State Snapshot

- **Last Updated**: 2026-10-14
- **Version**: workspace `0.1.45` · latest tag `v0.1.44`
- **Branch**: `chore/release-v0.1.45` (release candidate based on `origin/main` @ `bdd3ff07`)
- **Open PRs**: none open (run `gh pr list --state open`) — release preparation is the next intentional PR
- **Open issues**: 8 open (run `gh issue list --state open`: #1068, #1076, #1089, #1109, #1137, #1164, #1165, #1168)
- **Active plan**: release closeout for [`GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`](GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md) — all currently open PRs are merged; #1068, #1089, and #1076 remain post-release follow-ups
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md) (per-issue verdicts)
- **Release**: `v0.1.45` candidate prepared; latest tag remains `v0.1.44`; ship after the release-preparation PR merges (drift issues #1137/#1165)
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

- **Watermark follow-ups**: #1068, #1089 remain queued after the merged #1149 implementation.
- **Embedding follow-up**: #1076 remains queued after merged #1151/#1160 implementations.
- **Merge pipeline**: empty; #1153 was the final open PR and merged serially through the readiness gate.
- **Release/closeout**: merge the release-preparation PR, then ship `v0.1.45`; #1109 manual crates.io steps remain maintainer-external.

## Current truth flags

```text
release_current                   = false (v0.1.44 tagged 2026-10-02; v0.1.45 candidate ready — ship after release PR)
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
required_ci_causal                = true  (ruleset 9591004 requires Codacy + `CI / Required`; same-run aggregate)
release_dispatch_truthful         = true  (#1123/#1125 — OIDC-only publish chain, verify-a-release docs)
```

## Closed campaigns (pointer)

| Campaign | Result |
|----------|--------|
| Release-pipeline hardening #1109 C1–C7 | ✅ code-side merged (#1121, #1123, #1125); manual crates.io registrations + secret deletion remain maintainer-external |
| ADR-079/CIT/PTA/RAT CI-trust + attribution | ✅ code-side closed (#947, 2026-08-12); ADR-079 stage 4 fault-inject proof + ADR-080/081/082 lifecycle = maintainer-external |
| Retrieval judgment + rerank | ✅ #1041, #1042 (2026-09-24) |
| Feedback-to-ranking adaptation (ADR-082) | ✅ #952 (2026-08-13); lifecycle `Proposed` |
| v0.1.44 ship + post-bump | ✅ 2026-10-02 |
| PR queue cleanup (GOAP swarm) | ✅ 2026-07-27 |
| v0.1.36 ship + post-bump | ✅ 2026-07-22…23 |

Details: `plans/archive/2026-07-consolidation/completed-sprints/`.
