---
name: ci-poll
description: "Poll GitHub CI status with exponential backoff until all checks complete. Use after pushing to a PR branch to monitor CI until merge-ready or failure. Covers the wait-and-check loop that avoids manual sleep cycles."
---

# CI Poll Skill

Poll GitHub Actions CI status with structured backoff until all checks resolve.

## When to Use

- After pushing code to a PR branch and waiting for CI
- After triggering a Codacy reanalysis
- When you need to know if a PR is merge-ready without manual polling
- After updating a branch to re-trigger CI

## Quick Start

```bash
# Single check (no polling)
gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
  --jq '{state: .mergeStateStatus, pending: [.statusCheckRollup[] | select(.status != "COMPLETED") | .name], failing: [.statusCheckRollup[] | select(.status == "COMPLETED" and .conclusion != "SUCCESS" and .conclusion != "SKIPPED") | {name: .name, conclusion: .conclusion}]}'
```

## Polling Procedure

### Step 1: Initial Wait (60s)

CI rarely completes in under 60s. Wait once before first check.

```bash
sleep 60 && gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
  --jq '{state: .mergeStateStatus, pending: [.statusCheckRollup[] | select(.status != "COMPLETED") | .name], failing: [.statusCheckRollup[] | select(.status == "COMPLETED" and .conclusion != "SUCCESS" and .conclusion != "SKIPPED") | {name: .name, conclusion: .conclusion}]}'
```

### Step 2: Exponential Backoff

If checks are still pending, increase wait time:

| Round | Sleep | Timeout |
|-------|-------|---------|
| 1 | 60s | 120s |
| 2 | 120s | 180s |
| 3 | 300s | 360s |
| 4+ | 600s | 720s |

```bash
# Round 2
sleep 120 && gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
  --jq '{state: .mergeStateStatus, pending: [.statusCheckRollup[] | select(.status != "COMPLETED") | .name], failing: [.statusCheckRollup[] | select(.status == "COMPLETED" and .conclusion != "SUCCESS" and .conclusion != "SKIPPED") | {name: .name, conclusion: .conclusion}]}'

# Round 3+
sleep 300 && gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
  --jq '{state: .mergeStateStatus, pending: [.statusCheckRollup[] | select(.status != "COMPLETED") | .name], failing: [.statusCheckRollup[] | select(.status == "COMPLETED" and .conclusion != "SUCCESS" and .conclusion != "SKIPPED") | {name: .name, conclusion: .conclusion}]}'
```

### Step 3: Interpret Result

| `mergeStateStatus` | Meaning | Action |
|--------------------|---------|--------|
| `CLEAN` | All checks pass, no conflicts | ✅ Ready to merge |
| `BEHIND` | Branch is behind base | Update branch: `gh api repos/{owner}/{repo}/pulls/{n}/update-branch -X PUT` |
| `BLOCKED` | Required checks still pending | Wait more or investigate |
| `UNSTABLE` | Non-required checks failing | Check if failures are pre-existing |
| `DIRTY` | Merge conflicts | Resolve conflicts |
| `BLOCKED` + all checks `SUCCESS` | Hooks or rules blocking | Investigate branch protection |

### Step 4: Terminal Conditions (stop polling)

Stop polling when ANY of:
1. `mergeStateStatus` = `CLEAN` → success
2. Any required check has `conclusion: "FAILURE"` → investigate
3. Max 5 rounds reached → report current state
4. `mergeStateStatus` = `DIRTY` → resolve conflicts

## For Run-Level Monitoring

When monitoring a specific workflow run (not full PR state):

```bash
# Check a specific run
gh run view <run_id> --json conclusion,status

# List recent runs for a branch
gh run list --workflow=ci.yml --branch <branch> --limit 3 --json status,conclusion,name
```

## Common Pitfalls

- **Don't poll faster than 60s** — GitHub Actions needs time to schedule runners
- **Don't assume SKIPPED is failure** — SKIPPED is expected (e.g., Release workflow on PRs)
- **Don't forget CANCELLED** — Re-trigger if a required check was cancelled
- **Always check `mergeStateStatus`, not just CI** — A PR can have green CI but be unmergeable due to conflicts
- **Set timeout on bash calls** — Use `timeout: 360000` (6 min) for 5-minute sleep calls

## Codacy Integration

When waiting for Codacy reanalysis after a fix:

```bash
# Trigger reanalysis
codacy pull-request gh <owner> <repo> <PR> --reanalyze-and-wait 2>&1

# Or poll Codacy status
codacy pull-request gh <owner> <repo> <PR> 2>&1 | head -25
```

Codacy typically takes 2-5 minutes. Use the same backoff pattern.
