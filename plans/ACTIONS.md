# GOAP Actions Backlog

- **Last Updated**: 2026-10-04
- **Active plan**: **audit-backlog wave** — `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md`. 22 open code issues (#1063–#1092)
  were filed at `9f50c607`, never entered in these trackers, and re-validation at `74a44a15` found **0 of them fixed**.
  Wave slices ACT-370…ACT-375 below; queued chains ACT-376…ACT-383.
- **Previous plan**: release pipeline hardening (issue #1109) — **code complete**: C1/C2 (#1123), C3/C4 (#1121, #1110),
  C5–C7 (#1125). Remaining manual steps for crates.io: four trusted-publisher registrations, `CARGO_REGISTRY_TOKEN`
  deletion, and restricting the `crates.io` environment to `v*` tags. See ADR-078 amendment + LESSON-030…034.
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Active actions (2026-10-04 — audit-backlog wave W1…W6)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-370 | Drop the ranking read guard before `.await` in `recommend_patterns_for_task`; clone the snapshot instead of holding `ranking_index.read()` across `get_all_patterns()` and the recommendation pipeline; add a contention test proving a concurrent `refresh_ranking_index()` is not stalled | #1077 (R06) | 🔄 W1 |
| ACT-371 | Measure MCP request latency from a monotonic `Instant` (`as_millis()`), keeping epoch seconds only for event timestamps; regression test that a 1–50 ms operation reports non-zero and that min/max/avg/export stay valid | #1086 (M05) | 🔄 W2 |
| ACT-372 | Roll back the Turso tag transaction on every failure path in `save_episode_tags` (and make `delete_episode_tags` atomic); failure-injection/deterministic test that fails after the deletes and proves the prior tag set survives and the connection stays usable | #1088 (M07) | 🔄 W3 |
| ACT-373 | Compare `(timestamp, session_id)` inside the same redb transaction before overwriting the episode→session recommendation index, so write order stops beating recency; keep `TableDefinition<&str,&str>` to stay compatible with the fail-closed schema check | #1066 (S07) | 🔄 W4 |
| ACT-374 | Replace the constant `0.5` in `calculate_keyword_similarity` with a bounded deterministic query-aware lexical score; thread query text through `calculate_pattern_score`; tests for matching-vs-unrelated ordering, case/punctuation/repeated terms/empty query, and a guard against re-introducing the constant | #1075 (R04) | 🔄 W5 |
| ACT-375 | Make MCP health probes real: use configured handles with bounded per-backend `health_check`, report `healthy`/`degraded`/`unavailable` independently, wire live query-cache metrics + synchronizer state + process uptime, and **redact `TURSO_DATABASE_URL`, paths and raw backend errors** (escalation E1) | #1085 (M04) | 🔄 W6 |

## Queued actions (2026-10-04 — validated backlog, sequenced)

| ID | Action | Rec | Depends on |
|----|--------|-----|-----------|
| ACT-376 | Modification watermark: `Episode.updated_at` + Turso column + redb `SCHEMA_VERSION` 4→5, keyset pages on `(updated_at, episode_id)`, cursor through the sync loop; add a defaulted `query_episodes_modified_since` rather than changing the required method (E5: 22 implementations) | #1067 (S08) | — |
| ACT-377 | Revision-aware merge: one helper used by both `get_all_episodes` and retrieval backfill; wire the existing orphan `resolve_episode_conflict`; document the missing-revision policy | #1068 (S09) | ACT-376 |
| ACT-378 | Single-transaction redb `store_episodes_batch` override, then batch the synchronizer and honor the dead `SyncConfig.batch_size` | #1089 (M08) | ACT-376 |
| ACT-379 | Identity-scoped `EmbeddingStorageBackend` adapter over `Arc<dyn StorageBackend>`, namespaces + `kind:model:dims`/revision in the key, truthful ephemeral mode; keep `find_similar_*` working through the key-shape change | #1073 (R02) | — |
| ACT-380 | Route the 5 remaining pattern/management call sites through `live_semantic_service()` — closes escalation E2 (production pattern search currently ignores the query) | #1074 (R03) | ACT-374, before ACT-381 |
| ACT-381 | Reuse stored pattern vectors and batch only the misses; extend coalescing to key on full provider identity/revision | #1076 (R05) | ACT-379, ACT-380 |
| ACT-382 | Capability truth for the 23 faking `StorageBackend` defaults — **split into 4 PRs**: cleanup pair (+ delete the dead `redb_cache.rs` impl), procedural ×4, relationships ×9, inventory doc + lint so new defaults can't be added silently | #1087 (M06) | — |
| ACT-383 | Ignored-test integrity — **split into 3 PRs**: inventory artifact + ADR-027 truth, un-ignore the ~11 validation-only `security_tests.rs` cases, isolated nightly Turso job with a non-green crash/timeout signal (today 118 ignored Turso tests run nowhere) | #1091 (Q02) | — |
| ACT-384 | Lint-suppression integrity — **split into 4 PRs**: turso `unsafe_code` peel to its sites, `#![allow]` ceiling ratchet in `ci.yml` (the `allow_attributes = "deny"` sensor is currently inert), per-crate `#![expect]` conversion, then re-enable `unwrap_used`/`expect_used` | #1092 (Q03) | — |
| ACT-385 | Remaining validated P1s: #1063 adaptive pool monitor/permits/cooldown (E4 — `adaptive_tests.rs:140-146` asserts the broken value), #1064 scoped `with_connection` + 85-site migration, #1070 capacity eviction atomic-or-repairable, #1071 strict row decode (+ raw-query partial success), #1065 millisecond recommendation timestamps + deterministic tie-break, #1078 incremental ranking + benches, #1079 execution-backed provenance | see GAP file | — |
| ACT-368 | OIDC-only crates.io publish — manual steps: 4 crates.io registrations, delete `CARGO_REGISTRY_TOKEN`, restrict the `crates.io` env to `v*` | #1109 C1/C2 | ⏸ maintainer |

## Active actions (2026-10-03 — release pipeline hardening, issue #1109)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-368 | OIDC-only crates.io publish: official `crates-io-auth-action`, shared `.github/actions/publish-crate` action, skipped-tolerant `needs` chain (single-crate dispatch no longer silently skipped), `release.yml` dispatch on the tag (bot-published releases never fired `release: published`; crates.io was stale at 0.1.34), and hardening (tag-ref-only publishes, tag↔version binding, verify-then-`--no-verify`, pinned semver-checks, concurrency, fail-closed `needs`) | #1109 C1/C2 | ✅ merged in PR #1123 (`68eda3d0`) — manual follow-ups: 4 crates.io registrations, delete `CARGO_REGISTRY_TOKEN`, restrict `crates.io` env to `v*` |
| ACT-369 | Release-path C5–C7: book "Verify a release" page + SECURITY.md commands (signer-pinned attestations), release.yml PR trigger removed (ran only skipped jobs) + `v`-prefixed tag glob, Pages boundary recorded, publish-path `persist-credentials: false`, `host` OIDC blanked on SBOM build steps, `--skip-local-tests` documented as ci-check-parity emergency, mdBook `#`-stub files purged | #1109 C5/C6/C7 | ✅ merged in PR #1125 (`4fb0e20f`) |

## Completed actions (2026-09-24 — retrieval judgment + rerank + merge tooling)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-364 | Implement the provider-neutral typed semantic judgment interface (#1030) | R-F11 / #1030 | ✅ merged in PR #1041 — `RetrievalJudge`, typed atomic judgments, ID/score validation, bounded telemetry, 12 gate-satisfying tests |
| ACT-365 | Implement opt-in semantic shortlist rerank with deterministic fusion (#1031) | R-F12 / #1031 | ✅ merged in PR #1042 — `SemanticRerankConfig`, single `finish_ranked` path, `RerankStatus` telemetry, offline `--rerank` eval comparison (regression PASSED) |
| ACT-366 | Gate merges on readiness and check tracker drift; capture coverage-waiver knowledge | tooling | 🔄 PR #1046 — `merge-pr.sh`, `validate-plans.sh --tracker-drift`, `coverage-waivers` skill |
| ACT-367 | File harness friction upstream (sensor re-runs, beat scoping, PR readiness CLI, coverage classifier, project-check sensors, ci-explain, learn drafting, PR-loop metrics) | harness | ✅ 8 issues in `d-o-hub/do-harness` (#238–#245) |

## Completed actions (2026-08-12 — ranking adaptation + registry canonicalization)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-362 | Implement feedback-to-ranking adaptation (ADR-082) — derived Wilson weight, capability-gated `list_recommendation_*` methods on Turso/redb, recommend re-rank, e2e tests | ADR-082 §5 exit | ✅ merged in PR #952 (2026-08-13) — evidence-backed (e2e + backend contract + unit tests) |
| ACT-363 | Canonicalize ADR-025/054 aliases (move to `plans/adr/_aliases/`) to make the ADR registry unique | G-P1-8 | ✅ merged in PR #952 (2026-08-13) — `validate-plans.sh --identifiers` sees 51 unique ADR numbers |

## Completed actions (2026-08-11 — #947, merged 2026-08-12)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-334 | Accept ADR-079 and freeze the `CI / Required` aggregate contract | CIT-A1 | ✅ Accepted — ruleset `9591004` requires `[Codacy Static Code Analysis, CI / Required]` (verified live) — merged in #947 (2026-08-12) |
| ACT-335 | Implement and fault-inject same-run required aggregation | CIT-A1 | ✅ same-run `commitlint` + `fast-gate`; `ci-required-evaluate.sh` accepts only `success`, rejects `skipped`/`cancelled`/`timed_out`/`failure`/missing/unknown; `--required-aggregate` fixtures — merged in #947 (2026-08-12) |
| ACT-340 | Require the verified aggregate in ruleset `9591004` and validate blocking | CIT-A1/PTA-A9 | ✅ ruleset already requires `CI / Required` (stage 3); deliberate live fault-injection merge-block proof (stage 4) remains external maintainer evidence — merged in #947 (2026-08-12) |
| ACT-348 | Validate episode existence; unify malformed-ID rejection across core/MCP/CLI | RAT-B4 | ✅ evidence-backed — merged in #947 (2026-08-12) |
| ACT-349 | Add fallible playbook retrieval; no session on generation failure | RAT-B5 | ✅ evidence-backed — merged in #947 (2026-08-12) |
| ACT-350 | Add `persist_feedback_checked`; give manual MCP/CLI commands receipt semantics | RAT-B6 | ✅ evidence-backed — merged in #947 (2026-08-12) |
| ACT-351 | Declare `episode_id` in both MCP registries + registry-agreement test | RAT-B7 | ✅ evidence-backed — merged in #947 (2026-08-12) |
| ACT-352 | Deduplicate CLI rendering; replace the `too_many_arguments` suppression with a request struct | RAT-B8 | ✅ evidence-backed — merged in #947 (2026-08-12) |
| ACT-353 | Restart-safety, receipt-matrix, MCP snapshot, and CLI e2e tests to ≥ 90% | RAT-B9 | ✅ evidence-backed (coverage % measured by the controller's final validation) — merged in #947 (2026-08-12) |
| ACT-354 | Docs + plans-registry + repo hygiene (`API_REFERENCE`, ADR-058 duplicate, `.gitignore`) | RAT-B10 | ✅ evidence-backed — merged in #947 (2026-08-12) |

## Completed actions (2026-08-09 — fuzz nightly + LTO-off wave)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| T1 | Build all 3 fuzz targets locally with LTO off (`CARGO_PROFILE_RELEASE_LTO=false`) | G1 | ✅ Finished, no link error |
| T2 | Independent LTO root-cause review | G1 | ✅ nightly toolchain + `lto="fat"` config vs `-Zsanitizer` |
| T4 | Add `CARGO_PROFILE_RELEASE_LTO: false` to fuzz.yml job env | G1 | ✅ merged in #934 |
| T6 | Re-trigger fuzz workflow on branch | G1 | ✅ `success` (2026-08-09 17:20Z) |
| T9 | Merge #934 | G2 | ✅ merged 2026-08-09 |

## Active actions (2026-08-07 — PR review & CI fix wave)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-355 | Review/roast open PRs #928 + #927; fix all failing CI incl. pre-existing | GOAP | ✅ 2026-08-07 (see wave plan) |
| ACT-356 | Repair #928 commit messages (rewrap bodies ≤100, drop no-op commits) | commitlint | ✅ pushed; CI green |
| ACT-357 | Break #927 drift deadlock via `release-preparation` label | drift | ✅ Release Drift Check green |
| ACT-358 | Raise #927 Codecov patch coverage (receipt matrix + MCP + CLI dedup) | RAT-B8/B9 | ✅ pushed `68457631`→`52276c50` |
| ACT-359 | Ship v0.1.38 via release-guard to clear repo-wide drift for all PRs | R-A3 | ✅ shipped 2026-08-08 (workspace bumped to 0.1.39) |

## Active actions (2026-08-12 — post-closure, maintainer-external)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-360 | Deliver ADR-079 stage-4 live fault-injection merge-block proof | ADR-079 | ⏸ external maintainer evidence |
| ACT-361 | Accept ADR-080/081 lifecycle (move from Proposed on acceptance) | ADR-080/081 | ⏸ Proposed — maintainer |

## Active actions (2026-08-06)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-334 | Accept ADR-079 and freeze the `CI / Required` aggregate contract | CIT-A1 | ✅ Accepted (ruleset `9591004` requires `CI / Required`) — see 2026-08-11 completed table |
| ACT-335 | Implement and fault-inject same-run required aggregation | CIT-A1 | ✅ same-run gates + fail-closed evaluator — merged in #947 (2026-08-12) |
| ACT-336 | Fail closed on cancellation/missing/commitlint and restore Dependabot/fork assertion parity | CIT-A2 | ✅ waiters fail closed + commit-lint wait + downstream actor parity (2026-08-10) |
| ACT-337 | Reconcile test/Clippy/quality scopes and add semantic gate-contract fixtures | CIT-A3 | ✅ semantic validator + negative fixtures + actor-parity/ruleset-context fixtures (2026-08-10) |
| ACT-338 | Remove broken release dispatch and make publish selection/dependency planning truthful | CIT-A4 | ✅ Done (2026-08-06) |
| ACT-339 | Preserve fuzz/mutation evidence, then measure and remove duplicate CI work | CIT-A5 | ✅ Done (fuzz half; mutants already durable — 2026-08-06) |
| ACT-340 | With approval, require the verified aggregate in ruleset `9591004` and validate blocking | CIT-A1/PTA-A9 | ✅ ruleset already requires `CI / Required` (stage 3); stage 4 live fault-inject proof = external maintainer evidence |
| ACT-302 | `./scripts/release-manager.sh ship --execute` for `v0.1.36` | R-A1 | ✅ Done |
| ACT-303 | Post-release workspace bump to 0.1.37 | R-A2 | ✅ #886 |
| ACT-315 | Plans progress truth (open PRs, post-ship) | R-G* | ✅ #889 |
| ACT-316 | Land #887 changelog hygiene | docs | ✅ #887 merged |
| ACT-317 | Review/merge #888 cosine perf | perf | ✅ #888 merged |
| ACT-318 | Mark ADR-074 as Accepted / Implemented | docs | ✅ Done (#891) |
| ACT-319 | Gap analysis tasks: pattern extract + ADR-074 docs | G-P1-12/ACT-317/318 | ✅ #891 merged |
| ACT-320 | R-F8 CLI relationship show polish | R-F8 | ✅ #893 merged |
| ACT-321 | R-F9 HNSW persistence + capacity eviction | R-F9 | ✅ #893 merged |
| ACT-322 | Add 6 domain skills (40 total, all routed) | skills | ✅ #894 |
| ACT-323 | ADR-077 A1-A5 runtime embedding activation | ADR-077 | ✅ main (`9ef4b742`, `e0f7f712`) |
| ACT-324 | ADR-077 A6 validate/document/gate (docs + concurrency + zero-unsafe redaction tests) | ADR-077 | ✅ #897 merged |
| ACT-312 | R-F* GO spike artifacts written + validated (2026-07-28) | R-F* | ✅ Done |
| ACT-325 | Implement R-F10 OIDC trusted publishing in publish-crates.yml | R-F10 | ✅ Done (`id-token: write` + OIDC exchange; plans refreshed) |
| ACT-326 | Implement R-F4 SIMD cosine acceleration + benchmark variants | R-F4 | ✅ Done (`cosine_similarity_simd` + simd bench variant) |
| ACT-341 | Make non-`csm` cascade retrieval capability-truthful | PTA-A1 | ✅ Implemented |
| ACT-342 | Make CLI storage metrics measured/estimated/unavailable explicitly | PTA-A2 | ✅ Implemented |
| ACT-343 | Hide unsupported `eval set-threshold` command | PTA-A3 | ✅ Implemented |
| ACT-328 | Accept ADR-080 and freeze attributed contracts | RAT-A1 | Proposed |
| ACT-329 | Add capability-aware checked attribution persistence | RAT-A2 | Blocked by ADR acceptance |
| ACT-330 | Add core attributed pattern/playbook operations | RAT-A3 | Blocked by ACT-329 |
| ACT-331 | Enforce session and feedback integrity | RAT-A4 | Blocked by ACT-330 |
| ACT-332 | Wire optional attribution through MCP and CLI | RAT-A5/A6 | Blocked by ACT-331 |
| ACT-333 | End-to-end validation, docs, and authority update | RAT-A7/PTA-A9 | Blocked by ACT-341…343 + RAT chain |

All ACT-300…ACT-324 items are complete. ACT-341…ACT-343 (PTA-A1/A2/A3) are
implemented 2026-08-01. ACT-325/326 (R-F10/R-F4) and ACT-338/339 (CIT-A4/A5)
are implemented 2026-08-06. ADR-080/081 attribution merged in #927 (receipt-matrix
tests #930); the ADR-081 §2 capability-truth gap closed 2026-08-10 via
`supports_recommendation_attribution` + capability-gated `persist_session_checked`.
PR #947 (merged 2026-08-12) landed the same-run
fast gate, the fail-closed evaluator, the waiter/anchor removal (ADR-079 stage 5),
and the ADR-080/081 acceptance evidence (ACT-348…354, ACT-334/335/340). The
rank/adaptation PR landed ACT-362 (ADR-082 feedback-to-ranking adaptation) and
ACT-363 (ADR-025/054 alias canonicalization). Remaining
open items are maintainer-external: ADR-079 stage 4 live fault-injection
merge-block proof, and ADR-080/081 lifecycle acceptance (both stay `Proposed`).

## Completed actions (summary)

All ACT-190…ACT-279 series and 2026-07 recommendation waves are **complete**.  
Full tables: `plans/archive/2026-07-consolidation/completed-sprints/`

### Prevention permanently (do not regress)

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
