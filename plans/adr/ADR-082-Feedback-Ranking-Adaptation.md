# ADR-082: Feedback-to-Ranking Adaptation

- **Status**: Proposed — code evidence lands in this PR (`feat/ranking-adaptation`);
  lifecycle stays `Proposed` until maintainer acceptance
- **Date**: 2026-08-12
- **Deciders**: Project maintainers
- **Plan**: this PR (feedback-to-ranking adaptation + ADR-025/054 docs cleanup)
- **Supersedes nothing**; **completes** the deferred exit of [ADR-080 §5](ADR-080-Automatic-Recommendation-Attribution.md) and
  [ADR-081 §8](ADR-081-Attribution-Capability-Truth-And-Feedback-Resolution.md)
- **Related**: ADR-044 (recommendation attribution), ADR-080 (automatic attribution),
  ADR-081 (capability truth / feedback resolution)
- **Code evidence**: `memory-core/src/memory/attribution/ranking.rs`,
  `memory-core/src/memory/ranking.rs`, `memory-core/src/memory/pattern_api.rs`,
  `memory-core/src/memory/pattern_search/recommendation.rs`, `memory-core/src/memory/api.rs`,
  `memory-core/src/memory/types.rs`, `memory-core/src/memory/init.rs`,
  `memory-core/src/storage/backend.rs`, `memory-storage-turso/src/storage/recommendations.rs`,
  `memory-storage-turso/src/trait_impls/mod.rs`, `memory-storage-turso/src/resilient.rs`,
  `memory-storage-turso/src/cache/wrapper_backend.rs`,
  `memory-storage-redb/src/backend_impl.rs`, `memory-storage-redb/src/recommendations.rs`

---

## Status update (2026-08-12)

This PR closes the self-learning loop for recommendation ranking. Attributed
feedback now derives a durable per-pattern learned weight, the recommendation path
re-ranks its candidate pool by base relevance + that weight, and the weight is a pure
deterministic function of durable history (idempotent, replacement-safe,
rollback-safe by rebuild). `RecommendationStats` (`get_recommendation_stats`)
remains unchanged; this ADR only re-ranks future recommendations.

---

## Context

ADR-080 §5 and ADR-081 §8 defer feedback-to-ranking adaptation, and GOAP_STATE
records `feedback_updates_ranking = false`. Today `RecommendationFeedback` updates
only global `RecommendationStats` (`stats.rs` plus duplicated per-backend scans).
Nothing a ranker reads is ever written by attribution feedback, so the self-learning
loop is not closed: the system records which recommendations helped, but never acts
on that signal.

The extension must be:
- **Durable** — survive a process restart, reading only persisted history.
- **Idempotent** — a pure deterministic function of history, so a rebuild always
  converges to the same index.
- **Replacement-safe** — re-submitted feedback for a session supersedes the prior
  record (storage already upserts by `session_id`).
- **Rollback-safe** — the learned state is derived, never a destructive journal;
  dropping it and rebuilding loses nothing but the derived index.
- **Non-breaking** — backends that do not support the read surface and callers that
  do not opt in see identical behavior to today.

## Decision

### 1. Learned weight = Wilson lower bound on success-after-application

For each pattern, feedback evidence reduces to `(recommended, applied, succeeded)`.
The learned weight is the Wilson lower-bound success rate at `z = 1.96`
(`RANKING_WILSON_Z`, matching episode ranking), with zero evidence → `0.0`.
A single success is weighted conservatively (≈0.207), so boost is small until
evidence accumulates.

### 2. Durability via the existing session/feedback stores, behind a capability gate

Two new derived-`StorageBackend` defaults unlock the read surface:
`supports_ranking_adaptation() -> bool` (default `false`) and
`list_recommendation_sessions()` / `list_recommendation_feedback()`
(`Ok(Vec::new())` by default). Turso and redb (and their resilient/cached wrappers,
delegating) advertise `true` and return a full scan of their `recommendation_sessions`
/ `recommendation_feedback` tables. Non-capable backends contribute nothing.

### 3. Latest-feedback-per-session wins (replacement semantics)

Feedback is reduced to the LATEST per `session_id` (map overwrite), mirroring the
storage upsert. Re-submitting feedback for the same session replaces the learned
evidence.

### 4. Recommend-path re-rank: overfetch → boost → truncate

Only `SelfLearningMemory::recommend_patterns_for_task` re-ranks: candidate pool
overfetched `RECOMMEND_OVERFETCH_FACTOR` (3×) so a boosted pattern can enter the
top-N, sorted stable by `base_relevance + LEARNED_BOOST_SCALE * wilson`, then
truncated to `limit`. Search, discovery, and retrieval pass no learned index →
identical to today. Sorting uses `sort_by` with an explicit comparator (no
allocation; one `partial_cmp` per pair, ≤ 3N results).

### 5. Refresh hooks close the loop

`record_recommendation_feedback` (both legacy and checked variants) refresh the
index immediately after persistence, so the next recommendation reflects the new
evidence. Session recording alone does not refresh (sessions carry no success
evidence). The index is lazily loaded from durable history on first recommend
(`ensure_ranking_index_loaded`) and rebuilt from the in-process tracker plus every
capable backend.

### 6. Scope

Statistics (`get_recommendation_stats`) are unchanged. Attributed recommendations
carry the learned re-rank automatically because they call the same
`recommend_patterns_for_task`; the session is built from the IDs actually returned
(the boosted order), keeping captured exposure consistent with what the agent saw.

## Consequences

### Positive

- The self-learning loop closes for recommendation ranking (ADR-080 §5 exit).
- Derived state is rebuildable and rollback-safe; no destructive journal.
- No broad API change: one optional `learned: Option<&RankingIndex>` parameter on a
  crate-internal free function; the public `SelfLearningMemory` surface is unchanged.
- Non-capable backends and empty stores behave exactly as before (nothing learned).

### Negative and trade-offs

- `StorageBackend` grows three default methods; every backend must consider them,
  though the `false`/empty defaults are non-breaking.
- The recommend path pays one extra lock read and up to one `HashMap` lookup per
  candidate; overfetching 3× costs at most 2N extra scored candidates on the
  recommendation path only.
- Wilson lower bound is conservative at low evidence: a single success lifts a
  pattern modestly; the effect is deliberately gradual.

## Alternatives considered

1. **Bayesian update on a per-pattern beta prior.** Rejected: stateful and order-
   dependent, breaking idempotence/rollback-safety and complicating durable storage.
2. **Rewrite recommendation scores in the persisted patterns.** Rejected:
   destructive and not rollback-safe; derivation must not write durable rows.
3. **Re-rank by raw adoption rate or agent rating.** Rejected: small-sample noisy
   and unbounded; the Wilson lower bound is conservative and bounded in `[0,1]`.
4. **Apply the learned weight to generic search/discovery/retrieval.** Rejected:
   scope is recommendation ranking only; the plan keeps those paths identical.
5. **Persist the index as a new table.** Rejected: unnecessary — the index is a pure
   function of history already persisted; a table would need reconciliation and
   invariant maintenance for no benefit.

## Acceptance criteria

1. With no feedback, `recommend_patterns_for_task` returns the baseline relevance
   order identically to today.
2. Success feedback for a session recommending pattern P lifts P to the top of the
   next recommendation (limit 2, two competing patterns).
3. Replacing that session's feedback with a Failure removes the boost and the order
   returns toward baseline.
4. A cold restart (fresh in-memory tracker, same redb file) rebuilds the learned
   index from `list_recommendation_sessions` / `list_recommendation_feedback`, and
   the recommend order still reflects it.
5. A non-capable backend's stored attribution rows are ignored after a cold restart;
   the recommend order is unchanged.
6. Turso and redb advertise `supports_ranking_adaptation() == true` (concrete and
   resilient/cached delegating wrappers) and their `list_recommendation_*` methods
   round-trip every stored entry.
7. All CI in this PR passes; changed-crate coverage intent ≥ 90% (new modules carry
   unit + e2e tests; the coverage gate runs optional).
