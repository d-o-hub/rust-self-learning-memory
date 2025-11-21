# AGENTS.md

## Project overview

Episodic memory backend for AI agents: start → execute → score → learn → retrieve
**Stack:** Rust (async/Tokio), Turso/libSQL (durable), redb (cache), optional embeddings
**Crates:** `memory-core`, `memory-storage-turso`, `memory-storage-redb`, `memory-embed`

## Setup commands

```bash
rustup override set stable
cargo build --all
cargo test --all

# Debug tests
cargo test -- --nocapture
RUST_LOG=debug cargo test
```

## Code style

- Follow `rustfmt` and Clippy
- Max **500 LOC per file** — split when exceeded
- `anyhow::Result` for public APIs
- Use `tracing` not `println!`

## Core workflows

### 1. Create episode
```rust
memory.start_episode(desc, TaskContext {
    language: "rust", domain: "coding", tags: vec!["async"]
}).await?;
```

### 2. Log steps
```rust
memory.log_step(id, ExecutionStep {
    tool: "compiler", action: "build",
    latency_ms: 1250, success: true
}).await?;
```
Batch steps during high-frequency operations.

### 3. Complete episode
```rust
memory.complete_episode(id, TaskOutcome {
    verdict: Verdict::Success, ...
}).await?;
```
Auto-generates scores, reflections, and patterns.

### 4. Retrieve context
```rust
memory.retrieve_relevant_context(
    "implement async storage", context, limit: 5
).await?;
```
Uses embeddings if configured, falls back to indexing.

## Storage notes

**Turso/libSQL** (source of truth):
- Tables: `episodes`, `patterns`, `heuristics`
- Nested fields as JSON strings
- Use parameterized queries
- `INSERT OR REPLACE` for upserts
- Credentials in env vars only

**redb** (hot cache):
- Read transactions for concurrency
- Short write transactions
- Use `spawn_blocking` for writes
- Sync via `sync_memories()` if stale

## Performance

```rust
// Bounded concurrency
let sem = Arc::new(Semaphore::new(10));
for item in items {
    let permit = sem.clone().acquire_owned();
    tokio::spawn(async move {
        let _p = permit.await;
        process(item).await
    });
}
```

Config: `MAX_EPISODES_CACHE=1000`, `BATCH_SIZE=50`

## Testing

Unit tests: use `TempDir`, in-memory DBs, keep <100ms

```rust
#[tokio::test]
async fn test_episode() {
    let dir = TempDir::new()?;
    let mem = Memory::new_test(dir.path()).await;
    // test logic
}
```

Integration: ephemeral test DB, cleanup in teardown

## CI checks

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --all
```

## Security

- Never hardcode Turso tokens
- Sanitize artifacts (no secrets in logs)
- Rate limit episode creation
- Validate all inputs

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Low retrieval recall | Check embeddings, run backfill |
| Stale cache | Run `sync_memories()` |
| Slow pattern updates | Reduce batch size, limit concurrency |
| Test failures | `RUST_LOG=debug cargo test` |

## Development flow

**Feature:**
```bash
git checkout -b feat/name
# Implement in src/, keep ≤500 LOC
# Add tests
cargo fmt && cargo clippy && cargo test
# Clean up: remove temp files, test fixtures, debug scripts
```

**Bug fix:**
```bash
cargo test --test failing_test -- --nocapture
# Add regression test first, then fix
# Clean up any temporary debug/helper files
git commit -m "fix(module): description

Fixes X by Y. Added test in tests/Z.rs"
```

## Commit format

`[module] description` or `fix(module): description`

Examples:
- `[tests] fix storage: handle empty results`
- `feat(retrieval): add semantic search`

## Environment vars

- `TURSO_URL` — database URL
- `TURSO_TOKEN` — auth token
- `MAX_EPISODES_CACHE` — cache size limit