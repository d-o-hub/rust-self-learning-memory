# Validation Latest — 2026-10-16 (v0.1.45 shipped, closeout queue in review)

**Goal**: record the verified state of the release closeout — what shipped, what is in review, and the CI/local
evidence attached to each head SHA (PR monitoring guardrail from the `goap-agent` skill).

**Workspace**: `0.1.46` (bumped post-release in the closeout PR) · **Branch**: `main` @ `8ee711bd` · **Tag**: `v0.1.45` (GitHub Release 2026-10-16T10:30:09Z)
**Plan**: [`../GOAP_RELEASE_CLOSEOUT_2026-10-16.md`](../GOAP_RELEASE_CLOSEOUT_2026-10-16.md)
**Previous record**: [`archive/2026/VALIDATION_2026-10-09.md`](archive/2026/VALIDATION_2026-10-09.md) (audit-backlog wave A)

## Method

- Release state read from the source of truth, not from the tracker: `git tag --sort=-creatordate`,
  `gh release list`, `gh run list --workflow release.yml`, and the crates.io API per crate
  (`-A` user-agent required; an unauthenticated request returns a non-JSON body and must not be read as "absent").
- Every nightly claim reproduced **locally with the exact nightly invocation**, in both feature configurations,
  before and after the change. A job whose report file is empty is not evidence (see 2026-10-09 record).
- PR status computed as **latest run per check name** from the check-runs API, because `statusCheckRollup` keeps
  superseded runs (that is what #1177 fixes).
- Nothing in this closeout was skipped, waived, `#[ignore]`d, or bypassed. Standing instruction: *all CI must pass,
  including Codacy — never skip any*.

## Verified on `main`

| Item | Evidence |
|------|----------|
| `v0.1.45` tagged + released | tag → `8ee711bd` (#1169 merge); `gh release list` → published 2026-10-16T10:30:09Z; shipped via `./scripts/release-manager.sh ship --execute` |
| 2 high + 2 moderate advisories cleared | #1174 merged as `2d5ed2b0`; `main` CI green |
| **crates.io did NOT receive `v0.1.44`/`v0.1.45`** | newest published: `do-memory-core`/`-storage-redb`/`-storage-turso` = `0.1.34`, `do-memory-mcp` = `0.1.31`; release run `38883104363` = `success`. The publish chain is exactly those **4** crates (`publish-crates.yml`: core → redb → turso → mcp); `do-memory-cli` answers 404 because it ships as a GitHub Release asset and `do-memory-test-utils` is `publish = false` (`test-utils/Cargo.toml:6`), so neither 404 is a publish failure |
| …because the publish step was skipped while green | dispatched run `38887674688`: `Check if version exists` → `Crates.io latest: 0.1.34 / will publish`, `Authenticate … Retrieved token successfully`, `Publish to crates.io` → `conclusion=skipped;duration_ms=0` |
| Nightly red since 2026-10-13 | `gh run list --workflow nightly-tests.yml --limit 5` → 5 × `failure`; run `38883104337` (`8ee711bd`) → `106 tests run: 70 passed, 36 failed, 514 skipped` |

## In review (head SHA + evidence at 2026-10-16)

`main` advanced twice while this closeout was in review — `8ee711bd` (#1169, the `v0.1.45` release merge) then
`791c01a3` (#1173 docs sync, 13:11Z). Under `strict_required_status_checks_policy` each advance puts every branch
`BEHIND` and re-queues its required runs, so the heads below are the *post-update* SHAs and will move again until the
queue drains: re-read with `gh pr view <n> --json headRefOid,mergeStateStatus` rather than trusting this row.

| PR | Head | Latest-run-per-name status | Local evidence |
|----|------|---------------------------|----------------|
| #1179 publish guard | `c36a874d` | re-queued on the new main (18 names, 1 green so far) | `yamllint`, `shellcheck`, `bash -n`, `./scripts/validate-gate-contract.sh --ci-parity` PASS, `./scripts/test-release-workflow.sh --publish-fixtures` OK; sensor confirmed to bite `origin/main:197` |
| #1177 readiness run ordering | `826f5c7f` | re-queued on the new main (20 names) | latest-per-name reduction of the check-runs API reproduces the fix: #1179's two stale `Check for unreleased changes` **failures** are superseded by run `116964525934` = `success` |
| #1175 guardrail docs | `7d8dac23` | 27 success / 1 skipped / 1 in progress (`Quality Gates`) | owns `AGENTS.md` + `agent_docs/LESSONS.md`; no code touched |
| #1180 nightly Turso | `34774f4d` | 24 success / 5 in progress / 1 skipped / 1 neutral / **1 failure = `Check for unreleased changes`** | the only red is the drift gate, which clears when the `0.1.46` bump lands — nothing in the PR fails. `--run-ignored only` → **86 passed / 0 failed** (default schema); `--features turso_multi_dimension --test multi_dimension_routing --test phase1_validation --run-ignored all` → **20 passed / 0 skipped**; `86 + 20 = 106` = pre-fix total; `cargo clippy -p do-memory-storage-turso --lib` clean in both feature configs; `yamllint`/`actionlint -shellcheck=` rc 0 |
| plans closeout + `0.1.46` bump (this PR) | `33953794` + main `791c01a3` | queued on the merged head | `./scripts/check-release-drift.sh` → `severity=clean reason=within_cadence`; `./scripts/validate-plans.sh` green (active-set, version-state `cargo=0.1.46 tag=0.1.45`, tracker-drift mismatches=0); `./scripts/check-docs-integrity.sh` green; `cargo update --workspace --offline` for the lockfile |

## Findings (2026-10-13…16)

| Finding | Evidence / consequence |
|---------|------------------------|
| Composite-action inputs are strings, so `!inputs.dry-run` is a **truthiness bug** | `!"false"` → `false`; the live publish step was skipped on every tag since `493cfb01` while the job stayed green. Use `inputs.dry-run != 'true'` (the form the neighbouring guards already used). Two releases silently unpublished. |
| A green job is not evidence of work | the only sensor that ever noticed was the *next* crate's dependency timeout (`dependency do-memory-core@0.1.45 did not appear on crates.io within 300s`). `Verify publish landed` now polls crates.io inside the publishing job. |
| `statusCheckRollup` mixes superseded runs | the same check name appears multiple times with different conclusions; the readiness gate must reduce to the latest run per name (#1177) |
| `#[ignore]` + `--run-ignored only` cannot express "only with this feature" | nightly force-ran the multi-dimension suites against the default schema. Feature-gated suites need `[[test]] required-features`; per-test `cfg_attr(…, ignore)` is not a build gate. |
| `--run-ignored only` also **silently empties** a suite | with the feature on, those tests stop being ignored, so `only` filters all of them out: `6 passed, 14 skipped` looked fine. Use `--run-ignored all` and assert the run count. |
| Asserts that map `Result` → `bool` hide production defects | `assert!(result.is_ok(), "Should succeed")` let three real bugs (metrics columns, metrics units, pooled checkout) sit in a red suite for days with no diagnosis. Print `{result:?}`. |
| Hand-written SQL drifted from the DDL | `agent_metrics.last_execution_time` / `task_metrics.avg_completion_time` do not exist (`last_execution`, `avg_completion_time_ms`), and `avg_duration_ms` was written in **seconds** and read back `/1000`. Turso monitoring metrics could never be stored. Needs a schema-vs-SQL sensor (ACT-406). |
| `get_connection_with_id()` is direct-connection only | pooled deployments fail before SQL; scoped checkout (`with_connection`) must wrap the whole closure. |
| The workflow's own `GITHUB_TOKEN` label does not re-trigger a run | the drift check reads the trigger payload's labels, so `release-preparation` must be re-applied from outside (`gh pr edit --remove-label/--add-label`) to clear a stale `version_not_advanced`. That re-label was used on #1179 and worked, but it is the documented deadlock breaker, not the fix: **the fix is the `0.1.46` bump PR landing first**, which makes the check clean for the whole queue with no override. |
| Local floating stable breaks `--all-targets` clippy | `test-utils/src/lib.rs` unfulfilled `#![expect(...)]` reproduces on untouched `main` — environmental, pre-existing, **not** caused by #1180; recorded rather than suppressed. |

## Live tracker state

`gh pr list --state open` = 10 · `gh issue list --state open` = 5 (#1068, #1076, #1089, #1109, #1165) ·
Dependabot alerts = 12 (2 high, 2 moderate, 8 low — high/moderate fixed on `main`, the 8 low need `libsql 0.10.0`).
