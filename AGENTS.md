## Project overview

* Purpose: maintain an episodic memory backend for AI agents (start -> execute -> score -> learn -> retrieve).
* Languages: Rust (async/Tokio). Storage: Turso/libSQL for durable structured storage; redb for hot key-value caching.
* Main entrypoint crates: `memory-core`, `memory-storage-turso`, `memory-storage-redb`, `memory-embed` (optional).
* Expected outputs: episode JSON blobs, pattern records, cached embeddings.

## Quick setup

```bash
# Rust toolchain
rustup override set stable
cargo fetch

# Project build & tests
cargo build --all
cargo test --all
```

> Agents: run `cargo test -- --nocapture` when troubleshooting failing tests.

## File & code style

* Rust: follow the project `rustfmt` and Clippy rules.
* Keep each source file <= **500 LOC**. If a module grows, split into submodules (`storage/turso.rs`, `storage/redb.rs`).
* Use `anyhow::Result` for top-level functions and propagate typed errors internally.

## Agent responsibilities (common tasks)

### 1) Create / start an episode

* Use `SelfLearningMemory::start_episode(task_description, context)`.
* Ensure `TaskContext` contains `language`, `domain` and `tags` for accurate retrieval.
* Persist to both Turso (durable) and redb (fast cache).

### 2) Log execution steps

* Use `log_step(episode_id, ExecutionStep)`.
* Include `tool`, `action`, `latency_ms`, `tokens`, `success`, and `observation`.
* Avoid frequent tiny writes — batch steps when many occur in short bursts.

### 3) Complete & score episodes

* Call `complete_episode(episode_id, TaskOutcome)` after finalization.
* The system computes `RewardScore`, `Reflection`, and `Patterns` — update patterns and heuristics.

### 4) Retrieval

* Use `retrieve_relevant_context(description, context, limit)`.
* If `embedding_service` is configured, prefer semantic search first for recall quality.

## Storage & schema notes

### Turso / libSQL

* Tables: `episodes`, `patterns`, `heuristics`.
* Store large nested fields (steps, context, artifacts) as JSON strings.
* Indexes: `task_type`, `timestamp DESC`, `verdict`.
* Use parameterized queries and `INSERT OR REPLACE` for upserts.

### redb (cache)

* Tables: `episodes`, `patterns`, `embeddings`, `metadata`.
* Keep redb as hot-cache; do not treat it as only source-of-truth.
* Use read transactions for concurrent reads; wrap writes in short-lived write transactions.

## Patterns, heuristics & embeddings

* Patterns are small, typed records (ToolSequence, DecisionPoint, etc.).
* Heuristics are condition→action rules learned from episodes.
* Embeddings (optional): store as raw bytes in `embeddings` table; cache embedding IDs in episode metadata.

## Concurrency & performance

* Use Tokio with `async` functions for all IO (Turso operations are async). redb is synchronous — perform redb writes in dedicated tasks to avoid blocking async runtime.
* Batch pattern updates and use a semaphore (e.g., `Semaphore::new(10)`) when parallelizing heavy work.
* Keep `MAX_EPISODES_CACHE` configurable (default 1000).

## Testing & CI

* Unit tests: small, deterministic, fast (TempDir + in-memory DB where possible).
* Integration tests: use ephemeral Turso/test DB and a temp redb file.
* CI: ensure `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test` in pipeline.

## Security & secrets

* Read Turso credentials from environment variables (do not hardcode tokens).
* Sanitize any artifact before storing (avoid storing secrets in episode artifacts or logs).
* Set RBAC on remote DB; restrict write permissions where applicable.

## Operational guidance

* Backup strategy: periodically export episodes (SST/ JSONL) from Turso for long-term archival.
* Monitoring: emit telemetry with `tracing` and expose metrics (episode creation rate, retrieval latency, pattern update failures).
* Rolling upgrades: support schema migrations with SQL migration files and keep backward compatibility for older episode JSON shapes.
* Quota management: Monitor episode creation rates and implement rate limiting for production deployments. Handle `QuotaExceeded` and `RateLimitExceeded` errors gracefully with backoff strategies.

## Maintenance & extensibility

* Keep `episode` and `pattern` structures backwards-compatible: add nullable fields; avoid renaming.
* Split code into small modules under `src/` so each file stays <500 LOC.
* Add feature flags for optional components (e.g., `embedding`, `experimental-patterns`).

## Troubleshooting checklist for agents

1. If retrieval returns few results: check embeddings are computed and cached. If embeddings missing, rely on `task_type` index.
2. If redb is stale: run `sync_memories()` to reconcile with Turso.
3. If pattern updates are slow: reduce batch size or limit concurrency.
4. If tests fail intermittently: run with `RUST_LOG=debug cargo test` and inspect `tracing` logs.

---

## Minimal agent task templates

### Fix failing test

```
- Run: `cargo test --test <name>`
- If failing: reproduce locally, add/adjust test, run `cargo test`.
- Commit: include test and fix with clear message: `[tests] fix <module>: <short>`
```

### Implement feature

```
- Create branch: `feat/<short-desc>`
- Add small module <= 500 LOC; wire into crate root.
- Add unit tests and integration tests if required.
- Run `cargo clippy && cargo test` and submit PR.
```
