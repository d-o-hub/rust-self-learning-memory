---
description: list all PRs and check mergeability
subtask: true
---

list all PRs and check mergeability

## Overview

Use this workflow to review all pull requests and determine which are ready to merge.

## List All PRs

```bash
# List all open PRs
gh pr list

# List all PRs (including closed and merged)
gh pr list --state all

# List PRs with mergeability status
gh pr list --json number,title,author,state,mergeable,reviewDecision,headRefName,baseRefName,labels
```

## Check Mergeability

### Quick Mergeability Check

```bash
# List PRs that can be merged immediately
gh pr list --json number,title,author,mergeable,reviewDecision \
  --jq '.[] | select(.mergeable == "MERGEABLE" and .reviewDecision == "APPROVED")'
```

### Detailed PR Status

```bash
# Get detailed information for a specific PR
gh pr view <PR_NUMBER> --json mergeable,mergeStateStatus,reviewDecision,statusCheckRollup
```

## Mergeability Criteria

A PR is merge-ready when ALL of these are satisfied:

1. **Merge Status**: `mergeable` field is `MERGEABLE` (not `CONFLICTING` or `UNKNOWN`)
2. **No Conflicts**: `mergeStateStatus` is `CLEAN` or `HAS_HOOKS`
3. **Reviews**: `reviewDecision` is `APPROVED` (not `REVIEW_REQUIRED`, `CHANGES_REQUESTED`)
4. **CI Checks**: All required status checks in `statusCheckRollup` are `COMPLETED` with `SUCCESS`
5. **Not Draft**: `isDraft` is `false`
6. **State**: PR must be `OPEN`

## Common Scenarios

### Find Merge-Ready PRs

```bash
# PRs ready to merge (approved, passing CI, no conflicts)
gh pr list --json number,title,author,headRefName,baseRefName \
  --jq '.[] | select(.mergeable == "MERGEABLE") | select(.reviewDecision == "APPROVED")'
```

### Find PRs Blocking Merge

```bash
# PRs with conflicts
gh pr list --json number,title,author,headRefName,mergeable \
  --jq '.[] | select(.mergeable != "MERGEABLE")'

# PRs awaiting review
gh pr list --json number,title,author,reviewDecision \
  --jq '.[] | select(.reviewDecision == "REVIEW_REQUIRED")'

# PRs with failing CI
gh pr checks --json bucket | jq '.[] | select(.bucket == "fail")'
```

### Check CI Status

```bash
# View all checks for a PR
gh pr checks <PR_NUMBER>

# Watch checks until completion
gh pr checks <PR_NUMBER> --watch

# Get JSON output for programmatic checking
gh pr checks <PR_NUMBER> --json name,state,bucket
```

## Workflow

1. **List all open PRs**
   ```bash
   gh pr list --state open
   ```

2. **Identify merge-ready candidates**
   ```bash
   gh pr list --json number,title,author,mergeable,reviewDecision \
     --jq '.[] | select(.mergeable == "MERGEABLE" and .reviewDecision == "APPROVED") | "#\(.number) \(.title) - \(.author.login)"'
   ```

3. **Verify CI checks for candidates**
   ```bash
   gh pr checks <PR_NUMBER>
   ```

4. **Review detailed PR information**
   ```bash
   gh pr view <PR_NUMBER>
   ```

5. **Merge if ready**
   ```bash
   gh pr merge <PR_NUMBER> --merge
   ```

## Advanced Filtering

### Filter by Label

```bash
# Find PRs with specific labels that are merge-ready
gh pr list --label "ready-to-merge" --json number,title,mergeable,reviewDecision
```

### Filter by Author

```bash
# Find your merge-ready PRs
gh pr list --author "@me" --json number,title,mergeable,reviewDecision
```

### Filter by Branch

```bash
# Find PRs targeting a specific base branch
gh pr list --base main --json number,title,mergeable,reviewDecision
```

## Merge Strategies

When merging, choose the appropriate strategy:

```bash
# Merge commit (preserves full history)
gh pr merge <PR_NUMBER> --merge

# Squash and merge (single commit)
gh pr merge <PR_NUMBER> --squash

# Rebase and merge (linear history)
gh pr merge <PR_NUMBER> --rebase

# Enable auto-merge (merge when requirements met)
gh pr merge <PR_NUMBER> --auto

# Delete branch after merge
gh pr merge <PR_NUMBER> --delete-branch
```

## Troubleshooting

### PR Shows "UNKNOWN" Merge Status

This can happen when:
- CI checks haven't completed yet
- Branch is outdated with base
- GitHub is recalculating mergeability

**Solution**: Update the branch
```bash
gh pr update-branch <PR_NUMBER>
```

### Conflicting PRs

**Solution**: Resolve conflicts locally or in web interface
```bash
gh pr checkout <PR_NUMBER>
git merge main
# Resolve conflicts
git push
```

### Pending Reviews

**Solution**: Request review from team
```bash
gh pr edit <PR_NUMBER> --add-reviewer @user1,@user2
```

### Failing CI Checks

**Solution**: Fix the issues or re-run failed jobs
```bash
gh run list --pr <PR_NUMBER>
gh run rerun <RUN_ID>
```

## Key Principles

1. **Never merge without review**: Ensure `reviewDecision` is `APPROVED`
2. **Never merge failing CI**: All required checks must pass
3. **Check for conflicts**: `mergeable` must be `MERGEABLE`
4. **Use auto-merge for efficiency**: `--auto` flag saves time
5. **Keep history clean**: Choose appropriate merge strategy (squash for features, merge for docs)

## Best Practices

- Use `--json` and `--jq` for programmatic checks
- Always verify CI status before merging
- Use labels to organize PR workflow (e.g., `ready-to-merge`, `needs-review`)
- Enable branch protection rules to enforce checks
- Use `gh pr checks --watch` to monitor long-running CI
- Update branches before merging to reduce conflicts
- Delete feature branches after merge to keep repository clean

## Examples

### Batch Check All PRs

```bash
#!/bin/bash
# Check mergeability of all open PRs
echo "=== Merge-Ready PRs ==="
gh pr list --json number,title,author,mergeable,reviewDecision \
  --jq '.[] | select(.mergeable == "MERGEABLE" and .reviewDecision == "APPROVED") | "#\(.number) \(.title) (\(.author.login))"'

echo ""
echo "=== PRs Awaiting Review ==="
gh pr list --json number,title,author,reviewDecision \
  --jq '.[] | select(.reviewDecision == "REVIEW_REQUIRED") | "#\(.number) \(.title) (\(.author.login))"'

echo ""
echo "=== PRs with Conflicts ==="
gh pr list --json number,title,author,mergeable \
  --jq '.[] | select(.mergeable != "MERGEABLE") | "#\(.number) \(.title) (\(.author.login))"'
```

### Quick PR Status Summary

```bash
gh pr list \
  --json number,title,author,mergeable,reviewDecision,statusCheckRollup \
  --jq '.[] | {
      number, title, author: .author.login,
      mergeable: .mergeable,
      review: .reviewDecision,
      ci: [.statusCheckRollup[] | select(.conclusion != null)] | length
    }'
```
