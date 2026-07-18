# Lessons Log

Compact log for non-obvious workflow learnings. Pair each entry here with a short distilled note in the nearest `AGENTS.md`.

## LESSON-011: NEVER rush merges — wait for ALL CI, never use --admin

- Issue: PR #806 merged when `mergeStateStatus` was `UNSTABLE` (not `CLEAN`) using `gh pr merge --admin`. This bypassed branch protection and caused pre-existing CI failures to ship to main. Required follow-up PRs (#810, #811) to fix the failures.
- Root Cause: Agent rushed to "complete the task" instead of following the mandatory PR health check procedure. The pr-readiness skill was never loaded. The `--admin` flag was used as a shortcut instead of fixing the underlying issues.
- Solution: 
  1. ALWAYS load `.agents/skills/pr-readiness/SKILL.md` before any merge action
  2. NEVER use `--admin` or `--force` — fix the code, don't bypass the gate
  3. NEVER merge when `mergeStateStatus` is anything other than `CLEAN`
  4. Wait for ALL checks to reach terminal state before merging
  5. Follow the Mandatory Pre-Merge Checklist in AGENTS.md

## LESSON-010: Coderabbitai review loop — verify each finding against current code

- Issue: Conversation summaries implied fixes were already applied, but code still had the original unfixed patterns (e.g., `_min_sequence_len` in `with_thresholds`, `contributing_tiers` hardcoding `api_fallback_needed`).
- Root Cause: Trusting historical memory over actual file state. Multiple rounds of review comments can accumulate; fixes described in summaries may not match current tree.
- Solution: Always `read_files` on the target file before acting on a finding. Search for callers independently of cached results. Verify each finding against current code before declaring it resolved.

## LESSON-001: Verify release/package truth before roadmap edits

- Issue: Planning docs treated `v0.1.30` as unreleased even though GitHub already had a published release and publishable crates were still at `0.1.30`.
- Root Cause: `plans/` had drifted from repo reality and version work was being planned from stale documents instead of live release/package data.
- Solution: Check `gh release view <tag>` and `cargo metadata --no-deps --format-version 1` before changing roadmap, status, or version-bump plans.

## LESSON-002: Sprint reprioritization must update all live plan files together

- Issue: CPU/token-efficiency work was added in one place while adjacent GOAP and status docs still described the old sprint goal.
- Root Cause: The live planning surface is split across roadmap, goals, actions, GOAP state, and status files.
- Solution: Update `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`, and `plans/STATUS/CURRENT.md` in one pass.

## LESSON-003: `learn` workflow needs both the distilled note and the log file

- Issue: `.agents/skills/learn/SKILL.md` required dual-write to `agent_docs/LESSONS.md`, but that file did not exist.
- Root Cause: The workflow contract was documented before the backing log file was added.
- Solution: Keep `agent_docs/LESSONS.md` as the verbose log and reserve root `AGENTS.md` for compact workflow notes only.

## LESSON-004: Highest-leverage efficiency work is retrieval-path focused

- Issue: The repo's best near-term CPU/token wins were easy to miss because they are split across plans and code placeholders.
- Root Cause: Performance opportunities span both implementation gaps and prompt-assembly workflow docs.
- Solution: Prioritize cached retrieval wiring, QueryCache contention, bounded context assembly, hierarchical reranking, and compact high-frequency skill/docs before larger research features.

## LESSON-005: Secure JWT validation requires signature verification

- Issue: The MCP server used simplified JWT parsing that skipped signature verification, allowing token forgery.
- Root Cause: Development-only validation logic was left in place without enforcing secure production defaults.
- Solution: Use reliable libraries like `jsonwebtoken` for verification. Enforce mandatory secret configuration for production modes and include security tests for signature and claim (iss, aud, exp, sub) validation.

## LESSON-006: CI Optimization - paths-based benchmark triggering

- Issue: Benchmark workflow runs ~54 min on every PR, even docs-only changes.
- Root Cause: No path filtering on benchmark workflow triggers.
- Solution: Add `paths` filter for perf-critical code only (storage, core, benches). GitHub Actions doesn't support `paths` + `paths-ignore` at same trigger level - use `paths` alone.
- Key insight: Use `.claude/skills/github-workflows` and `.claude/skills/ci-fix` skills for CI issues.

## LESSON-007: cosine_similarity normalization range [-1,1]→[0,1]

- Issue: Coverage test assertions expected raw cosine values (-1 to 1), but implementation normalizes to 0-1 range.
- Root Cause: `cosine_similarity` in `similarity.rs` normalizes: `(similarity + 1.0) / 2.0` to keep all values positive.
- Solution: Test assertions must account for normalization:
  - Identical vectors: 1.0 → 1.0 (unchanged)
  - Opposite vectors: -1.0 → 0.0
  - Orthogonal vectors: 0.0 → 0.5
  - Low similarity (orthogonal): 0.0 → 0.5
- Location: `memory-storage-turso/src/retrieval/similarity.rs` lines 52-54
## LESSON-008: Optimized Batch Eviction in Turso Storage

- Issue: Capacity eviction was performing O(N) database roundtrips, deleting episodes and embeddings one-by-one in a loop, causing significant latency during large evictions.
- Root Cause: Sequential 'DELETE' statements in a loop (N+1 query problem).
- Solution: Implemented batch deletion using SQL 'IN (...)' clauses. Episodes and embeddings are now collected and deleted in a single batch operation per table.
- Results: Benchmarking via SQLite simulation showed a 98.7% reduction in deletion time (from 1782ms to 23ms for 100 items).
- Key insight: Always prefer batch operations for bulk deletions in remote or file-based storage to minimize I/O overhead. Ensure multi-dimension embedding tables are also cleared in batch.

## LESSON-009: Accurate Logical Cache Size with Lazy Invalidation

- Issue: `effective_size()` in the query cache was potentially returning inaccurate values after physical LRU evictions.
- Root Cause: Simply subtracting the count of all invalidated hashes from the total cache size didn't account for entries that might have already been physically evicted by the LRU policy.
- Solution: Changed the calculation to only count invalidated hashes that are still physically present in the cache (using `peek()`).
- Key insight: When combining LRU eviction with lazy invalidation, logical size calculations must intersect the two sets to remain accurate.

## LESSON-011: GitHub Actions upload-artifact LCA causes deep nesting on download

- Issue: PR #609 benchmark workflow uploaded successfully (5.4 MB) but downstream "Check for Performance Regression" job posted "⚠️ Benchmark artifacts not available" because `bench_results.txt` was nowhere to be found at the workspace root.
- Root Cause: `actions/upload-artifact@v7` computes the **least common ancestor (LCA)** of all input paths. Mixing workspace-relative paths (e.g., `./bench_results.txt`) with `${{ runner.temp }}/cargo-target/criterion/` produced an LCA of `/home/runner/work`. The archive stored `bench_results.txt` at `rust-self-learning-memory/rust-self-learning-memory/bench_results.txt` and criterion files at `_temp/cargo-target/criterion/...`. After `download-artifact@v8` extraction, `bench_results.txt` lived at `<workspace>/rust-self-learning-memory/rust-self-learning-memory/bench_results.txt`, not at the workspace root where the regression check looked.
- Solution: Co-locate all upload inputs under a single parent. Copy workspace-relative files into `${{ runner.temp }}/cargo-target/` and upload only that directory so the LCA is `${{ runner.temp }}/cargo-target/`. Add `if-no-files-found: error` to surface silent failures instead of masking them.
- Reference: <https://github.com/actions/upload-artifact#upload-using-multiple-paths-and-exclusions>; local docs: `agent_docs/github_actions_patterns.md` ("Upload Artifact LCA Pitfall (2026-06-05)").

## LESSON-012: One-off scripts in root create maintenance burden

- Issue: `cleanup_yaml.py` and `remove_redundant_cache.py` were committed to the repository root as one-off CI cleanup utilities.
- Root Cause: Ad-hoc scripts created during CI fixes were committed directly to root without considering long-term maintenance or placement conventions.
- Solution: Place reusable scripts in `scripts/`, delete one-off scripts after use, and never commit `.py` or `.sh` files to the repository root. Use `plans/` for design notes, `target/` for build artifacts.
- Reference: `AGENTS.md` "Disk Space" section — "No Temporary Files in Root" guard rail.

## LESSON-015: Feature-gated test code requires matching cfg on ALL imports

- Issue: `tests/hybrid_storage_recovery.rs` had `use std::sync::Arc`, `use async_trait::async_trait`, `struct FailingStorage` etc. defined unconditionally, but they were only used inside `#[cfg(feature = "redb")]` test functions. CI runs `cargo clippy --tests -- -D warnings` without features, causing unused-import and dead-code errors that block ALL open PRs.
- Root Cause: When adding feature-gated tests, developers often gate only the `#[test]` function with `#[cfg(feature = "X")]` but forget to gate the imports, helper structs, and impl blocks that the test uses. Without the feature enabled, those become dead code.
- Solution: Every import, struct, and impl block that is ONLY used inside feature-gated code must also carry matching `#[cfg(...)]` attributes. Use the same or broader cfg predicate as the most restrictive consumer.
- 2026 best practice: Run clippy in CI both with `--tests` (no features) AND with `--all-features --tests` to catch both dead-code (no features) and missing-import (all features) errors. Consider `cargo-matrix` for comprehensive feature combination testing.
- Key insight: A single ungated import in a test file can block the entire CI pipeline for all PRs, since the `e2e-tests` crate is checked as part of workspace clippy.
- Location: `tests/hybrid_storage_recovery.rs`
- Reference: Rust Reference §Conditional Compilation; <https://effective-rust.com/features.html>

## LESSON-016: GOAP swarm for PR batch merges with auto-merge

- Issue: 5 PRs with CI still running needed coordinated merge in dependency order
- Root Cause: Manual sequential merge waiting is wasteful when PRs are independent
- Solution: Enable `gh pr merge --squash --auto` on all PRs simultaneously. GitHub handles merge ordering via branch protection. Required Check Anchor is the only true merge gate; CANCELLED checks (Coverage, YAML Lint) are non-required noise.
- Key insight: Auto-merge + squash is safe for independent PRs. Monitor via `gh pr view --json state,autoMergeRequest` rather than polling individual checks.

## LESSON-018: Audit log size init + nested redaction (S1.7)

- Issue: Audit file rotation tracked `current_file_size` as `0` after open even when the file already existed and was large; nested JSON secrets under objects/arrays were never redacted; `writeln!` ran on the async request path.
- Root Cause: Size was read into a local and discarded; redaction walked only top-level object keys; file I/O was synchronous under `async fn log_event`.
- Solution: Dedicated bounded `sync_channel` + OS writer thread initializes size from metadata; recursive case-insensitive key redaction; drop counter when queue is full (`dropped_writes()`).
- Key insight: Startup metadata must seed runtime accounting; “sensitive field” policies must recurse JSON structure; disk I/O belongs off the Tokio worker.

## LESSON-019: Skill evals need a dedicated lightweight CI workflow (K3.1b)

- Issue: K3.1a delivered `run-evals.sh` but nothing required fixtures on every PR, so schema regressions could merge green.
- Root Cause: Gate contract documented skill evals as optional until CI wiring; no workflow invoked the runner.
- Solution: `.github/workflows/skill-evals.yml` (no Rust compile): always fixtures + gate-contract; PRs also `--changed` with `fetch-depth: 0`; full suite on schedule/dispatch.
- Key insight: Schema validation CI must not depend on cargo; keep skill-eval and gate-contract checks on a cheap path so they always run.

## LESSON-017: CLI pattern list empty across processes + `--db-path` no-op (#830 / #831)

- Issue: After `episode complete` logged "Successfully cached pattern", a fresh CLI process showed `pattern list` = 0. Separately, `--db-path` / `MEMORY_DB_PATH` appeared ignored.
- Root Cause:
  1. **#831**: `Pattern` used `#[serde(tag = "type")]` (internally tagged) — **postcard cannot deserialize** that form, so durable redb/Turso reads failed silently / returned empty. Also `get_all_patterns` only consulted the empty in-memory map in a new process.
  2. **#830**: `--db-path` only set Turso `database.db_path`; `Config::default()` already filled `redb_path` with the XDG cache path, so the local redb backend never used the user path.
- Solution:
  1. Use postcard-compatible (externally tagged) serde for types persisted with postcard; keep a postcard round-trip unit test on `Pattern`.
  2. Hydrate patterns via `StorageBackend::get_all_patterns` (memory → redb → Turso) on list/search.
  3. Always set **both** `db_path` and `redb_path` from `--db-path` / `MEMORY_DB_PATH`.
  4. Smoke across processes: create → log-step `--success` → complete → `pattern list` with the same `--db-path`.
- Key insight: Unit tests in one process mask cross-process durability bugs. Never use internally tagged enums with postcard. CLI path flags that choose "where data lives" must override redb, not only Turso.
- Prevention docs: `.agents/skills/do-memory-cli-ops/SKILL.md`, `troubleshooting.md`
- References: issues #829–#832; `plans/GOAP_CLI_UX_PATCH_0.1.35_2026-07-15.md`
## LESSON-013: Local storage mode requires explicit temp directory for redb fallback

- Issue: When `--storage-mode local` was used without configuring `redb_path`, the combination logic created redb with literal `:memory:` path, creating a file named `:memory:` in CWD or failing silently. Episodes appeared to save but weren't persisted across CLI invocations.
- Root Cause: `RedbStorage::new(Path::new(":memory:"))` doesn't create an in-memory database — it creates a file literally named `:memory:`. The redb crate doesn't support SQLite-style `:memory:` URLs.
- Solution: Use `tempfile::tempdir()` for ephemeral redb cache when no explicit path is configured. Leak the TempDir handle (`std::mem::forget`) so the file persists for the process lifetime but cleans up on exit. Also ensure parent directories exist before opening redb files.
- Key insight: Test storage paths end-to-end across process boundaries. A single-process test may pass because in-memory state masks the persistence failure.
- Location: `memory-cli/src/config/storage/combination.rs`, `memory-cli/src/config/storage/mod.rs`

## LESSON-014: CI publish pipeline — polling beats sleeping

- Issue: `sleep 30` between crate publishes in `publish-crates.yml` is fragile. Under crates.io load, 30s isn't enough for index propagation, causing downstream publishes to fail with dependency resolution errors.
- Root Cause: Crates.io sparse index propagation time varies (10s to 120s+) depending on server load and registry cache state.
- Solution: Poll `https://crates.io/api/v1/crates/<name>/versions` until the expected version appears (max 20 attempts × 15s = 5min ceiling). Also use `--locked` on all publish commands and declare explicit `needs` for all transitive workspace dependencies.
- 2026 best practice: Trusted Publishing (OIDC) eliminates API token management entirely. Consider migration for crates.io publishing.
- Reference: <https://forge.rust-lang.org/infra/docs/trusted-publishing.html>, <https://crates.io/docs/trusted-publishing>
