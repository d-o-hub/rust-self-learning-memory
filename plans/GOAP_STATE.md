# GOAP State Snapshot

- **Last Updated**: 2026-10-06
- **Version**: workspace `0.1.45` · latest tag `v0.1.44`
- **Branch**: main @ `0485bb66` (2026-10-05)
- **Open PRs**: run `gh pr list --state open` (3 at 2026-10-06: #1138 W5, #1140 W6, #1130 Q02)
- **Open issues**: run `gh issue list --state open` (21 at 2026-10-06 — audit backlog minus 4 closed, + #1109 code-complete/manual, + #1137 release drift)
- **Active plan**: audit-backlog wave [`GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`](GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md) — W1–W4 merged 2026-10-05; W5/W6 in review; chains ACT-376…385 queued
- **Register**: [`STATUS/GAP_ANALYSIS_LATEST.md`](STATUS/GAP_ANALYSIS_LATEST.md) (per-issue verdicts)
- **Release**: ✅ `v0.1.44` shipped 2026-10-02 (`4f4f4ba8`); workspace bumped to `0.1.45`; next release pending (drift issue #1137)
- **Archive**: `plans/archive/2026-07-consolidation/`

---

## Wave status — audit backlog (2026-10-05/06)

| Slice | Issue | PR | Status |
|-------|-------|-----|--------|
| W0 | trackers + wave doc | #1129 (`26b12fc8`) | ✅ merged 2026-10-05 |
| W1 | #1077 ranking guard across await | #1131 (`874df209`) | ✅ merged 2026-10-05 |
| W2 | #1086 sub-second MCP latency | #1135 (`75007b51`) | ✅ merged 2026-10-05 |
| W3 | #1088 Turso tag-tx rollback | #1134 (`347b3296`) | ✅ merged 2026-10-05 |
| W4 | #1066 redb recommendation index order | #1139 (`f1c31699`, `0485bb66`) | ✅ merged 2026-10-05 |
| W5 | #1075 query-aware lexical fallback | #1138 | 🔄 in review — required checks green; cancelled workflows re-running |
| W6 | #1085 real MCP health probes + URL redaction (E1) | #1140 | 🔄 in review — cancelled workflows re-running; aggregate failed closed on the cancelled set |
| Q02 | #1091 ignored-test inventory + isolation | #1130 (jules) | 🔄 rescued: the PR tip had reverted W1–W4; a clean branch re-applies only the real work and removes the `continue-on-error`/`|| true` silent-pass, then force-push |

Adjacent merges in the same window: #1128 docs-integrity fix, #1133 local hook sensors, #1136 dependabot actions bump.

## Next queue (ACT-376…ACT-385)

- **Watermark chain** (E3/E5): #1067 → #1068 → #1089.
- **Embedding identity chain** (E2): #1073 → #1074 (remaining 5 pattern sites) → #1076.
- **Independent**: #1063 (E4 — fix rewrites `adaptive_tests.rs:140-146`), #1064, #1070, #1071, #1078, #1079.
- **Splits**: #1087 (4 PRs), #1092 (4 PRs), #1091 remainder (2 PRs after Q02).
- **Release/closeout**: ship `v0.1.45` (#1137) once the wave is green; #1109 manual crates.io steps remain maintainer-external.

## Current truth flags (campaign-scoped)

```text
release_current                   = false (v0.1.44 tagged 2026-10-02; drift issue #1137 open — next release follows the wave)
audit_backlog_registered          = true  (#1129 — every #1063–#1092 issue has a verdict in GAP_ANALYSIS_LATEST.md)
w1_ranking_guard_released         = true  (#1131)
w2_subsecond_latency_released     = true  (#1135)
w3_turso_tag_tx_rollback          = true  (#1134)
w4_redb_rec_index_ordering        = true  (#1139 — repair pass heals write-order winners without a schema bump)
w5_lexical_fallback               = in_review (#1138)
w6_mcp_health_truthful_and_redacted = in_review (#1140)
ignored_test_inventory            = in_review (#1130 — 159 entries, script-validated, isolated nightly job made fail-visible)
production_pattern_query_relevance = false until #1074 lands (E2 — semantic_service has no writer; both fallbacks constant)
episodes_created_at_is_watermark  = false until #1067 lands (E3 — absent from the Turso INSERT column list)
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
