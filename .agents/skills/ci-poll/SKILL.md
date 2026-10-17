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
# Exit 0 = all listed checks passed
# Exit 8 = still pending (when not using --watch)
# Non-zero (other) = failure
gh pr checks <PR> --required --watch --interval 30

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

`statusCheckRollup` keeps **every** check run for the head SHA, so a re-run leaves a stale `FAILURE` row sitting next to a newer `SUCCESS` (LESSON-052). Reduce to **latest run per check name** on that SHA before reporting anything as failed:

```bash
# Latest run per check name on the current head — real non-successes only
gh api "repos/{owner}/{repo}/commits/$(gh pr view <PR> -q .headRefOid)/check-runs" --paginate \
  --jq '(.check_runs | group_by(.name) | map(max_by(.id))
         | map(select(.status == "completed" and .conclusion != "success" and .conclusion != "skipped")) | .[])'
```

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

1. `mergeStateStatus` = `CLEAN` and required checks pass on the **current** `headRefOid`  
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
- Re-read `headRefOid` at the start of **every** cycle — polling a superseded SHA under strict up-to-date checks fakes a permanent red  
- `statusCheckRollup` keeps superseded runs → report only failures that survive latest-run-per-name reduction on the current head  
- Cancelled ≠ failed: confirm with the jobs API before calling a row a failure, and never cancel runs of a PR you intend to merge  
- Batch edits into one push — each push restarts the whole required suite, so one push per file is a polling-cost multiplier  
- `SKIPPED` is often expected (e.g. Release workflow on PRs)  
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
