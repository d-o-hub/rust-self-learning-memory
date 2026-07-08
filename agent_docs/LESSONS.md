# Lessons Log

Compact log for non-obvious workflow learnings. Pair each entry here with a short distilled note in the nearest `AGENTS.md`.

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
