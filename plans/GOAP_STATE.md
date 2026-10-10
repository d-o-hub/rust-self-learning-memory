# GOAP State Snapshot

- **Last Updated**: 2026-10-09
- **Version**: workspace `0.1.45` · latest tag `v0.1.44`
- **Branch**: main @ `5054c15a` (2026-10-08; merges through #1142)
- **Open PRs**: run `gh pr list --state open` (17 at 2026-10-09 — wave-A slices in review: #1138, #1140, #1144, #1145, #1146, #1147, #1149, #1150, #1151, #1152, #1153, #1154, #1155, #1157, #1158, #1159 + plans #1141)
- **Open issues**: run `gh issue list --state open` (16 at 2026-10-09: 1063, 1064, 1067, 1068, 1070, 1071, 1073, 1074, 1075, 1076, 1078, 1085, 1087, 1089, 1109, 1137)
- **Active plan**: audit-backlog wave [`GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`](GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md) — W1–W4 + Q02 merged; the wave-A slices are in review; chains #1067→#1068→#1089 and #1073→#1074→#1076 in flight
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md) (per-issue verdicts)
- **Release**: ✅ `v0.1.44` shipped 2026-10-02 (`4f4f4ba8`); workspace bumped to `0.1.45`; next release pending (drift issue #1137)
- **Archive**: `plans/archive/2026-07-consolidation/`

---

## Wave status — audit backlog (2026-10-09)

| Slice | Issue | PR | Status |
|-------|-------|-----|--------|
| W0 | trackers + wave doc | #1129 (`26b12fc8`) | ✅ merged 2026-10-05 |
| W1 | #1077 ranking guard across await | #1131 (`874df209`) | ✅ merged 2026-10-05 |
| W2 | #1086 sub-second MCP latency | #1135 (`75007b51`) | ✅ merged 2026-10-05 |
| W3 | #1088 Turso tag-tx rollback | #1134 (`347b3296`) | ✅ merged 2026-10-05 |
| W4 | #1066 redb recommendation index order | #1139 (`f1c31699`, `0485bb66`) | ✅ merged 2026-10-05 |
| W5 | #1075 query-aware lexical fallback | #1138 | 🔄 in review — roast MAJORs fixed (query-only tokens, context double-count), CI re-running |
| W6 | #1085 real MCP health probes + URL redaction (E1) | #1140 | 🔄 in review — roast MAJOR fixed (live retrieval-cache metrics), rebased, CI re-running |
| Q02 | #1091 ignored-test inventory + isolation | #1130 (`8a363505`) | ✅ merged 2026-10-06 — 159-entry inventory, 14 tests un-ignored, isolated fail-visible nightly job; issue closed with evidence |

### Wave A (2nd batch) in review

| Issue | PR | Notes |
|-------|----|-------|
| #1065 recommendation precision | #1142 (`5054c15a`) | ✅ merged 2026-10-08 |
| #1079 execution-backed provenance | #1148 (`9fdbc765`) | ✅ merged 2026-10-07 |
| #1092 lint suppressions (slice 1) | #1143 (`cdb70594`) | ✅ merged 2026-10-06; slice 2 #1153 in review (317→0 crate-root allows) |
| #1071 strict row decode + query builder | #1145 | 🔄 in review (doctest fixed, rebased) |
| #1070 capacity eviction | #1150 | 🔄 in review (clippy fixes pushed) |
| #1063 adaptive pool capacity | #1146 | 🔄 in review (E4 test replaced) |
| #1064 scoped pool checkout | #1147 | 🔄 in review (roast BLOCKER fixed: no nested checkout) |
| #1067 modification watermark | #1149 | 🔄 in review (6 roast MAJORs fixed: txn atomicity, monotonic revisions, durable watermark, loud default, Turso backfill) |
| #1073 identity-scoped embedding adapter | #1151 | 🔄 in review (LOC decomposition done) |
| #1078 incremental ranking index | #1154 | 🔄 in review (100k: 222 ms → 0.84 µs) |
| #1087 capability truth | #1144, #1152, #1158 | 🔄 in review (cleanup pair; procedural+relationships; inventory + checker) |
| #1074 live provider snapshot (pattern sites) | — | 🔄 in flight |
| nightly JSON flag fix (follow-up to #1130) | #1159 | 🔄 in review |

Adjacent merges: #1128 docs-integrity, #1133 local hook sensors, #1136 dependabot.

## Next queue

- **Watermark chain** (E3/E5): #1067 (#1149 in review) → #1068, #1089 (spawn after #1149 merges).
- **Embedding identity chain** (E2): #1073 (#1151 in review) → #1074 (in flight) → #1076.
- **Merge pipeline**: 17 PRs must merge serially (strict up-to-date policy re-runs required checks after every merge).
- **Release/closeout**: ship `v0.1.45` (#1137) once the wave is green; #1109 manual crates.io steps remain maintainer-external.

## Current truth flags

```text
release_current                   = false (v0.1.44 tagged 2026-10-02; drift issue #1137 open — release follows the wave)
audit_backlog_registered          = true  (#1129 — every #1063–#1092 issue has a verdict in GAP_ANALYSIS_LATEST.md)
w1_ranking_guard_released         = true  (#1131)
w2_subsecond_latency_released     = true  (#1135)
w3_turso_tag_tx_rollback          = true  (#1134)
w4_redb_rec_index_ordering        = true  (#1139 — repair pass heals write-order winners without a schema bump)
w5_lexical_fallback               = in_review (#1138 — roast-fixed)
w6_mcp_health_truthful_and_redacted = in_review (#1140 — roast-fixed)
ignored_test_inventory            = true  (#1130; nightly job fail-visible, JSON flag follow-up in #1159)
recommendation_precision          = true  (#1142 — ms ordering + UUID tie-break + backfill)
retrieval_provenance_execution_backed = true (#1148)
lint_crate_root_allows_zero       = in_review (#1143 merged slice 1; #1153 slice 2 drives crate-root allows to 0)
production_pattern_query_relevance = pending #1074 (E2 — semantic_service has no writer; snapshot seam in flight)
watermark_source_of_truth         = in_review (#1149 — revision table + keyset pages + durable watermark)
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
