# Project Status — Self-Learning Memory System

**Last Updated**: 2026-11-04
**Released Version**: v0.1.48 (release target; tag and publication pending green main CI)
**Workspace Version**: 0.1.48 (release preparation; latest published tag is v0.1.47, `854f6f8d`, 2026-10-28)
**Edition**: Rust 2024 (toolchain pinned `1.99.1`)
**Active plan**: **release preparation for `0.1.48`** — `v0.1.47` shipped and its released CLI was dogfooded on a persistent database; the observed `storage stats` and retrieval-ranking defects are fixed in #1225/#1226,
the advisory fix (#1224), generated-lockfile alert (#1227), version restore (#1228), toolchain pin (#1222), and crates.io publishing settings (#1109) are complete, and the harness wave (#1241, #1242, #1244, #1248, #1250, #1251, #1249) is merged.
Remaining: land this preparation, verify green main CI, ship `v0.1.48`, then verify the GitHub Release, the released-CLI dogfood, and all four crates the publish chain owns. Upstream-blocked Dependabot ACT-404 and maintainer-external ADR lifecycle acceptance stay open.
**Branch**: release preparation based on `main` @ `2426638a`.

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | The release-preparation PR is the only open pull request; `gh pr list --state open` is the live source. Merged since `v0.1.47`: #1220, #1222, #1232/#1233, #1237, #1238, #1239, #1240, #1241, #1242, #1244, #1245, #1248, #1250, #1251, #1249. |
| Open issues | #1236, the auto-maintained release-drift issue (32 commits / 6 days since `v0.1.47`); `check-release-blockers.sh` excludes it by label so the gate cannot deadlock on its own bookkeeping, and it closes when this version ships. `gh issue list --state open` is the live source. |
| Dependency audit | The remaining Dependabot alert rows are upstream-blocked (ACT-404); consult the security workflow's latest run for current advisory status. |

## Corrected claim (2026-10-04)

The "P0 plan gaps: **0 open code-side**" line once below this section described the **CI-trust / attribution** campaign (ADR-079,
CIT-A1…A5, PTA, RAT) and is still true for that campaign. It was never true of the repository as a whole: the
22-issue audit backlog was open the entire time and absent from the trackers, which is how "0 open gaps" got
restated at each refresh. Read any campaign claim as campaign-scoped; the repo-wide register is
`STATUS/GAP_ANALYSIS_LATEST.md`.

## Recent completed (2026-10-30 — v0.1.47 dogfood follow-ups + toolchain pin)

| Slice | PR | Evidence |
|-------|-----|----------|
| RUSTSEC-2026-0418 (`libz-sys` build-script env leak) | #1224 (`0bc68aad`) | `libz-sys` 1.1.28→1.1.31 in `Cargo.lock`, 1.1.29→1.1.31 in `fuzz/Cargo.lock`; CI 32 pass; local `quality-gates.sh` passed |
| `storage stats` counted only the in-memory fallback | #1225 (`0a461028`) | released 0.1.47 CLI: `patterns.total_count=0` vs `pattern list=2` on the same DB; patched: `2 == 2`; regression test reopens the redb file in a second process |
| Same-domain context match saturated the 40% budget | #1226 (`e12c7101`) | pre-fix `matching language (0.4)` vs `nonmatching (0.4)`; post-fix all five Rust episodes rank ahead of same-domain Python/TypeScript candidates |
| Generated `.opencode/package-lock.json` tracked | #1227 (`6b28c2a0`) | Dependabot alert #35 (`zod`, high) state `fixed` after untracking + ignoring the file |
| Workspace reverted to the released 0.1.47 by #1220 | #1228 (`7dd67587`) | `cargo metadata --locked` → 0.1.48 for all 9 workspace packages and the 4 path deps in `fuzz/Cargo.lock`; drift `severity=clean`; issue #1223 closed |
| Rust toolchain floated on `stable` | #1222 (`16a31396`) | `rust-toolchain.toml` + `setup-rust` default = `1.99.1`; `cargo check -p do-memory-core --all-targets` on 1.99.1 passed; `release-preparation` label removed |
| ADR-079 stage 4 merge-block proof | #1229 (closed unmerged) | injected frontmatter fault → `Quick PR Check` + `CI / Required` failed, `mergeStateStatus=BLOCKED` with `mergeable=MERGEABLE`, `merge-pr.sh` exit 1 |

## Recent completed (2026-11-01 — CLI output contract follow-up)

| Slice | PR | Evidence |
|-------|-----|----------|
| Episode-create output contract | #1232/#1233 | Merged on 2026-11-01; output and test contract unified across feature configurations. See `VALIDATION_LATEST.md` for check and smoke evidence. |

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

## Recent completed (2026-10-13…16 — v0.1.45 ship + closeout)

| Slice | PR / commit | State |
|-------|-------------|-------|
| v0.1.45 release | `8ee711bd` → tag `v0.1.45` | ✅ shipped via `release-manager.sh ship --execute` (never `gh release create`) |
| rustls + quinn-proto advisories | #1174 (`2d5ed2b0`) | ✅ merged — cleared the 2 high + 2 moderate alerts and unblocked `main` |
| Release docs + version bump | #1169 | ✅ merged as `8ee711bd` |
| crates.io publish false-green | #1179 | ✅ merged `01d1bd7e` **and verified externally at `v0.1.46`** — the string-coerced `!inputs.dry-run` guard had skipped every real upload since `493cfb01` (v0.1.44/v0.1.45 never published); `v0.1.46` published all four registered crates, each confirmed by crates.io version-membership over HTTP 200 |
| Readiness gate run ordering | #1177 | 🔄 in review — count the latest check run per name (superseded runs were reported as current) |
| Guardrail docs (advisory/ruleset/drift traps) | #1175 | 🔄 in review — owns `AGENTS.md` + `agent_docs/LESSONS.md` |
| Turso nightly: 36 failures → 4 real defects | #1180 | 🔄 in review — metrics column/unit drift, multi-dimension read gap, pooled-checkout misuse, error-swallowing asserts; `required-features` + feature-specific nightly step |

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
| Repo-wide open issues | the authoritative count + list is the **Open issues** row under *Open tracker (live)* — keep it in that one place; this row used to carry a stale copy of it. Note on **#1182**: the canonical `release-drift` issue, filed and updated **in place** by the `maintain-issue` job at `release-drift.yml:98-149` — it closed itself at 2026-10-18 11:14:15Z once `v0.1.46` was tagged, which is the designed exit (`version_not_advanced` clears when the tag matches), never by hand |
| Merged since 2026-10-06 | #1130, #1142, #1143, #1148, #1138, #1140, #1144, #1145, #1146, #1147, #1149, #1150, #1151, #1152, #1154, #1158, #1159, #1160, #1161, #1153, #1174, #1172, #1173, #1175, #1176, #1177, #1179, #1180, #1181, #1185, #1186, #1187, #1193, #1194, #1201, #1203, #1204, #1207, #1220, #1222, #1224, #1225, #1226, #1227, #1228, #1230, #1231, #1232, #1233 |
| Production pattern query relevance | ✅ **#1074 merged in #1160** — pattern operations use one runtime provider snapshot |
| `episodes.created_at` watermark | ✅ **#1067 merged in #1149** — durable modification watermark and bounded keyset pages |
| Release | `v0.1.47` tagged `854f6f8d` (2026-10-28), GitHub Release published 2026-10-28T20:41:48Z; workspace is `0.1.48` |
| crates.io | ✅ **`0.1.47` published** — `do-memory-core`, `do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp`; the publish chain has now landed two consecutive releases (`0.1.46`, `0.1.47`) since the #1179 guard fix. `do-memory-cli` / `do-memory-examples` are still not registered (404) |
| Released-CLI dogfood (v0.1.47) | ✅ mandatory post-release step now codified in `.agents/skills/release-guard/SKILL.md` — asset digest matched (`76621912…ee58`), `--version` = 0.1.47, episode `8c6c3261…236e` completed with 3 steps and 5 learned patterns; it exposed the two defects fixed in #1225/#1226 |
| Nightly (Turso native) | ✅ Scheduled run #41288205136 (2026-11-01): Full Test Suite, Slow Tests, Isolated Turso Native Tests and Nightly Summary succeeded; isolated dimension suite 20/20 passed, 0 skipped |
| Code execution | Fail-closed; S1.1c Wasmtime/WASI **NO-GO** |
| Serialization | Postcard required |
| Skill evals / routes | 40/40 |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Land the **release-prep PR** for `0.1.48` and verify green main CI | ACT-412 | ⏳ this PR; `v0.1.47` preparation and ship are recorded below as done |
| P0 | Then cut tag `v0.1.48` and verify the four owned crates resolve on crates.io | ACT-409 | ⏳ `v0.1.47` shipped at tag `854f6f8d` with all four published and the released-CLI dogfood performed |
| P1 | Dependabot register is **upstream-blocked, not a bump away**: 18 open alerts across 9 advisory IDs and two lockfiles; see `ACTIONS.md` / `GAP_ANALYSIS_LATEST.md` for pinned chains | ACT-404 | ⏸ upstream; alert #35 is fixed, and RUSTSEC-2026-0418 (`libz-sys`) was cleared by #1224 |
| P1 | Sensors so this class of bug fails locally: schema-vs-SQL column check; LESSON + AGENTS trap entries for `required-features` vs `cfg_attr(..., ignore)` and error-swallowing asserts | ACT-406, ACT-407 | ✅ schema/SQL sensor landed in #1201; LESSON/AGENTS entries in #1203 |
| P2 | Decide whether scheduled/tag nightly runs gate releases (`do_ci_check` `.event` filtering) — hardening only, never weakened at ship time | ACT-405 | ✅ decision + ADR-083 — only `push`/`pull_request` gate; a tag-triggered nightly (a `push`) still gates |
| P2 | Strip the auto-applied `release-preparation` label from non-release PRs; the drift gate is clean again, so no label is needed | ACT-411 | ✅ removed from #1194, #1204 never carried it, and #1222 was stripped before it merged |
| P0 | ADR-079 stage 4 live fault-injection proof | maintainer | ✅ performed 2026-10-30 on probe PR #1229 (closed unmerged) — see ADR-079 acceptance criteria |
| P0 | ADR-080/081 lifecycle acceptance | maintainer | ⏸ external |
| P2 | Research/product spikes (R-F1...R-F7, R-F10) | R-F* | ⏸ DEFER |

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Active closeout: `plans/GOAP_RELEASE_CLOSEOUT_2026-10-16.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Validation: `plans/STATUS/VALIDATION_LATEST.md`
- Archive: `plans/archive/2026-07-consolidation/`
