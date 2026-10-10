# GOAP: audit-backlog reconciliation + missing-task wave (2026-10-04)

- **Date**: 2026-10-04
- **Orchestrator**: `goap-agent` skill (ANALYZE → DECOMPOSE → STRATEGIZE → COORDINATE → EXECUTE → SYNTHESIZE)
- **Trigger**: `plans/` said "no active plan / 0 open code gaps" while `gh issue list --state open` returned **22 unfixed
  code issues** (#1063–#1092, plus the #1109 release proposal). The trackers had no reference to any of them.
- **Baseline validated**: `74a44a15` (main). Audit baseline of the issues themselves: `9f50c607` (2026-09-28).

## ANALYZE — why the drift happened, and what constrains the fix

The 22 issues were filed by a static-analysis swarm at `9f50c607`. The waves that shipped afterwards
(#1095 pool ownership, #1096 redb fail-closed, #1097 embedding activation identity, #1100, #1101 OAuth,
#1107/#1112/#1113, #1117, #1121/#1123/#1125 release pipeline) closed *other* issues (#1060–#1062, #1069,
#1072, #1080–#1084, #1090, #1094, #1098, #1116) and were recorded in the trackers — but nothing reconciled the
22 survivors, so each tracker refresh restated "0 open gaps" from the completed-wave evidence rather than from
the live issue list.

Constraints that shape any implementation slice:

- **AGENTS.md invariants**: no locks held across `.await`; parameterized SQL only; Postcard serialization;
  clippy `-D warnings`; ≤500 LOC per source file; ≥70% coverage floor; new public types re-exported from `lib.rs`.
- **ADR-081/082** — capability advertisement is the established pattern for "this backend cannot do that"
  (`supports_recommendation_attribution`, `supports_ranking_adaptation`); it is the template for #1087.
- **ADR-074** — provenance must describe what actually ran; `candidate_count` is pre-truncation. Bounds #1079.
- **ADR-072 §4** — one release authority, tag-only. #1109 must not be reopened by this wave.
- **ADR-027** — ignored-test strategy; its counts are now demonstrably stale (#1091).
- **Schema-migration paths exist** for both backends: Turso `PRAGMA table_info` → conditional `ALTER TABLE`
  (`turso_config.rs:207-240`), redb `SCHEMA_VERSION` + `migrate_schema()` (`lib.rs:183-199`,
  `storage_ops/migration.rs:31`, added by #1096). #1067 therefore has a known, tested migration route.

## VALIDATE — per-issue verdicts (5 parallel read-only agents, evidence at `74a44a15`)

**Result: 20 OPEN, 2 PARTIAL. Zero of the 22 were fixed by later PRs** — no commit in `9f50c607..HEAD` references
any of these numbers, and `git log 9f50c607..HEAD -- <path>` shows the cited regions untouched.

| Issue | Key | Verdict | Decisive evidence at HEAD | Effort |
|---|---|---|---|---|
| #1063 | S04 | OPEN (worse than filed) | `pool/adaptive.rs:112,120,133` + `:152,160,173` — both ctors spawn an empty monitor task; repo-wide grep for `add_permits` = 0 hits; `:251`/`:290` store `now.elapsed()` from a fresh `Instant` (≈0 ns) so the cooldown predicate is always true → scale-up/scale-down are permanent no-ops; `adaptive_tests.rs:140-146` *asserts* the broken behavior | M |
| #1064 | S05 | OPEN | `lib_impls/helpers.rs:47-57`, `:59-66`, `:68-71` consume the guard via `into_inner()` before any SQL; no `with_connection` exists anywhere; 27 `get_connection()` + 58 `get_connection_with_id()` call sites | L |
| #1065 | S06 | OPEN | `storage/recommendations.rs:41` `session.timestamp.timestamp()` (whole seconds), `:93` `ORDER BY timestamp DESC LIMIT 1` with no tiebreaker; `schema/mod.rs:65` coarse INTEGER column | M |
| #1066 | S07 | OPEN | `memory-storage-redb/src/recommendations.rs:58-59` unconditional `episode_index.insert(...)`; value type is a bare `&str` (`lib.rs:139-140`), no ordering key | S/M |
| #1067 | S08 | OPEN (worse than filed) | `backend.rs:103` doc says "modified since", `:115` implements `start_time`; Turso `episodes/query.rs:108-110`; redb `episodes_queries.rs:63`; `synchronizer.rs:141-155` one capped page, no cursor; `:229` re-queries last hour. **New**: `episodes.created_at` (`schema/mod.rs:22`) is absent from the INSERT column list (`storage/episodes/crud.rs:16-22`), so it re-defaults on every write — it is not even a creation stamp | L |
| #1068 | S09 | OPEN, **hard-blocked by #1067** | `memory/queries/mod.rs:63-68` then `:91-96`, both `entry(id).or_insert_with(...)` → stale redb wins by visit order; same shape at `retrieval/context.rs:179-181`/`:196-198`. `sync/conflict.rs:20-40 resolve_episode_conflict` exists but is wired to nothing outside its own tests | M |
| #1070 | S11 | OPEN | `storage/capacity.rs:25` persists before `:28 enforce_capacity`; `:101 let _ = self._delete_embeddings_batch_internal(...)` drops both `Err` and rows-affected; no FK/CASCADE on embeddings (`schema/mod.rs:89-100`); episode delete on a second connection | M |
| #1071 | S12 | OPEN | `episodes/row.rs:31 unwrap_or_default()` (invalid `start_time` → chrono **year-0** sentinel, not 1970), `:34` → `None`, `:40/:43/:46 .ok()` silently absent; `episodes/raw_query.rs:59-65,115-121` and `patterns/raw_query.rs:58-65,99-105` skip bad rows and return `Ok(partial)` | M |
| #1073 | R02 | OPEN | `memory-mcp/.../configure.rs:47 let storage = Box::new(InMemoryEmbeddingStorage::new())` (its own comment: "A future ADR may add storage composition here"); the `EmbeddingStorage<T>` wrapper (`storage.rs:263`) is constructed nowhere; `ConfigureEmbeddingsOutput` has no storage-scope field | M |
| #1074 | R03 | **PARTIAL** | Fixed for completion + retrieval by #1097 (`completion_embedding.rs:20`, `retrieval/context.rs:116`, `context_branches.rs:39`). Still static: `pattern_api.rs:62,82,140,242` and `management.rs:364` pass `self.semantic_service.as_ref()` | S/M (was M) |
| #1075 | R04 | OPEN | `pattern_search/scoring.rs:132-135` — `calculate_keyword_similarity(_pattern, _context) -> f32 { 0.5 }`; both parameters ignored. Also `:51` returns `0.5` when there is no service; only coverage asserts non-emptiness (`recommendation.rs:299`) | S |
| #1076 | R05 | OPEN, dormant until R03 lands | `scoring.rs:48 service.embed_pattern(pattern)` inside the per-candidate loop (`recommendation.rs:69-90`) — provider call + write per candidate per search; `get_pattern_embedding` has no caller outside the backend impls; `coalescing.rs:101-103` keys on model-name + text, not provider identity | M |
| #1077 | R06 | OPEN — **invariant violation** | `pattern_api.rs:134` read guard born, alive across `:135 get_all_patterns().await` and `:136-144 recommend_patterns_for_task(...).await`; blocks `ranking.rs:64`'s write for the whole call | S |
| #1078 | R07 | OPEN | `api.rs:221`/`:263` run the full `refresh_ranking_index()`; `ranking.rs:46-51` full-scans both backends, `:63` rebuilds from all history. No ranking/feedback Criterion bench exists (20 `[[bench]]` targets, none for ranking) | L |
| #1079 | R08 | OPEN | `provenance_api.rs:55` + `:57` two `query_cache.get()` calls, each mutating telemetry (`cache/lru.rs:95,106-137`), plus a *third* inside `retrieval/context.rs:118`; `:66-68` fabricates `candidate_count = Some(result_count)` under a comment saying it is unknown | M |
| #1085 | M04 | OPEN — **escalated, see E1** | `jsonrpc.rs:117-118` env-var existence == connected; `:120-123` formats the raw `TURSO_DATABASE_URL` into the response; `:135-141` cache metrics hard-coded `0`; `:152 uptime_seconds = 0`. Real probes already exist (`turso lib_impls/helpers.rs:190`, `redb storage_ops/stats.rs:58`) | M |
| #1086 | M05 | OPEN | `monitoring/core.rs:113-114` `elapsed_secs.saturating_mul(1000)` from `as_secs()` values → any sub-second request records `0 ms`, inherited by avg/min/max and `endpoints.rs:142`. No histogram/percentile exists (the issue overstates) | S |
| #1087 | M06 | OPEN | 27 default methods on `StorageBackend`; only 2 are capability-gated → **23 can fake durable success**. `cleanup_episodes`/`count_cleanup_candidates` are overridden by *no* production backend and called from no production path; `relationships/mod.rs:94`, `api.rs:346,371` discard with `let _ =`; dead duplicate `impl StorageBackend for RedbStorage` in `redb_cache.rs` (never declared in `lib.rs`) | L → split 4 |
| #1088 | M07 | OPEN | `tag_operations.rs:40 BEGIN`, `?` early returns at `:50/:61/:75`, `:79 COMMIT`; the token `ROLLBACK` does not appear in the file | S |
| #1089 | M08 | OPEN (**+ gap the issue understates**) | `synchronizer.rs:162-181` loops `redb.store_episode`; **redb has no `store_episodes_batch` override** (`backend_impl.rs:12` overrides only `store_embeddings_batch`) → routing through the trait alone yields zero speedup. `SyncConfig.batch_size = 100` is dead: `StorageSynchronizer::new` never accepts it | S-M |
| #1091 | Q02 | OPEN (partially mitigated) | **173** `#[ignore]` attrs (turso 118 / core 38 / mcp 11 / cli 2); ADR-027 claims 121/71/37/9 → stale by 52 and omits the largest bucket (`monitoring_capacity_tests.rs` 30); `nightly-tests.yml` runs `--run-ignored only` in two jobs that both exclude `do-memory-storage-turso` → **118 ignored Turso tests execute nowhere** | S+M → split 3 |
| #1092 | Q03 | OPEN | 395 crate-root `#![allow(...)]` lines in `*/src` (cli 66 + `main.rs` 62, mcp 64, turso 57, redb 25, core 23); `memory-storage-turso/src/lib.rs:3` **allows `unsafe_code`** while the other four crates deny it; `Cargo.toml:113` sets `allow_attributes = "deny"` and `HARNESS.md:36` advertises "No `#[allow(...)]`" — the sensor is inert | L → split 4 |

### Escalations found during validation (not in any issue body)

- **E1 — health output leaks the Turso database URL.** #1085 was filed as an observability defect;
  `jsonrpc.rs:120-123` actually interpolates `TURSO_DATABASE_URL` into a health response. Redaction is a
  security fix, not a nice-to-have.
- **E2 — production pattern search has no query relevance at all.** `semantic_service` is `None` on every
  production construction path (`memory/init.rs:133`, `:288`, `core/builder.rs:72`; `with_semantic_config`
  at `init.rs:355-366` stores config only), so `scoring.rs:51` returns the `0.5` no-service constant *and*
  `scoring.rs:132-135` returns the `0.5` lexical constant. Ordering therefore depends only on
  context/effectiveness/recency — the query text is ignored. This is what makes #1074 + #1075 one coupled fix,
  and is why #1076 (per-candidate provider calls) must not land before them.
- **E3 — `episodes.created_at` is not a creation timestamp** (absent from the Turso INSERT column list), so any
  reasoning that assumes it is a lower bound is wrong. Feeds #1067's design.
- **E4 — the adaptive pool's cooldown is provably dead by inspection** and the existing unit test asserts the
  broken value, so a fix must change that test rather than preserve it.

## DECOMPOSE — dependency chain and slices

```
#1067 (S08 watermark) ──> #1068 (S09 revision merge)
        └──> #1089 (M08 batch writes, + redb batch override)

#1073 (R02 identity-scoped adapter) ──> #1076 (R05 batch/reuse)
#1074 (R03 pattern APIs → live snapshot) ──> #1076 (R05 becomes hot)
#1075 (R04 query-aware lexical fallback) ── independent, coupled by E2 to #1074

#1060/#1061/#1062 (closed) ──> #1064 (S05 scoped checkout) ; #1063 (S04) needs only #1060
```

Independent, no trait/schema churn: #1077, #1086, #1088, #1066, #1065, #1085, #1075.

## STRATEGIZE — this wave

Criterion: fix a real defect, do not touch the `StorageBackend` trait signature (22 implementations exist —
4 production backends + 16 test stubs + 1 dead duplicate), do not require a schema migration, and keep each
slice independently revertable.

| Slice | Issue | Key | Change | Branch |
|---|---|---|---|---|
| W0 | — | — | this wave doc + tracker reconciliation | `plans/audit-wave-2026-10-04` |
| W1 | #1077 | R06 | clone the ranking snapshot, drop the read guard before any `.await` | `fix/core-ranking-lock-await` |
| W2 | #1086 | M05 | monotonic `Instant` elapsed → real millisecond latency | `fix/mcp-subsecond-latency` |
| W3 | #1088 | M07 | roll back the tag transaction on every failure path | `fix/turso-tag-tx-rollback` |
| W4 | #1066 | S07 | compare `(timestamp, session_id)` before overwriting the redb episode index | `fix/redb-recommendation-index-order` |
| W5 | #1075 | R04 | deterministic query-aware lexical fallback (replaces the `0.5` constant) | `fix/pattern-lexical-fallback` |
| W6 | #1085 | M04 | bounded real backend probes + live cache/sync/uptime + URL redaction (E1) | `fix/mcp-health-probes` |

Deliberately **out of this wave**, with reason recorded:

- **#1067 → #1068 → #1089** — a modification watermark means an `Episode` field + Turso column + redb
  `SCHEMA_VERSION` bump. Highest-value structural fix, but it must be its own sequenced PR with migration
  tests, not folded into a hygiene wave.
- **#1073 → #1076** — the identity-scoped embedding adapter changes key shape in both backends
  (`redb embeddings_backend.rs:23,33,101,205` slices `&key[8..]`; turso tags by `"episode"|"pattern"`), which
  breaks `find_similar_*` until done together. #1076 must not precede it (E2).
- **#1064** — 85 call sites; additive `with_connection` first, migration after.
- **#1087 / #1091 / #1092** — each needs 3–4 atomic PRs; split proposed in the backlog table below.
- **#1078, #1079, #1070, #1071, #1063, #1065, #1074** — queued behind W1–W6.

## COORDINATE — execution model

- Worktree per slice under `../worktrees/` (never on `main`), one conventional commit per slice, PR per slice.
- W1–W5 are file-disjoint → implementation may proceed in parallel; **builds may not** (single 8-core box,
  no `target/` at wave start), so compilation/tests are serialized through the shared target dir.
- Read-only review agents run against the diff for each slice; every finding must be verified against source
  before it is acted on.
- Gates per slice: `./scripts/code-quality.sh fmt`, `clippy --workspace`, `./scripts/build-rust.sh check`,
  `cargo nextest run --all`, `cargo test --doc`, `./scripts/quality-gates.sh`, `do-harness verify --record`,
  then `./scripts/check-pr-readiness.sh <PR>`; merge only via `./scripts/merge-pr.sh <PR> --execute`.
- Trackers are updated **only** by W0 and the closing refresh, so code PRs stay atomic and conflict-free.

## STATUS — wave progress (2026-10-05/06)

| Slice | Issue | PR | Status |
|-------|-------|----|--------|
| W0 | trackers + this doc | #1129 (`26b12fc8`) | ✅ merged 2026-10-05 |
| W1 | #1077 | #1131 (`874df209`) | ✅ merged 2026-10-05 |
| W2 | #1086 | #1135 (`75007b51`) | ✅ merged 2026-10-05 |
| W3 | #1088 | #1134 (`347b3296`) | ✅ merged 2026-10-05 |
| W4 | #1066 | #1139 (`f1c31699`, `0485bb66`) | ✅ merged 2026-10-05 — includes an open-time repair pass that heals stale write-order winners without a schema bump |
| W5 | #1075 | #1138 | 🔄 in review — required checks green; cancelled non-required workflows re-running (2026-10-06) |
| W6 | #1085 | #1140 | 🔄 in review — cancelled non-required workflows re-running; aggregate failed closed on the cancelled set, as designed |
| Q02 | #1091 | #1130 (jules → clean branch) | 🔄 rescued: the PR tip (`da0c867c`) reverted W1–W4, plans and scripts; a clean branch re-applies only `b9331ce3`'s five files and removes the `continue-on-error`/`\|\| true` silent-pass before force-push |

Adjacent merges in the same window: #1128 docs-integrity fix, #1133 local hook sensors, #1136 dependabot actions bump.

## SYNTHESIZE — success criteria

1. `plans/` no longer contradicts `gh issue list`: every one of the 22 issues appears with a verdict, evidence
   and a queue position. ✅ (#1129 + this refresh)
2. W1–W6 merge with green CI, each closing (or advancing) a numbered issue with test evidence. 🔄 W1–W4 done;
   W5/W6/Q02 in review.
3. E1 (URL redaction) is fixed and regression-tested. 🔄 in #1140.
4. The remaining chain order (#1067→#1068→#1089, #1073→#1074→#1076) is written down with its migration rationale, so
   the next wave does not have to re-derive it. ✅ (ACT-376…ACT-381).
5. Next-wave execution follows this doc's queue: independent slices in parallel worktrees; watermark and
   embedding chains sequenced; release `v0.1.45` (#1137) after the wave is green.

