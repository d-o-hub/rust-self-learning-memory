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

## Fallback polling (only if watch is unavailable)

CI rarely completes in under 60s. Use **bounded** sleep + re-check (max 5 rounds):

| Round | Sleep |
|-------|-------|
| 1 | 60s |
| 2 | 120s |
| 3 | 300s |
| 4+ | 600s |

```bash
sleep 60
gh pr checks <PR> --required --json name,state,bucket
```

## Interpret merge readiness

| `mergeStateStatus` | Meaning | Action |
|--------------------|---------|--------|
| `CLEAN` | Checks OK, no conflicts | Ready to merge (still address PR comments) |
| `BEHIND` | Branch behind base | `gh pr update-branch <PR>` |
| `BLOCKED` | Required checks pending | Keep watching |
| `UNSTABLE` | Non-required checks failing | Inspect whether blocking |
| `DIRTY` | Merge conflicts | Resolve conflicts |

### Terminal conditions (stop)

1. `mergeStateStatus` = `CLEAN` and required checks pass  
2. Required check `bucket=fail` → fix and re-push  
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
