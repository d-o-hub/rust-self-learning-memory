# GOAP: CI Health Analysis & Remediation — 2026-06-11

- **Created**: 2026-06-11
- **Version context**: `0.1.32` (workspace, sprint in flight)
- **Branch (local)**: `main` (1 commit ahead of `origin/main`)
- **Related ADR**: `plans/adr/ADR-057-CI-Health-PR616-Nightly-Timeout.md`
- **Detailed code changes**: `plans/CODE_CHANGES_CI_REMEDIATION_2026-06-11.md`
- **Source data**: `gh run list`, `gh pr list`, `gh issue list` (live, 2026-06-11)

---

## 1. Current State (observed)

### Open PRs
| PR | Title | Author | Status |
|----|-------|--------|--------|
| #616 | `perf(encoder): optimize cosine similarity calculation` | Jules (bot, @d-o-hub) | ❌ CI failing |

### Open Issues
None.

### Latest CI Runs
| Workflow | Trigger | Result |
|----------|---------|--------|
| Nightly Full Tests | schedule (main) | ❌ failure |
| CI (PR #616) | pull_request | ❌ failure |
| Quick Check (PR #616) | pull_request | ❌ failure |
| Coverage / Security / Perf Benchmarks / File Structure (PR #616) | pull_request | ❌ failure (cascade) |
| Supply Chain Security / CodeQL / PR Check Anchor (PR #616) | pull_request | ✅ success |

---

## 2. Root-Cause Analysis

### G1 — PR #616 Quick Check failure (clippy) ⛔ BLOCKING the PR
- **Symptom**: `Quick PR Check (Format + Clippy)` exits 101; `cargo clippy --tests -- -D warnings` fails. `CI`, `Coverage`, `Security`, `Performance Benchmarks` all cascade-fail because they gate on Quick Check via `lewagon/wait-on-check-action`.
- **Exact error**:
  ```
  error: this `MutexGuard` is held across an await point
    --> memory-mcp/src/bin/server_impl/storage.rs:294:13
  note: these are all the await points this lock is held through
    --> memory-mcp/src/bin/server_impl/storage.rs:305:47
  ```
- **Cause**: PR #616 (attempting to de-flake env-var tests) added
  `static TEST_LOCK: parking_lot::Mutex<()>` and takes
  `let _lock = TEST_LOCK.lock();` at the top of each `#[tokio::test]`. The
  `parking_lot` guard is then held across `initialize_*().await`, tripping
  `clippy::await_holding_lock` under `-D warnings`.
- **Scope**: Introduced by the PR; **`main` is clean** (current
  `storage.rs` tests carry no `TEST_LOCK`).
- **Secondary defects in PR #616**:
  - `Semver Check` exits 2.
  - The PR is cut from a **stale `main`** and *deletes* recently-landed plans
    (`GOAP_MAINTENANCE_2026-06-10.md`, `GOAP_REMOTE_ANALYSIS_2026-06-10.md`,
    `remote-repo-analysis/synthesis-2026-06-10.md`, large chunk of
    `GOAP_STATE.md`). Merging as-is would regress documentation.

### G2 — Nightly Full Tests failure (main) ⚠️ NON-BLOCKING for PRs
- **G2a — Slow integration test timeout (real)**:
  ```
  TIMEOUT [120.003s] do-memory-core::async_extraction
      should_scale_processing_with_different_worker_counts
  Summary [120.005s] 37 tests run: 36 passed, 1 timed out, 1627 skipped
  ```
  - The test (`memory-core/tests/async_extraction.rs:291`, `#[ignore]`,
    nightly-only) loops `worker_count ∈ [1,2,4,8]`, each iteration spins a fresh
    worker pool, enqueues 20 episodes and `sleep(3s)`. It prints results for the
    `workers=1` iteration then hangs; nextest's 120s slow-timeout kills it.
    Likely a worker-pool shutdown/startup interaction across iterations (pools
    are not stopped between iterations) rather than a sleep-budget issue.
- **G2b — Full Test Suite "regular tests" (infra)**: step exits 95 right after
  the `Insufficient disk space (<5G)` pre-check — a runner disk-space failure,
  not a code defect.
- **G2c — Mutation Testing**: hits the 2h job ceiling (expected for cargo-mutants
  scope; tracked separately, not a regression).

### G3 — Missing implementation (carried sprint backlog) 📋 TRACKED
Confirmed still-stubbed in current `main` (no `todo!`/`unimplemented!` macros;
47 `TODO`/`FIXME` comments total):
| WG | Location | Stub |
|----|----------|------|
| WG-160 | `memory-storage-turso/src/cache/wrapper.rs:142` | `query_hits: 0, // Not yet implemented` |
| WG-156 | pattern match scoring | hard-coded `0.8` |
| WG-157 | `memory_usage_mb` | hard-coded `50.0` |
| WG-158 | `episode_success_rate` | hard-coded `99.0` |
| WG-161 | cascade `analyze_query` | stub |
| WG-162 | `generate_simple_embedding` | placeholder |

Follow-up plan already exists: `plans/GOAP_PRE_EXISTING_ISSUES_FOLLOWUP_2026-06-09.md`.

---

## 3. Goal State

- PR #616 either fixed (clippy-clean, no plans deletions) or closed/superseded.
- Nightly `should_scale_*` test no longer times out (or is correctly bounded).
- Backlog WGs remain tracked; no new regressions on `main`.

---

## 4. GOAP Action Plan

| ID | Action | Pre-conditions | Effect | Cost | Owner |
|----|--------|----------------|--------|------|-------|
| A1 | Replace `parking_lot::Mutex` lock-across-await with `tokio::sync::Mutex` (`.lock().await`) **or** drop the guard before `.await` in `storage.rs` tests | PR branch writable | Quick Check clippy passes → CI/Coverage/Security un-cascade | M | maintainer / re-task Jules |
| A2 | Rebase PR #616 onto current `main`; restore deleted `plans/*` files | A1 | No doc regression; Semver Check re-run | M | maintainer |
| A3 | Bound `should_scale_processing_with_different_worker_counts`: stop worker pools between iterations and/or replace fixed `sleep(3s)` with a drain-poll loop; verify under `--ignored` | repro locally | Nightly slow-tests green | M | maintainer |
| A4 | (Infra) Confirm nightly disk-space pre-check headroom; treat exit-95 as infra-flake, add retry/cleanup if recurring | G2b recurs | Full Test Suite stable | S | CI owner |
| A5 | Keep WG-156–162 on the existing follow-up plan; no action this sprint | — | Backlog stays visible | S | — |

**Recommended execution**: A1→A2 (sequential, same PR) ∥ A3 (independent, `main`).
A4/A5 are watch-items.

---

## 5. Validation Gates

Per `AGENTS.md` before any merge:
```
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
cargo nextest run --all
cargo test --doc
cargo nextest run -p do-memory-core --run-ignored ignored-only -E 'test(should_scale_processing_with_different_worker_counts)'
```

---

## 6. Decision

Captured in **ADR-057**. PR #616's perf change to `similarity.rs` is sound; the
PR is blocked solely by its **test-harness clippy violation** and a **stale-base
plans deletion**. The nightly failure is a **pre-existing slow-test timeout**
plus **runner disk infra**, independent of PR #616.
