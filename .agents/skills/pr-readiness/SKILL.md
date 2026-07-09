---
name: pr-readiness
description: "Comprehensive PR health check that verifies merge state, CI status, conflicts, and cancelled checks before recommending merge. Prevents the common mistake of declaring 'CI green, ready to merge' when the PR has conflicts or is behind main."
---

# PR Readiness Check Skill

Verify that a Pull Request is truly ready to merge — not just that CI is green.

## When to Use

- Before recommending any PR for merge
- When analyzing open PRs for a health report
- When asked to "fix CI" on open PRs
- After pushing conflict resolution or branch updates

## Critical Rule

**NEVER recommend merge based solely on CI status.** You MUST also check:
1. `mergeable` field (conflicts?)
2. `mergeStateStatus` field (behind? blocked? dirty?)
3. All required checks passed (not just non-required ones)
4. No stale CANCELLED checks that should have run

## Full Procedure

### 1. Query All PR State

```bash
# Get everything in one call
gh pr list --state open --json number,title,mergeable,mergeStateStatus,statusCheckRollup,headRefName,baseRefName
```

### 2. Interpret mergeStateStatus

| State | Meaning | Mergeable? | Fix |
|-------|---------|------------|-----|
| `CLEAN` | All clear, ready to merge | ✅ YES | None needed |
| `BEHIND` | Branch is behind base, no conflicts | ⚠️ Not yet | Update branch |
| `BLOCKED` | Required checks pending/failing | ❌ NO | Wait or fix CI |
| `UNSTABLE` | Non-required checks failing | ⚠️ Maybe | Check if failures matter |
| `DIRTY` | Merge conflicts exist | ❌ NO | Resolve conflicts |
| `HAS_HOOKS` | Pre-receive hooks blocking | ❌ NO | Investigate hooks |
| `UNKNOWN` | GitHub hasn't computed yet | ⏳ Wait | Re-query in 30s |

### 3. Interpret CI Check Conclusions

| Conclusion | Meaning | Action |
|------------|---------|--------|
| `SUCCESS` | Check passed | ✅ None |
| `SKIPPED` | Expected skip (Release on PRs, Coverage badge on non-main) | ✅ None |
| `CANCELLED` | Workflow was cancelled — may be stale or dependent | ⚠️ Investigate: was it cancelled because a prerequisite failed? Or stale from a previous push? Re-run if stale. |
| `FAILURE` | Check failed | ❌ Must fix |
| `pending`/`IN_PROGRESS` | Still running | ⏳ Wait — do NOT recommend merge |
| `NEUTRAL` | Informational only | ✅ None |

### 4. Fix Procedures

#### Branch Behind Main (BEHIND)
```bash
# Via GitHub API (preferred — no local checkout needed)
gh api repos/{owner}/{repo}/pulls/{number}/update-branch -X PUT -f update_method=merge

# If API says "no new commits" but state still shows BEHIND, the state is stale.
# Push an empty commit or wait for GitHub to recompute.
```

#### Merge Conflicts (DIRTY / CONFLICTING)
```bash
# Checkout the PR branch
gh pr checkout {number}

# Merge main
git merge origin/main

# Resolve conflicts in editor
# Look for <<<<<<< ======= >>>>>>> markers
grep -rn "<<<<<<" .

# After resolving:
git add <resolved-files>
git commit --no-edit
git push
```

#### Cancelled CI Checks
```bash
# Find the run ID from statusCheckRollup detailsUrl
# Re-run the workflow
gh run rerun {run_id}

# Or push empty commit to re-trigger all CI
git commit --allow-empty -m "chore: re-trigger CI" && git push
```

#### Failed CI Checks
```bash
# Get failed job logs
gh run view {run_id} --log-failed

# Diagnose and fix the code
# Common patterns in .agents/skills/ci-fix/SKILL.md
```

### 5. Verification After Fix

After any fix (conflict resolution, branch update, CI re-trigger):

```bash
# Wait ~30s for GitHub to recompute, then re-check
gh pr view {number} --json mergeable,mergeStateStatus,statusCheckRollup
```

A PR is ready to merge ONLY when:
- `mergeable` = `MERGEABLE`
- `mergeStateStatus` = `CLEAN`
- All required `statusCheckRollup` entries show `conclusion: SUCCESS` or `SKIPPED`

## Common Mistakes

### ❌ Wrong: Check only CI status
```
"All checks pass → ready to merge"
```
This ignores merge conflicts and behind-main state.

### ✅ Correct: Check CI + merge state + conflicts
```
"All checks pass, mergeStateStatus=CLEAN, mergeable=MERGEABLE → ready to merge"
```

### ❌ Wrong: Recommend merge while checks are pending
```
"Most checks pass, a few are pending → probably fine"
```

### ✅ Correct: Wait for all checks
```
"3 checks still pending → waiting for completion before recommending merge"
```

### ❌ Wrong: Ignore CANCELLED checks
```
"CANCELLED = skipped, doesn't matter"
```

### ✅ Correct: Investigate CANCELLED
```
"Coverage check CANCELLED — was it because Quick Check failed? Or stale from old push?"
```

## Output Format

When reporting PR readiness, always include:

```
## PR #{number}: {title}
- **Merge State**: {mergeStateStatus} ({mergeable})
- **CI Status**: {pass_count} pass, {fail_count} fail, {pending_count} pending, {cancelled_count} cancelled
- **Codacy**: {status}
- **Verdict**: {READY TO MERGE | NEEDS FIX: {reason}}
- **Action**: {specific action needed, or "None — merge when ready"}
```

## Related Skills

- `.agents/skills/ci-fix/SKILL.md` — CI failure diagnosis and repair
- `.agents/skills/github-workflows/SKILL.md` — Workflow patterns
- `AGENTS.md` → "PR Health Check" section — Quick reference table
