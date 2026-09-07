
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-23 — Resource Exhaustion via Unbounded Field Projection
**Vulnerability:** Public MCP tools accepted unbounded arrays of field names for JSON projection, leading to potential CWE-770 (Resource Exhaustion).
**Learning:** Security bounds must be applied not just to scalar 'limit' parameters, but also to collection sizes (vectors/arrays) provided by users.
**Prevention:** Use `.truncate(MAX_CONSTANT)` for user-provided lists and ensure all numeric/floating-point inputs are clamped to safe ranges.

## 2026-09-07 — Jules-bundled deletes ride along with one-line security fixes (PR triage wave)
**Pattern:** Bot-generated PRs (#989, #986, #980, #978, #972) bundled full-file infra deletes (`bunnyshell.yaml` -72, `bunnyshell/builder.Dockerfile` -43, `scripts/bns-cargo.sh` -220) plus 30–40 files of one-line churn alongside a 20–45-line real fix. Signal-to-noise as bad as ~30/365 lines (#989).
**Learning:** Review the survivor set, not the diff: real hunk + regression test + LEARNINGS entry. Everything else is suspect until justified.
**Prevention:** Strip with `git checkout main -- <unrelated paths>` on the PR branch; infra removal gets its own `chore(...)` PR, never a ride-along. Post the survivor-set rule in the roast review so authors self-serve next time.

## 2026-09-07 — Duplicate bot PRs for the same bound (#972 vs #980)
**Pattern:** Two parallel bot runs produced the identical gist reranker `top_k` bound with identical bundled churn; only `Cargo.toml` (+2) and +22 vs +24 lines differed.
**Learning:** Duplicate detection = compare the core hunk (`git diff` on the claimed file), not titles/scopes (`fix(singularity)` vs `fix(framework)` disguised the dup).
**Prevention:** Close the dup fast with a pointer comment, keep the scope-correct survivor, and port any unique test from the closed PR before merging the survivor.

## 2026-09-07 — Empty and artifact PRs consume triage cycles (#987, #1000)
**Pattern:** #987 shipped a 0-file diff (no-op automation run); #1000 shipped generated `coverage/lcov.info` (~54k additions churn) as a mergeable PR.
**Learning:** Generated artifacts and no-op runs look like real PRs in list views; only the file list exposes them (`--json files` is the cheapest triage signal).
**Prevention:** Close with an explanatory comment (badge refresh belongs to CI artifacts, not checked-in lcov). Consider a CI guard rejecting `coverage/lcov.info` diffs and 0-file PRs at creation.

## 2026-09-07 — Merge-sync commits break Commit Message Lint across the wave
**Pattern:** Every BEHIND bot PR synced via `Merge branch 'main' into...` (1–2 per branch); #989/#986/#980 went red on Commit Message Lint (conventional-type + header-max-length 100 rules). `gh pr update-branch` is merge-based, so it re-infects lint-gated branches.
**Learning:** On lint-gated repos, branch-sync method is a correctness issue, not style.
**Prevention:** Rebase-only syncs (`git rebase origin/main` + `push --force-with-lease`); `@dependabot rebase` for dependabot branches. Never `gh pr update-branch` where commitlint runs.

## 2026-09-07 — Always verify force-push tips with ls-remote (caught live on #980)
**Pattern:** A push to a PR branch pointed it at an unrelated commit (local workdir fault); caught within minutes by auditing `git ls-remote` vs local HEAD, repaired with a corrective force-push before any CI consequence. Nearby pushes (#992, #999) audited clean via `gh pr view --json files`.
**Learning:** Force-push output is easy to misread; the remote tip is ground truth and cheap to check.
**Prevention:** After every force-push to a shared PR branch: `git ls-remote origin <branch>` + `gh pr view N --json files` must match intent. Post a transparency comment if a wrong tip was ever live.

## 2026-09-07 — Patch-coverage gaps hide in unexercised match arms (#992)
**Pattern:** Codecov flagged 2 missing lines in `memory-core/src/retrieval/eval/runner.rs`: the acceptance test exercised `Adaptive` + `AlwaysEmbed` but never `LocalOnly`, leaving the `FallbackPolicy::LocalOnly` arm uncovered.
**Learning:** A strategy/policy enum with N variants needs all N exercised in the acceptance test, not just the headline pair.
**Prevention:** Extend the test to assert per-variant baselines (here: LocalOnly reports 0 embedding calls/query). Coverage for new match statements = one test leg per arm.
