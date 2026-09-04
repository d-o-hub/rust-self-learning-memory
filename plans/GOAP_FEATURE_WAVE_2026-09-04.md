# GOAP Feature Wave — Retrieval Confidence, Ranking Bounds, Storage Batching, Compact Handoff, Observability

- **Date**: 2026-09-04
- **Issues**: #968 (retrieval fallback) · #966 (ranking bounds) · #967 (storage batching) · #965 (compact handoff) · #962 (observability)
- **Branch track**: one atomic branch/PR per issue; this doc rides with the #968 PR and is normative for the rest.
- **Related ADRs**: ADR-075 (durability), ADR-077 (embedding activation), ADR-082 (ranking adaptation), ADR-074 (provenance/redaction)

## D1 — ADR-075 amendment for #967 (decision)

ADR-075 requires `complete_episode` to return a hard error when **any** configured
backend fails a durable write. A background write queue cannot satisfy that
contract literally and still move remote I/O off the completion path.

**Decision**: split durability into two tiers (implemented in the #967 PR):

1. **Local durability stays synchronous and hard-error.** `redb`/cache writes
   remain in the completion call; any failure returns `Err` exactly as today.
   Read-after-write and the CLI verify-after-write rule are unchanged.
2. **Remote (Turso) durability becomes journal-backed eventual.** Completion
   enqueues the write to a bounded queue (`QuotaExceeded` backpressure, modeled
   on `PatternExtractionQueue`), returns `Ok` once local state commits, and a
   worker performs transactional batch commits with retry/backoff. Failures go
   to the `OperationJournal` (new `JournalOpKind::EpisodeComplete`) and are
   reconciled via an explicit `flush`/`drain` path used by CLI shutdown, tests,
   and operators.

`completion_durability.rs` / `hybrid_storage_recovery.rs` assertions move to the
new contract: local-failure → `Err`; remote-failure → `Ok` + journal entry +
`flush()` surfaces the error. If the maintainer rejects this split, #967 falls
back to intra-call batching only (smaller win, no queue).

## D2 — `retrieval.csm` configuration (decision)

Issue #968 proposes `[retrieval.csm]` TOML keys. The cascade is currently
constructed only by the eval harness and tests — there is **no production call
path** — so file-based keys would be an advertised non-operation (the PTA-A3
anti-pattern).

**Decision**: the policy lives on `CascadeConfig` (public library API with
documented defaults) and is exercised by the eval baseline runner
(`AlwaysEmbed`/`LocalOnly`/`Adaptive` strategies map 1:1 to policies).
`[retrieval.csm]` TOML/CLI wiring is **explicitly deferred** until the cascade
is wired into production query paths, and is documented as such — no TOML keys
are advertised as supported by this wave.

Keys (effective now via API + eval): `fallback_policy = "adaptive" |
"always_embed" | "local_only"` (default `"adaptive"`),
`local_confidence_threshold = 0.78`, `minimum_score_margin = 0.08`.

## D3 — Telemetry contract for #962 (normative)

All wave PRs emit structured `tracing` events; #962 adds counters/dashboards on
top. Event fields use these bounded label values (never raw queries, IDs, tags,
or provider error strings — F4.1/ADR-074 redaction rule):

- `policy`: `adaptive | always_embed | local_only`
- `fallback_reason`: `local_tier_sufficient | local_confident |
  insufficient_confidence | no_local_results | always_embed_policy |
  local_only_policy`
- numeric fields (`top_score`, `score_margin`, latencies, counts) are never
  labels.

`CascadeResult` carries `fallback_reason`, `top_score`, `score_margin` so the
eval harness and (later) #962 counters read decisions without parsing logs.

## Sequencing

1. #968 (this branch): policy + confidence gating + eval wiring.
2. #966: per-tier candidate budgets + post-top-N MMR; reuses candidate counts
   from the eval harness (`avg_candidate_before/after`).
3. #967: bounded write queue per D1. Parallelizable with #966.
4. #965: compact handoff schema + budgets. Parallelizable with #966/#967.
5. #962: counters + dashboard examples consuming D3 events. Last.

## Preconditions (owner: maintainer)

- Ship v0.1.39 → v0.1.40 release drift is `critical` (issue #976); wave PRs
  rebase onto the tagged base.
- Triage colliding queue PRs before #966: #971, #977, #978, #986 touch
  ranking/diversity code. Merge or record supersession on each.
