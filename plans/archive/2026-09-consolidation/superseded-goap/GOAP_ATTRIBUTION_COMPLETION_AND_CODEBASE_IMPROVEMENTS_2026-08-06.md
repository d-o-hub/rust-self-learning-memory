# GOAP — Attribution Completion & Codebase Improvements (2026-08-06)

- **Date**: 2026-08-06
- **Workspace version**: `0.1.38` · latest tag `v0.1.37`
- **Branch analysed**: `feat/adr-080-automatic-recommendation-attribution`
- **Working tree**: 15 modified files, 1 new file (`memory-core/src/memory/attribution/receipt.rs`), **+832 / −176**
- **Build state**: `cargo check --workspace --all-targets` → **exit 0** (verified 2026-08-06)
- **Governing ADRs**: [ADR-080](adr/ADR-080-Automatic-Recommendation-Attribution.md) (Proposed), [ADR-081](adr/ADR-081-Attribution-Capability-Truth-And-Feedback-Resolution.md) (Proposed, this analysis)
- **Scope of this document**: planning only. No source files are modified by this plan.

---

## 1. World state

### 1.1 What the working tree already delivers

Nine of ADR-080's decisions are implemented and compile cleanly. Detailed inventory:

| # | Change | Location | Detail |
|---|--------|----------|--------|
| 1 | New module `attribution::receipt` | `memory-core/src/memory/attribution/receipt.rs` (new, 199 LOC) | `PersistenceReceipt` (4 variants, `#[serde(tag = "state", rename_all = "snake_case")]`), `AttributedPatternResult<R>`, `AttributedPlaybookResult<P>`, accessors `session_id()` / `episode_id()` / `is_durable()` / `is_restart_safe()`, 5 unit tests |
| 2 | Module wiring | `attribution/mod.rs:63,67` | `mod receipt;` + `pub use receipt::{AttributedPatternResult, AttributedPlaybookResult, PersistenceReceipt};` |
| 3 | Public re-export | `memory-core/src/lib.rs:307-310` | Adds `AttributedPatternResult`, `AttributedPlaybookResult`, `PersistenceReceipt` to the crate root |
| 4 | Multi-session index | `attribution/tracker.rs:55-56, 93-100` | `episode_sessions: Arc<RwLock<HashMap<Uuid, Uuid>>>` → `HashMap<Uuid, Vec<Uuid>>`; `insert` → `entry().or_default().push()` |
| 5 | Deterministic latest lookup | `attribution/tracker.rs:187-207` | `get_session_for_episode` picks `max_by(timestamp, then session_id.as_bytes())` |
| 6 | New accessor | `attribution/tracker.rs:209-233` | `get_all_sessions_for_episode` returns all sessions sorted oldest-first with the same tie-break |
| 7 | Feedback integrity | `attribution/tracker.rs:129-166` | Unknown session → `Error::InvalidInput`; applied IDs must be a subset of `recommended_pattern_ids`; replacement overwrites (idempotent) |
| 8 | Checked persistence | `memory-core/src/memory/persistence.rs:192-266` | `persist_session_checked(&RecommendationSession) -> PersistenceReceipt`; counts configured backends, tallies success/failure, emits the 4-state receipt with stable backend identifiers `"turso"` / `"redb"` |
| 9 | Attributed pattern op | `memory-core/src/memory/pattern_api.rs:142-186` | `recommend_patterns_attributed(episode_id, task_description, context, limit)`; nil-episode rejection; session built from `r.pattern.id()` of the exact returned results |
| 10 | Attributed playbook op | `memory-core/src/memory/retrieval/playbooks.rs:171-236` | `retrieve_playbooks_attributed(...)`; nil-episode rejection; dedupes supporting pattern IDs; records both pattern and playbook IDs |
| 11 | Hidden nil session removed | `memory-core/src/memory/retrieval/playbooks.rs` (−16 lines) | The `Uuid::nil()` side-effect session in `retrieve_playbooks` is deleted (ADR-080 §2) |
| 12 | MCP attribution envelope | `memory-mcp/src/mcp/tools/pattern_search.rs:37-43, 54-56, 225-227, 245-268` | `AttributionEnvelope { session_id, episode_id, receipt }`; `SearchPatternsOutput.attribution: Option<_>` with `skip_serializing_if`; `RecommendPatternsInput.episode_id: Option<Uuid>` |
| 13 | MCP playbook branch | `memory-mcp/src/bin/server_impl/tools/feature_handlers.rs:147-186` | Optional attributed retrieval; audit log now uses a `playbook_count` captured from either branch |
| 14 | CLI flags | `pattern/core/types.rs:92-94`, `playbook/types.rs:39-41` | `--episode-id <UUID>` on `pattern recommend` and `playbook recommend`, threaded through `dispatch_pattern.rs:55,63` and `playbook/dispatch.rs:23,34` |
| 15 | CLI attributed rendering | `pattern/core/search.rs:154-290`, `playbook/commands.rs:42-...` | Hard-errors on malformed UUID; human mode prints an `--- Attribution Tracking (ADR-080) ---` block with per-state durability wording; JSON/YAML emit the full attributed envelope |
| 16 | Tests added | `tracker.rs:408-512`, `receipt.rs:126-199`, `pattern_search.rs:312,315` | 4 tracker integrity tests, 5 receipt tests, input-default assertions |
| 17 | Snapshot updated | `memory-mcp/tests/snapshot_tests.rs:590` | `attribution: None` added to the existing search snapshot |

### 1.2 Verified gaps

Each row was confirmed by reading current source, not by inference.

| ID | Severity | Gap | Evidence |
|----|----------|-----|----------|
| **G1** | **Blocker — regression** | Feedback resolves only the in-process map; no storage hydration. Because unknown-session now hard-errors, feedback against a durable session **fails after restart** where it previously succeeded | `tracker.rs:133-142` resolves from `self.sessions` only; `api.rs:176-185` calls it without hydration; contrast `api.rs:191-205` which *does* fall back to storage for reads |
| **G2** | **Blocker** | Capability assumed, not advertised. Any configured backend counts as a write even though `store_recommendation_session` has a default no-op `Ok(())` → a non-persisting backend yields `Persisted` | `persistence.rs:203-244` vs `memory-core/src/storage/backend.rs:374-377`; 18 such no-op defaults exist in `backend.rs` |
| **G3** | **Blocker** | Nonexistent episode IDs create orphan sessions; only `is_nil()` is checked | `pattern_api.rs:153`, `playbooks.rs:189` |
| **G4** | **Blocker** | MCP silently drops a malformed `episode_id` and returns an unattributed result as success; CLI hard-errors — the surfaces disagree | `feature_handlers.rs:147-150` (`.ok()`) vs `search.rs:154-160` (`map_err`) |
| **G5** | **Blocker** | Playbook generation failure is indistinguishable from valid emptiness, so a *failed* generation records an empty session as a real abstention | `playbooks.rs:161-165` returns `vec![]` on `Err`; consumed by `playbooks.rs:196-202` |
| **G6** | High | Manual session/feedback paths stay warning-only and report unconditional `success: true` | `api.rs:129-134` → `persistence.rs:17-38`; `recommendation_feedback/tool.rs:69-90`; CLI `feedback/core.rs` |
| **G7** | High | No `persist_feedback_checked`; feedback writes have no receipt at all | `persistence.rs:41-60` is warning-only with no checked counterpart |
| **G8** | **Gate blocker** | `attribution/tracker.rs` is **557 LOC**, over the hard 500-LOC gate | `wc -l` |
| **G9** | High | `episode_id` absent from both MCP tool schemas — clients cannot discover it from `tools/list` | `server/tool_definitions.rs:171-200`; `server/tools/registry/builder.rs:109-155` |
| **G10** | Medium | Two independent tool registries declare overlapping tools with no agreement test | same two files |
| **G11** | Medium | `docs/API_REFERENCE.md` (202 LOC) names both tools at lines 90-91 with no attribution parameters or receipt semantics | `docs/API_REFERENCE.md:90-91` |
| **G12** | Medium | No restart-safety test, no receipt-state matrix against real backends, no MCP snapshot of a populated envelope, no CLI e2e for `--episode-id` (repo gate ≥ 90%) | `snapshot_tests.rs:590` adds only `attribution: None` |
| **G13** | Medium | ~130 lines of human-mode rendering duplicated between the attributed and unattributed branches; new `#[expect(clippy::too_many_arguments)]` added | `search.rs:157-290` vs `291-395`; `playbooks.rs:175` |
| **G14** | Low — latent | `PersistenceReceipt` is internally tagged (`#[serde(tag = "state")]`) and re-exported from `lib.rs`, while sessions are postcard-serialized in redb. Standing rule: never `#[serde(tag=)]` on postcard types | `receipt.rs:22`; `memory-storage-redb/src/recommendations.rs:22` |

**ADR-080 acceptance scorecard: 6 of 12 met.** Failing: malformed/nonexistent
rejection, error-must-not-create-session, no-op-default exclusion, manual-command
checked semantics, restart retrievability, documentation.

### 1.3 Findings outside ADR-080

| ID | Severity | Finding | Evidence |
|----|----------|---------|----------|
| **X1** | Medium | Capability blindness is systemic, not attribution-specific: 18 no-op `Ok(())`/`Ok(None)`/`Ok(Vec::new())` defaults in `StorageBackend` let any backend silently under-implement | `memory-core/src/storage/backend.rs` |
| **X2** | Medium | 72 clippy suppressions workspace-wide, led by `too_many_arguments` ×19 and `excessive_nesting` ×10. AGENTS.md's steering loop mandates a harness fix after 2 firings in a sprint; this threshold is long exceeded with no skill created | `grep -rn '#\[expect(clippy\|#\[allow(clippy' memory-*/src` |
| **X3** | Medium | Plans registry integrity: ADR numbers **025, 054, 058** duplicated. The ADR-058 duplicate is **untracked** and materially differs from the tracked file (different title, status, date, related-ADR list). `GOAP_STATE.md:85` records `plan_registry_unique ≈ true`, which understates this | `ls plans/adr` + `git ls-files plans/adr`; `diff` of the two ADR-058 files |
| **X4** | Low | Repo hygiene: `memory-cli/default_5961729753683299812_0_277361.profraw`, root `package.json` / `package-lock.json`, `.codex/`, `.commandcode/`, `.claude/commands/` are untracked and unignored (`.gitignore` covers only `node_modules/` at line 17) | `git status --short`; `.gitignore:17` |
| **X5** | Info | Ranking adaptation remains deferred by ADR-080 §5 — the self-learning loop is **not** closed and must not be described as such | ADR-080 §5 |
| **X6** | Info | ACT-325 (R-F10 OIDC publishing) and ACT-326 (R-F4 SIMD cosine) are still in progress and untouched by this analysis | `plans/ACTIONS.md:31-32` |

---

## 2. Goal state

```text
attribution_feedback_restart_safe   = true   # G1
attribution_capability_advertised   = true   # G2
attribution_episode_validated       = true   # G3
attribution_surface_parity          = true   # G4
playbook_error_vs_empty_distinct    = true   # G5
manual_paths_checked                = true   # G6, G7
loc_gate_clean                      = true   # G8
mcp_attribution_discoverable        = true   # G9, G10
attribution_docs_truthful           = true   # G11
attribution_coverage_ge_90          = true   # G12
attribution_no_new_suppressions     = true   # G13
receipt_postcard_safe               = true   # G14
plan_registry_unique                = true   # X3
repo_hygiene_clean                  = true   # X4
feedback_updates_ranking            = false  # X5 — deferred, follow-up ADR required
```

---

## 3. Action plan

Eleven actions, `ACT-344` … `ACT-354`. `ACT-344` is the ADR gate; nothing else
starts until ADR-081 is accepted. Detailed code changes follow in §4.

| ID | Action | Package | Depends on | Priority | Status |
|----|--------|---------|------------|----------|--------|
| ACT-344 | Accept ADR-081 and freeze the completed attribution contract | RAT-B0 | — | **P0** | Proposed — maintainer decision; ADR-081 stays `Proposed` |
| ACT-345 | Split `tracker.rs` under the 500-LOC gate | RAT-B1 | ACT-344 | **P0** | ✅ evidence-backed / landed in closure PR (`attribution/tracker/{mod,integrity,stats,tests}.rs`) |
| ACT-346 | Add `StorageBackend` attribution-capability advertisement; make `persist_session_checked` capability-aware | RAT-B2 | ACT-344 | **P0** | ✅ evidence-backed / landed 2026-08-10 (capability truth wave) |
| ACT-347 | Resolve feedback sessions through memory→storage; remove the restart regression | RAT-B3 | ACT-345, ACT-346 | **P0** | ✅ evidence-backed / landed in closure PR (cold-restart Turso-only + redb-only tests) |
| ACT-348 | Validate episode existence; unify malformed-ID rejection across core/MCP/CLI | RAT-B4 | ACT-346 | **P0** | ✅ evidence-backed / landed in closure PR |
| ACT-349 | Add fallible playbook retrieval; no session on generation failure | RAT-B5 | ACT-346 | **P0** | ✅ evidence-backed / landed in closure PR (`try_retrieve_playbooks` + unit seam test) |
| ACT-350 | Add `persist_feedback_checked`; give manual MCP/CLI commands receipt semantics | RAT-B6 | ACT-346, ACT-347 | P1 | ✅ evidence-backed / landed in closure PR |
| ACT-351 | Declare `episode_id` in both MCP registries + registry-agreement test | RAT-B7 | ACT-348 | P1 | ✅ evidence-backed / landed in closure PR |
| ACT-352 | Deduplicate CLI rendering; replace the `too_many_arguments` suppression with a request struct | RAT-B8 | ACT-348, ACT-349 | P1 | ✅ evidence-backed / landed in closure PR (`AttributedPlaybookRequest`) |
| ACT-353 | Restart-safety, receipt-matrix, MCP snapshot, and CLI e2e tests to ≥ 90% | RAT-B9 | ACT-345…352 | P1 | ✅ evidence-backed / landed in closure PR; coverage % measured by the controller's final validation |
| ACT-354 | Docs + plans-registry + repo hygiene (`API_REFERENCE`, ADR-058 duplicate, `.gitignore`) | RAT-B10 | ACT-353 | P2 | ✅ evidence-backed / landed in closure PR |

Closure PR: branch `fix/ci-attribution-truth-closure` (PR number / head SHA
recorded by the controller after creation). All rows above were "planned" in the
original wave; they are evidence-backed once the closure PR's tests pass under
the controller's final validation.

Execution strategy: **sequential** for ACT-344→347 (they share `persistence.rs` and
`tracker.rs`); **parallel** for {ACT-348, ACT-349} and {ACT-351, ACT-352} once their
dependencies land; ACT-353 is the convergence gate.

---

## 4. Detailed code changes

> Line anchors are against the **current working tree** and will shift as edits land.

### ACT-345 — Split `tracker.rs` (G8)

`memory-core/src/memory/attribution/tracker.rs` is 557 LOC. Split by responsibility,
not arbitrarily, following the repo's `file.rs → file/mod.rs + submodule.rs` convention:

| New file | Contents | Approx LOC |
|----------|----------|-----------|
| `attribution/tracker/mod.rs` | `RecommendationTracker` struct, `new`/`Default`, `record_session`, session/feedback getters, `get_all_sessions_for_episode` | ~180 |
| `attribution/tracker/integrity.rs` | `record_feedback` and the ADR-080 §4 integrity rules (unknown-session rejection, applied ⊆ recommended, idempotent replacement) | ~90 |
| `attribution/tracker/stats.rs` | `get_stats` aggregation | ~80 |
| `attribution/tracker/tests.rs` | All `#[tokio::test]` cases, including the 4 new ones | ~210 |

Do **not** nest `mod tests` inside `tests.rs` (project convention). `attribution/mod.rs`
keeps `pub use tracker::RecommendationTracker;` unchanged — no public API movement.

### ACT-346 — Capability advertisement (G2)

**`memory-core/src/storage/backend.rs`** — add above the ADR-044 block at line 371:

```rust
/// Whether this backend actually persists recommendation attribution (ADR-081 §2).
///
/// Defaults to `false` so that the no-op default implementations below are never
/// counted as a durable write. Backends overriding the recommendation methods
/// must override this to `true`.
fn supports_recommendation_attribution(&self) -> bool {
    false
}
```

Keep the four no-op defaults at `backend.rs:374-407` exactly as they are — the `false`
default makes them unreachable as evidence of a write, and leaving them keeps the change
non-breaking for external implementors.

**Overrides to add** (each returns `true`):

| Crate | File | Note |
|-------|------|------|
| `do-memory-storage-turso` | `src/trait_impls/mod.rs` (near line 165) | Real implementation present |
| `do-memory-storage-turso` | `src/resilient.rs` (near line 271) | Must **delegate to the inner backend**, not hardcode `true`, so a wrapped incapable backend stays incapable |
| `do-memory-storage-redb` | `src/backend_impl.rs` (near line 143) | Real implementation present |
| `do-memory-storage-redb` | `src/redb_cache.rs` (near line 113) | Second `impl StorageBackend for RedbStorage` — both must be updated |

**`memory-core/src/memory/persistence.rs:192-266`** — rewrite the capability test in
`persist_session_checked`:

```rust
// before
let turso_configured = self.turso_storage.is_some();
let redb_configured  = self.cache_storage.is_some();

// after
let turso_capable = self.turso_storage.as_ref()
    .is_some_and(|s| s.supports_recommendation_attribution());
let redb_capable  = self.cache_storage.as_ref()
    .is_some_and(|c| c.supports_recommendation_attribution());
```

`MemoryOnly` is then returned when **no capable** backend exists (not merely when no
backend is configured), and each write arm is guarded by its capability flag. The
`configured` tally at `persistence.rs:246` becomes a `capable` tally. Backend identifier
strings `"turso"` / `"redb"` are unchanged — they are the stable wire identifiers ADR-080
§3 requires and must not become raw error text.

> MSRV note: the repo targets 1.70.0. `Option::is_some_and` is stable since 1.70 and is
> safe here; `is_none_or` is **not** (1.82) and must not be used.

### ACT-347 — Storage-resolved feedback (G1, the regression)

**`memory-core/src/memory/attribution/tracker/integrity.rs`** (post-ACT-345) — keep
`record_feedback` as the pure in-memory integrity check. Add a hydration entry point so
the memory layer can seed a storage-loaded session before validation:

```rust
/// Insert a session resolved from durable storage into the in-memory index (ADR-081 §1).
pub(crate) async fn hydrate_session(&self, session: RecommendationSession) { /* … */ }
```

`hydrate_session` reuses `record_session`'s indexing but must be idempotent — pushing the
same `session_id` twice into `episode_sessions` would corrupt the latest-lookup ordering,
so it checks membership before pushing.

**`memory-core/src/memory/persistence.rs`** — add alongside the existing
`fetch_session_for_episode_from_storage` (line ~62):

```rust
pub(crate) async fn fetch_session_by_id_from_storage(
    &self,
    session_id: Uuid,
) -> Option<RecommendationSession>
```

Chain: Turso → redb, first hit wins, capability-gated the same way as ACT-346.
`StorageBackend::get_recommendation_session` already exists (`backend.rs:380-387`).

**`memory-core/src/memory/api.rs:176-185`** — `record_recommendation_feedback` becomes:

1. If `recommendation_tracker.get_session(session_id)` is `None`, call
   `fetch_session_by_id_from_storage`; on a hit, `hydrate_session` it.
2. Call `recommendation_tracker.record_feedback(...)` — unchanged rejection semantics,
   now firing only after the chain is exhausted.
3. Persist via `persist_feedback_checked` (ACT-350).

This is the whole regression fix: the hard rejection at `tracker.rs:139-142` stays,
because it is correct; what changes is that it can no longer fire for a durable session.

### ACT-348 — Episode validation and surface parity (G3, G4)

**`memory-core/src/memory/pattern_api.rs:153`** and
**`memory-core/src/memory/retrieval/playbooks.rs:189`** — after the existing `is_nil()`
guard, add an existence check via the already-present episode lookup
(`SelfLearningMemory::get_episode`), returning
`Error::InvalidInput("Attributed … requires an existing episode; {episode_id} not found")`.

Both call sites share the guard — extract it once as a private helper on
`SelfLearningMemory` (suggested: `memory-core/src/memory/attribution/guard.rs`) so the
two attributed operations cannot drift.

**`memory-mcp/src/bin/server_impl/tools/feature_handlers.rs:147-150`** — replace the
silent-degradation parse:

```rust
// before — malformed UUID silently becomes an unattributed success
let episode_id = args.get("episode_id")
    .and_then(|v| v.as_str())
    .and_then(|s| uuid::Uuid::parse_str(s).ok());

// after — absent stays unattributed; malformed is an error (ADR-081 §3)
let episode_id = match args.get("episode_id").and_then(|v| v.as_str()) {
    Some(s) => Some(uuid::Uuid::parse_str(s)
        .map_err(|e| anyhow::anyhow!("Invalid episode_id '{s}': {e}"))?),
    None => None,
};
```

`RecommendPatternsInput.episode_id: Option<Uuid>` (`pattern_search.rs:225-227`) already
gets this for free — serde rejects a malformed UUID at deserialization. Only the
hand-rolled playbook parse is defective. The CLI paths
(`search.rs:154-160`, `playbook/commands.rs:42-48`) are already correct and need no change.

### ACT-349 — Playbook error vs. empty (G5)

**`memory-core/src/memory/retrieval/playbooks.rs`** — introduce the fallible path and
make the existing signature a shim:

```rust
/// Fallible playbook retrieval (ADR-081 §4). Generation failure is an error;
/// a valid empty result is `Ok(vec![])`.
pub async fn try_retrieve_playbooks(/* … */) -> Result<Vec<RecommendedPlaybook>>;

/// Compatibility shim — maps any error to an empty vector, exactly as today.
pub async fn retrieve_playbooks(/* … */) -> Vec<RecommendedPlaybook> {
    self.try_retrieve_playbooks(/* … */).await.unwrap_or_default()
}
```

The `Err(e) => { debug!(…); vec![] }` arm at `playbooks.rs:161-165` moves into the shim;
`try_retrieve_playbooks` propagates the generator error.

`retrieve_playbooks_attributed` (`playbooks.rs:196-202`) switches to
`try_retrieve_playbooks(...).await?` — so **generation failure returns `Err` before any
session is created**, while a legitimate empty result still creates the empty session
ADR-080 §2 requires for abstention measurement.

### ACT-350 — Checked manual paths (G6, G7)

**`memory-core/src/memory/persistence.rs`** — add `persist_feedback_checked`, mirroring
`persist_session_checked` exactly (same capability gating, same four states, same
`"turso"` / `"redb"` identifiers) over `store_recommendation_feedback`.

**`memory-core/src/memory/api.rs`** — add checked variants and demote the existing
functions to delegating shims:

| Existing (kept, source-compatible) | New |
|---|---|
| `record_recommendation_session(session) -> ()` (`api.rs:129`) | `record_recommendation_session_checked(session) -> PersistenceReceipt` |
| `record_recommendation_feedback(feedback) -> Result<()>` (`api.rs:176`) | `record_recommendation_feedback_checked(feedback) -> Result<PersistenceReceipt>` |

The shims call the checked variants and discard the receipt — one implementation, not two.

**`memory-mcp/src/mcp/tools/recommendation_feedback/types.rs`** — add to both
`RecordRecommendationSessionOutput` (line ~110) and `RecordRecommendationFeedbackOutput`
(line ~86):

```rust
/// Durability state of the attribution write (ADR-081 §5).
pub receipt: do_memory_core::PersistenceReceipt,
```

**`memory-mcp/src/mcp/tools/recommendation_feedback/tool.rs:69-90`** — `success` becomes
`receipt.is_durable()` rather than the current unconditional `true`, and `message`
reports the state. Same treatment for `record_feedback`.

**`memory-cli/src/commands/feedback/core.rs`** — `RecordSessionResult` /
`RecordFeedbackResult` gain a `receipt` field; their `write_human` implementations print a
durability line reusing the wording already established at `search.rs:266-290`
(`Persisted` / `⚠️ Partially Persisted` / `⚠️ Memory-only` / `❌ Persistence Failed`).
Extract that match into one shared helper rather than writing it a third time.

### ACT-351 — Schema discoverability + registry agreement (G9, G10)

Add to **both** registries:

- `memory-mcp/src/server/tool_definitions.rs:171-200` — `recommend_patterns`
- `memory-mcp/src/server/tools/registry/builder.rs:109-155` — `recommend_playbook`

```json
"episode_id": {
  "type": "string",
  "format": "uuid",
  "description": "Optional episode ID. When supplied, the call is attributed: the response adds an `attribution` envelope with session_id and a persistence receipt (persisted | partially_persisted | memory_only | persistence_failed). Omit for the unattributed legacy response shape."
}
```

`episode_id` stays out of `required`. Note that `tool_definitions.rs:183` and
`:213` contain pre-existing brace/indentation damage in the JSON literals — worth
repairing while editing, but it is cosmetic and does not affect the emitted JSON.

New test in `memory-mcp/tests/`: for every tool name present in both registries, assert
name, description, and `input_schema` are identical. This is the guard that keeps the
dual-source-of-truth defect from silently returning; merging the registries is
explicitly deferred (ADR-081 §6).

### ACT-352 — CLI dedup and suppression removal (G13)

**`memory-cli/src/commands/pattern/core/search.rs`** — the attributed branch
(157-290) and unattributed branch (291-395) print byte-identical pattern bodies. Extract:

```rust
fn print_pattern_recommendations(results: &[PatternSearchResult], task_description: &str);
fn print_persistence_receipt(receipt: &PersistenceReceipt);
```

Both branches then call the shared printers; the attributed branch additionally calls
`print_persistence_receipt`. This removes ~130 duplicated lines and keeps the file well
under the 500-LOC gate as it grows.

**`memory-core/src/memory/retrieval/playbooks.rs:175`** — delete
`#[expect(clippy::too_many_arguments)]` and introduce a request struct rather than
suppressing lint number 20 of that kind:

```rust
pub struct AttributedPlaybookRequest {
    pub episode_id: Uuid,
    pub task_description: String,
    pub domain: String,
    pub task_type: TaskType,
    pub context: TaskContext,
    pub max_playbooks: usize,
    pub max_steps_per_playbook: usize,
}
```

`retrieve_playbooks_attributed(&self, request: AttributedPlaybookRequest)` then takes two
arguments. Re-export the struct from `lib.rs` per the project's new-public-type rule.
Update the two call sites: `feature_handlers.rs:155-165` and
`playbook/commands.rs:50-60`.

Per the AGENTS.md steering loop, `clippy::too_many_arguments` has now fired far past the
2-per-sprint threshold (19 existing suppressions). ACT-352 also files the metrics event
at `.agents/events/2026/08/06/too-many-arguments-<ts>.json` and opens the question of a
`.agents/skills/` guide for the config-struct pattern — **the skill itself is out of scope
here** and is left as an explicit follow-up rather than smuggled into an attribution change.

### ACT-353 — Test and coverage closure (G12)

| Test | Location | Asserts |
|------|----------|---------|
| Restart safety — Turso | `memory-storage-turso/tests/` | Persist session → drop `SelfLearningMemory` → rebuild with cold tracker → feedback **accepted** (ADR-081 AC-1) |
| Restart safety — redb | `memory-storage-redb/tests/` | Same, redb-only |
| Truly-unknown session | `memory-core` | Session in no backend and no tracker → `InvalidInput` (AC-2) |
| Capability exclusion | `memory-core` | Backend that does not advertise capability → `MemoryOnly`, never `Persisted` (AC-3) |
| Receipt matrix | `memory-core` | Both OK → `Persisted`; one fails → `PartiallyPersisted` + correct identifier; both fail → `PersistenceFailed`; recommendations intact in all four (AC-4) |
| Episode validation | `memory-core` | nil / malformed / nonexistent rejected on all three surfaces (AC-5) |
| MCP malformed `episode_id` | `memory-mcp` | Tool error, never an unattributed success (AC-6) |
| Playbook failure vs. empty | `memory-core` | Generator error → `Err`, **zero** sessions created; valid empty → empty session created (AC-7) |
| MCP populated snapshot | `memory-mcp/tests/snapshot_tests.rs` | A snapshot with a populated `AttributionEnvelope` — the current addition at line 590 only covers `attribution: None` |
| Registry agreement | `memory-mcp/tests/` | Duplicated tools identical across registries (AC-9) |
| Postcard guard | `memory-core` | No postcard-serialized type transitively contains `PersistenceReceipt` (AC-12) |
| CLI e2e | `memory-cli/tests/` | `pattern recommend --episode-id` and `playbook recommend --episode-id` in human/JSON/YAML |

Gate: `./scripts/quality-gates.sh` with `QUALITY_GATE_COVERAGE_THRESHOLD=90`.

### ACT-354 — Docs, registry, hygiene (G11, G14, X3, X4)

**`docs/API_REFERENCE.md`** (currently 202 LOC; both tools named at lines 90-91) — add an
attribution section: the optional `episode_id`, the four receipt states with their exact
JSON discriminants, the attributed vs. unattributed response shapes, and an explicit
statement that this is attribution **capture**, not ranking adaptation (ADR-080 §5).

**`memory-core/src/memory/attribution/receipt.rs:19-22`** — add a doc warning that
`PersistenceReceipt` is JSON-only and must never be embedded in a postcard-serialized
type, cross-referencing the redb session encoding at
`memory-storage-redb/src/recommendations.rs:22`.

**Plans registry (X3)** — the untracked
`plans/adr/ADR-058-CI-Health-Gitleaks-Release-Drift-2026-06-14.md` differs from the
tracked `ADR-058-CI-Health-Gitleaks-Release-Drift.md` in title, status, date, and
related-ADR list. Decide one canonical file, record the other as an alias in
`plans/adr/README.md` alongside the existing 025/054 entries, and correct
`GOAP_STATE.md:85` from `plan_registry_unique ≈ true`. Same for the untracked
`plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-14.md`, whose siblings all live under
`plans/archive/2026-07-consolidation/analyses/`. Validate with
`./scripts/validate-plans.sh --adrs --identifiers`.

**Hygiene (X4)** — `.gitignore` currently covers only `node_modules/` (line 17). Add
`*.profraw`, and decide explicitly whether `package.json` / `package-lock.json`,
`.codex/`, `.commandcode/`, and `.claude/commands/` are tracked or ignored. Delete
`memory-cli/default_5961729753683299812_0_277361.profraw`.

---

## 5. Sequencing and quality gates

```
ACT-344 (ADR-081 accepted)
   │
   ├─► ACT-345 split tracker ──┐
   ├─► ACT-346 capability ─────┼─► ACT-347 feedback resolution (regression fix)
   │                           ├─► ACT-348 episode validation ─┐
   │                           └─► ACT-349 playbook Result ────┤
   │                                                           ├─► ACT-352 dedup + request struct
   ACT-347 ──────────────────► ACT-350 checked manual paths    │
   ACT-348 ──────────────────► ACT-351 schemas + agreement ────┤
                                                               └─► ACT-353 tests ─► ACT-354 docs/hygiene
```

Per-action gate (AGENTS.md change workflow, all mandatory):

```
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace     # zero warnings, -D warnings
./scripts/build-rust.sh check
cargo nextest run -p <crate> && cargo nextest run --all
cargo test --doc
cargo doc --no-deps --document-private-items     # bare-URL check
./scripts/quality-gates.sh                       # coverage >= 90
git status                                       # everything staged
```

Additional gates for this plan: every touched file ≤ 500 LOC (ACT-345 exists solely to
satisfy this), and **no new** `#[expect(clippy::…)]` in any diff.

Commit discipline — one atomic commit per action, `feat(module):` / `fix(module):`:

| Action | Message |
|--------|---------|
| ACT-345 | `refactor(attribution): split tracker.rs into mod/integrity/stats/tests under 500 LOC` |
| ACT-346 | `feat(storage): advertise recommendation-attribution capability on StorageBackend` |
| ACT-347 | `fix(attribution): resolve feedback sessions from storage before rejecting` |
| ACT-348 | `fix(attribution): validate episode existence and reject malformed episode_id on MCP` |
| ACT-349 | `feat(playbooks): add fallible retrieval so generation failure creates no session` |
| ACT-350 | `feat(attribution): return persistence receipts from manual session/feedback commands` |
| ACT-351 | `feat(mcp): declare episode_id in both tool registries and assert registry agreement` |
| ACT-352 | `refactor(cli): deduplicate recommendation rendering and drop too_many_arguments suppression` |
| ACT-353 | `test(attribution): restart-safety, receipt matrix, MCP snapshot, CLI e2e` |
| ACT-354 | `docs(attribution): document episode_id and receipt states; fix plans registry and hygiene` |

ACT-347 is the only entry that must ship as `fix(` — it repairs a regression the current
working tree would otherwise introduce, and the commit message should say so plainly.

---

## 6. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| The working tree is committed/merged before ACT-347 | Feedback breaks after restart for every durable deployment — a live regression | Treat ACT-347 as a merge blocker for the branch; do not open a PR on the current slice alone |
| `supports_recommendation_attribution` missed on one of the four `impl StorageBackend` blocks | Silent `MemoryOnly` where durability exists — the inverse false report | ACT-353 capability test exercises every backend; `resilient.rs` must delegate, not hardcode |
| Episode-existence check adds a read to every attributed call | Latency on attribution paths | Attribution paths only; retrieval hot path untouched; verify against the < 100 ms retrieval target in benches |
| `try_retrieve_playbooks` shim diverges from the original error-swallowing behavior | Silent behavior change for existing callers | Shim is `unwrap_or_default()` — byte-identical to the current arm; assert with a test |
| Splitting `tracker.rs` collides with the in-flight branch | Merge pain | ACT-345 runs first, before the functional actions touch the same file |
| Receipt shape changes after MCP clients adopt it | Wire break | ADR-081 §7 freezes the JSON shape now; any postcard need requires a new representation, not a mutation |

---

## 7. Explicitly out of scope

- **Feedback → ranking adaptation.** Deferred by ADR-080 §5; requires its own ADR
  covering idempotent durable updates, replacement semantics, and rollback. Until then
  the project must not claim a closed learning loop.
- **Generalizing capability advertisement to all 18 `StorageBackend` no-op defaults**
  (X1) — follow-up.
- **Merging the two MCP tool registries** (G10) — ACT-351 only prevents drift.
- **A `.agents/skills/` guide for the `too_many_arguments` steering-loop violation** (X2)
  — ACT-352 files the metrics event and names the follow-up.
- **ACT-325 (R-F10 OIDC) and ACT-326 (R-F4 SIMD)** — in progress, untouched here.

---

## 8. Traceability

| ADR-081 AC | Action | Test |
|-----------|--------|------|
| 1, 2 | ACT-347 | Restart safety ×2, truly-unknown session |
| 3, 4 | ACT-346 | Capability exclusion, receipt matrix |
| 5, 6 | ACT-348 | Episode validation, MCP malformed `episode_id` |
| 7 | ACT-349 | Playbook failure vs. empty |
| 8 | ACT-350 | Manual command receipt assertions |
| 9 | ACT-351 | Registry agreement |
| 10 | ACT-345 | LOC gate in `quality-gates.sh` |
| 11 | ACT-352 | Clippy `-D warnings`, no-new-suppression diff check |
| 12 | ACT-354 | Postcard guard |
| 13 | ACT-353 | `quality-gates.sh` ≥ 90 |
| 14 | ACT-354 | Docs integrity check |
