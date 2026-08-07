# GOAP: PR Review & CI Fix Wave (2026-08-07)

- **Status**: ✅ Fixes landed on both open PRs; CI re-running to terminal state
- **Date**: 2026-08-07
- **Orchestration**: GOAP skill — analysis swarm → targeted fixes → validation swarm → plan refresh
- **Related**: `plans/GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md`, `plans/GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md` (ADR-081), ADR-079/080/081
- **Learnings**: `agent_docs/LESSONS.md` LESSON-024

## Open PRs at wave start

| PR | Branch | State | Failures at start |
|----|--------|-------|-------------------|
| #928 | `ci/cit-a4-a5-plan-truth-2026-08-06` | UNSTABLE | Commit Message Lint + 5 fail-closed waiters (cascade) |
| #927 | `feat/adr-080-automatic-recommendation-attribution` | UNSTABLE | Release Drift `commit_limit` (pre-existing) + `codecov/patch` 24.88% |

## Root causes and fixes

### #928 — commit messages violate commitlint

- **Cause**: 5 of 8 branch commits had body/footer lines > 100 chars
  (`ci(publish)`, `ci(fuzz)`, `docs(plans)`, `test(ci)`, and the #929 squash
  `b5e42cf6`); 2 `chore(ci): re-trigger workflow runs` no-op commits added noise.
- **Fix** (no content change, `git diff d8180a82..HEAD` empty):
  1. `GIT_SEQUENCE_EDITOR` rebase to drop the 2 re-trigger commits;
  2. `git filter-branch --msg-filter 'fold -s -w 100'` rewraps every long line;
  3. `npx commitlint --from 92db07bf --to HEAD --verbose` → 6/6 clean;
  4. `git push --force-with-lease` → CI green (Commit Message Lint + waiters pass).
- **Lesson**: the fail-closed waiters from CIT-A2 work as designed — a lint
  failure now surfaces in 5 workflows instead of being masked.

### #927 — pre-existing release drift + Codecov patch coverage

- **Drift**: `commit_limit` (workspace 0.1.38 vs tag v0.1.37, 43 unreleased
  commits; v0.1.38 was prepared in #921 but never shipped). Resolved with the
  documented deadlock breaker: `release-cadence-manager.sh resolve --pr 927`
  adds the `release-preparation` label; `Release Drift Check` then passes.
  Root-cause fix (actually shipping v0.1.38) is deferred to the release-guard
  flow — see TODO below.
- **Codecov patch 24.88% (459 lines)**: added
  - `memory-core/tests/attribution_receipt_matrix.rs` (new, 9 tests): nil
    episode rejection for both attributed entry points, session recording with
    the exact recommended IDs, the full `persist_session_checked` receipt
    matrix (MemoryOnly / Persisted / PartiallyPersisted / PersistenceFailed),
    and restart-safe feedback after durable persistence;
  - MCP `pattern_search` tests: attributed `execute_recommend` attaches the
    `AttributionEnvelope` and records a resolvable session; unattributed path
    has no envelope;
  - CLI render dedup (ADR-081 RAT-B8/G13): shared `print_pattern_results_human`
    and a new `attribution_output::print_attribution_block`, removing ~150
    duplicated uncovered lines from the diff and fixing the drift risk between
    attributed/unattributed output.
- **Review comments**: none human; Codecov + stale pr-readiness review replied
  to on the PR thread. Codacy 0 issues.

## Main-branch pre-existing CI

- Skill Evals run 31124726735 and Performance Benchmarks run 31124727114 at
  `92db07bf` were **cancelled** (no code failure); both re-run via
  `gh run rerun --failed`.

## Validation

- `cargo clippy -p do-memory-core -p do-memory-mcp -p do-memory-cli --all-targets` → 0 warnings
- `cargo fmt --all -- --check` → clean (after formatting the new files)
- `cargo test -p do-memory-core --lib` 1258 ✅ · `do-memory-mcp --lib` 262 ✅ · `do-memory-cli --lib` 218 ✅
- `memory-core --test attribution_receipt_matrix` 22 ✅ · MCP `pattern_search` 5 ✅ · snapshot_tests 37 ✅
- Memory CLI + DB (`./data/cache.redb`): 3 episodes recorded/learned this wave;
  `pattern recommend --episode-id` end-to-end validated the ADR-080 attributed
  flow (session created, receipt `Persisted`).

## TODO / decisions for maintainer

- Ship `v0.1.38` via release-guard (main green) to clear the repo-wide drift
  for all future PRs (the label only breaks the deadlock per-PR).
- Merge #928 then #927; then bump workspace to 0.1.39.
- ADR-080/081 acceptance; live-ruleset `CI / Required` migration (ADR-079).
- Neither open PR qualifies for close: both carry substantive codebase impact.
