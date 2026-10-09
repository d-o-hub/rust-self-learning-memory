# GOAP Actions Backlog

- **Last Updated**: 2026-10-06
- **Active plan**: **audit-backlog wave** — `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`. W1–W4 merged 2026-10-05;
  W5/W6 + Q02 in review; queued chains ACT-376…ACT-385 below.
- **Previous plan**: release pipeline hardening (#1109) — **code complete**: C1/C2 (#1123), C3/C4 (#1121), C5–C7 (#1125).
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

## Queued actions (2026-10-04 — validated backlog, sequenced)

| ID | Action | Rec | Depends on | Status |
|----|--------|-----|-----------|--------|
| ACT-376 | Modification watermark: `Episode.updated_at` + Turso column + redb `SCHEMA_VERSION` 4→5, keyset pages on `(updated_at, episode_id)`, cursor through the sync loop; add a defaulted `query_episodes_modified_since` (E5) | #1067 (S08) | — | ⏳ |
| ACT-377 | Revision-aware merge: one helper used by `get_all_episodes` and retrieval backfill; wire the orphan `resolve_episode_conflict`; document the missing-revision policy | #1068 (S09) | ACT-376 | ⏳ |
| ACT-378 | Single-transaction redb `store_episodes_batch` override, then batch the synchronizer and honor the dead `SyncConfig.batch_size` | #1089 (M08) | ACT-376 | ⏳ |
| ACT-379 | Identity-scoped `EmbeddingStorageBackend` adapter over `Arc<dyn StorageBackend>`, namespaces + `kind:model:dims`/revision in the key, truthful ephemeral mode | #1073 (R02) | — | ⏳ |
| ACT-380 | Route the 5 remaining pattern/management sites through `live_semantic_service()` — closes E2 | #1074 (R03) | ACT-379, after ACT-374 | ⏳ |
| ACT-381 | Reuse stored pattern vectors and batch only the misses; coalescing keyed on full provider identity/revision | #1076 (R05) | ACT-379, ACT-380 | ⏳ |
| ACT-382 | Capability truth for the 23 faking `StorageBackend` defaults — **4 PRs**: cleanup pair (+ delete dead `redb_cache.rs` impl), procedural ×4, relationships ×9, inventory doc + lint | #1087 (M06) | — | ⏳ |
| ACT-383 | Ignored-test integrity — **3 PRs**: inventory artifact + ADR-027 truth (in review #1130), un-ignore the ~11 validation-only `security_tests.rs` cases, isolated nightly Turso job | #1091 (Q02) | — | 🔄 1/3 |
| ACT-384 | Lint-suppression integrity — **4 PRs**: turso `unsafe_code` peel, `#![allow]` ceiling ratchet in `ci.yml`, per-crate `#![expect]` conversion, re-enable `unwrap_used`/`expect_used` | #1092 (Q03) | — | ⏳ |
| ACT-385 | Remaining validated P1s: #1063 adaptive pool monitor/permits/cooldown (E4), #1064 scoped `with_connection`, #1070 capacity eviction, #1071 strict row decode, #1065 ms recommendation timestamps, #1078 incremental ranking, #1079 execution-backed provenance | see GAP file | — | ⏳ |
| ACT-368 | OIDC-only crates.io publish — manual steps: 4 crates.io registrations, delete `CARGO_REGISTRY_TOKEN`, restrict the `crates.io` env to `v*` | #1109 C1/C2 | — | ✅ code merged #1123; ⏸ maintainer |
| ACT-369 | Release-path C5–C7: verify-a-release page, trigger slimming, publish-path hygiene | #1109 C5/C6/C7 | — | ✅ merged #1125 (`4fb0e20f`) |

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
