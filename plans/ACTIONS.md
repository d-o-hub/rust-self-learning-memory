# GOAP Actions Backlog

- **Last Updated**: 2026-10-17
- **Active plan**: **release closeout** — `GOAP_RELEASE_CLOSEOUT_2026-10-16.md` (ACT-399…ACT-412). `v0.1.45` shipped
  but published nothing to crates.io, and the Turso nightly has been red since 2026-10-13. The queue is serial and
  starts with the closeout PR, which carries the `0.1.46` bump (ACT-402 → #1179 → #1177 → #1175 → #1180 → tag `v0.1.46`).
- **Previous plan**: **audit-backlog wave** — `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`; W1–W6, Q02 and the wave-A
  slices all merged (ACT-370…ACT-398 closed), including #1153 as `b6326d90`.
- **Earlier plan**: release pipeline hardening (#1109) — **code complete**: C1/C2 (#1123), C3/C4 (#1121), C5–C7 (#1125).
  Remaining manual steps for crates.io: four trusted-publisher registrations, `CARGO_REGISTRY_TOKEN` deletion, and
  restricting the `crates.io` environment to `v*` tags. See ADR-078 amendment + LESSON-030…034.
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions (2026-10-04 — audit-backlog wave)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-370 | Drop the ranking read guard before `.await` in `recommend_patterns_for_task`; clone the snapshot instead of holding `ranking_index.read()` across `get_all_patterns()` and the recommendation pipeline; contention test | #1077 (R06) | ✅ merged #1131 (`874df209`) |
| ACT-371 | Measure MCP request latency from a monotonic `Instant` (`as_millis()`), keeping epoch seconds only for event timestamps; sub-ms regression test | #1086 (M05) | ✅ merged #1135 (`75007b51`) |
| ACT-372 | Roll back the Turso tag transaction on every failure path; failure-injection test proving the prior tag set survives | #1088 (M07) | ✅ merged #1134 (`347b3296`) |
| ACT-373 | Compare `(timestamp, session_id)` inside the same redb transaction before overwriting the episode→session index; repair pass heals stale write-order winners on open | #1066 (S07) | ✅ merged #1139 (`f1c31699`, `0485bb66`) |
| ACT-374 | Replace the constant `0.5` in `calculate_keyword_similarity` with a bounded deterministic query-aware lexical score; matching-vs-unrelated ordering tests | #1075 (R04) | 🔄 in review #1138 — roast MAJORs fixed (raw-query tokens only) |
| ACT-375 | Real MCP health probes: configured handles, bounded per-backend `health_check`, live cache/sync/uptime, **redact `TURSO_DATABASE_URL`/paths/raw errors** (E1) | #1085 (M04) | 🔄 in review #1140 — roast MAJOR fixed (retrieval-cache metrics) |
| ACT-383a | Ignored-test inventory + un-ignore the pure protocol/token security tests + isolated fail-visible nightly Turso job + ADR-027 truth | #1091 (Q02) | ✅ merged #1130 (2026-10-06); nightly JSON-flag follow-up #1159 in review |

## Wave A actions (2026-10-06…09, in review)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-386 | Recommendation precision: ms ordering + UUID tie-break + legacy backfill + ranked index | #1065 (S06) | ✅ merged #1142 (`5054c15a`) |
| ACT-387 | Execution-backed retrieval provenance (one lookup, real pre-truncation count) | #1079 (R08) | ✅ merged #1148 (`9fdbc765`) |
| ACT-388 | Lint slice 1: turso denies `unsafe_code` + crate-root allow ratchet | #1092 (Q03) | ✅ merged #1143 (`cdb70594`) |
| ACT-389 | Lint slice 2: crate-root allows → expects with reasons (317 → 0) | #1092 (Q03) | 🔄 in review #1153 |
| ACT-390 | Strict row decode + allowlisted query builder | #1071 (S12) | 🔄 in review #1145 |
| ACT-391 | Capacity eviction: durable outbox + single transaction + partial outcome | #1070 (S11) | 🔄 in review #1150 |
| ACT-392 | Adaptive pool capacity model + monotonic cooldown (E4) | #1063 (S04) | 🔄 in review #1146 |
| ACT-393 | Scoped pool checkout (`with_connection`) + call-site migration | #1064 (S05) | 🔄 in review #1147 |
| ACT-394 | Watermark chain: revision table, keyset pages, durable watermark, Turso backfill | #1067 (S08) | 🔄 in review #1149 |
| ACT-395 | Identity-scoped embedding adapter + truthful ephemeral mode | #1073 (R02) | 🔄 in review #1151 |
| ACT-396 | Incremental ranking index + Criterion benches | #1078 (R07) | 🔄 in review #1154 |
| ACT-397 | Capability truth: cleanup pair / procedural+relationships / inventory+lint | #1087 (M06) | 🔄 in review #1144, #1152, #1158 |
| ACT-398 | Nightly libtest-JSON flag fix | #1091 follow-up | 🔄 in review #1159 |

## Active actions (2026-10-16 — release closeout)

Full context, evidence and acceptance criteria: [`GOAP_RELEASE_CLOSEOUT_2026-10-16.md`](GOAP_RELEASE_CLOSEOUT_2026-10-16.md).

| ID | Action | Rec | Depends on | Status |
|----|--------|-----|-----------|--------|
| ACT-399 | Merge #1179: publish guard `inputs.dry-run != 'true'`, in-job `Verify publish landed` crates.io poll, membership-shaped version check, gate-contract + publish-fixture sensors | #1109 (release) | ACT-402 | 🔄 in review `c36a874d` (main `791c01a3` merged 13:15Z) |
| ACT-400 | Merge #1177: readiness gate counts the latest check run per name so superseded failures stop shadowing current ones | CI trust | after ACT-399 | 🔄 in review `826f5c7f` (main `791c01a3` merged) |
| ACT-401 | Merge #1175 → #1180 **serially** (strict up-to-date invalidates the others on every merge) | guardrails/tests | after ACT-400 | 🔄 #1175 `7d8dac23`, #1180 `34774f4d` — heads move with every `main` merge, re-read with `gh pr view <n> --json headRefOid` |
| ACT-402 | **`0.1.46` bump, landed in the closeout PR**: `Cargo.toml` + `Cargo.lock` + version references across the active plan set (template `b3d1b944`; the CHANGELOG heading moves at release time, not here). This is what makes the drift gate clean instead of overriding it | release | — | 🔄 in review — acceptance: `check-release-drift.sh` → `severity=clean` |
| ACT-403 | Next nightly green (schedule + dispatch) with the new `turso_multi_dimension` step reporting 20/20 | #1091 (Q02) | after #1180 merges | ⏳ |
| ACT-404 | Dependabot register is **upstream-blocked**: 12 alerts = 6 advisories × 2 lockfiles (`Cargo.lock`, `fuzz/Cargo.lock`) — 8 `rustls-webpki 0.102.8` (2 high, via `libsql → hyper-rustls 0.25 → rustls 0.22.4`), 2 `lru <0.16.3` (via `augurs-changepoint 0.10.2` + `agentfs-sdk 0.6.4`, both newest), 2 `libsql-sqlite3-parser` (no patch). Do **not** open the `libsql 0.10.0` bump PR: 0.10.0 pulls the same chain (verified by `cargo tree` in a scratch crate) | Dependabot | — | ⏳ blocked upstream; re-test when libsql moves to hyper 1.x |
| ACT-410 | Pin a dated `stable` for the advisory/lint gates so a floating toolchain cannot break compilation or invent deny lints between releases | toolchain | — (local hygiene) | ⏳ |
| ACT-411 | Strip the auto-applied `release-preparation` label from #1171/#1172/#1175/#1176/#1177/#1179/#1180 once the bump is on `main`; review the parked #1171/#1172/#1176/#1178 on their own merits (tip-vs-merge-base diff first) | release drift | after ACT-402 | ⏳ |
| ACT-405 | Decide whether scheduled/tag nightly runs gate releases (`do_ci_check` `.event` filtering). Hardening only — never weaken the gate at ship time | release | after ACT-403 | ✅ decision + ADR-083 — `schedule`/`workflow_dispatch` are advisory; `push`/`pull_request` gate (a tag-triggered nightly is a `push` and still gates) |
| ACT-406 | New `validate-schema-sql.sh` sensor: columns referenced in hand-written SQL vs the DDL constants, wired next to `validate-gate-contract.sh` in `skill-evals.yml` | Turso (S03) | after #1180 merges | ⏳ |
| ACT-407 | LESSON + `AGENTS.md` trap entries: `required-features` vs `cfg_attr(…, ignore)` under `--run-ignored only`; asserts must print the `Err`; a step with `conclusion=skipped` in a green job is not evidence | Q02/Q03 | after #1175 merges (it owns both files) | ⏳ |
| ACT-409 | **Cut tag `v0.1.46`** once ACT-399…ACT-401 **and ACT-412** land — the first tag that can publish, because the chain runs from the tag SHA | release | after ACT-399, ACT-401, ACT-412 | ⏳ acceptance: `index("0.1.46") != null` for core/redb/turso/mcp **and** `Verify publish landed` green |
| ACT-412 | **Release-prep PR before the tag**: `CHANGELOG.md` `## [0.1.46]` section + the `Released Version`/version-doc flip, because `release-manager.sh` Phase 0 runs `verify-release-state.sh --check-unreleased`, which today reports 3 mismatches (`ROADMAP_ACTIVE says v0.1.45`, `CURRENT references v0.1.45`, `CHANGELOG missing entry for v0.1.46`) and would refuse to ship. Also install `parse-changelog-cli` so the local check uses the parser `release.yml` uses | release | after ACT-401 (#1180) | ⏳ this is the PR that legitimately carries `release-preparation` |
| ACT-408 | Close #1165 with the release evidence; re-queue #1068/#1089/#1076 behind the closeout | release drift | after ACT-409 | ⏳ |

## Queued actions (2026-10-04 — validated backlog, sequenced)

| ID | Action | Rec | Depends on | Status |
|----|--------|-----|-----------|--------|
| ACT-376 | Modification watermark: `Episode.updated_at` + Turso column + redb `SCHEMA_VERSION` 4→5, keyset pages on `(updated_at, episode_id)`, cursor through the sync loop; add a defaulted `query_episodes_modified_since` (E5) | #1067 (S08) | — | ⏳ |
| ACT-377 | Revision-aware merge: one helper used by `get_all_episodes` and retrieval backfill; wire the orphan `resolve_episode_conflict`; document the missing-revision policy | #1068 (S09) | ACT-376 | ⏳ |
| ACT-378 | Single-transaction redb `store_episodes_batch` override, then batch the synchronizer and honor the dead `SyncConfig.batch_size` | #1089 (M08) | ACT-376 | ✅ code merged #1171 (`16426d08`); criteria 1–3 evidenced, 4 open |
| ACT-379 | Identity-scoped `EmbeddingStorageBackend` adapter over `Arc<dyn StorageBackend>`, namespaces + `kind:model:dims`/revision in the key, truthful ephemeral mode | #1073 (R02) | — | ⏳ |
| ACT-380 | Route the 5 remaining pattern/management sites through `live_semantic_service()` — closes E2 | #1074 (R03) | ACT-379, after ACT-374 | ⏳ |
| ACT-381 | Reuse stored pattern vectors and batch only the misses; coalescing keyed on full provider identity/revision | #1076 (R05) | ACT-379, ACT-380 | ⏳ |
| ACT-382 | Capability truth for the 23 faking `StorageBackend` defaults — **4 PRs**: cleanup pair (+ delete dead `redb_cache.rs` impl), procedural ×4, relationships ×9, inventory doc + lint | #1087 (M06) | — | ⏳ |
| ACT-383 | Ignored-test integrity — **3 PRs**: inventory artifact + ADR-027 truth (in review #1130), un-ignore the ~11 validation-only `security_tests.rs` cases, isolated nightly Turso job | #1091 (Q02) | — | 🔄 1/3 |
| ACT-384 | Lint-suppression integrity — **4 PRs**: turso `unsafe_code` peel, `#![allow]` ceiling ratchet in `ci.yml`, per-crate `#![expect]` conversion, re-enable `unwrap_used`/`expect_used` | #1092 (Q03) | — | ⏳ |
| ACT-385 | Remaining validated P1s: #1063 adaptive pool monitor/permits/cooldown (E4), #1064 scoped `with_connection`, #1070 capacity eviction, #1071 strict row decode, #1065 ms recommendation timestamps, #1078 incremental ranking, #1079 execution-backed provenance | see GAP file | — | ⏳ |
| ACT-368 | OIDC-only crates.io publish — manual steps: 4 crates.io registrations, delete `CARGO_REGISTRY_TOKEN`, restrict the `crates.io` env to `v*` | #1109 C1/C2 | — | ✅ code merged #1123; ⏸ maintainer |
| ACT-369 | Release-path C5–C7: verify-a-release page, trigger slimming, publish-path hygiene | #1109 C5/C6/C7 | — | ✅ merged #1125 (`4fb0e20f`) |

**ACT-378 evidence (2026-10-17)** — code half: `memory-storage-redb/src/backend_impl.rs:20` +
`episodes.rs:99-127` (one write transaction per batch), `memory-core/src/sync/synchronizer.rs:298`
(`page.chunks(batch_size)`) and `:48`/`:67` (`SyncConfig.batch_size` is no longer dead). Acceptance
criteria for #1089, each named to the test that proves it: 1 → `storage_sync.rs:68`, `:794`
(100/1,000 volume); 2 → `storage_sync.rs:278`, `sync_watermark.rs:383`; 3 → `sync_watermark.rs:306`
and `:383` **plus** `sync_partial_failure_cursor.rs` (#1187), where the batch path and the cursor are
finally asserted together; **4 → still open** — `sync_watermark.rs:268` proves exactly-once delivery
("no duplicates or omissions") but no sync test asserts page **order**, so that box stays unchecked.

## Completed actions (pointer)

| Series | Outcome |
|--------|---------|
| 2026-10-05 wave-window hygiene | ✅ #1128 docs-integrity, #1129 trackers+wave doc, #1131 W1, #1133 local sensors, #1134 W3, #1135 W2, #1136 dependabot, #1139 W4 |
| 2026-09-24…10-04 release pipeline + receipts + toolchain | ✅ #1107, #1108, #1110, #1112, #1113, #1117, #1121, #1123, #1125 |
| 2026-09-24 retrieval judgment + rerank + merge tooling | ✅ #1041, #1042, #1046 |
| 2026-08-06…13 CI trust + attribution + capability truth | ✅ #916, #927, #930, #934, #938, #940, #947, #952 |
| 2026-07 recommendation wave, ships v0.1.36–v0.1.41, skills | ✅ #873…#897 family |
| Pre-2026-07 (ACT-190…ACT-363) | ✅ complete — full tables in `plans/archive/2026-07-consolidation/completed-sprints/` |

Maintainer-external items still open: ADR-079 stage 4 live fault-injection proof; ADR-080/081 lifecycle acceptance;
#1109 crates.io registrations + secret deletion.

## Prevention permanently (do not regress)

- Never `#[serde(tag=)]` on postcard types
- StorageBackend new methods → all backends
- CLI path flags → set `redb_path`
- Cross-process storage features → e2e CLI test
- No manual `gh release create`; use release-manager + `release.yml`
- No soft-pass on cargo deny / required cancelled checks
- Required status must be a causal same-run aggregate; an echo anchor is not a gate
- Dependabot/fork trust changes permissions and secret access, not test assertions
- Fail-closed `execute_agent_code` unless approved capability backend
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+)
- Docs integrity: do not re-check `plans/archive/**` link rot as a ship blocker
- After tag `vX.Y.Z`, immediately bump workspace to next patch before more feat/fix commits
- Commit bodies must stay ≤ 100 chars; repair long bodies mechanically with `git filter-branch --msg-filter 'fold -s -w 100'` and verify with `npx commitlint --from <base> --to HEAD --verbose`
- No-op `chore(ci): re-trigger workflow runs` commits are lint-noise — drop them via rebase, never push them
- Pre-existing repo-wide release drift blocks every PR: fix the root cause (ship the release), use the `release-preparation` label only as a documented deadlock breaker
- Codecov patch coverage: dedupe duplicated rendering (removes uncovered lines from the denominator) AND add targeted tests for new core paths
- Bot/agent PR branches can silently revert main: before merging any long-lived bot branch, diff the tip against `origin/main` and reject clobbers (PR #1130 tipped this)
- Machine-readable test output can be empty: `--message-format libtest-json` needs `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`; assert report artifacts are non-empty before reading a red job as a test failure (PR #1159)
- A version bump alone does not make a repo shippable: `release-manager.sh` Phase 0 runs `verify-release-state.sh --check-unreleased`, which validates `CHANGELOG.md` and the version-doc lines against **`workspace_version`**. Run it before promising a tag date, and land the CHANGELOG/doc flip in a release-prep PR (the pattern #1169 used for v0.1.45)
- Dependabot (GHSA) alerts and `cargo audit`/`cargo deny` (RUSTSEC) are **different registers**: clearing the advisory-db gate closes zero Dependabot alerts (#1174 fixed audit, all 12 alerts stayed open), and one advisory can appear as several alerts — one per lockfile
- An advisory can be invisible to **both** local gates in two different ways: RustSec `informational` advisories (e.g. `lru` RUSTSEC-2026-0002) are routed by cargo-audit to `.warnings.unsound`, never to `.vulnerabilities`, and the CI audit step fails only on `.vulnerabilities.count` — so it reddens neither `cargo audit` nor `cargo deny`; and a GHSA that never reached RustSec (`libsql-sqlite3-parser`, CVE-2025-47736) is not in the advisory DB at all. Measured on this lock: in-repo `cargo audit --no-fetch --json` → rc 0 with 5 `.warnings.unsound` + 1 `.unmaintained` surviving, while the same lock from `/tmp` (so `.cargo/audit.toml` ignores do not apply) → rc 1 with 8 `.vulnerabilities`. **A green `cargo audit` means "nothing un-ignored reached `.vulnerabilities`", not "no advisories"** — cite the ignore list and the `.warnings` count whenever you use it
- Never plan a dependency-upgrade PR from a changelog claim: resolve the candidate version in a scratch crate and `cargo tree` it. That is how "libsql 0.10.0 clears 8 alerts" was disproved — 0.10.0 still pins `hyper-rustls 0.25 → rustls 0.22.4 → rustls-webpki 0.102.8`
- Record an unfixable advisory as **upstream-blocked with the pinning chain named** (`augurs-changepoint 0.10.2` → `changepoint 0.14.2` → `rv 0.16.5` → `lru 0.9.0`), not as a to-do the next bump will close
- Floating `stable` adds deny-by-default lints between releases (1.99: clippy `assert_is_empty`, rustdoc `redundant_explicit_links`): `rustup update stable` before diagnosing CI-only lint failures; fix, never suppress (PRs #1145/#1150/#1152)
- Strict up-to-date checks make merges serial; every merge invalidates the other PRs' runs — merge as a pipeline and update the next branch immediately (ruleset 9591004)
- Shared `CARGO_TARGET_DIR` across worktrees can replace test binaries and fake mass failures — re-run the failing test in isolation before believing it
- Composite-action inputs are strings: `!inputs.<x>` is a truthiness bug (`!"false"` → `false`). Compare literally (`inputs.dry-run != 'true'`) and keep every guard in the action in the same form
- A green job is not evidence of work: a step with `conclusion=skipped;duration_ms=0` inside a `success` job hid two unpublished releases. Assert the *effect* (poll crates.io in the publishing job), not the conclusion
- `statusCheckRollup` keeps superseded runs of the same check name — reduce to the latest run per name before calling a PR red or green
- `#[ignore]` + `--run-ignored only` cannot express "only under this feature": nightly force-runs those tests against the wrong schema. Feature-gated suites need `[[test]] required-features`; with the feature on, `only` then filters everything out, so use `--run-ignored all` and assert the run count
- Never assert `result.is_ok()` without printing the error — an `Err` rendered as a fixed sentence cost days on three real Turso defects
- Hand-written SQL must be checked against the DDL constants (column names *and* declared units): `*_ms` columns written with `as_secs_f64()` round-tripped wrong by 1000×, and nonexistent columns made metrics permanently unwritable
- `get_connection_with_id()` is direct-connection only; pooled call sites must wrap the whole closure in `with_connection`/`with_connection_with_id`
- After shipping, the drift gate reds every PR until the version advances; the fix is the bump PR, not a waiver. `release-preparation` only re-evaluates when re-applied from outside (`gh pr edit`), because a `GITHUB_TOKEN` label does not trigger a run
