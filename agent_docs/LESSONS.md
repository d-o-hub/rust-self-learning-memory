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

## LESSON-021: Never gate cheap CI jobs on a 15m Quick Check wait

- Issue: `YAML Lint / Check Quick Check Status` repeatedly **CANCELLED after ~15m**, skipping `yamllint` and `actionlint`. Quick Check itself often took 15–20m (queue + cold cache). PR #860 stayed `UNSTABLE` with a permanent cancelled check.
- Root Cause: Multiple workflows used `lewagon/wait-on-check-action` with `timeout-minutes: 15` equal to (or less than) Quick Check's real runtime. GitHub marks job timeouts as **cancelled**. Downstream `needs: check-quick-check` + `result == 'success'` then **skipped** the real lint jobs. #871 fixed `running-workflow-name` / concurrency but left the 15m race.
- Solution (permanent):
  1. **Do not gate cheap path-filtered jobs** (yaml-lint) on Quick Check — run them immediately.
  2. Raise remaining wait jobs to **`timeout-minutes: 40`** (must exceed Quick Check job timeout + queue headroom).
  3. Raise Quick Check job timeout to **25m**.
  4. Set `fail-on-no-checks: false` and poll interval ≥15s.
  5. Prefer `concurrency.group: ${{ github.workflow }}-${{ github.sha }}` so mid-wait runs are not torn down by same-ref re-triggers.
- Key insight: A wait job that times out is worse than no gate — it produces CANCELLED checks and skips the work it was meant to protect.
- Prevention: `agent_docs/github_actions_patterns.md`, `.agents/skills/github-workflows/`, `ci-fix`

## LESSON-022: Insta snapshots must not use `{:?}` on f32/f64

- Issue: macOS Multi-Platform failed `behaviour_harness::snapshot_store_and_recall_exact_match` while Linux passed — same bits, different Debug text (`1.885494` vs `1.8854939`).
- Root Cause: Platform-dependent float Debug formatting in snapshot strings.
- Solution: Format rewards/scores with fixed precision (`format!("{:.4}", v)`) before `insta::assert_snapshot!`.
- Key insight: Snapshot exact-string equality + Debug floats = cross-platform flakes.

## LESSON-020: Advance workspace version immediately after a release tag

- Issue: PR CI failed Release Drift with `version_not_advanced` after `v0.1.35` shipped while workspace remained `0.1.35` with new feat commits.
- Root Cause: Drift policy treats equal workspace+latest-tag with unreleased commits as invalid (not “clean”).
- Solution: Bump `[workspace.package] version` (and path-dep version pins + Cargo.lock) to the next SemVer step in the first post-release PR.
- Key insight: Released Version docs stay at the tag; workspace version leads by one until the next ship.

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

## LESSON-023: Two CI killers on every commit — rustdoc links and commit body line length

- Issue: `cargo doc --no-deps --document-private-items` with `RUSTFLAGS="-D warnings"` treats broken intra-doc links as errors (`rustdoc::broken-intra-doc-links`). A reference like `[activate_semantic_service]` without a qualifying path (`Self::`, `crate::path::`) fails in CI even though it compiles fine. Commitlint also enforces `body-max-line-length ≤ 100`.
- Root Cause: (1) Rustdoc resolves intra-doc links in the scope of the item, not the module. Methods on `Self` need `Self::method_name`. Free functions need full crate paths. (2) Commit bodies with lines > 100 characters (including bullet points with long Rust type names) fail commitlint.
- Solution:
  1. Always use `Self::method_name` for links to other methods on the same type.
  2. Run `cargo doc --no-deps --document-private-items 2>&1 | grep error` locally before pushing.
  3. Keep every commit body line ≤ 100 chars. Wrap inline code references to next line if needed.
  4. The pre-push gate is: `cargo doc --no-deps --document-private-items` (already in AGENTS.md Required Checks).
- Prevention: This lesson extends the AGENTS.md "Required Checks Before Commit" list — `cargo doc` is listed there; rustdoc link errors are the silent part of it.
- Reference: PR #896, CI run 30197204039.

## LESSON-024: Repairing commitlint failures and pre-existing drift on PR branches (2026-08-07)

- Issue: PR #928 failed Commit Message Lint because 5 of 8 branch commits had
  body/footer lines > 100 chars; two `chore(ci): re-trigger workflow runs`
  no-op commits polluted history. PR #927 failed Release Drift
  (`commit_limit`) — a repo-wide pre-existing condition (workspace 0.1.38 vs
  tag v0.1.37, 43 unreleased commits), not caused by the PR.
- Fixes:
  1. Enumerate failures locally first: `npx commitlint --from <base-sha> --to HEAD --verbose`.
  2. Drop no-op commits non-interactively:
     `GIT_SEQUENCE_EDITOR='sed -i "/re-trigger workflow runs/d"' git rebase -i <base>`.
  3. Rewrap every long line mechanically, no content change:
     `git filter-branch -f --msg-filter 'fold -s -w 100' -- <base>..HEAD`,
     then verify `git diff <old-head> HEAD` is empty and commitlint is clean.
  4. Push with `--force-with-lease`; the fail-closed waiters (CIT-A2) then
     surface green instead of cascading 5 workflow failures.
  5. For pre-existing drift: the documented deadlock breaker is
     `./scripts/release-cadence-manager.sh resolve --pr <n>` (adds the
     `release-preparation` label); the root-cause fix is shipping the
     prepared release via release-guard.
- Key insight: a lint/cadence failure under fail-closed waiters is a feature —
  it makes the root cause visible in every workflow instead of masked green.
- Prevention: `.agents/skills/commit/SKILL.md` (repair section), LESSON-023,
  `release-cadence-manager` skill.
- References: PR #928, PR #927; `plans/archive/2026-09-consolidation/superseded-goap/GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md`.
## LESSON-025: Criterion writes three estimates files per benchmark; only new/ is absolute (2026-09-15)

- Issue: the `Performance Benchmarks` workflow ingested every `estimates.json` under the
  criterion dir, stripped `/new/`, `/base/` and `/change/` down to one benchmark name, and
  emitted all of them as absolute ns. Reproduced with a scratch Criterion 0.8 bench:
  `base/estimates.json` is a byte-identical copy of `new/estimates.json`, and once a
  previous run exists in the same target dir, `change/estimates.json` holds the RELATIVE
  delta (e.g. -0.099) which the old code emitted as `-1 ns/iter`. Two sources under one
  name also produced duplicate rows (masked only because `sort -u` collapsed the identical
  `new`/`base` pairs) and silently dropped benchmarks: `floor()` turned sub-1ns means to 0.
- Evidence (CI run 34879770281, job `Run Benchmarks`): `Criterion estimates.json files
  found: 232` -> `Converted benchmark lines: 222` -> `Total benchmark entries: 111`
  (2 files per benchmark; every pair identical because the CI target dir is fresh).
- Fixes:
  1. Ingest only `-path '*/new/estimates.json'` (fresh, absolute, one row per benchmark).
  2. Emit 2-decimal floats instead of flooring, so sub-1ns means survive
     (`compression_overhead/without_compression` measures ~0.31 ns and was silently
     dropped from every run); fail closed on an estimate with no positive ns mean.
  3. Fail closed when converted rows != fresh file count (provenance) and when a duplicate
     benchmark name survives conversion (silent last-row-wins in the store step).
  4. Guard fixtures live in `./scripts/test-benchmark-workflow.sh --fixtures`.
- Why it matters for gating: the store step compares each benchmark against the previously
  stored run, so ingesting `change/` values would inject numbers unrelated to any
  measurement, and dropped rows shift the dataset (observed 111<->139 entries), making
  alerts compare across different benchmark sets.
- Gate calibration (follow-up PR `fix/benchmark-store-threshold`): same-code repeats on
  the FIXED ingestion pipeline measure the real envelope -
  (a) commits `af36e3c1` vs `80dcda16` (identical ingestion code): median ratio 1.199,
  max 1.72x, min 0.70x, 77% of 120 benchmarks >10%, 33% >25%, 1% >50%;
  (b) one SHA measured twice (`04744db9`, push vs `workflow_dispatch`): median 1.212,
  max 2.28x (`domain_invalidation_latency_100` 13.97us -> 31.88us), 0 benchmarks >3x.
  The pre-fix history (14 stored runs, 1480 comparisons) reaches 4.00x on a 5ns
  timer-floor benchmark (3.82x for means >=1us).
  With the default `fail-threshold = alert-threshold` (110%), 13/13 run pairs breached and
  `store-benchmark` failed on every main push; a same-code repeat (`e4a62748`) raised 75
  alerts on its own. `fail-threshold: '500%'` (strict `>`, so >=25% margin over the
  observed 4.00x envelope) keeps a catastrophic-regression gate, while
  `alert-threshold: '110%'` still comments and cc's @maintainers. The blocking perf gate
  stays deterministic: `eval benchmark --fail-on-regression` in the
  `Retrieval Quality & Cost Benchmark` job.
- Prevention: never infer measurement noise from cross-commit ratios alone - repeat the
  same commit and compare (stored history lives in `gh-pages:dev/bench/data.js`), and
  never set a failure threshold below the measured same-code envelope; a gate that fails
  every run carries no signal.
- References: `.github/workflows/benchmarks.yml` (`benchmark`, `store-benchmark`),
  `scripts/test-benchmark-workflow.sh`, `agent_docs/github_actions_patterns.md`.

## LESSON-026: A sensor that scans missing paths reports success forever (2026-09-15)

- Issue: `scripts/check-loc.sh` — the do-harness `loc` sensor and the pre-commit
  `loc` hook — walked `$ROOT/src` and `$ROOT/crates`. Neither directory exists in
  this workspace (crates live at `memory-core/`, `memory-cli/`, `memory-mcp/`, ...),
  so `find` matched nothing, the script printed `check-loc OK` and exited 0 on every
  commit. Six source files silently drifted past the 500 LOC invariant between
  v0.1.40 and now: `checkpoint.rs` 330->554, `episode/core/types.rs` 490->503,
  `memory/checkpoint/compact.rs` (new file) 514, `patterns/similarity.rs` 395->533,
  `storage/backend.rs` 480->502, `mcp/tools/checkpoint/tool.rs` 415->713.
- Impact: main CI stayed green (the CI `Quality Gates` job runs the security audit
  and coverage only), but the canonical release path runs
  `scripts/quality-gates.sh`, whose blocking `run_source_file_size_gate` scans
  `git ls-files '*.rs'` for real and failed the release. The drift was discovered
  only when `release-manager ship` stopped at the quality-gates phase.
- Fixes:
  1. `scripts/check-loc.sh` now selects files with `git ls-files '*.rs'` plus
     untracked ones (excluding `target/` and `benches/`) and applies the same
     test-file carve-out as the gate, so sensor and gate agree.
  2. The six files were split at cohesive seams: inline `#[cfg(test)] mod tests`
     bodies moved to sibling `*/tests.rs` files, `types.rs` payload enums/DTOs moved
     to `types/models.rs` (re-exported), and the compact byte-budget helpers moved to
     `memory/checkpoint/compact/budget.rs` (`pub(super)`).
- Prevention: a sensor must fail closed when its search root is absent — assert the
  scan found files (or count them) before reporting OK, and keep the sensor's file
  selection identical to the gate it stands in for. Grep any new sensor's paths
  against `cargo metadata` before trusting a green result.
- References: `scripts/check-loc.sh`, `scripts/quality-gates.sh`
  (`run_source_file_size_gate`), `do-harness.toml` (sensor `loc`), AGENTS.md core
  invariant "Files: <=500 LOC per source file".

## LESSON-027: A push-triggered workflow that self-cancels blocks the release gate (2026-09-17)

- Issue: `release-manager.sh ci-check` requires every completed workflow run on
  `origin/main` HEAD to conclude success/skipped/neutral. `Mutation Testing`
  (`mutants.yml`) triggers on push to main when `memory-core/src/{reward,
  retrieval,retry,patterns}/**` change, and its four shards each carry
  `timeout-minutes: 60` while needing longer: GitHub kills the shard, the job and
  the whole run conclude `cancelled`, and `ci-check` counts that as one failed
  run on HEAD. All 12 runs in the preceding month concluded `cancelled` for the
  same reason, so any main commit touching those paths blocked the release path
  (the path filter is why most merges ship fine — e.g. release-prep merges touch
  only Cargo.toml/CHANGELOG/docs and never start the workflow).
- Impact: `fix/loc-invariant-restore` (PR #1026) merged with every run green
  except `Mutation Testing`, and `ship` refused to tag `452eb21a`.
- Fix: the shard step now wraps `cargo mutants` in
  `timeout --signal=TERM --kill-after=60s "$BUDGET"` (default 3000s, override
  with `MUTANTS_SHARD_BUDGET_SECONDS`), so the step exits before the job timeout
  and the run concludes `success`. rc 124/137 emits a truncation notice and any
  other non-zero rc emits a warning; the job stays `continue-on-error: true`
  because Phase 1 is informational (issue #747).
- Prevention: never let a workflow's own timeout be the normal termination path
  for a job — bound the work inside the job budget. Before adding a push trigger,
  ask what `release-manager.sh ci-check` will report when the job is cancelled.
- References: `.github/workflows/mutants.yml`, `scripts/release-manager.sh`
  (`do_ci_check`), `.agents/skills/release-guard/SKILL.md`.

## LESSON-028: Measure a test's real runtime before trusting a timeout to be the bug (2026-09-17)

- Issue: `cargo nextest run --all` (the local gate `release-manager ship` runs)
  killed `e2e-tests::cli_workflows::test_bulk_operations`,
  `do-memory-cli::relationship_command_tests::test_relationship_full_cycle` and
  `test_pattern_discovery` at their `profile.default` caps (120s, 120s, 270s).
  The tests were passing — every assertion logged before the kill — but each
  spawns the debug CLI binary once per command, so their runtime tracks machine
  speed. The plan's remedy (idle machine, no contention) did not help: running
  them single-threaded still exceeded the caps.
- Measurements (uncapped, idle, sequential):
  `cargo test -p e2e-tests --test cli_workflows test_bulk_operations -- --exact`
  → 134.5s; `... relationship_command_tests test_relationship_full_cycle` →
  179.0s; `... cli_workflows test_pattern_discovery` → 400.5s.
- Fix: `.config/nextest.toml` `profile.default` gained whole-binary overrides
  (`binary(cli_workflows)`, `binary(relationship_command_tests)`) with
  `period = "240s", terminate-after = 2` (kill at 480s, above the slowest
  measured test). The redundant per-test default override for
  `test_pattern_discovery` was removed: when several overrides match, the
  binary-level one wins, so two entries with different caps for the same test
  made the effective limit unpredictable. `profile.ci` is untouched (it retries
  twice and passes on its runners).
- Prevention: to size a timeout, bypass the harness and measure —
  `cargo test -p <crate> --test <binary> <test> -- --exact` runs without
  nextest's caps. Set the cap above the slowest machine you must support, keep
  one override per test binary, and never treat "the job was killed at the
  limit" as a slow test's fault without that measurement.
- Follow-up (same day): per-test overrides fixed those three tests but not the
  class. The next full run killed a *different* test —
  `do-memory-cli config::types::simple_config_tests::test_simple_config_with_cloud_platform`,
  ~37s uncontended — starved past the 120s base cap while the CLI waves held all
  four cores for ~6 minutes. `profile.default` was the only profile that gates
  local releases *and* the only one with no retries, so it now has `retries = 1`
  and a 240s base cap: retries cannot mask a deterministic failure, and nextest
  runs the retry serially after the parallel pass, which is exactly when a
  starved test succeeds. `cargo nextest run --all` then reported 3956/3956
  passed, exit 0, with no test flagged flaky.
- References: `.config/nextest.toml`, `scripts/release-manager.sh`
  (`ship --execute` → `cargo nextest run --all`), LESSON-027.

## LESSON-029: A floating `stable` toolchain can fail every PR without touching it (2026-10-01)

- Issue: `rust-toolchain.toml` pins `channel = "stable"`, so CI (and the release
  gate) follow the runner's stable. When stable moved to 1.99, its new
  `clippy::assert_is_empty` lint fired at ~100 pre-existing sites
  (`assert!(x.is_empty())` / `assert!(!x.is_empty())` in tests across
  memory-core, memory-mcp, memory-storage-turso/redb, memory-cli and e2e). The
  required `Quick PR Check` then failed on a PR that touched none of those
  files (the flagged sources were byte-identical to `main`, whose last runs
  were green only because they predated the toolchain bump).
- Impact: every open PR was blocked at merge time, and the failure read like a
  regression introduced by the PR under review.
- Fix: migrate the sites (`assert!(x.is_empty())` → `assert_eq!(x.len(), 0)`,
  `assert!(!x.is_empty())` → `assert_ne!(x.len(), 0)`), verified with the exact
  toolchain: `rustup toolchain install 1.99.0 --component clippy`,
  `cargo +1.99.0 clippy --workspace --all-targets -- -D warnings`.
- Prevention: when CI and local disagree about clippy, suspect a stable bump
  before suspecting the diff — run the gate with the version CI will use
  (`rustup toolchain install <version>` / `cargo +<version> clippy`). If
  surprise migrations are unwanted, pin an exact toolchain instead of `stable`;
  that is a policy choice, not a code fix.
- References: `rust-toolchain.toml`, `.github/workflows/ci.yml`
  (`./scripts/code-quality.sh clippy --workspace`), PR #1107.

## LESSON-030: `GITHUB_TOKEN`-published releases never fire `release: published` (2026-10-03)

- Issue: the draft-first release flow publishes the GitHub Release with the
  repository `GITHUB_TOKEN`. Events raised by that token do **not** create
  workflow runs (only `workflow_dispatch` and `repository_dispatch` are
  exceptions), so `publish-crates.yml` — which listens to `release: published` —
  had not run for a release since 2026-04-22 (the last release published by the
  user account). crates.io sat at `0.1.34` (`do-memory-mcp`: `0.1.31`) while the
  workspace moved to `0.1.45`, and no check noticed.
- Impact: five months of releases were never published to crates.io, and every
  user-visible "release" claim was true for GitHub Releases but false for the
  registry.
- Detection: `gh release view <tag> --json author` → `github-actions[bot]` means
  the event was suppressed; compare `curl -s -A ci-verify https://crates.io/api/v1/crates/<crate> | jq -r .crate.max_version`
  with the workspace version (the `User-Agent` is required — without it crates.io
  returns an error body and the command prints nothing); `gh run list --workflow=publish-crates.yml --event=release`
  shows the last run that actually fired.
- Fix: `release.yml`'s `dispatch-publish` job (needs `actions: write`) calls
  `gh workflow run publish-crates.yml --ref <tag> -f dry-run=false` after the
  release is published; the `release: published` trigger stays as a safety net
  for user-published releases, and re-runs no-op on already-published versions.
- Prevention: never rely on a token-raised event to start a second workflow —
  dispatch it explicitly (documented exception), or use an App/PAT token. When
  an event-triggered workflow "does not run", check the *author* of the
  triggering object before debugging the workflow file.
- References: `.github/workflows/release.yml`, `.github/workflows/publish-crates.yml`,
  `agent_docs/github_actions_patterns.md`, PR #1123.

## LESSON-031: crates.io trusted publishing matches the CALLING workflow, never the ref (2026-10-03)

- Issue: the trusted-publisher config stores owner + repository + **workflow
  filename** + environment. crates.io reads the OIDC `workflow_ref` claim — for a
  job defined in a *called* reusable workflow that is the caller's filename
  (`job_workflow_ref` is not read at all) — and it does not validate the git ref
  or tag. Two traps follow: (a) moving publish steps into `x.yml` while the
  registration says `publish-crates.yml` fails the exchange; (b) a workflow
  dispatch with `dry-run=false` from a branch can publish for real unless the
  environment and the workflow itself restrict refs.
- Impact: a refactor can silently break publishing at release time, and a
  write-access collaborator could publish an arbitrary version from a branch.
- Fix: keep the auth step inside the registered workflow file (a composite
  action keeps `workflow_ref` = `publish-crates.yml`; a reusable workflow would
  also need the caller's filename registered and cannot elevate
  `id-token: write`), register all four crates once (crate → Settings → Trusted
  Publishing), restrict the `crates.io` environment to `v*` tags, and gate real
  publishes in-workflow on `github.ref_type == 'tag'` with a tag↔version check.
- Prevention: treat the trusted-publisher config as part of the workflow's
  interface; pin the exchange action by SHA (`rust-lang/crates-io-auth-action`);
  remember the token is single-use with a 30-minute lifetime and that each job
  performs its own exchange. `pull_request_target`/`workflow_run` triggers are
  rejected by crates.io.
- References: `plans/adr/ADR-078-Trusted-Publishing-OIDC.md` (amendment),
  `.github/actions/publish-crate/action.yml`, `SECURITY.md`, PR #1123.

## LESSON-032: A skipped dependency makes every downstream job skip (2026-10-04)

- Issue: `release.yml` kept a `pull_request` trigger for a "dist plan smoke
  test". `preflight` is gated on `!github.event.pull_request`, and `plan`
  declares `needs: [preflight]` without `always()`, so on every PR the whole
  workflow ran preflight(skipped) → plan(skipped) → build-*(skipped) →
  host/announce(skipped). The PR run executed nothing, yet it appeared in every
  PR's check list and its comments claimed a smoke test that never ran.
- Impact: a no-op required-looking run; misleading rationale; a path filter
  "optimised" something that did nothing. Nothing was validated by it, so
  removing it changed no coverage.
- Detection: on any PR, look at the run's jobs — if the first job is `skipped`,
  all dependents are skipped unless they use `always()`. Compare the check names
  a workflow actually produces (`gh pr checks`).
- Fix: drop the trigger and state the truth in the workflow comment (release
  validation happens at tag time; code PR coverage lives in `ci.yml`). See
  PR #1125.
- Prevention: before adding or gating a trigger, run it once and confirm at
  least one job executes; `needs:` without `always()` propagates skips.
- References: `.github/workflows/release.yml`,
  `plans/GOAP_RELEASE_PIPELINE_C5_C7_2026-10-04.md`, PR #1125.

## LESSON-033: mdBook `create-missing` turns `#anchor` placeholders into files (2026-10-04)

- Issue: `book/src/SUMMARY.md` listed un-written chapters as anchor links
  (`- [Architecture](#architecture) <!-- TODO -->`). With `create-missing = true`
  in `book/book.toml`, `mdbook build` treated the anchor as a path and created
  one-line files literally named `book/src/#architecture`, `#patterns`, …; a
  `git add -A` then committed nine of them.
- Impact: stray files in the repo, pages rendered as `#architecture.html` that
  cannot be navigated to, and diffs polluted by build artifacts.
- Fix: remove the files and use mdBook draft chapters (`- [Title]()`), which
  create nothing; rebuild and confirm the tree is clean.
- Prevention: after any `mdbook build`, check `git status` before `git add -A`;
  keep TODO chapters as `[]()` drafts, never as `#anchor` links.
- References: `book/src/SUMMARY.md`, `book/book.toml`, PR #1125.

## LESSON-034: `gh attestation verify` without `--signer-workflow` under-binds (2026-10-04)

- Issue: the new docs told users `gh attestation verify <file> -R <owner/repo>`
  while the prose claimed the artifact "was built by this repository's
  workflow". The default policy checks the repository/owner identity only, so
  any workflow in the repository that can request an attestation satisfies it.
- Impact: a documented verification weaker than it reads, on a page whose whole
  purpose is trust.
- Fix: add `--signer-workflow <owner>/<repo>/.github/workflows/release.yml`
  (GitHub CLI manual: "Ideally, the path of the signer workflow is also
  validated using `--signer-workflow`"), plus `--source-ref refs/tags/vX.Y.Z`
  for tag-pinned verification. Applied to the book page, `SECURITY.md` and the
  `release-guard` skill.
- Prevention: when documenting attestation verification, always pin the signer
  (workflow or certificate identity), not just the repository.
- References: `book/src/verify-a-release.md`, `SECURITY.md`,
  `.agents/skills/release-guard/SKILL.md`, PR #1125.

## LESSON-035: Bot/agent PR branches can silently revert merged work (2026-10-06)

- Issue: the PR #1130 tip (`da0c867c`, via clobber commit `0401fa9c`) contained a
  full revert of already-merged main work — `transaction_scope.rs`,
  `recommendation_index*.rs`, `ranking_guard_tests.rs`, monitoring tests, the
  plans trackers and `install-hooks.sh` — pushed as a "stale branch" diff.
  `mergeable=MERGEABLE` said nothing about it because GitHub compares against
  the merge base, not the tree the branch actually carries.
- Impact: merging it would have deleted four merged PRs' worth of code and
  docs. The visible symptom was an unrelated commitlint failure hiding a
  destructive diff.
- Fix: rebuilt the branch from `origin/main` with only the real change
  (`b9331ce3`'s five files) as one commit, force-pushed, and commented the
  evidence on the PR.
- Prevention: before merging a long-lived or bot-authored branch, run
  `git diff <merge-base> <tip> --stat` (or `git diff 810db30b da0c867c`) and
  reject any branch whose tip removes files it never touched; treat
  `mergeable=MERGEABLE` as "no textual conflict", never as "no lost work".
- References: PR #1130, `plans/STATUS/VALIDATION_LATEST.md`.

## LESSON-036: `nextest --message-format libtest-json` needs the experimental flag (2026-10-09)

- Issue: the new isolated Turso nightly job piped
  `cargo nextest run … --message-format libtest-json > report.json`. Nextest
  refuses that flag without `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`, so the job
  failed during startup — both uploaded report files were 0 bytes and the
  ignored tests never ran. The nightly was red for the wrong reason, which
  looked identical to the libsql crash the job exists to expose.
- Impact: three nightlies proved nothing about the ignored-test population
  while appearing to.
- Fix: set `NEXTEST_EXPERIMENTAL_LIBTEST_JSON: 1` on the step (PR #1159);
  verified locally that JSON is emitted.
- Prevention: when redirecting a tool's machine-readable output to a file,
  assert the file is non-empty (or check the tool's version gate) before
  treating a red job as a test failure; copy the env from the existing job that
  already uses the flag.
- References: `.github/workflows/nightly-tests.yml`, PR #1159, issue #1091.

## LESSON-037: floating `stable` adds deny-by-default lints between releases (2026-10-09)

- Issue: CI runs floating `stable` (1.99) while the local toolchain was 1.98, so
  two new lints failed PR CI invisibly to local runs: clippy
  `assert_is_empty` (`assert!(x.is_empty())` in new tests) and rustdoc
  `redundant_explicit_links` (`[`X`](crate::X)` when the label already
  resolves). Three PRs burned a CI cycle each.
- Impact: "clippy clean locally" was not evidence; each failure needed a CI log
  round trip.
- Fix: `rustup update stable` (→ 1.99.0) and fix the lints at the source
  (`assert_eq!(x.len(), 0)` and the bare label) — no `#[allow]`.
- Prevention: before diagnosing a CI-only lint failure, check the toolchain
  versions (`rustc --version` vs the CI log) and update local stable; treat a
  green local clippy on a floating-toolchain repo as weakly informative, and
  prefer the canonical `./scripts/code-quality.sh clippy --workspace` scope
  (per-package clippy differs under feature unification).
- References: PRs #1145, #1150, #1152, `scripts/code-quality.sh`.

## LESSON-038: strict up-to-date checks make merges serial; shared target dirs make test failures flaky (2026-10-09)

- Issue: ruleset `9591004` sets `strict_required_status_checks_policy`, so every
  merge to `main` invalidates the other open PRs' required runs; the 17-PR
  backlog needs one branch update + CI cycle per merge. Separately, several
  worktrees sharing one `CARGO_TARGET_DIR` produced mass test failures
  (including a pure cosine-similarity test) when a sibling build replaced test
  binaries mid-run.
- Impact: merge throughput is CI-bound, not work-bound; and an unisolated test
  failure can look like a regression when it is target-dir thrash.
- Fix/prevention: plan merges as a serial pipeline (update the next branch
  immediately after each merge); before believing a mass failure, re-run the
  failing test in isolation or in a fresh target dir, and prefer `-E` filters
  over full-suite runs while siblings compile.
- References: `.agents/skills/ci-poll/SKILL.md`, `scripts/merge-pr.sh`, PRs
  #1150/#1154.

## LESSON-039: one missing `cargo fmt --all` reds three CI surfaces at once (2026-10-11)

- Issue: the rebuilt #1147 branch was pushed after conflict resolution without a
  final `cargo fmt`. The fast gate failed, and `e2e-tests/tests/quality_gates.rs`
  `quality_gate_formatting` failed *inside* both the Storage Matrix job and the
  Code Coverage job (which runs the whole test suite), so one local miss produced
  three red jobs and a wasted ~1h CI cycle.
- Impact: a formatting miss is never local; suite-wide gates re-run it.
- Fix/prevention: run `cargo fmt --all -- --check` (or `./scripts/code-quality.sh
  fmt`) as the **last** step before every push, including after conflict
  resolution and after hand-indenting merged blocks (rustfmt does not always
  reindent a closure passed as a call argument — verify, do not assume).
- References: `.agents/skills/code-quality/SKILL.md`, PR #1147.

## LESSON-040: floating toolchains break dependency compilation, not just lints (2026-10-04..11)

- Issue: the fuzz workflow floated `nightly`; a nightly-only change to
  `core::intrinsics` broke the transitive `branches 0.4.5` crate with no
  repository change, so every fuzz target exited 1 as a "startup failure" and
  the weekly run was red for two cycles.
- Impact: a scheduled gate goes red with zero code change and no obvious cause.
- Fix/prevention: pin dated toolchains for every job that must not float
  (`RUSTUP_TOOLCHAIN: nightly-2026-09-26` + the same in `dtolnay/rust-toolchain`);
  validate a pin by building the offending crate locally with that toolchain.
- References: `.github/workflows/fuzz.yml`, PR #1161.

## LESSON-041: `gh pr checks` prints cancelled checks as `fail` (2026-10-11)

- Issue: after stale runs were cancelled, `gh pr checks` showed ~20 rows as
  `fail` with durations, which read as genuine failures; the jobs API showed
  `completed/cancelled`. Follow-on effect: cancelled required checks also keep
  `mergeStateStatus` from reaching `CLEAN`, so the merge gate refuses.
- Impact: wasted diagnosis, and cancelling other PRs' runs silently blocks them.
- Fix/prevention: classify a row with `gh api
  repos/{owner}/{repo}/actions/jobs/<job_id> --jq '.status, .conclusion'` before
  treating it as failed; do not cancel runs of PRs you intend to merge — re-run
  them with a fresh push/`gh pr update-branch` instead.
- References: `.agents/skills/ci-fix/SKILL.md`, `.agents/skills/pr-readiness/SKILL.md`.

## LESSON-042: same-file structural splits in parallel PRs force manual rebuilds (2026-10-11)

- Issue: #1149 and #1151 both split `memory-core/src/storage/backend.rs` into a
  `backend_trait.rs` module while #1152 split the same file into
  `backend/mod.rs` + `capabilities.rs`. Any two of them conflict on the file
  layout, and the later branches had to be rebuilt by hand (4 rebuilds), each
  repeating the same conflict resolution.
- Impact: merge order becomes load-bearing; rebuilding costs a full local
  verification cycle plus a CI cycle.
- Fix/prevention: land the structural split first, then rebuild the rest on top
  of it (keep the winner's layout and re-apply the other PR's intent); never
  merge a stacked PR before its base; when a branch is rebuilt, re-run fmt +
  workspace clippy + the PR's own tests before pushing.
- References: PRs #1149, #1151, #1152.

## LESSON-043: the capability checker distinguishes fail-closed defaults (2026-10-11)

- Issue: `list_embedding_ids` (a defaulted `StorageBackend` method that returns
  `Error::CapabilityUnavailable`) fit no class in the inventory vocabulary:
  `capability-gated` requires naming a predicate, and the typed-fallback class
  warns forever.
- Impact: the fail-closed gate either blocks a correct default or keeps the
  inventory noisy.
- Fix/prevention: the checker now has a `fail-closed` class (coherence: the
  default body must be a `CapabilityUnavailable` error, with a justification) and
  an F6 fixture; a new defaulted `StorageBackend` method must be classified in
  `scripts/storage-backend-capabilities.toml` in the same PR that adds it.
- References: `scripts/check-storage-backend-capabilities.sh`, PR #1158.

## LESSON-044: the drift check re-runs its original event payload (2026-10-11)

- Issue: `Check for unreleased changes` fails when drift is critical and
  auto-adds the `release-preparation` label, but re-running the failed job
  re-evaluates the *original* payload (no label) and fails again.
- Impact: a re-run cannot clear the check; only a new event can.
- Fix/prevention: after the label appears, push a commit or run
  `gh pr update-branch <pr>` so the new `synchronize` event carries the label.
- References: `.github/workflows/release-drift.yml`, PRs #1161 and the wave.

## LESSON-045: fixture timestamps must encode the assertion's ordering (2026-10-11)

- Issue: the #1068 regression test built the "newer" copy with an `end_time`
  that preceded the "older" copy's `start_time`, so the assertion measured the
  inverse of the intended comparison and failed on correct code.
- Impact: a green-implementation red test wastes a cycle and can trigger a wrong
  fix.
- Fix/prevention: construct fixture timestamps as explicit absolute values in
  the order the assertion assumes; treat a failing assertion on freshly written
  code as a fixture hypothesis before touching the implementation.
- References: `memory-core/tests/get_all_episodes_merge.rs`, PR #1068.

## LESSON-046: advisories are gated by two tools in one step — deny alone is not the truth (2026-10-15)

- Issue: `origin/main` HEAD went red on a fresh batch of RustSec advisories. The
  Quality Gates job runs **both** `cargo deny check advisories` and
  `cargo audit --json` (then fails on `.vulnerabilities.count > 0`) — see
  `.github/workflows/ci.yml:380-398`. A rustls-only bump cleared deny but left
  five quinn-proto advisories that only audit reports, because the two tools
  consult different advisory DB clones and surface different sets.
- Impact: an advisory PR scoped from deny output alone re-reds main and needs a
  second full CI cycle; waiver text gets written twice, in two files.
- Fix/prevention: before proposing an advisory fix, enumerate the **union** of
  `cargo deny check advisories` and `cargo audit --json` against the target
  lockfile. Every ignore needs an entry in **both** `deny.toml`
  (`[advisories] ignore`) and `.cargo/audit.toml` — the lists are independent
  and both are blocking.
- References: `.github/workflows/ci.yml`, `.agents/skills/code-quality/SKILL.md`, PR #1174.

## LESSON-047: a stale advisory DB clone makes `cargo deny` report a false green (2026-10-15)

- Issue: local `cargo deny` reported "advisories ok" against the exact lockfile
  CI proved failing. Its DB clone under `~/.cargo/advisory-dbs/…` was pinned at
  commit `e2111519` (2026-09-25) while `cargo audit`'s clone
  `~/.cargo/advisory-db` was current; the refresh was blocked by a `db.lock`
  left behind by an earlier crash.
- Impact: a false green mis-scopes the fix, and the PR is declared done before
  it can possibly pass.
- Fix/prevention: check the DB's checkout date before believing a green advisory
  run, and cross-check with the second tool. Investigate a `db.lock` (find the
  holding process) rather than deleting it. For per-advisory facts, use the
  already-cloned markdown at `~/.cargo/advisory-db/crates/<pkg>/<ID>.md` or
  `rustsec.org/advisories/<ID>.html` — `api.rustsec.io` and `crates.io/api/v1`
  returned empty bodies to `curl` here.
- References: LESSON-046, `.agents/skills/code-quality/SKILL.md`.

## LESSON-048: bump to the minimal patched version, then re-run the WASM gate (2026-10-15)

- Issue: `cargo update -p quinn-proto` chose 0.11.19 although the advisory's
  `[versions] patched` range only required 0.11.18. The bump also dragged
  `rand` 0.9→0.10 and `getrandom` 0.3→0.4 across majors — and `getrandom` is
  exactly the crate whose `--cfg getrandom_backend="wasm_js"` RUSTFLAGS are
  hardcoded in the WASM Compile Check gate (`.github/workflows/ci.yml:505-512`).
- Impact: an over-pull multiplies lockfile churn and review surface; a
  `getrandom` major can break the wasm target without touching any code path
  the advisory is about.
- Fix/prevention: read `[versions] patched` from the advisory and pin the floor
  with `cargo update -p <pkg> --precise <min-patched>`. When a bump crosses a
  `getrandom` major, re-run the WASM job's exact command *with its RUSTFLAGS*
  locally. Keep advisory PRs lockfile-only; `Cargo.toml` changes belong in a
  separate PR.
- References: `agent_docs/dependency_upgrades.md`, PR #1174.

## LESSON-049: bot-applied labels cannot clear the drift check — toggle them as a human (2026-10-15, extends LESSON-044)

- Issue: `release-drift.yml` fails when cadence is critical *and*
  `github.event.pull_request.labels` lacks `release-preparation`, then applies
  that label with `GITHUB_TOKEN`. GitHub suppresses workflow-triggered actions,
  so no `labeled` run fires, and `gh run rerun <id>` only replays the original
  label-less payload — it failed again. A human label toggle emitted a fresh
  `labeled` event carrying the label and passed.
- Impact: the documented retry path (`push a commit` / `gh pr update-branch`)
  re-triggers the whole suite, which under `strict_required_status_checks_policy`
  invalidates the PR's own required runs.
- Fix/prevention: `gh pr edit <n> --remove-label release-preparation && gh pr
  edit <n> --add-label release-preparation` (needs `labeled` in the workflow's
  `pull_request` types). Separate fact: a permanently-`skipping`
  `Maintain canonical drift issue` row on PRs is structural
  (`if: github.event_name != 'pull_request'`), not a skipped gate.
- References: `.github/workflows/release-drift.yml`, LESSON-044, PR #1174.

## LESSON-050: `main` is guarded by a ruleset, not branch protection (2026-10-15)

- Issue: `gh api repos/{owner}/{repo}/branches/main/protection` 404s while merges
  are still gated. The guard is ruleset `main-protection` (id 9591004). Required
  contexts are exactly `CI / Required` and `Codacy Static Code Analysis`;
  `strict_required_status_checks_policy=true`,
  `required_review_thread_resolution=true`,
  `required_approving_review_count=0`,
  `dismiss_stale_reviews_on_push=true`,
  `require_extra_approval_for_unattributed_changes=true`.
- Impact: merge-state questions get answered from the wrong API, and green
  non-required rows are mistaken for blockers (or vice versa).
- Fix/prevention: read `gh api repos/{owner}/{repo}/rules/branches/main` — the
  payload is an array of *rule* objects (each carrying `ruleset_id`), and the
  contexts live at `.parameters.required_status_checks` (not `.context`), so dump
  it to a file before scripting jq. `CI / Required` is not a per-job row: it is
  the aggregate job `required` (`name: CI / Required`, `needs:` the substantive
  jobs, fail-closed on any non-success dependency incl. `skipped`), so it only
  appears after the slowest dependency (often `multi-platform`) finishes —
  `mergeStateStatus` stays `BLOCKED` while every visible row is green. Required-
  check drift is a ruleset change, never a workflow edit.
- References: `.github/workflows/ci.yml` (`required` job), ADR-079, PR #1174.

## LESSON-051: `ci-check` counts scheduled runs by head SHA, so an always-red nightly blocks releases (2026-10-15)

- Issue: `do_ci_check` in `scripts/release-manager.sh` lists
  `gh run list --branch main --commit <head>` and counts every completed
  non-success run without fetching the `event` field — so a `schedule`-triggered
  nightly failure attached to the release SHA blocks the ship even though the
  commit itself is unchanged (`isolated-turso-native-tests` has been red since
  it first ran 2026-10-07).
- Impact: releases are held by pre-existing debt; advancing main HEAD only opens
  a *window*, because the next nightly (cron `0 2 * * *`, observed start
  06:36-08:37 UTC) re-attaches to the new SHA.
- Fix/prevention: for v0.1.45 the decision was to keep the gate strict and ship
  inside the window, not to weaken `ci-check`. The durable fix — filter on
  `.event`, or allowlist per-event failures with a tracked issue — needs an
  explicit decision first; do not silently exclude schedule runs.
- Resolution (2026-10-20, ADR-083, PR #1204): the explicit decision is now made —
  `schedule` and `workflow_dispatch` runs are **advisory** for release gating, and only
  `push`/`pull_request` runs gate a release. `do_ci_check` classifies runs through
  `scripts/lib/release-gate-events.sh` (an allowlist, so a newly-introduced event type does
  not silently become gating), queries gating and advisory runs **separately** so advisory
  noise cannot consume the `--limit` window and hide the real `CI` run, and
  `scripts/test-release-workflow.sh` drives the real `do_ci_check` against a stubbed `gh`.
  A tag-triggered nightly is a `push` event and therefore **still gates** a release.
- References: `scripts/release-manager.sh`, `scripts/lib/release-gate-events.sh`,
  `.github/workflows/nightly-tests.yml`, ADR-027, ADR-083.

## LESSON-052: `statusCheckRollup` keeps superseded runs, so a label toggle fakes a CI failure (2026-10-15)

- Issue: after the human `release-preparation` toggle cleared the drift check on
  #1174, the head SHA still carried the two earlier `FAILURE` rows for
  `Check for unreleased changes`. `scripts/check-pr-readiness.sh` counts raw
  rollup entries, so it reported `32 pass, 2 fail` and `merge-pr.sh` refused with
  "readiness gate failed" while `mergeStateStatus` was `CLEAN` and GitHub's
  evaluation saw the newest run (success) as authoritative.
- Impact: a correct, fully green PR becomes unmergeable through the sanctioned
  path; the tempting workaround (delete the failing runs, or merge around the
  gate) is a bypass and is not acceptable.
- Fix/prevention: re-run on a **fresh head SHA** — rollups are per SHA, so
  `git commit --allow-empty -m "chore: re-trigger CI …" && git push` drops the
  stale rows (the `synchronize` payload now carries the label, so the drift check
  passes). Durable fix: reduce the rollup to the latest run per check name
  (prefer an unfinished entry over a finished one) before classifying. Never
  delete workflow runs to silence a gate.
- References: `scripts/check-pr-readiness.sh`, `scripts/merge-pr.sh`, LESSON-041, LESSON-049.

## LESSON-053: a negated input guard makes the crates.io publish chain falsely green (2026-10-16)

- Issue: `release-manager.sh ship --execute` published v0.1.45, `release.yml`
  dispatched `publish-crates.yml` on the tag and the run went **success** for
  `Publish do-memory-core` — yet `crates.io/api/v1/crates/do-memory-core/versions`
  still listed `0.1.34` as newest. The log shows
  `Version not on crates.io, will publish`, `aborting upload due to dry run`,
  `Retrieved token successfully`, then
  `##[endaction … conclusion=skipped;duration_ms=0]` for `Publish to crates.io`.
  Cause: `if: ${{ !inputs.dry-run && … }}`. Composite-action inputs are
  **strings**, and GitHub coerces any non-empty string to `true`, so the value
  `publish-crates.yml` renders (`dry-run: ${{ inputs.dry-run == true }}` →
  `"false"`) makes `!inputs.dry-run` **false** and skips the real upload.
  Introduced 2026-10-03 in `493cfb01` (#1109); v0.1.44 and v0.1.45 never
  published, and `do-memory-mcp` has been stuck at 0.1.31.
- Impact: the release looks complete — release assets, attestations and a green
  publish workflow all say "shipped" — while the crates.io audience gets nothing
  and the gap only surfaces as the *next* crate's
  `dependency … did not appear on crates.io within 300s` timeout. Retagging is
  not a fix either: a run takes the workflow **and** the composite action from
  its tag SHA, and `Verify tag matches version` binds tag to manifest, so a
  superseded tag can never publish retroactively.
- Fix/prevention: never negate an input in a gate — compare to the literal
  (`inputs.dry-run != 'true'`), the form the tag guard and auth step already used
  in the same file. Make a green publish prove itself (`Verify publish landed`
  polls the versions endpoint, bounded like LESSON-014) and query version
  **membership** (`[(.versions // [])[].num] | index($v) != null`) instead of
  `.versions[0].num` equality, so re-runs stay idempotent once a newer version
  exists. Sensors: `scripts/validate-gate-contract.sh` fails any `!inputs.` guard
  in the publish files and `scripts/test-release-workflow.sh --publish-fixtures`
  requires the proof step. After any release, assert the publish externally —
  `curl -sS -A ci-verify https://crates.io/api/v1/crates/<crate>/versions | jq -r --arg v "$TAG" '[(.versions // [])[].num] | index($v) != null'`
  must print `true`; a green workflow is not evidence that an artifact exists.
  **Send the `User-Agent`:** crates.io rejects the request without one, and the empty
  body it returns makes a bare `curl | jq` print nothing — which looks like a missing
  version and, worse, is easy to skim past (the same empty-report trap as the
  libtest-json gate above).
- References: `.github/actions/publish-crate/action.yml`,
  `.github/workflows/publish-crates.yml`, ADR-078 (amendment 2026-10-16),
  LESSON-014, #1179.

## LESSON-054: the tag-push changelog automation rewrites the whole CHANGELOG — close, never merge (2026-10-16)

- Issue: every tag push runs `.github/workflows/changelog.yml`, which executes
  `git-cliff --config cliff.toml --output CHANGELOG.md` and opens an
  `automated-pr` (`#1178` for `v0.1.45`). Because git-cliff regenerates the file
  **from git history**, the PR replaces the curated Keep-a-Changelog prose with
  one-line subjects all the way back to `## [0.1.10]` (+405/−264 across the same
  single `CHANGELOG.md`, hunk header `@@ -7,317 +7,458 @@`) and re-dates the
  section to the tag day. It looks
  mergeable and even "fixes" the release date, which is what makes it dangerous.
- Impact: merging it deletes the evidence-bearing entries the release process
  requires (each `### Fixed` bullet names the PR and the behaviour it restored),
  and it collides with any open PR editing the top of `CHANGELOG.md`. Precedent:
  `#1119`, `#1105`, `#1054`, `#1036` — every previous auto-changelog PR was
  closed without merging.
- Fix/prevention: treat `chore(changelog): update CHANGELOG.md for v*` as a
  **close-only** PR; the changelog is written by the release-prep PR
  (`chore(release): prepare vX.Y.Z changelog and version docs`) per AGENTS.md.
  Land release-facing `CHANGELOG.md` changes *before* reviewing that automation,
  and mention the date truth in the prep PR instead of accepting the regen. If
  the automation stays unwanted, the fix is to its trigger/config in its own PR —
  never a mid-release edit.
- References: `.github/workflows/changelog.yml`, `cliff.toml`,
  `scripts/release-manager.sh`, LESSON-044.

## LESSON-055: nightly test configuration masquerades as a storage bug (2026-10-16)

- Issue: three knobs move the numbers with no code changing. `nightly-tests.yml`
  cron is `0 2 * * *` but scheduled runs actually started 06:30–06:49Z (queue
  delay), and the workflow also fires on `push` of `v*.*.*` — `workflow_dispatch`
  is the only reliable clock. `[[test]] required-features` is a BUILD-level gate:
  with the feature off the target is not compiled at all, so test counts
  silently change. `--run-ignored only` FORCE-RUNS `#[ignore]`d tests. The red
  nightly (106 run / 36 failed, run 38883104337 at 8ee711bd) was partly a
  *configuration artifact*, proven by the split: `--run-ignored only` →
  86 passed / 0 failed (default schema) plus
  `--features turso_multi_dimension --test multi_dimension_routing --test phase1_validation --run-ignored all`
  → 20 passed / 0 skipped, and `86 + 20 = 106` = the pre-fix total.
- Impact: a schema/config mismatch surfaced by force-running ignored tests reads
  as a Turso storage defect, and a count that silently omits feature-gated
  targets makes two runs of the same commit look contradictory — so the fix
  gets aimed at storage instead of at the test configuration.
- Fix/prevention: never quote a nightly pass/fail count without the feature
  matrix and the `--run-ignored` mode that produced it. A schema/config mismatch
  surfaced by force-running ignored tests is a **TEST CONFIGURATION** finding,
  not a storage bug. Use `--run-ignored all` when you also want feature-gated
  targets compiled.
- References: `.github/workflows/nightly-tests.yml`, `Cargo.toml`,
  `.agents/skills/ci-fix/SKILL.md`, `.agents/skills/github-workflows/SKILL.md`,
  LESSON-051, ACT-403, ACT-407.

## LESSON-056: three advisory registers, and `cargo audit` is gated on `.vulnerabilities` only (2026-10-16)

- Issue: 12 Dependabot alerts were **6 advisories × 2 lockfiles**
  (`Cargo.lock` + `fuzz/Cargo.lock`); `Cargo.toml:14` sets `exclude = ["fuzz"]`,
  so no local gate ever reads the fuzz lockfile. PR #1174 cleared the audit/deny
  gates and closed ZERO Dependabot alerts. Two distinct mechanisms make an
  advisory invisible to BOTH CI gates: (1) RustSec `informational` advisories —
  `lru` RUSTSEC-2026-0002
  (`~/.cargo/advisory-dbs/<db>/crates/lru/RUSTSEC-2026-0002.md`:
  `informational = "unsound"`, `aliases = ["GHSA-rhfx-m35p-ff5j"]`,
  `patched = [">= 0.16.3"]`) is routed by cargo-audit to `.warnings.unsound`,
  NEVER to `.vulnerabilities`, and both CI audit steps
  (`.github/workflows/ci.yml:383-402`,
  `.github/workflows/security.yml:77-95`) fail only when
  `.vulnerabilities.count > 0`; `cargo deny` does not gate informational ids
  either. (2) A GHSA never adopted into RustSec (`libsql-sqlite3-parser`,
  CVE-2025-47736 — the advisory DB has `libsqlite3-sys` but no
  `libsql-sqlite3-parser`) is invisible to both by absence. Measured twice on
  the current lock (cargo-audit 0.22.2): in-repo `cargo audit --no-fetch --json`
  → rc 0, `.vulnerabilities.list` empty, with 5 `.warnings.unsound` (git2
  RUSTSEC-2026-0183/0184, lru RUSTSEC-2026-0253 ×3) + 1
  `.warnings.unmaintained` (RUSTSEC-2024-0449) still reported; the same
  `Cargo.lock` run from `/tmp` (so `.cargo/audit.toml` ignores do NOT apply) →
  rc 1 with 8 `.vulnerabilities` (rsa 2023-0071,
  rustls-webpki 2026-0049/0098/0099/0104, h2 2026-0258, rustls 2026-0346/0349).
- Impact: a green audit and a green deny prove only that nothing un-ignored
  reached `.vulnerabilities` — every one of the three registers can still list
  advisories while both CI gates pass, which is exactly how #1174 could go green
  and close no alerts.
- Fix/prevention: a green `cargo audit` proves only "nothing un-ignored reached
  `.vulnerabilities`" — always cite the ignore list and the surviving
  `.warnings` count. Classify an advisory by **register** before promising a
  fix. Dedupe alerts by `security_advisory.ghsa_id`, never by row count, and
  inspect `.security_advisory | keys` before writing a bulk `jq` projection
  (wrong keys silently yield empty columns). Corrected in this session: an
  earlier draft claimed informational advisories DO fail `cargo audit` —
  measurement said otherwise, which is itself the lesson.
- References: `.github/workflows/ci.yml`, `.github/workflows/security.yml`,
  `.cargo/audit.toml`, `Cargo.toml`, `fuzz/Cargo.lock`,
  `.agents/skills/code-quality/SKILL.md`, `.agents/skills/ci-fix/SKILL.md`,
  LESSON-053, ACT-404, PR #1174.

## LESSON-057: a version bump does not make a repo shippable, and the drift gate files its own issue (2026-10-16)

- Issue: `release-manager.sh` Phase 0 runs
  `./scripts/verify-release-state.sh --check-unreleased`, which validates
  `CHANGELOG.md`, `ROADMAP_ACTIVE.md` "Released Version" and `STATUS/CURRENT.md`
  against `workspace_version`. After the 0.1.46 bump PR (#1181) was queued it
  still printed: `❌ ROADMAP_ACTIVE.md says v0.1.45 (expected v0.1.46)`,
  `❌ STATUS/CURRENT.md references v0.1.45`,
  `❌ CHANGELOG.md missing entry for v0.1.46`,
  `⚠️ parse-changelog not installed`. Second half: `check-release-drift.sh`
  returns `critical/version_not_advanced` whenever
  `workspace_version == latest_tag` with commits after the tag (the normal
  post-ship state), and its `maintain-issue`
  job (`.github/workflows/release-drift.yml:98-149`, concurrency group
  `release-drift-canonical-issue`) **creates-or-updates one canonical
  `release-drift` issue from main**, closing it at `:156` when drift resolves —
  #1182 was opened 2026-10-16T13:32Z by `app/github-actions`.
- Impact: a bump-only branch is still not shippable, and without
  `parse-changelog` installed the release notes silently fall back. The drift issue is a
  symptom, not a task: closing it by hand only makes the next detection
  re-create it — and because it is a real issue it moves the open-issue count
  the plans validators compare (5 → 6), which is how `validate-plans.sh`
  tracker-drift caught a stale duplicate count inside `STATUS/CURRENT.md`.
- Fix/prevention: land CHANGELOG `## [X.Y.Z]` + the version-doc flip as **ONE
  release-prep PR before any tag date is promised** (precedent #1169), and
  install the tool (`cargo install --locked parse-changelog`) or release
  notes silently fall back — `release-preparation` is that PR's label, never an
  override on unrelated PRs. Every copy of this instruction previously named the
  crate **`parse-changelog-cli`, which does not exist** on crates.io (HTTP 404;
  the real crate is `parse-changelog` 0.6.17, binary `parse-changelog`), so an
  agent following the hint failed the install and concluded the tool was
  unavailable. Verify a crate name against crates.io before writing an install
  command into a guide. Resolve drift by advancing the state the gate
  reads, never by hand-closing #1182, and keep the sanctioned paths: ship only
  through `./scripts/release-manager.sh ship --execute` (`release-guard`,
  **never** `gh release create`) and merge only through
  `./scripts/merge-pr.sh <PR> --execute` (never `--admin`/`--force`).
- References: `scripts/release-manager.sh`, `scripts/verify-release-state.sh`,
  `scripts/check-release-drift.sh`, `scripts/validate-plans.sh`,
  `.github/workflows/release-drift.yml`, `CHANGELOG.md`,
  `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/STATUS/CURRENT.md`,
  `.agents/skills/release-guard/SKILL.md`,
  `.agents/skills/release-cadence-manager/SKILL.md`, #1181, ACT-411, ACT-412.

## LESSON-058: a "does not teach X" eval reddens the moment you write "never do X" (2026-10-16)

- Issue: the skills distill (#1175) added a `### Gate Semantics` bullet to
  `.agents/skills/release-cadence-manager/SKILL.md` saying ship only through
  `release-manager.sh ship --execute`, **never** `gh release create`. The prose
  is correct; the fixture was not. `Skill schema + evals` failed on
  `no-manual-ship ... FAIL`, `Command: ! rg -q 'gh release create'
  .agents/skills/release-cadence-manager/SKILL.md` (run 38914237079, job
  117017480826, exit 1). A presence-only predicate treats *any* mention of the
  anti-pattern as a violation, so it punishes the strongest available wording.
  24 skills carry this shape under `no-unguarded-gh-release-create`.
- Impact: the sensor is inverted — a skill that states the prohibition and a
  skill that teaches the bad command score identically, so it cannot
  distinguish the fix from the bug, and any future guardrail wording in those
  24 skills silently reddens CI. Second-order trap: `run-evals.sh --changed`
  derives its skill list from `git diff --name-only origin/main...HEAD`
  (`scripts/run-evals.sh:169-174`), i.e. **committed state only**, so uncommitted
  fixture edits are invisible and a local green run proves nothing about the push.
- Fix/prevention: write such fixtures as *require the canonical command AND
  reject unguarded mentions* — `rg -q 'release-manager\.sh ship --execute'
  SKILL.md && ! (rg -n 'gh release create' SKILL.md | rg -vi
  'never|no t|do not|don.t|avoid|forbid|instead')` (the form
  `release-guard/blocks-manual-release` already used). When editing a skill,
  read its `evals/evals.json` before adding prose about what is forbidden, and
  run the named skill (`./scripts/run-evals.sh <skill>`), not `--changed`, while
  iterating — then re-run after committing so CI and local see the same tree.
  Same family, second instance the same day: `coverage-waivers` fixture
  `documents-default-feature-pipeline` asserted
  `rg -qi 'default features only|no feature flags'`, so **correcting** that claim
  (coverage actually runs `--features do-memory-mcp/oauth`, per
  `.github/workflows/coverage.yml:98` — measured, not assumed) reddened the eval
  for stating the truth. Rule: a fixture must pin the *substance* (which file,
  which flag, which command), never a literal phrase, and it must be rewritten in
  the same change that corrects the claim — otherwise the sensor actively defends
  the stale wording.
- References: `.agents/skills/release-cadence-manager/evals/evals.json`,
  `.agents/skills/release-guard/evals/evals.json`, `scripts/run-evals.sh`,
  `.github/workflows/skill-evals.yml`, `agent_docs/coding_workflow.md`, PR #1175.

## LESSON-059: `((i++))` under `set -e` silently truncates a report loop (2026-10-16)

- Issue: `scripts/check-pr-readiness.sh` runs `set -euo pipefail` and counted
  problems with `((ISSUES_FOUND++))`. A post-increment evaluates to the **old**
  value, so the first increment from `0` makes the arithmetic command return
  exit status 1, and `set -e` kills the script mid-loop. Measured:
  `bash -c 'set -euo pipefail; X=0; ((X++)); echo survived'` → rc 1, nothing
  printed; `((++X))` and `X=$((X+1))` both survive.
- Impact: the all-PRs sweep printed **2 of 9** PRs and exited, with empty stderr
  and a plausible-looking report. Because rc=1 is also the script's legitimate
  "issues found" summary exit, an abort was indistinguishable from a normal
  failure — so the truncated output read as complete. The hidden consequence:
  PR #1171's 3 failing checks (`clippy::cast_precision_loss` at
  `memory-core/tests/storage_sync.rs:645`, plus Commit Message Lint) never
  surfaced, because #1171 sits past the abort point in `gh pr list` order.
- Fix/prevention: use `VAR=$((VAR + 1))` (or `((++VAR))`) for counters inside
  `set -e` scripts; never rely on a bare `((VAR++))`. Treat a summary tool's
  output length as an assertion, not an assumption — the sweep must report
  `N` PRs where `N == gh pr list --state open | wc -l`, or say plainly that it
  stopped early. Same rule as the empty-report trap in AGENTS.md's
  machine-readable-output bullet: a short report is a broken report until proven
  otherwise, and fixing it here turned "2 PRs, nothing else to see" into 9 PRs
  with one genuinely red branch.
- References: `scripts/check-pr-readiness.sh`, PR #1177, PR #1171,
  AGENTS.md (machine-readable-output trap), LESSON-026, LESSON-036
  (vacuous-green / empty-artifact class).

## LESSON-060: a package-scoped clippy run turns `#![expect(...)]` gates into false errors (2026-10-16)

- Issue: verifying a test-only change with
  `cargo clippy -p <crate> --tests -- -D warnings -A clippy::expect_used -A
  clippy::unwrap_used -A clippy::uninlined_format_args` (flags copied verbatim
  from `scripts/code-quality.sh:144`) reported two `unfulfilled lint expectation`
  errors in files the change never touched: `test-utils/src/lib.rs:14`
  (`#![expect(clippy::doc_markdown, ...)]`) and `memory-storage-turso/src/lib.rs:4`
  (`expect(clippy::expect_used, reason = "expect in compression code and tests")`).
  Neither is a defect. An `expect(...)` gate is fulfilled only if the lint
  actually fires in that crate, and whether it fires depends on the feature set:
  CI runs `cargo clippy --workspace --tests`, where workspace feature unification
  turns on `compression`, while `-p <crate>` resolves only that package's own
  default features.
- Impact: two red errors pointing at unrelated crates, sitting next to the two
  findings that *were* real (`clippy::unnecessary_to_owned`,
  `clippy::field_reassign_with_default`). Reading the artifacts as defects would
  have meant "fixing" `#![expect]` gates in crates this PR never touched — the
  same class as LESSON-055, where the nightly's test configuration looked like a
  storage bug.
- Fix/prevention: before judging a lint result, run the gate CI runs
  (`./scripts/code-quality.sh clippy --workspace`). In a scoped run, treat
  `unfulfilled lint expectation` as an artifact of the narrower feature set and
  re-confirm it under the workspace run; note that `-A clippy::expect_used` on
  the command line cannot satisfy a crate-level `expect`, because the attribute
  overrides the command-line level. Prefer the real findings by filtering on the
  changed file paths, not by the presence of `error:` lines.
- References: `scripts/code-quality.sh:144,158`, `.github/workflows/ci.yml:94`,
  `memory-storage-turso/src/lib.rs:4`, `test-utils/src/lib.rs:14`, LESSON-055,
  `.agents/skills/code-quality/SKILL.md`.

## LESSON-061: the CI coverage artifact is the patch-coverage ground truth, not a local run (2026-10-16)

- Issue: classifying PR #1180's "57 lines missing" from a local
  `cargo llvm-cov` run of `do-memory-storage-turso` disagreed with CI on which
  lines were missing. Locally `trait_impls/mod.rs` 330/346 looked uncovered (CI
  covers them) and `storage/monitoring.rs` reported 0 hits across 171
  instrumented lines, while CI's per-file counts were 25/17/9/3/3. Reason:
  `.github/workflows/coverage.yml:98` instruments a **whole-workspace** test run,
  so other crates' tests contribute hits into this crate's lines; a `-p` run
  cannot see them.
- Impact: the classification would have waived lines that are plainly coverable
  and demanded tests for lines CI already covers. It also first mislabeled
  `storage/mod.rs`'s missing block as `turso_multi_dimension`-gated (therefore
  uncoverable). The CI lcov shows `DA:177..201,0` inside the
  `#[cfg(not(feature = "turso_multi_dimension"))]` arm — default-build code, so
  tests were owed, not a waiver.
- Fix/prevention: take the report Codecov actually parsed, then intersect it with
  the patch:
  `gh run list --workflow coverage.yml --branch <pr-branch> --json
  databaseId,headSha` → `gh run download <id> --name coverage-report` (the
  artifact *is* `lcov.info`) → compare `DA:<line>,0` against the hunk ranges from
  `git diff -U0 <merge-base> <head> -- <file>`. That reproduced the comment's
  totals exactly (25+17+9+3+3 = 57) and supplied the line numbers the comment
  omits. Use local `cargo llvm-cov` only to prove that a *new* test executes a
  line, never to decide which lines were missing.
- References: PR #1180, `.github/workflows/coverage.yml:98,151-154`,
  `.agents/skills/coverage-waivers/SKILL.md`, LESSON-052 (measuring the wrong
  surface class).

## LESSON-062: "unreachable defensive arm" was a missing-setup problem, not an unreachable one (2026-10-16)

- Issue: most of PR #1180's missing patch lines were `map_err(...)` arms in
  `capacity_intents.rs` and `embeddings_internal.rs`. Reaching "the statement
  prepared, then the query failed" looks like it needs a prepared-cache race, and
  the pooled path that owns the cache is `#[ignore]`d (ADR-027). Worse, direct
  connection mode cannot race at all: `get_connection_with_id()`
  (`lib_impls/helpers.rs:87-93`) calls `get_connection_id()`, which is
  `NEXT_ID.fetch_add(1, Ordering::Relaxed)` (`prepared/cache.rs:102-108`), so every
  checkout gets a fresh id, the cache never hits, and the arm stays cold.
- Resolution: invalidate the **schema** instead of racing the cache. Write
  successfully, then `DROP TABLE` the table the *next* statement needs (a
  `drop_table()` helper using `storage.get_connection()` +
  `conn.execute("DROP TABLE IF EXISTS …")`). Prepare/execute then fails for real and
  the `map_err` runs: 4 capacity-intent arms plus the `#[cfg(not(feature =
  "turso_multi_dimension"))]` prepare-failure arm, 8/8 new tests green in the default
  build (whole crate lib suite 339 run / 339 passed / 7 skipped).
- Impact: those lines were about to be waived as an unreachable defensive arm — a
  false waiver on 20+ patch lines, and the kind of waiver that hides real untested
  error handling.
- Fix/prevention: before waiving an error arm, try destructive setup on the local
  `libsql::Builder::new_local` database. A waiver must name the *specific* mechanism
  that makes the line unreachable (e.g. "per-checkout cache id, so the cache cannot
  hit outside the pooled path"), never a general "needs a race" or "needs a pool".
- References: PR #1180, `memory-storage-turso/src/lib_impls/helpers.rs:87-93`,
  `memory-storage-turso/src/prepared/cache.rs:102-108`,
  `.agents/skills/coverage-waivers/SKILL.md` (destructive setup), LESSON-061.

## LESSON-063: two release affordances in our own guides do not exist (2026-10-17)

- Issue: while fixing the preflight warning for v0.1.46, four copies of
  `cargo install parse-changelog-cli` were spread across
  `scripts/verify-release-state.sh`, `agent_docs/LESSONS.md`,
  `release-guard` and `github-release-best-practices`. That crate is **404 on
  crates.io**; the real one is `parse-changelog` (binary `parse-changelog`,
  0.6.17 installed here). An agent following the hint gets a failed install and
  concludes the tool is unavailable, so the warning persists and release notes
  silently fall back to a download table.
- Second affordance: `verify-release-state.sh --help` advertised
  `--fix  Auto-fix version references in .md files`. `FIX_MODE` was assigned at
  the flag and read nowhere — `--fix` exited 0 having changed nothing, while the
  drift errors it claimed to repair stayed up.
- Detection: `curl -sS -A "<ua>" https://crates.io/api/v1/crates/<name>` (the
  `-A` is mandatory — LESSON-053) and, for flags,
  `grep -c 'FIX_MODE\|--fix' scripts/<x>.sh` — an option whose variable appears
  only in the assignment is a no-op.
- Fix/prevention: a guide may only name an affordance that is verified to exist.
  Corrected the crate name everywhere, pinned it with the
  `names-the-real-parse-changelog-crate` /
  `no-phantom-crate-in-install-hint` evals, and deleted the dead `--fix` flag
  and its help line rather than leaving a documented lie (no caller used it;
  `check-pr-readiness.sh --fix` is a different, working flag).
- References: `scripts/verify-release-state.sh`, LESSON-053, LESSON-057,
  `.agents/skills/release-guard/SKILL.md`,
  `.agents/skills/github-release-best-practices/SKILL.md`.

## LESSON-064: a bot branch's "Merge branch 'main'" resolution rolled a dependency bump back (2026-10-17)

- Issue: PR #1176 is one line of production code (`limit.min(MAX_QUERY_LIMIT)` in
  `SpatiotemporalIndex::query_range`) plus a test. Its `--stat` against the merge
  base also carried `memory-cli/Cargo.toml`, `memory-mcp/Cargo.toml`,
  `tests/Cargo.toml` and `Cargo.lock`. Those four hunks set `sysinfo = "0.38"`
  and delete the `objc2`/`dispatch2`/`objc2-foundation` tree, while **both** the
  merge base (`2e1d7537`) and `main` (`c34c68ed`) say `0.39` — the branch had
  merged main four times (`ffe8c021`, `824f3869`, `ec2b0a9c`, `fc3f5924`) and one
  resolution kept the stale side.
- Why `MERGEABLE` hid it: a 3-way merge takes "theirs" for a line only the
  branch changed, so merging #1176 would have **downgraded sysinfo on main** and
  rewritten `Cargo.lock`. GitHub reported `MERGEABLE`/`CLEAN`-equivalent status,
  Codacy reported 0 issues and Codecov reported every coverable line covered —
  none of those signals touch dependency state.
- Detection: `for r in $MB $TIP origin/main; do git show $r:memory-cli/Cargo.toml | grep '^sysinfo'; done`
  (compare merge base, tip, main for the same manifest line), or read the branch
  diff restricted to manifests:
  `git diff $MB $TIP -- Cargo.lock '*/Cargo.toml'`. Any manifest hunk on a branch
  whose title is not a dependency change is the finding.
- Fix/prevention: repair the branch, do not merge and revert —
  `git checkout origin/main -- Cargo.lock '*/Cargo.toml'`, commit, push once
  (the branch needs a CI cycle anyway because it is `BEHIND`). AGENTS.md's
  bot/agent-branch trap and `pr-readiness` rule 7 now name this dependency
  variant explicitly, pinned by the
  `catches-dependency-rollback-on-stale-branches` eval.
- References: PR #1176, LESSON-035 (`MERGEABLE` ≠ no lost work),
  `.agents/skills/pr-readiness/SKILL.md`.

## LESSON-065: a `[skip ci]` token written as prose permanently blocks the merge (2026-10-17)

- Issue: PR #1186 (changelog-generator fix) had every visible check green
  (Codacy, CodeQL) and `mergeable=MERGEABLE`, yet `mergeStateStatus` never left
  `BLOCKED`. The cause was in the **commit message body**: the change was
  described as "drops the `[skip ci]` marker", and GitHub's skip matcher scans
  the entire last commit message for CI control tokens — backticks, quotes and
  prose context do not neutralize it. The branch therefore had
  **zero `pull_request` runs** — the decisive probe is `gh api
  "repos/{owner}/{repo}/commits/<sha>/check-runs" --jq '.total_count'`, which
  returned `0` on the original head and `8` on the corrected one, so nothing
  could ever satisfy the aggregate required context `CI / Required`.
- Why it reads as "still pending" instead of "broken": the two checks that do
  **not** honor the token — CodeQL (`Analyze (rust)`) and Codacy (a GitHub App) —
  still produce rows, so the PR page looks healthy with two greens and no
  pending item. A `BLOCKED` state with no visible `pending` row is the tell, not
  the slowest dependency (LESSON-049/050).
- Second-order hazard: GitHub's default squash-merge uses the PR title **plus the
  commit body**, so merging that PR would have put the same control token on the
  main merge commit and suppressed **push** CI on main — blinding the release
  gate for exactly the commit the ship depends on (`release-manager.sh ci-check`
  counts runs by head SHA; no runs, no green).
- Detection: `gh api "repos/{owner}/{repo}/commits/<sha>/check-runs" -q
  '[.check_runs[] | select(.name | test("Codacy|CodeQL|Analyze"))] | length'`
  vs `.total_count`; a PR whose only check rows are App-based has been
  self-suppressed. Also grep any message you are about to push:
  `git log -1 --format=%B | grep -nE '\[(skip|ci skip|no ci)\]'`.
- Fix/prevention: reword so the literal token never appears (write *skip-ci
  marker*, not the bracketed form), `git commit --amend -F <file>`, then
  `git push --force-with-lease`. Prove the rewrite was message-only with
  `git rev-parse HEAD^{tree}` identical before and after, and re-assert
  `total_count > 0` plus the expected per-workflow run list on the new SHA.
  Never take the destructive shortcut of deleting/recreating the PR.
- References: PR #1186, LESSON-049/050 (ruleset required contexts, aggregate
  `CI / Required` row), LESSON-041 (never cancel runs you need),
  `.agents/skills/pr-readiness/SKILL.md`, `.agents/skills/ci-poll/SKILL.md`.

## LESSON-066: a fixed generator can be range-correct and still corrupt the changelog (2026-10-18)

- Issue: `v0.1.46` shipped, the generator opened PR #1188, and the part #1186
  fixed really worked — it covered only commits since `v0.1.45` and left
  published history alone. But it is **not idempotent**: release-prep PR #1185
  had already written a hand-authored `## [0.1.46] - 2026-10-17`, and the
  generator inserted `## [0.1.46] - 2026-10-18` above it. Head file: line 10 the
  new heading, line 75 the existing one, line 144 `## [0.1.45]`.
- Why it matters: two identical version headings are exactly what
  `verify-release-state.sh` rejects (its duplicate check is
  `grep -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' | sort | uniq -d`), and it is the
  same condition that produced **blank** release notes for v0.1.35. Merging
  #1188 would have red-shipped the gate on main. The published `v0.1.46` notes
  were unaffected because cargo-dist read the CHANGELOG **at the tag**.
- Detection: after any generator run, diff heading sets rather than trusting the
  PR's green CI —
  `gh api repos/{o}/{r}/contents/CHANGELOG.md?ref=<head> --jq .content |
  base64 -d | grep -E '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' | uniq -c`. CI does not
  run the duplicate gate on a PR that only touches CHANGELOG.md — but in #1188
  nothing ran at all: its eight workflow runs are `action_required` (LESSON-067),
  which is why the rollup looked quietly green instead of failing.
- Fix/prevention: the generator must treat "section for this tag already exists"
  as replace-or-skip, and the generator job itself should fail on a duplicate
  heading before it opens the PR.
- Related tracker-integrity finding found while verifying: #1089 closed as
  COMPLETED at 2026-10-17 12:25:12Z (via #1171) with **two acceptance criteria
  still unchecked**; criterion 3 got its test only the next day (#1187, merged
  2026-10-18, shipped in `v0.1.46`) and criterion 4 still has no ordering
  assertion anywhere. A `stateReason: COMPLETED` is not evidence that the
  acceptance list was satisfied —
  `gh issue view <n> --json body -q .body | grep '^- \[ \]'` is. Re-verify
  criterion state before repeating an issue's completion in a roadmap or
  CHANGELOG line.
- References: PR #1188, PR #1186, issue #1178, issue #1089, LESSON-057
  (the bump is the drift exit), `.agents/skills/release-guard/SKILL.md`
  (empty-notes failure mode).

## LESSON-067: a bot-committer PR never runs CI, so its required check can never report (2026-10-18)

- Issue: PR #1188 (opened by `peter-evans/create-pull-request` from the
  `Generate Changelog` workflow) looked healthy: zero failing rows in
  `statusCheckRollup`, and the only entries present were CodeQL and Codacy, all
  `SUCCESS`. In fact none of the eight repository workflows ever executed:

  ```
  $ gh run list --branch chore/update-changelog-v0.1.46 --json name,conclusion
  CI, Release Drift Check, Coverage, Security, Supply Chain Security,
  Storage Matrix Tests, Skill Evals, File Structure Validation
      -> every one conclusion=action_required
  $ gh api repos/{owner}/{repo}/actions/runs/39122545647 \
        --jq '.conclusion, .triggering_actor.login, .head_commit.committer.name'
  action_required  github-actions[bot]  github-actions[bot]
  $ gh api repos/{owner}/{repo}/actions/runs/39122545647/jobs --jq .total_count
  0
  ```

- Why it matters: GitHub withholds workflow execution for runs whose commit and
  trigger identity it does not trust — here both are `github-actions[bot]` —
  until a maintainer approves them. So `CI / Required` is not pending and not
  failing, it is **absent**, while the services GitHub pushes itself (CodeQL,
  Codacy) report regardless. `mergeStateStatus` then sits at `BLOCKED` with
  `pending: 0`, which reads as "still waiting on CI" and actually means "CI is
  never coming". That is the second, independent reason generated changelog PRs
  cannot land: #1178's equivalent was closed rather than merged in the same
  shape, on top of its content defect (LESSON-066).
- Detection: never treat an empty failing-row list as green. Count the rows
  against the workflows a PR must trigger, then confirm the runs —
  `gh run list --branch <head-branch> --json name,status,conclusion` and
  `gh api repos/{owner}/{repo}/actions/runs/<id> --jq .conclusion`.
  `action_required` together with `jobs.total_count == 0` means unapproved, not
  passed. A rollup whose expected rows are silently missing is the same failure
  class as an empty report file: assert the artifact exists before believing it.
- Fix/prevention: prefer designs where the bot does not need a mergeable PR at
  all — #1190 makes the generator skip a version the release-preparation PR
  already wrote, so the normal case opens nothing. When a generated change
  really is required, re-author the commit under a human identity
  (`GIT_COMMITTER_NAME`/`GIT_COMMITTER_EMAIL` plus a user token) or have a
  maintainer approve the runs **after reading the diff**; never blanket-approve
  a bot's workflows just to make a rollup report.
- References: PR #1188 (CI run `39122545647`), PR #1178's closed auto-PR,
  LESSON-065 (the other way a rollup lies), LESSON-066 (the content defect in
  the same PR), `.agents/skills/pr-readiness/SKILL.md` (green rows are not a
  merge signal).
## LESSON-068: a lint that only fires in the `lib test` target makes a `--lib` clippy run a false green (2026-10-19)

- Issue: PR #1184 head `7ab5e46e` failed `Quick PR Check (Format + Clippy)` with exactly one
  error — `clippy::items_after_test_module` at `memory-core/src/sync/conflict.rs:109`, "could
  not compile `do-memory-core` (lib test)". Reproducing it the obvious way was green:

  ```
  $ cargo clippy -p do-memory-core --lib -- -D warnings     # rc 0 — lint absent
  $ cargo clippy -p do-memory-core --tests -- -D warnings    # rc 101 — reproduces
  ```

- Why it matters: the lint is evaluated per **compilation target**, and items after
  `#[cfg(test)] mod tests` only exist as a shape in the test build. A `--lib` check therefore
  certifies the file clean while CI still fails, and the natural conclusion is "CI is
  flaky / the toolchain moved". The same class of target-scoped blindness made the pre-fix
  coverage picture look fine (`[[test]] required-features` is a build gate, LESSON-055).
- Detection: reproduce a clippy failure with the **exact command CI runs**, including its
  target flags. Here that is `ci.yml:94` → `./scripts/code-quality.sh clippy --workspace` →
  `cargo clippy --workspace --tests -- -D warnings …` (`code-quality.sh:144,158`). If a
  suspected-lint reproduction comes back green, suspect the target set before the lint.
- Fix/prevention: fix the shape, not the gate — move the items above the test module
  (`03870115`, pure reorder, 274 → 274 lines, no `#[allow]`), and record that any future
  `items_after_*` diagnosis must use `--tests`.
- References: PR #1184 (head `03870115`), `cargo clippy --explain
  clippy::items_after_test_module` (clippy 0.1.99 / rustc 1.99.0 `b940084d7e`),
  `.agents/skills/ci-fix/SKILL.md`, LESSON-060 (the sibling clippy misreport).

## LESSON-069: dropping the repo's `-A` allow-list invents a second clippy failure (2026-10-19)

- Issue: while reproducing #1184's lint, running the gate "stricter" than CI — i.e.
  `cargo clippy --workspace --tests -- -D warnings` without the three `-A` flags — produced
  an additional `clippy::expect_used` error in
  `memory-core/tests/pattern_search_integration_test.rs:188` (the `.expect("store pattern")` call),
  which CI does not report.
- Why it matters: the repo's authoritative gate is not "-D warnings" alone; it is
  `-D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used`
  (`scripts/code-quality.sh:144,158`). "More strict than CI" reads as *more failures than CI*,
  and sends you fixing (or suppressing) code that is not blocking anything. LESSON-060 is the
  mirror image: a *scoped* run misreports crate-root `#![expect(...)]` gates. Both directions
  come from treating "clippy" as one command when the gate is a specific flag set.
- Detection: copy the command out of the workflow and the script that expands it; diff your
  invocation's flags against it before believing an extra error. Note also that `-A` on the
  command line cannot override a crate-level attribute, so flags and attributes are not
  interchangeable levers.
- Fix/prevention: keep one canonical definition of the gate (the script), have CI call it, and
  quote the expanded command in any report of clippy state. The fix for #1184 was the lint
  itself; the `expect_used` finding was discarded as an artifact once the flags matched.
- References: `scripts/code-quality.sh:144,158`, `.github/workflows/ci.yml:94`, PR #1184,
  LESSON-060.

## LESSON-070: prove a conflict resolution lossless with index-stage set arithmetic, not with eyeballing (2026-10-19)

- Issue: re-basing PR #1189 onto `main` produced one conflicted file (`CHANGELOG.md`) whose two
  sides added **different, both-valid** sections — ours added `### CI/CD` + `### Tests`, theirs
  (from #1190) added `### Fixed`. The LESSON-064 failure mode is exactly this shape: a
  hand-resolved merge that silently keeps "ours" and rolls the other side's work back.
- Why it matters: `git diff` after the fact does not answer "did I lose a line from either
  side?" on a text file where ordering is meaningful, and a careless Edit injected a stray
  blank line during the first attempt. A merge of a docs file is where silent loss hides,
  because nothing fails to compile.
- Detection: after resolving, intersect the resolved file against both sides as sets, using the
  conflict stages rather than memory:

  ```
  # Read the two sides BEFORE staging: `git add` collapses stages 1/2/3 into stage 0,
  # after which `git show :2:` fails with "path is in the index, but not at stage 2".
  export LC_ALL=C                                    # sort must not be locale-dependent
  git show :2:CHANGELOG.md | LC_ALL=C sort -u > /tmp/ours
  git show :3:CHANGELOG.md | LC_ALL=C sort -u > /tmp/theirs
  comm -23 /tmp/ours <(sort -u CHANGELOG.md)          # lines lost from ours  -> must be empty
  comm -23 /tmp/theirs <(sort -u CHANGELOG.md)        # lines lost from theirs -> must be empty
  comm -13 <(cat /tmp/ours /tmp/theirs | sort -u) <(sort -u CHANGELOG.md)   # invented lines
  git grep -n '^<<<<<<<\|^=======\|^>>>>>>' -- CHANGELOG.md                 # leftover markers
  ```

  Measured: 0 lost from either side, 0 invented, 0 markers, net `+42/-0`, 34 `## [` headings all
  distinct, `parse-changelog CHANGELOG.md 0.1.46` rc 0.
- Two traps inside the technique, both reproduced in a scratch repo: (1) `git add` on a conflicted
  path collapses the conflict stages — after staging, `git show :2:<file>` fails with
  "fatal: path '<file>' is in the index, but not at stage 2", so the proof must read the stages
  *before* staging, not after; (2) `git checkout --conflict=merge -- <file>` **overwrites the
  working file with conflict markers**, destroying the resolution you just made — it is only
  needed to re-label `HEAD`/`base` markers as `ours`/`theirs`, so run it on a copy, or not at
  all. And plain `sort` is locale-collating, so `comm` warned "file is not in sorted order" and
  produced a bogus missing-lines list — `LC_ALL=C` is required for the proof to mean anything.
- Fix/prevention: run the set-difference proof as part of resolving, and verify the *content*
  diff versus `origin/main` separately (`git diff origin/main HEAD --stat`) so a docs PR cannot
  smuggle a `Cargo.toml`/`Cargo.lock` revert (LESSON-064).
- References: PR #1189 (`2a859aec`), PR #1190, LESSON-064, `.agents/skills/pr-readiness/SKILL.md`.

## LESSON-071: `--check-unreleased` failing after a development bump is the correct state, and the label override can hide it (2026-10-19)

- Issue: PR #1193 bumps the workspace to `0.1.47` with **no** `## [0.1.47]` CHANGELOG section,
  so `./scripts/verify-release-state.sh --check-unreleased` reports 4 version-doc/heading
  mismatches on the branch. Separately, PR #1192's `Check for unreleased changes` went red with
  `reason=version_not_advanced` while #1184's identical-looking row reported success.
- Why it matters: the temptation is to "fix" the bump PR by writing a `## [0.1.47] - 2026-10-19`
  heading — which would publish a release note for a release that has not happened, i.e. the
  exact tracker-integrity defect this workstream exists to remove (LESSON-057, #1089's closure).
  And the drift gate has two facts that are easy to conflate:
  `scripts/check-release-drift.sh:70-74` sets critical when `workspace_version ==
  latest_version` **and** `total > 0`; `release-drift.yml:87-96` guards the failing step with
  `!contains(github.event.pull_request.labels.*.name,'release-preparation')`, so a labelled PR
  reports green **without the check ever running**. #1184 (a bot storage PR) and #1192 (a test
  PR) both carry that label via the workflow's own best-effort auto-label step — their green
  rows are overrides, not passes.
- Detection: read the two numbers before arguing about either (`workspace_version`,
  `latest_tag`, `total`, `severity`, `reason`), and check the label list before trusting a drift
  row. A development bump is compliant with `release-guard`'s own rule — "Bump workspace to next
  patch for development in a **follow-up PR** (optional)" (`SKILL.md:160`).
- Fix/prevention: land the bump first when other PRs are blocked by
  `version_not_advanced` — after `0.1.47` is on `main`, `workspace_version != latest_version`,
  the `elif` chain reaches `is_valid_next_version`, and every later PR is clean **on merit**.
  Then remove the misapplied `release-preparation` labels. ADR-079 §6 is the governing precedent
  — "Release creation remains tag-only under ADR-072. Remove the broken Release
  `workflow_dispatch` surface rather than inventing a second release path" (line 105) — applied
  to the drift gate: delete the broken override surface instead of keeping a second, undocumented
  path. (§5's "No bypass flag is used" is scoped there to ruleset migration, so cite it as the
  repo's stance on bypasses rather than as a rule about labels.) Document the expected-fail state
  in the PR body instead of silencing the gate.
- The label is not free to use, and the workflow's own auto-label step breaks its contract:
  **ADR-058 D2** permits "a trusted collaborator to label **only the release-preparation PR** to
  break a **hard-limit deadlock**". Three of the four open PRs (#1184 bot storage fix, #1189
  tag-selector + docs fix, #1192 test-only fix) carried the label, none of them is a
  release-preparation PR, it was applied by `github-actions[bot]` rather than a trusted
  collaborator, and the repository was at 4 unreleased commits — not the ≥30 commits / ≥14 days
  hard limit the deadlock-breaker exists for — while the one PR that actually is the next-version
  bump (#1193) had no label.
- The asymmetry, measured on #1192 (workflow "Release Drift Check" run `39211445689` — run
  *number* 1596; cite the id, the number is not an API id — event `pull_request`, head
  `b192da92`): run created `06:42:17Z`, job started `06:42:32Z`,
  `github-actions[bot]` applied the label at `06:42:40Z`, the `Enforce release cadence` **step**
  failed at `06:42:41Z` (`##[error]Process completed with exit code 1` at `06:42:41.575Z`), the
  **job** closed `06:42:43Z` — all inside **one** run. Name the subject of each timestamp: the
  step and the job do not fail at the same second, and quoting the job time as the step time
  makes the measurement unverifiable against the step API.
  `github.event.pull_request.labels` is the snapshot taken when the event
  fired, so a label applied by a run's own best-effort step cannot suppress that run's failing
  step; and because a token-applied label produces no follow-up `labeled` run (LESSON-049), the
  *only* #1192 head has one drift row: the failing one. The label is therefore useless-against-the-
  current-failure and harmful-against-the-future: it silences the step on every later run (that is
  why #1184, labelled on `2026-10-17T14:04:06Z`, reports a green drift row that never ran) without
  ever clearing the red row it was applied to fix.
- Reproduced on a PR opened for this very lesson, which is the cleanest form of the evidence:
  #1194 (`docs/lessons-068-075`, a **docs-only** PR) opened `08:21:00Z`; its drift run
  `39222134818` ran `Auto-label release-preparation` `08:26:33→08:26:37Z` (label event recorded
  `08:26:36Z`, actor `github-actions[bot]`), and `Enforce release cadence` started and failed at
  `08:26:37Z` — **one second after** the label, because the label is a trigger-time snapshot and
  the step's own `if:` still saw a label-less payload. Exactly one drift run exists for that head
  — the token-applied label re-fired nothing, and it left #1194 labelled, red on the current head,
  and silently exempt on every future one. So "automation labels the release PR" is not a rare
  misfire to be argued about: it fires on any PR whose head still equals the latest tag, including
  a `agent_docs/`-only diff with no `Cargo.toml` change at all.
- The two runs of #1194 together are the whole defect lifecycle on a single PR, with no
  inference needed. Run `39222134818` (head `31f6265f`, created `08:21:00Z`): `Auto-label`
  applied the label at `08:26:36Z`, `Enforce release cadence` failed one second later at
  `08:26:37Z`, run conclusion **failure**. Run `39224885643` (head `bdad4e1d`, created `08:44:10Z`
  by the next push): the label was already in that event's payload, so **both** `Auto-label` and
  `Enforce release cadence` were `skipped` and the run conclusion was **success**. Nothing about
  the repository changed in those 18 minutes — measured at `08:59:24Z` the drift state was still
  `workspace=0.1.46 latest_tag=v0.1.46 total=11 severity=critical reason=version_not_advanced`.
  So the row reports green for a repo still in critical drift, on a docs-only PR, because a bot
  applied a label a commit earlier. That is the false green, and it is why the enforce step must
  be un-guardable rather than overridable.
- Why the expected-fail state is safe to land, and where it bites: `verify-release-state.sh`
  has **no CI caller** (`grep -rn verify-release-state .github/workflows/` → empty; the only
  executable references anywhere are `release-manager.sh:231`/`:304`,
  `release-cadence-manager.sh`, and `test-release-workflow.sh:87`, which merely asserts the file
  exists while `skill-evals.yml:99` runs `--publish-fixtures` against mocks). So merging the bump
  cannot
  redden `main`. What it does guarantee is the *next* cut's gate: measured on the `0.1.47`
  tree, `--check-unreleased` fails with exactly four mismatches — ROADMAP `Released Version`,
  STATUS version references, a missing `## [0.1.47]` heading, and `parse-changelog` rejecting
  the version — which `release-manager.sh` Phase 0 will refuse until a 0.1.47
  release-preparation PR lands. Drift-clean is therefore **not** shippable: the bump buys a
  clean cadence gate, and the #1169 release-prep pattern is still required before any tag.
- ADR basis for this distinction: ADR-072 §1 makes "Released version" authoritative only as a
  "matching immutable git tag plus successful automated GitHub release for the same commit" and
  lists **"Workspace version alone" as the non-authoritative source** — which is precisely what a
  development bump changes. The same table makes "Published crate version" require crates.io
  API/index evidence rather than release-plan claims, so verifying `0.1.46` membership over HTTP
  200 (not reading the workflow's own green step) is the compliant way to state it shipped.
- References: PRs #1192/#1193, `scripts/check-release-drift.sh:70-74`,
  `.github/workflows/release-drift.yml:73-96`, ADR-058 D2, ADR-072 §4 (step 3 is the sanctioned
  version bump), ADR-079 §5–§6, LESSON-049, LESSON-057.

## LESSON-072: `gh pr checks --required --watch` can exit 0 while a third of the checks have not run (2026-10-19)

- Issue: on PR #1189, `gh pr checks 1189 --required --watch` returned exit code 0 while 12 of 21
  rows were still `pending` and `mergeStateStatus` was `BLOCKED`.
- Why it matters: the ci-poll skill documents `--watch` as the preferred waiter, so exit 0 reads
  as "CI done, merge". It is not: the required subset is evaluated against what GitHub has
  *attached* as required, and an aggregate like `CI / Required` (LESSON-050/051) can be satisfied
  by rows that are not the slow jobs. Merging on that signal would merge on absent evidence.
- Detection: treat `mergeStateStatus` plus a pending count as the terminal condition, not the
  watcher's exit code, and assert the row count against the workflows a PR must trigger
  (LESSON-067's empty-rollup class):

  ```
  gh pr view <n> --json mergeStateStatus --jq .mergeStateStatus
  gh pr checks <n> --json bucket --jq '[.[] | select(.bucket=="pending")] | length'
  ./scripts/check-pr-readiness.sh <n>
  ```

  Here that sequence said BLOCKED with 12 pending while the watcher had already reported success,
  so the merge was held until the readiness gate itself went green.
- Fix/prevention: wire the merge step behind `check-pr-readiness.sh`, and never behind the exit
  code of a `--watch` invocation. Update `.agents/skills/ci-poll/SKILL.md`'s "Exit 0 = all listed
  checks passed" reading with the pending-count assertion.
- Same-class corollary, measured the other way: a **run object can report `status=queued` while
  its jobs are already running**. `gh run list --commit 2a859aec` showed `CI … queued`, and
  `gh api repos/{owner}/{repo}/actions/runs/39211135856` returned `status=queued` — while the
  **jobs** endpoint (`…/runs/<id>/jobs`) listed `Quick PR Check`, `Semver Check`,
  `Commit Message Lint`, `WASM Compile Check` as `completed/success` with `MCP Build` and
  `CSM Feature Tests` `in_progress`. (The run object carries no `jobs` field at all — only
  `jobs_url` — so the discriminator is `jobs.total_count` on the **jobs** endpoint, not a `jobs`
  key on the run object; that endpoint reported `total_count: 0` for the `action_required` case of
  LESSON-067, where no run had been started at all.) So `queued` at the run level is not evidence
  of a stalled or unapproved queue: read the **jobs** endpoint, and use `jobs.total_count == 0` as
  the discriminator for the `action_required` case — here it was a 37-minute runner wait, then
  work, not a withheld run.
- References: PR #1189 (run `39211135856`), PR #1184, `.agents/skills/ci-poll/SKILL.md`,
  LESSON-050, LESSON-051, LESSON-067.

## LESSON-073: measure release state in the tree you are reporting about (2026-10-19)

- Issue: `./scripts/release-cadence-manager.sh detect` run from the primary checkout printed
  `severity=clean reason=within_cadence`, while CI's `Check for unreleased changes` on a branch
  containing the same `main` said `severity=critical reason=version_not_advanced`. Both outputs
  were "true" — of different trees: the checkout was still on `d85efb5c` (the `v0.1.46` tag),
  4 commits behind `origin/main`.
- Why it matters: every release-pipeline number is a function of `HEAD`, the tag set and the
  working tree. A stale local `main` produces a confident, wrong answer, and because the wrong
  answer is the *comfortable* one ("within cadence") it gets believed. The same trap explains
  `git status` "working-tree cleanliness unknown" reports and the shared-`CARGO_TARGET_DIR`
  fake-mass-failure pattern.
- Detection: before quoting a release or drift number, assert the tree identity:

  ```
  git fetch origin main
  git rev-parse HEAD origin/main            # equal, or say which one you measured
  git rev-list --count HEAD..origin/main    # 0 expected
  git merge-base --is-ancestor v0.1.46 origin/main
  ```

  Re-measured after a fast-forward (`d85efb5c → 86aa6269`, clean tree, only the primary checkout
  holds `main`): `workspace_version=0.1.46 latest_tag=v0.1.46 total=4 severity=critical
  reason=version_not_advanced` — now matching CI.
- Fix/prevention: for one-off measurements prefer a detached worktree at `origin/main`
  (`git worktree add --detach <path> origin/main`, removed after) over the long-lived checkout,
  and state the SHA in any number you report. Related completeness fact found while here:
  `fuzz/Cargo.lock` pins the local crates at `0.1.39` (stale by 7 releases, harmless because
  `Cargo.toml` `exclude = ["fuzz"]` makes it a separate workspace and no CI job builds the repo
  with `--locked`), so a version bump PR is *not* expected to touch it.
- References: PRs #1192/#1193, `scripts/release-cadence-manager.sh`,
  `scripts/check-release-drift.sh`, AGENTS.md "Shared `CARGO_TARGET_DIR`", LESSON-056.

## LESSON-074: a two-dot diff against `main` manufactures "the bot branch reverted merged work" findings (2026-10-19)

- Issue: auditing PR #1184 (a `jules-*` bot branch) before `gh pr update-branch`, `git diff
  origin/main HEAD --stat` listed `scripts/test-update-changelog.sh | 167 ------`,
  `scripts/update-changelog.sh | 154 +----`, `plans/adr/ADR-045-… | 28 ---` and `CHANGELOG.md | 15 --`
  — i.e. it looked like the branch would delete the changelog-generator fix that #1190 had just landed.
  That reading is wrong: `git diff A B` reports the *tree difference*, which conflates "B reverted A's
  change" with "A advanced past B". Only the three-dot form (`git diff <merge-base> B`, which is exactly
  what `gh pr diff --name-only` shows) answers the revert question.
- Second, sharper trap: the per-file existence test that was supposed to confirm it produced a **fake
  alarm**. `git rev-parse <ref>:<path>` on a path that does not exist exits non-zero (rc=128) but
  still **echoes the literal argument on stdout** alongside the `fatal:` line, so a script that
  captured stdout without checking the exit status compared two error strings and printed
  `!!! BRANCH MODIFIED vs merge-base` for a file absent from *both* sides.
- Detection: prove the claim with merge-base blob equality, and use `git cat-file -e` for existence:

  ```
  MB=$(git merge-base origin/main HEAD)
  git diff --stat $MB HEAD                    # what the branch introduces (must equal the PR file list)
  for f in <suspect files>; do
    git cat-file -e $MB:$f 2>/dev/null && mb=yes || mb=no
    git cat-file -e HEAD:$f 2>/dev/null && hd=yes || hd=no
    printf '%s MB=%s HEAD=%s MAIN=%s\n' "$f" "$mb" "$hd" \
      "$(git cat-file -e origin/main:$f 2>/dev/null && echo yes || echo no)"
  done
  git diff --stat origin/main HEAD            # re-run after the merge; must still equal the branch's own delta
  ```

  #1184 resolved as safe: merge base `d85efb5c`, branch delta exactly 6 files (+638/−188), no
  `Cargo.toml`/`Cargo.lock` hunk, and `scripts/test-update-changelog.sh` was *added* by #1190's
  `2a9a7b6e` (`MB=absent HEAD=absent MAIN=present`) — a "main advanced" case, not a revert. Post-update
  head `a861ea3b` kept #1190's blobs identical to `origin/main` (`2b509cbc`, `f4d4189e`, `9ab34369`).
- Fix/prevention: the AGENTS.md bot-branch rule says "diff the tip against the merge base" — this is the
  arithmetic that makes it checkable, plus the asymmetry check (branch delta before and after the merge must
  be the same size; #1184 was 6 files/+638/−188 both times). Record the blob hashes, not the file list, when
  claiming a merge preserved merged work.
- References: PR #1184 (heads `03870115` → `a861ea3b`), PR #1190 (`2a9a7b6e`), AGENTS.md
  "Bot/agent branches", LESSON-064, LESSON-070.

## LESSON-075: check a test against production, not against the issue's prose (2026-10-19)

- Issue: #1089 criterion 4 demands "a dedicated assertion over the keyset `(updated_at, episode_id)`".
  PR #1192's `memory-core/tests/sync_keyset_cursor.rs` keys on `(modified_at, episode_id)`, which reads
  like a fidelity gap in the test — the obvious move is to ask the author to "fix" the test to match the
  tracker.
- Why it matters: the tracker was wrong. Production orders sync pages by **`modified_at`**:
  `memory-storage-turso/src/storage/episodes/query.rs:197` emits
  `ORDER BY r.modified_at_ms ASC, r.episode_id ASC LIMIT ?`, and the synchronizer's defensive
  normalisation (`memory-core/src/sync/synchronizer.rs:280-286`) retains `> (cursor_at, cursor_id)` and
  `sort_by_key(|a| (a.1, a.0.episode_id))`. `updated_at` is a real field and a common one
  (73 references across `memory-core`/`memory-storage-turso`, e.g. `constants.rs:114`,
  `procedural/types.rs:37`, `patterns/heuristic.rs:25`, the Turso pattern tables) — what it is *not*
  is a field of the **sync cursor**, which is keyed by `modified_at_ms`. "Fixing" the test to
  the prose would have desynchronised it from the code it is supposed to pin.
- Detection: before accepting or rejecting a test's fidelity, read the two production sites that create the
  property — the SQL `ORDER BY` and any in-process re-sort — and compare them with the fake's sort/filter:

  ```
  grep -rn "ORDER BY .*modified_at\|ORDER BY .*updated_at" memory-storage-*/src
  grep -rn "sort_by_key\|> (cursor_at, cursor_id)" memory-core/src/sync/synchronizer.rs
  ```

  #1192's fake sorts with the same key as `synchronizer.rs:286` and filters with the same tuple comparison
  as `:281-282`, and `should_renormalize_a_page_delivered_out_of_keyset_order` targets the defensive branch
  directly — so the test is faithful and the issue text needs the correction.
- Fix/prevention: when ticking an acceptance box, cite the production line the test pins (not just the test
  name), and correct the criterion's field name in the same edit. A tick justified by "the test matches the
  prose" is weaker than one justified by "the test matches the query the backend actually runs".
- Closing note: this is the gap #1189's LESSON-066 recorded as still open ("criterion 4 still has no
  ordering assertion anywhere"), so the two entries are a matched pair — the dated lesson states the
  absence, this one states that the fix must be verified against production rather than against the
  tracker's wording. LESSON entries are timestamped snapshots, so #1189's stale-sounding sentence is left
  as written; the live correction belongs in the issue body.
- References: issue #1089 criterion 4, PR #1192 (`b192da92`), `memory-core/src/sync/synchronizer.rs:280-286`,
  `memory-storage-turso/src/storage/episodes/query.rs:197`, LESSON-060 (triage sources that lie),
  LESSON-066 (the same gap, recorded as open).

## LESSON-076: a required analyzer's PII finding on a CI run id is fixed by naming the run, not by excluding the files (2026-10-20)

- Issue: the required `Codacy Static Code Analysis` context sat at `action_required` on
  this PR with one annotation: `Potential PII: phone number pattern found` at
  `AGENTS.md:94`. The attempted fix added a `.codacy.yml` excluding `'**/*.md'`
  (commit `dfe1fe9a`) — and the context stayed red, because the file used a key
  Codacy does not support.
- Why it matters: the reflex was to switch the analyzer off for the file class it
  complained about. Three things make that the wrong reflex here:
  (1) **the file was inert** — Codacy's documented key is `exclude_paths` (Java glob
  syntax), not `exclude_patterns`, and the file must start with a `---` line
  (<https://docs.codacy.com/repositories-configure/codacy-configuration-file>); and
  "If your repository has a Codacy configuration file, the *Ignored files* settings
  defined on the Codacy UI don't apply" — so adding one silently **disables** the
  UI ignore list repo-wide.
  (2) **the first two diagnoses were both wrong, and both were testable.** The
  hypothesis "two run IDs 57 characters apart" was **tested and rejected** — after
  separating the IDs the finding persisted. The replacement hypothesis "a dense
  cluster of `HH:MM:SS` timestamps" was **also tested and rejected** — after
  thinning the timestamps from five to two the finding still persisted. Both
  hypotheses were plausible, neither was the cause.
  (3) **the real trigger is a bare 10–15 digit run in `AGENTS.md`.** The green head
  `31f6265f` has **zero** lines with a 10–15 digit run in `AGENTS.md`; both red
  heads have exactly **one** — the GitHub Actions run id on the flagged bullet.
  `agent_docs/LESSONS.md` on the *same* green head already carries 9 such ids and
  is not flagged, so the detector is not a general "11 digits" rule either: the
  finding is reported for the **changed file**, and the run id in `AGENTS.md` is
  what it reads.
- Detection: before silencing a PII finding, find the *minimal* pattern that
  distinguishes the flagged file from the last green version of **the same file** —
  not from `main`, and not from the repository at large. Codacy only reports
  against changed files, so `main` is not a baseline for it, and a "grep the whole
  repo" sweep measures nothing:

  ```
  # 10-15 digit runs in one file, green head vs current head
  for sha in <green-sha> <current-sha>; do
    echo -n "$sha: "
    git show "$sha:AGENTS.md" | grep -cE '(^|[^0-9A-Za-z_-])[0-9]{10,15}([^0-9A-Za-z_-]|$)'
  done
  ```

  The answer was "0 on the green head, 1 on the red head", and the fix is to name
  the run by its **run number** in `AGENTS.md` and keep the API id in
  `agent_docs/LESSONS.md`, where the log's existing ids live and are not flagged.
  The lesson is that each hypothesis must be **falsified by a change and a
  re-run**, not merely by a plausible regex: two of the three here survived a
  careful reading of the evidence and were still wrong.
- Fix/prevention: name a CI run by its **run number** in `AGENTS.md` and keep the
  API id in `agent_docs/LESSONS.md`; do **not** add a `.codacy.yml` to silence a
  documentation finding. If a Codacy config file is ever needed, use
  `exclude_paths` with the `---` header, exclude the narrowest path, and prove the
  context flips back to `success` before claiming it works. Note also that Codacy
  always uses the config file **in the default branch**, so a PR-only config never
  silently changes the default-branch analysis.
- References: PR #1194 (heads `31f6265f` green → `76e588b2` red), Codacy check runs
  `117960009097` (success) / `117980639748` (action_required),
  <https://docs.codacy.com/repositories-configure/codacy-configuration-file>,
  LESSON-058 (a presence-only fixture punishes the correct wording),
  LESSON-063 (documented affordances that do not exist),
  LESSON-052 (reduce the rollup to the latest run per check name).
