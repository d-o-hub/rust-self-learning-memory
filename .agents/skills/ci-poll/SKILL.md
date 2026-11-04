---
name: ci-poll
description: "Poll GitHub CI status with exponential backoff until all checks complete. Use after pushing to a PR branch to monitor CI until merge-ready or failure. Prefer gh pr checks --watch and gh run watch over bare sleep loops."
---

# CI Poll Skill

Poll GitHub Actions CI status until all checks resolve. Prefer **typed `gh` waiters** from the [GitHub CLI manual](https://cli.github.com/manual/) over manual sleep loops.

## When to Use

- After pushing code to a PR branch and waiting for CI
- After triggering a Codacy reanalysis
- When you need to know if a PR is merge-ready without manual polling
- After updating a branch to re-trigger CI

## Preferred: `gh pr checks --watch` (R-H3)

```bash
# Required checks only; blocks until complete.
# Exit 0 = the *listed* rows finished — NOT proof of merge-readiness:
#   observed exit 0 with 12 of 21 rows still pending, because --required
#   prints a subset and a ruleset change shrinks it (LESSON-072).
# Exit 8 = still pending (when not using --watch)
# Non-zero (other) = failure
gh pr checks <PR> --required --watch --interval 30

# Never conclude from the exit code alone — re-read the rollup:
gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
  -q '"state=" + .mergeStateStatus + " pending=" +
      ([.statusCheckRollup[] | select(.status=="IN_PROGRESS" or .status=="QUEUED")] | length | tostring)'

# Structured one-shot status
gh pr checks <PR> --required --json name,state,bucket,workflow,link
```

| `bucket` (JSON) | Meaning |
|-----------------|---------|
| `pass` | Check succeeded |
| `fail` | Check failed |
| `pending` | Still running / queued |
| `skipping` | Skipped (often OK) |
| `cancel` | Cancelled — investigate / re-run |

**Cancelled ≠ failed.** `gh pr checks` prints `CANCELLED` rows as `fail` **with a duration**, which reads as a genuine failure (LESSON-041). Classify every suspect row through the jobs API before reporting it or starting a diagnosis:

```bash
gh api repos/{owner}/{repo}/actions/jobs/<job_id> --jq '.status, .conclusion'
# completed/cancelled => not a code failure;  completed/failure => real failure
```

Never cancel runs of a PR you intend to merge: a cancelled required check keeps `mergeStateStatus` away from `CLEAN`, so the merge gate refuses. Re-trigger with a push or `gh pr update-branch` instead.

## Preferred: `gh run watch` for a single workflow

```bash
# List runs for this branch / commit
gh run list --branch "$(git branch --show-current)" --limit 5 \
  --json databaseId,name,status,conclusion,url

# Block until one run finishes
gh run watch <run_id>

# Diagnose failures
gh run view <run_id> --log-failed
gh run rerun <run_id> --failed
```

## Quick status (no watch)

```bash
gh pr view <PR> --json mergeStateStatus,statusCheckRollup,mergeable \
  --jq '{state: .mergeStateStatus, mergeable: .mergeable, pending: [.statusCheckRollup[]? | select(.status != "COMPLETED") | .name], failing: [.statusCheckRollup[]? | select(.status == "COMPLETED" and .conclusion != "SUCCESS" and .conclusion != "SKIPPED") | {name: .name, conclusion: .conclusion}]}'
```

## Poll the current head (strict up-to-date checks)

Ruleset `main-protection` sets `strict_required_status_checks_policy=true`: **one merge onto `main` re-queues every required run on every other open branch** (LESSON-038, LESSON-050). While a queue drains the head SHA is ephemeral — a SHA captured at the start of a wait, or pinned in a doc or note, goes stale within minutes.

**Re-read the head at the START of every polling cycle:**

```bash
gh pr view <PR> --json headRefOid,mergeStateStatus \
  --jq '{sha: .headRefOid, state: .mergeStateStatus}'
```

Polling a superseded SHA creates a permanent "still red" illusion: the old run never improves and the current one is never looked at.

`statusCheckRollup` keeps **every** check run for the head SHA, so a re-run leaves a stale `FAILURE` row sitting next to a newer `SUCCESS` (LESSON-052). Reduce to **latest run per check name** on that SHA before reporting anything as failed — and make the reduction answer three questions, not one:

```bash
# set -o pipefail: without it the pipeline exits with jq's status, so a 422 from
# gh is invisible and a poll that fetched nothing still "succeeded".
set -o pipefail
# gh needs --json before -q; a bare `-q` prints an empty SHA, the URL 422s, and
# the poll reads nothing at all.
HEAD=$(gh pr view <PR> --json headRefOid -q .headRefOid)
gh api "repos/{owner}/{repo}/commits/$HEAD/check-runs?per_page=100" --paginate \
  | jq -sr '
      [.[] | (.check_runs // [])[]]                   as $rows
      | ($rows | group_by(.name) | map(max_by(.id))) as $latest
      | if ($latest | length) == 0 then
          "NO_ROWS - a poll that saw nothing is a failed poll, not a green one"
        else
          ($latest | map(select(.status == "completed"
                                and ((.conclusion | IN("success","skipped","neutral")) | not))
                               | "\(.name) => \(.conclusion)")) as $bad
        | ($latest | map(select(.status != "completed")) | length) as $pending
        | "names=\($latest | length) pending=\($pending) failed=\($bad | length)",
          ($bad[] | "  FAIL " + .)
        end'
```

Four details in that command are load-bearing, each one measured rather than assumed:

- **`--paginate` applies `--jq` to each page separately**, so `group_by(.name)` never sees the other pages and one check name is reduced twice with contradictory conclusions. Pipe the pages into `jq -s` instead.
- **`jq -s` also removes the silence.** jq runs its program once per input; with no input it prints nothing and exits 0, so "query failed" and "no failures" share one output — the vacuous-green class (LESSON-026, LESSON-059). `-s` guarantees exactly one input, so zero rows reaches the explicit `NO_ROWS` branch.
- **`failed=0` is not green without `pending=0`.** Only `pending=0 failed=0` is a verdict; anything else is still running.
- **Parenthesize the negation.** `select(.status == "completed" and (.conclusion | IN("success","skipped","neutral")) | not)` does not mean "completed, and not successful": jq binds `|` looser than `and`, so the whole conjunction is negated and every *queued* row (whose `conclusion` is `null`) matches it. Observed as 9 still-running checks printed as `FAIL` on a head that had no failures at all — the parentheses around the `IN(...) | not` in the command above are what make it read as intended.

### Status and conclusion vocabulary

From the [status-checks reference](https://docs.github.com/en/pull-requests/reference/status-checks):

| Meaning | Values |
|---------|--------|
| **Not terminal — keep waiting** | `queued`, `pending`, `in_progress`, `requested`, `waiting`, `expected` |
| **Terminal success** | `success`; `neutral` and `skipped` count as success for dependent checks |
| **Terminal non-success** | `failure`, `cancelled`, `timed_out`, `stale`, `action_required` |

`stale` means GitHub marked the run stale because it took too long — not a code failure, and not green. Re-trigger and re-poll.

**A skipped job reports its status as "Success" and does not block a required check** (official note, verbatim: *"A job that is skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if it is a required check"*). So `pending=0 failed=0` proves the checks **reported**, never that they **ran**. When the work inside a job is what you depend on, read the step:

```bash
gh run view <run_id> --json jobs \
  -q '.jobs[].steps[] | select(.name == "<the step that performs the effect>") | .conclusion'
```

`duration_ms: 0` on that step is a failed operation wearing a green job (LESSON-053).

Prefer `gh pr checks --watch` / `gh run watch`; use this scripted reduction only when their output is ambiguous.

**Batch pushes — a polling cost, not tidiness.** Every branch push restarts the entire required suite (LESSON-049), so N single-file edits cost N full CI cycles; land docs/skills edits as one push. PR body edits are free: `release-drift.yml` triggers on `opened, synchronize, reopened, labeled, unlabeled` — **not** `edited`.

## Fallback polling (only if watch is unavailable)

CI rarely completes in under 60s. Use **bounded** sleep + re-check (max 5 rounds):

| Round | Sleep |
|-------|-------|
| 1 | 60s |
| 2 | 120s |
| 3 | 300s |
| 4+ | 600s |

```bash
# 0. Every round starts here — last round's SHA may already be superseded
gh pr view <PR> --json headRefOid,mergeStateStatus --jq '.headRefOid'
sleep 60
gh pr checks <PR> --required --json name,state,bucket
```

A changed `headRefOid` (or `mergeStateStatus` flipping to `BEHIND`) means the suite re-queued under strict up-to-date checks — not that CI regressed. Re-run the reduction above on the new SHA before reporting.

## Interpret merge readiness

| `mergeStateStatus` | Meaning | Action |
|--------------------|---------|--------|
| `CLEAN` | Checks OK, no conflicts | Ready to merge (still address PR comments) |
| `BEHIND` | Branch behind base | `gh pr update-branch <PR>` |
| `BLOCKED` | Required checks pending | Keep watching |
| `UNSTABLE` | Non-required checks failing | Inspect whether blocking |
| `DIRTY` | Merge conflicts | Resolve conflicts |

### Terminal conditions (stop)

0. **Never terminal, always fix first:** an empty reduction, a 422, or a `headRefOid` that came back empty is a **broken poll**, not a green one. Repair the query before reporting any state — silence has no conclusion.  
1. `mergeStateStatus` = `CLEAN` and required checks pass on the **current** `headRefOid`, with `pending=0` in the reduction  
2. Required check `bucket=fail` → confirm it is `fail` and not `cancel`, and that the row survived latest-run-per-name reduction on the current head, then fix and re-push  
3. Max 5 rounds / policy timeout → report state  
4. `DIRTY` → stop and fix conflicts  

## Branch update (prefer typed CLI)

```bash
# Preferred (R-H4)
gh pr update-branch <PR>

# Fallback only if gh version lacks update-branch
gh api repos/{owner}/{repo}/pulls/<PR>/update-branch -X PUT -f update_method=merge
```

## Common pitfalls

- Prefer `gh pr checks --watch` over bare `sleep` loops  
- **Empty output is a broken poll, never a pass** — a reduction that returned nothing, or nothing but `failed=0`, must still show `pending=0` and a non-zero row count before it means anything  
- `--paginate` + `--jq` filters **per page**; group across pages with `jq -s`  
- Re-read `headRefOid` at the start of **every** cycle — polling a superseded SHA under strict up-to-date checks fakes a permanent red  
- `statusCheckRollup` keeps superseded runs → report only failures that survive latest-run-per-name reduction on the current head  
- Cancelled ≠ failed: confirm with the jobs API before calling a row a failure, and never cancel runs of a PR you intend to merge  
- Batch edits into one push — each push restarts the whole required suite, so one push per file is a polling-cost multiplier  
- `SKIPPED` is often expected (e.g. Release workflow on PRs) — but a skipped job **reports Success even when it is a required check**, so green never proves the step ran; read the step conclusion when the work matters  
- `stale` / `timed_out` / `action_required` are terminal **non**-successes — do not fold them into a "not failure" bucket  
- `CANCELLED` on required checks → re-run; do not merge  
- Green checks ≠ mergeable if conflicts (`DIRTY`) or comments remain  
- Never `gh pr merge --admin`  

## Codacy

```bash
codacy pull-request gh <owner> <repo> <PR> --reanalyze-and-wait 2>&1
```

## See also

- Official agent skill: `gh skill install cli/cli gh --scope user`  
- Manual: <https://cli.github.com/manual/gh_pr_checks> · <https://cli.github.com/manual/gh_run>  
- Merge readiness: `pr-readiness` skill  
