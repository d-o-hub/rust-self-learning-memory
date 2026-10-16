# Project Status — Self-Learning Memory System

**Last Updated**: 2026-10-14
**Released Version**: v0.1.45 (release candidate; latest tag v0.1.44)
**Workspace Version**: 0.1.45 (post-v0.1.44 bump)
**Edition**: Rust 2024
**Active plan**: **release closeout for the audit-backlog wave** (`GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`) — W1–W6, Q02, and wave-A slices are merged through #1161; the last open PR, #1153, merged as `b6326d90` on 2026-10-12. Follow-up issues #1068, #1089, and #1076 remain queued after the release.
**Branch**: `chore/release-v0.1.45` (release candidate based on `origin/main` @ `bdd3ff07`)

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | none open (run `gh pr list --state open`) — release preparation is the next intentional PR |
| Open issues | 8 open (run `gh issue list --state open`) — per-issue verdicts, evidence and queue position live in `STATUS/GAP_ANALYSIS_LATEST.md` |

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

## Recently merged (2026-10-09…12)

| Slice | PR | State |
|-------|-----|-------|
| W5 #1075 query-aware lexical fallback | #1138 | ✅ merged |
| W6 #1085 health probes + URL redaction (E1) | #1140 | ✅ merged |
| #1067 / #1064 / #1070 / #1063 / #1071 | #1149, #1147, #1150, #1146, #1145 | ✅ merged |
| #1073 / #1078 / #1074 | #1151, #1154, #1160 | ✅ merged |
| #1087 capability truth | #1144, #1152, #1158 | ✅ merged |
| #1092 lint slice 2 | #1153 | ✅ merged as `b6326d90`; crate-root allows 317 → 0 |
| Nightly JSON / fuzz toolchain fixes | #1159, #1161 | ✅ merged |
| Release-plan refresh | #1141 | ✅ merged |

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
| Repo-wide open gaps | **7 open issues**: follow-up storage/retrieval work, release drift, manual release-pipeline steps, and current docs work — register: `STATUS/GAP_ANALYSIS_LATEST.md` |
| Merged since 2026-10-06 | #1130, #1142, #1143, #1148, #1138, #1140, #1144, #1145, #1146, #1147, #1149, #1150, #1151, #1152, #1154, #1158, #1159, #1160, #1161, #1153 |
| Production pattern query relevance | ✅ **#1074 merged in #1160** — pattern operations use one runtime provider snapshot |
| `episodes.created_at` watermark | ✅ **#1067 merged in #1149** — durable modification watermark and bounded keyset pages |
| Release | v0.1.45 candidate prepared; latest tag remains v0.1.44 until the release workflow completes |
| Code execution | Fail-closed; S1.1c Wasmtime/WASI **NO-GO** |
| Serialization | Postcard required |
| Skill evals / routes | 40/40 |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Merge the release-preparation PR with green CI and current version docs | release | 🔄 |
| P0 | Ship `v0.1.45` through `release-manager.sh ship --execute` | release | ⏳ |
| P2 | Follow-up watermark sync improvements (#1068, #1089) | S09/M08 | ⏳ post-release |
| P2 | Follow-up embedding reuse and batching (#1076) | R05 | ⏳ post-release |
| P0 | ADR-079 stage 4 live fault-injection proof; ADR-080/081 lifecycle; #1109 crates.io registrations + secret deletion | maintainer | ⏸ external |
| P2 | Research/product spikes (R-F1…R-F7, R-F10) | R-F* | ⏸ DEFER |

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Validation: `plans/STATUS/VALIDATION_LATEST.md`
- Archive: `plans/archive/2026-07-consolidation/`
