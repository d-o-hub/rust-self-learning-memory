# Detailed Code Changes — CI Remediation (2026-06-11)

Concrete, copy-pasteable code changes to clear the CI failures analyzed in
`plans/GOAP_CI_ANALYSIS_2026-06-11.md` and `plans/adr/ADR-057-CI-Health-PR616-Nightly-Timeout.md`.

> Documentation only. These diffs are the **specification**; they are not yet
> applied to source. A1/A2 target the **PR #616 branch**; A3 targets **`main`**.

---

## A1 — Fix `clippy::await_holding_lock` in storage tests (PR #616 branch) ⛔ BLOCKING

**File**: `memory-mcp/src/bin/server_impl/storage.rs`
**Problem**: PR #616 added a `parking_lot::Mutex` and holds its guard across
`initialize_*().await`, which fails `cargo clippy --tests -- -D warnings`
(`error: this MutexGuard is held across an await point`).

### Option A1-preferred — use `tokio::sync::Mutex` (async-aware, keeps serialization)

`tokio`'s mutex guard is `Send`/await-safe, so clippy does not flag it.

```rust
// in: #[cfg(test)] mod tests { ... }

// BEFORE (added by PR #616 — fails clippy):
//     static TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
//     ...
//     let _lock = TEST_LOCK.lock();

// AFTER:
use tokio::sync::Mutex as AsyncMutex;

// Serialize tests that mutate process-wide environment variables.
static TEST_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

#[tokio::test]
async fn test_initialize_turso_local_succeeds() {
    let _lock = TEST_LOCK.lock().await;   // <-- await the async lock
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    // SAFETY: single-threaded test environment (enforced by TEST_LOCK)
    unsafe {
        std::env::set_var("MEMORY_DB_PATH", db_path.to_str().unwrap());
        std::env::set_var("REDB_CACHE_PATH", dir.path().join("cache.redb").to_str().unwrap());
    }
    let result = initialize_turso_local().await;
    unsafe {
        std::env::remove_var("MEMORY_DB_PATH");
        std::env::remove_var("REDB_CACHE_PATH");
    }
    assert!(result.is_ok());
}
```

Apply the identical `let _lock = TEST_LOCK.lock().await;` to the other three
guarded tests:
- `test_initialize_redb_only_storage_succeeds`
- `test_initialize_memory_system_in_memory_mode`
- `test_initialize_memory_system_unknown_mode_falls_back`

`AsyncMutex::const_new` is `const`, so the `static` initializer remains valid.
No extra `Cargo.toml` change is needed: `tokio` is already a dependency of
`memory-mcp`.

### Option A1-alt — drop the std/parking_lot guard before every `.await`

If a sync mutex must stay, the guard must not live across an await. Because each
test awaits, this means binding the guard in a non-async helper that returns
*after* the env setup, which is awkward. Prefer Option A1-preferred.

If `serial_test` is acceptable (it is already used elsewhere in the workspace),
the simplest correct form is:

```rust
// remove TEST_LOCK entirely; annotate each test:
#[tokio::test]
#[serial_test::serial]
async fn test_initialize_turso_local_succeeds() { /* unchanged body, no _lock */ }
```

> Note the PR comment claimed `serial_test` was avoided due to "CI binary"
> issues; if that is real, ship Option A1-preferred (`tokio::sync::Mutex`).

### Validation
```
cargo clippy -p do-memory-mcp --tests -- -D warnings
cargo nextest run -p do-memory-mcp
```

---

## A2 — Rebase PR #616 and restore deleted plans (PR #616 branch)

PR #616 was cut from a stale `main` and its diff **deletes** files that landed
afterward. These must be restored (do **not** let the merge remove them):

| Deleted by PR #616 | Action |
|--------------------|--------|
| `plans/GOAP_MAINTENANCE_2026-06-10.md` | restore from `main` |
| `plans/GOAP_REMOTE_ANALYSIS_2026-06-10.md` | restore from `main` |
| `plans/remote-repo-analysis-2026-06-10.md` | restore from `main` |
| `plans/remote-repo-synthesis-2026-06-10.md` | restore from `main` |
| `plans/GOAP_STATE.md` (−51 lines) | restore `main` content, then re-add the A-prefixed CI sweep entry |

Recommended commands (run on the PR #616 branch checkout):
```
git fetch origin main
git rebase origin/main
# resolve in favour of main for the plans/ files:
git checkout origin/main -- \
  plans/GOAP_MAINTENANCE_2026-06-10.md \
  plans/GOAP_REMOTE_ANALYSIS_2026-06-10.md \
  plans/remote-repo-analysis-2026-06-10.md \
  plans/remote-repo-synthesis-2026-06-10.md \
  plans/GOAP_STATE.md
git add plans/
```

This also re-triggers `Semver Check` (was exit 2 against the stale base).

### Validation
```
git diff --stat origin/main...HEAD   # confirm no plans/* deletions remain
```

---

## A3 — Bound the nightly slow test (`main`)

**File**: `memory-core/tests/async_extraction.rs`
**Test**: `should_scale_processing_with_different_worker_counts` (line ~291,
`#[ignore]`, nightly-only). It times out at nextest's 120s slow-limit because it
(a) spins a new worker pool per iteration without stopping the previous ones and
(b) uses a fixed `sleep(Duration::from_secs(3))` that does not adapt to load.

Replace the fixed sleep with a bounded drain-poll on `get_queue_stats()`, and
keep iterations independent. The memory facade exposes `get_queue_stats()`
(returns `current_queue_size`); use it to poll.

```rust
#[tokio::test]
#[ignore = "slow integration test - runs for >60s, run explicitly with --ignored"]
async fn should_scale_processing_with_different_worker_counts() {
    for worker_count in [1, 2, 4, 8] {
        let config = QueueConfig {
            worker_count,
            poll_interval_ms: 10,
            ..Default::default()
        };

        let memory = Arc::new(
            SelfLearningMemory::with_config(test_memory_config())
                .enable_async_extraction(config),
        );
        memory.start_workers().await;

        let episode_count = 20;
        let start = std::time::Instant::now();

        for i in 0..episode_count {
            let episode_id = create_test_episode(&memory, &format!("Task {i}"), 3).await;
            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] },
                )
                .await
                .unwrap();
        }

        // Bounded drain instead of a fixed sleep(3s):
        // poll queue size until empty or a hard 20s per-iteration cap.
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            let stats = memory.get_queue_stats().await.unwrap();
            if stats.current_queue_size == 0 {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "workers={worker_count}: queue did not drain within 20s \
                 (remaining={})",
                stats.current_queue_size
            );
            sleep(Duration::from_millis(50)).await;
        }

        let duration = start.elapsed();
        let stats = memory.get_queue_stats().await.unwrap();
        println!(
            "Workers: {}, Episodes: {}, Duration: {:?}, Processed: {}, Failed: {}",
            worker_count, episode_count, duration, stats.total_processed, stats.total_failed
        );

        assert_eq!(stats.current_queue_size, 0, "Queue should be empty");
    }
}
```

Rationale:
- The poll exits as soon as the queue drains, so fast iterations no longer pay a
  flat 3s; slow ones fail fast with a clear message well under the 120s cap.
- Each iteration's `memory` is dropped at loop end, releasing its worker pool, so
  later iterations don't contend with stale workers.

> Optional hardening (separate change): expose
> `SelfLearningMemory::wait_until_empty(timeout)` /
> `SelfLearningMemory::shutdown_workers()` as thin facades over the existing
> `PatternExtractionQueue::wait_until_empty` / `shutdown`
> (`memory-core/src/learning/queue/operations.rs:294,312`) and call them here
> instead of hand-rolling the poll loop.

### Validation
```
cargo nextest run -p do-memory-core --run-ignored ignored-only \
  -E 'test(should_scale_processing_with_different_worker_counts)'
```

---

## A4 — Nightly disk-space exit-95 (infra, no code change)

`Run regular tests` in the nightly **Full Test Suite** exits 95 right after the
`Insufficient disk space (<5G)` pre-check. No Rust change. If it recurs, tighten
the workflow's cleanup step (`.github/workflows/`):
- run the aggressive `jlumbroso/free-disk-space` step *before* the heavy build,
- and/or split the regular vs slow test steps onto separate runners.

No diff is prescribed here; treat as an infra watch-item.

---

## A5 — Missing-implementation backlog (no change this sweep)

WG-156–162 stubs remain on `main`, confirmed:
- `memory-storage-turso/src/cache/wrapper.rs:142` → `query_hits: 0, // Not yet implemented`
- pattern match score hard-coded `0.8`, `memory_usage_mb` `50.0`,
  `episode_success_rate` `99.0`, cascade `analyze_query` stub,
  `generate_simple_embedding` placeholder.

Tracked in `plans/GOAP_PRE_EXISTING_ISSUES_FOLLOWUP_2026-06-09.md`. No code change
in this remediation.

---

## Combined validation gate (per `AGENTS.md`)

```
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
./scripts/build-rust.sh check
cargo nextest run --all
cargo test --doc
cargo nextest run -p do-memory-core --run-ignored ignored-only \
  -E 'test(should_scale_processing_with_different_worker_counts)'
git status   # verify all intended changes staged, no plans/* deletions
```
