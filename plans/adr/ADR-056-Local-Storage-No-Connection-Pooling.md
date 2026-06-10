# ADR-056: Local/In-Memory Turso Storage Disables Connection Pooling and Keep-Alive

- **Status**: 🟢 Accepted
- **Date**: 2026-06-09
- **Deciders**: Project maintainers
- **Related**: ADR-024 (lazy MCP tools), PR #611, Issue #610, GOAP_PR611_CI_FIX_2026-06-09

## Context

PR #611 makes local/offline SQLite a first-class storage path by adding
`StorageMode` and explicit constructors
(`TursoStorage::new_local`, `new_in_memory`, `new_remote`, `from_storage_mode`).

The original constructors delegated to `new()` →
`with_config(url, token, TursoConfig::default())`. The default config enables:

- `enable_pooling = true` → builds a multi-connection `ConnectionPool`
- `enable_keepalive = true` (under the `keepalive-pool` feature) → builds a
  `KeepAlivePool` and spawns a background task via `start_background_task()`

These defaults are correct for **remote** libSQL/Turso Cloud connections, where
connection reuse and keep-alive pings amortize network latency and prevent idle
disconnects.

For a **local file** or **`:memory:`** database they are inappropriate:

1. **No benefit** — there is no network round-trip or idle-timeout to optimize.
2. **SIGSEGV** — the keep-alive background task holds libsql connections. When a
   short-lived runtime (e.g. a `#[tokio::test]`) is dropped, those connections
   are dropped *outside* a Tokio runtime context, which aborts with signal 11.
   This surfaced as 8 crashing relationship tests in CI (Tests + both
   Multi-Platform jobs) once the workspace's `keepalive-pool` feature was
   unified onto the turso test binary.
3. **`:memory:` correctness** — each pooled `db.connect()` to `:memory:` opens a
   *separate, empty* in-memory database, so schema created by
   `initialize_schema()` on one connection is invisible to the next.

## Decision

Local and in-memory Turso storage modes **must not** use connection pooling or
the keep-alive background task. A dedicated config is used by `new_local` and
`new_in_memory`:

```rust
fn local_config() -> TursoConfig {
    TursoConfig {
        enable_pooling: false,
        #[cfg(feature = "keepalive-pool")]
        enable_keepalive: false,
        ..TursoConfig::default()
    }
}
```

Both constructors call `with_config(url, "", Self::local_config())`. Remote
construction (`new_remote` / `new`) is unchanged and keeps pooling + keep-alive.

## Consequences

### Positive
- Eliminates the SIGSEGV; relationship tests pass with `keepalive-pool` enabled.
- Local/in-memory storage uses a single libsql connection — correct semantics
  for `:memory:` (schema persists) and zero background tasks to leak.
- Production `new_local` is now safe for short-lived/ephemeral agent processes.
- No change to remote behavior or public API surface.

### Negative / Trade-offs
- Local storage forgoes pooled concurrency. For single-file SQLite this is a
  non-issue (libsql serializes file access); throughput is unaffected for the
  intended local/dev/test workloads.
- A future high-concurrency local workload would need an explicit opt-in
  (construct via `with_config` with a custom `TursoConfig`), which remains
  available.

### Neutral
- The decision is localized to two constructors; `with_config` retains full
  flexibility for advanced callers.

## Validation

- Reproduced pre-fix: `cargo nextest run -p do-memory-storage-turso --features
  keepalive-pool relationships::tests` → 8 SIGSEGV.
- Post-fix: same command → 8 passed; full turso suite 378 passed; turso+cli
  739 passed; `fmt` clean; `clippy --features keepalive-pool --all-targets` zero
  warnings.

## Alternatives Considered

1. **Construct `TursoStorage` struct directly in tests (bypass `with_config`)** —
   the original SIGSEGV fix attempt. Rejected: it leaves the *production*
   `new_local`/`new_in_memory` constructors crash-prone, only masking the bug in
   tests, and it failed to compile (missing `TursoConfig` import).
2. **Disable the `keepalive-pool` feature in the workspace** — rejected: removes
   a legitimate optimization for remote connections and treats the symptom.
3. **Gate connection drops behind explicit runtime handles** — rejected: large,
   invasive change to pool/keepalive lifecycle for a problem that does not apply
   to local storage at all.
