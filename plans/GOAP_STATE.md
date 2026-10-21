# GOAP State Snapshot

- **Last Updated**: 2026-10-20
- **Version**: workspace `0.1.47` (post-v0.1.46 development bump, merged in #1193) · latest tag `v0.1.46` (`d85efb5c`, GitHub Release published 2026-10-18)
- **Branch**: `main` (`7411a336`) — no release branch; the `0.1.47` release-prep PR is the next intentional PR
- **Open PRs**: 1 open (run `gh pr list --state open`) — #1204 (release-gate CI event filtering, ADR-083).
  #1194 (LESSON-068…076) merged 2026-10-20 as `24a90054`. Merges are serial:
  `strict_required_status_checks_policy` puts every other branch `BEHIND` and re-queues their required runs
- **Open issues**: 4 open (run `gh issue list --state open`: #1206, #1205, #1200, #1109)
- **Active plan**: **release closeout** — [`GOAP_RELEASE_CLOSEOUT_2026-10-16.md`](GOAP_RELEASE_CLOSEOUT_2026-10-16.md)
  (publish stall, red nightly, drift gate); ACT-405 (the release-gate event decision) is the last open action
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md) (per-issue verdicts)
- **Release**: `v0.1.46` shipped and **reached crates.io** (all four registered crates verified by version
  membership, `http=200`) — the first release off the `0.1.34`/`0.1.31` stall, proving the #1179 guard fix
  externally. The next tag is `v0.1.47`, gated on a release-prep PR.
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

- **Merge pipeline (serial, forced by strict up-to-date + drift gate)**: this plans + `0.1.46` bump PR **first**
  (it makes `check-release-drift.sh` clean instead of overriding it with `release-preparation`) → #1179 publish
  guard → #1177 readiness gate → #1175 guardrail docs → #1180 nightly Turso → tag `v0.1.46`, which is the publish
  vehicle because the chain runs from the tag SHA.
- **Publish repair (ACT-402)**: `v0.1.44`/`v0.1.45` never reached crates.io; `0.1.46` must be verified with
  `index("0.1.46") != null` per crate **and** a green `Verify publish landed`.
- **Nightly repair (ACT-403)**: #1180 turns the 36-failure nightly green and exposes four real defects
  (metrics column/unit drift, multi-dimension read gap, pooled-checkout misuse, error-swallowing asserts).
- **Dependabot (ACT-404)**: 12 alerts open, but they are **6 advisories duplicated across `Cargo.lock` and
  `fuzz/Cargo.lock`** and all are upstream-blocked: 8 × `rustls-webpki 0.102.8` (2 high) reached only through
  `libsql → hyper-rustls 0.25 → rustls 0.22.4`, 2 × `lru <0.16.3` through `augurs-changepoint 0.10.2` and
  `agentfs-sdk 0.6.4` (both already newest), 2 × `libsql-sqlite3-parser` with no upstream patch. `libsql 0.10.0`
  changes none of this (verified with a scratch `cargo tree`), so no bump PR is planned until upstream moves.
- **Watermark follow-ups**: #1068, #1089 remain queued after the merged #1149 implementation.
- **Embedding follow-up**: #1076 remains queued after merged #1151/#1160 implementations.
- **Release/closeout**: `v0.1.45` shipped; #1109 manual crates.io registrations + secret deletion remain
  maintainer-external.

## Current truth flags

```text
release_current                   = true  (v0.1.45 tagged 2026-10-16 on 8ee711bd; workspace bumped to 0.1.46 here)
crates_io_publish_truthful        = false (guard `!inputs.dry-run` string-coerced to true → uploads skipped since
                                           493cfb01; newest published 0.1.34/mcp 0.1.31 — fixed in #1179, lands on 0.1.46)
nightly_turso_green               = false (36 of 106 failing at 8ee711bd; repaired locally in #1180, awaiting CI)
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
release_dispatch_truthful         = partial (#1123/#1125 made the chain OIDC-only + docs, but the publish step
                                           itself was false-green until #1179; see crates_io_publish_truthful)
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
