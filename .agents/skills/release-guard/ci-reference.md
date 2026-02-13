# CI/CD Verification Reference

## Key Principles
- **Require ALL jobs**: No partial passes (e.g., Quick/Security only).
- **No queued**: >5min queued = investigate runners/limits.
- **Main branch only**: Post-merge verification.
- Branch protection: Settings > Branches > Require status checks.

## Essential Commands

### 1. List Latest Runs on Main
```
gh run list --branch main --limit 5 --json databaseId,status,conclusion,headBranch,createdAt,updatedAt,workflowName
```
- **status**: queued, in_progress, completed.
- **conclusion**: success, failure, cancelled, skipped, neutral, timed_out, startup_failure.
- PASS: All latest run: status=completed, conclusion=success.[web:113][web:105]

Example output (JSON):
```
[
  {
    "databaseId": 123456,
    "status": "completed",
    "conclusion": "success",
    "headBranch": {"name": "main"}
  }
]
```

### 2. Check Specific Run Jobs
```
gh run view <RUN_ID> --json jobs --jq '.jobs[] | {name, status, conclusion}'
```
- Verify EVERY job: completed + success.
- Queued stuck? Check runners: Settings > Actions > Runners.[web:97][web:99]

### 3. Filter Queued/In-Progress
```
gh run list --branch main --status queued,in_progress --limit 10
```
- Empty = good. Non-empty: BLOCK & notify.[web:101][web:109]

### 4. PR-Linked Run (Post-Merge)
```
gh run list --branch main --limit 3 --json workflowId | jq '..workflowId'
gh run list --workflow <ID> --limit 1
```
- Ties to latest PR merge.

## Common Issues & Fixes
| Issue | Symptom | Fix |
|-------|---------|-----|
| Queued Forever | status=queued >10min | Check runner capacity, concurrency limits. Cancel stale: `gh run cancel <ID>`[web:97][web:109] |
| Partial Jobs | Some skipped/neutral | Require in branch protection. Skipped=success but review.[web:100] |
| No Runs | Empty list | Recent merge? Wait 1min post-merge trigger. |

## Verification Checklist
- [ ] Latest main run: completed + success?
- [ ] All jobs in run: success?
- [ ] No queued runs?
- [ ] Matches PR merge time?

## Branch Protection Setup
Repo Settings > Branches > main rule:
- Require status checks: Select ALL workflows/jobs.
- Require review, no force-push.[web:103][web:115]

Use `gh api repos/:owner/:repo/branches/main/protection` for JSON status.
