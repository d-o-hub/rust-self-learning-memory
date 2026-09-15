# GOAP: ADR-081 §2 Attribution Capability Truth (2026-08-10)

> **HISTORICAL — completed slice.** Superseded by the closure PR from branch
> `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the
> controller after creation). Retained for history per ADR-039; do not treat as
> active backlog.

- **Date**: 2026-08-10
- **Branch**: `feat/adr081-capability-truth`
- **Goal**: Implement ADR-081 §2 / acceptance criterion 3 — attribution capability
  advertisement — the genuinely-open missing task in the current plans. Ranked
  adaptation remains deferrED by ADR-081 §8 / ADR-080 §5 and is NOT in scope.
- **Orchestrator**: GOAP hybrid — parallel swarm for file-disjoint crates, then
  controller synthesis + quality gates.
- **ADR context**: ADR-081 §2 (capability truth), ADR-080 §3 (checked receipt
  never counts no-op default as a write), ADR-075 (durability truth).

## Root-cause (verified this session)

`StorageBackend` has no capability query. `persist_session_checked`
(`memory-core/src/memory/persistence.rs:232-298`) decides durability purely from
`turso_storage.is_some()` / `cache_storage.is_some()`, then counts any backend's
`Ok(())` as a successful write. Because the trait default
`store_recommendation_session` returns `Ok(())` no-op (`backend.rs:374-377`), a
configured backend that does not actually persist still yields `Persisted` — a
false durability claim. ADR-081 §2 requires the receipt to count only
advertised-capable backends.

## Goal hierarchy

| Goal | Success criteria |
|------|------------------|
| G1 capability_trait | `StorageBackend::supports_recommendation_attribution()` defaults `false`; Turso + redb advertise `true` (all impl blocks); resilient/cached wrappers delegate to inner |
| G2 persist_checked | `persist_session_checked` gates each write arm and the MemoryOnly/Persisted tally on capability; a non-advertising configured backend never yields `Persisted`; no capable backend → `MemoryOnly` |
| G3 receipt_matrix | `attribution_receipt_matrix.rs` + `attribution_feedback_restart.rs` updated to advertize capability where a mock truly persists; AC-3 test (non-advertising backend → MemoryOnly, never Persisted) added |
| G4 green | workspace builds, clippy clean, nextest + doctests pass, coverage ≥ 90% across changed crates |
| G5 plan_truth | GOAP_STATE two stale flags flipped; GAP/GOALS/CURRENT + new plan file updated |

## Tasks

### Phase 1 — SWARM (parallel, file-disjoint) — ✅ ALL COMPLETE
| Task | Agent | Files | Output |
|------|-------|-------|--------|
| T1 core capability + persist rewrite + tests | task | `memory-core/src/storage/backend.rs`, `memory-core/src/memory/persistence.rs`, `memory-core/tests/attribution_receipt_matrix.rs`, `memory-core/tests/attribution_feedback_restart.rs` | ✅ method + rewrite + AC-3 test |
| T2 turso overrides | task | `memory-storage-turso/src/trait_impls/mod.rs`, `memory-storage-turso/src/resilient.rs`, `memory-storage-turso/src/cache/wrapper_backend.rs` | ✅ `true` / delegate |
| T3 redb overrides | task | `memory-storage-redb/src/backend_impl.rs`, `memory-storage-redb/src/redb_cache.rs` | ✅ `true` |

**Cross-task contract**:
- Method name: `fn supports_recommendation_attribution(&self) -> bool` (default `false`).
- Turso `ResilientStorage` + `CachedTursoStorage` must **delegate** to their inner
  backend (`self.storage.supports_recommendation_attribution()`), never hardcode
  `true`; plain `TursoStorage` impl returns `true`.
- Redb `backend_impl.rs` + `redb_cache.rs` (both `impl StorageBackend for RedbStorage`)
  return `true` only on the block that couples to real redb persistence.
- No validation (build/lint/test) inside tasks — controller runs gates after all land.
- Only the named task's files may be edited.

### Phase 2 — SEQUENTIAL (controller)
- T4 `cargo check --workspace` → resolve integration issues.
- T5 Non-advertising-receipt test + matrix green; clippy/fmt.
- T6 Quality gates (`./scripts/code-quality.sh`, nextest, doctests, coverage >= 90%).
- T7 Plan truth: flip GOAP_STATE `release_dispatch_truthful`,
  `informational_ci_evidence_durable`; update GAP/GOALS/CURRENT/ROADMAP.
- T8 Commit + push + open PR.

## Cross-task contracts / acceptance
- Every `impl StorageBackend` in-tree that touches real recommendation
  persistence advertises capability; no-op mocks (`InertBackend`, `StubBackend`,
  `MockStorage`) keep the default `false`.
- MSRV-safe Rust (no `is_none_or`); `is_some_and` is fine.
- Receipt backend identifiers remain `"turso"` / `"redb"` (ADR-080 §3 wire contract).
- No `#[serde(tag = ...)]` on postcard types introduced; receipts stay JSON-only.
