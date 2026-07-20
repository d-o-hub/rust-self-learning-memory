---
name: pr-readiness
description: "Comprehensive PR health check: merge state, CI status, conflicts, cancelled checks, AND all PR comments/reviews (human + bots). Requires addressing actionable feedback before recommending merge. Prevents 'CI green, ready to merge' when conflicts, pending comments, or Codecov/Codacy findings remain."
---

# PR Readiness Check Skill

Verify that a Pull Request is truly ready to merge — not just that CI is green.

## When to Use

- Before recommending any PR for merge
- When asked to "review PR", "check PR", "fix PR", or "babysit PR"
- When analyzing open PRs for a health report
- When asked to "fix CI" on open PRs
- After pushing conflict resolution or branch updates

## Critical Rule

**NEVER recommend merge based solely on CI status.** You MUST also check:

1. `mergeable` field (conflicts?)
2. `mergeStateStatus` field (behind? blocked? dirty?)
3. All required checks passed (not just non-required ones)
4. No stale CANCELLED checks that should have run
5. **All PR comments and reviews** (human + bots) — actionable feedback addressed or explicitly waived with reason

## Hard Blockers (NEVER BYPASS)

These rules have NO exceptions. Do not use `--admin`, `--force`, or any bypass mechanism:

1. **NEVER merge when `mergeStateStatus` is `UNSTABLE`** — Wait for all checks to complete.
2. **NEVER merge when checks are `pending`** — Wait for ALL checks to reach a terminal state.
3. **NEVER use `gh pr merge --admin` to bypass branch protection** — Fix the code, don't bypass the gate.
4. **NEVER merge a PR you haven't personally verified** — Run the full check and confirm ALL conditions.
5. **NEVER skip loading this skill** — Load this skill before any merge recommendation.
6. **NEVER ignore PR comments** — Fetch inline reviews, review bodies, and issue comments; address actionable feedback before declaring ready.

---

## Full Procedure

### 1. Query All PR State

```bash
# Single PR (preferred when number known)
gh pr view {n} --json number,title,url,mergeable,mergeStateStatus,statusCheckRollup,headRefName,baseRefName,headRefOid,reviews,comments

# All open PRs
gh pr list --state open --json number,title,mergeable,mergeStateStatus,statusCheckRollup,headRefName,baseRefName
```

Also use the helper script:

```bash
./scripts/check-pr-readiness.sh [PR_NUMBER]
./scripts/check-pr-readiness.sh --fix [PR_NUMBER]   # also update BEHIND branches
```

### 2. Interpret mergeStateStatus

| State | Meaning | Mergeable? | Fix |
|-------|---------|------------|-----|
| `CLEAN` | All clear (merge state only) | ✅ if CI + comments OK | Still verify CI + comments |
| `BEHIND` | Branch behind main, no conflicts | ⚠️ Not yet | Update branch |
| `BLOCKED` | Required checks still pending | ❌ NO | Wait for CI |
| `UNSTABLE` | Non-required checks failing | ⚠️ Maybe | Investigate |
| `DIRTY` | Merge conflicts exist | ❌ NO | Resolve conflicts |
| `HAS_HOOKS` | Pre-receive hooks blocking | ❌ NO | Investigate hooks |
| `UNKNOWN` | GitHub hasn't computed yet | ⏳ Wait | Re-query in 30s |

### 3. Interpret CI Check Conclusions

| Conclusion | Meaning | Action |
|------------|---------|--------|
| `SUCCESS` | Check passed | ✅ None |
| `SKIPPED` | Expected skip (Release on PRs, Coverage badge on non-main) | ✅ None |
| `CANCELLED` | Workflow was cancelled — may be stale or dependent | ⚠️ Investigate; re-run if stale |
| `FAILURE` | Check failed | ❌ Must fix |
| `pending` / `IN_PROGRESS` | Still running | ⏳ Wait — do NOT recommend merge |
| `NEUTRAL` | Informational only | ✅ None |

### 4. Fetch ALL PR Comments (MANDATORY)

**Do not stop at CI.** Always pull every feedback channel:

```bash
OWNER_REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
N={pr_number}

# A) Inline review comments (file/line threads)
gh api "repos/${OWNER_REPO}/pulls/${N}/comments" --paginate > /tmp/pr${N}_inline.json

# B) Submitted reviews (APPROVED / CHANGES_REQUESTED / COMMENTED + body)
gh api "repos/${OWNER_REPO}/pulls/${N}/reviews" --paginate > /tmp/pr${N}_reviews.json

# C) Issue-style conversation comments (Codecov, Codacy, humans, bots)
gh api "repos/${OWNER_REPO}/issues/${N}/comments" --paginate > /tmp/pr${N}_issue.json

# Optional: conversation via gh
gh pr view ${N} --comments
```

Parse and classify every item:

| Source | Typical author | Treat as |
|--------|----------------|----------|
| Human review (inline or body) | real users | **Must address** unless clearly outdated |
| `CHANGES_REQUESTED` review | reviewers | **Hard blocker** until fixed or re-reviewed |
| Codecov bot | `codecov[bot]` | Actionable if patch coverage low / missing lines listed |
| Codacy bot | `codacy*` | Actionable if new issues > 0 |
| CodeRabbit / Cursor / other bots | varies | Actionable if concrete code suggestions |
| GitHub Actions summary bots | `github-actions[bot]` | Usually informational (benchmarks) unless FAIL |
| Dependabot / security | bots | Actionable if security findings on the PR |

### 5. Address Feedback (not just acknowledge)

For each actionable comment:

1. **Understand** — reproduce or map to code
2. **Fix or waive** — implement the change, or document why not (with evidence)
3. **Verify** — tests / clippy / targeted nextest as appropriate
4. **Close the loop** — push commits; reply on the PR thread summarizing what was done
5. **Re-check readiness** — CI + merge state + no new unresolved threads

#### Common bot feedback patterns

| Bot / finding | Typical fix |
|---------------|-------------|
| Codecov: low patch % / N missing lines | Add unit tests for listed files; extract pure helpers for I/O-heavy paths |
| Codacy: new issues | Fix lint/complexity/duplication at cited lines |
| Reviewer: correctness bug | Fix code + regression test |
| Reviewer: docs drift | Align CHANGELOG/README with behavior |
| Stale comment on old line | Reply that it was fixed in commit SHA or is no longer applicable |

#### Waiving feedback (rare)

Only waive when:

- Comment is outdated (code already changed) — reply with commit SHA
- Comment is incorrect — reply with counter-evidence
- Purely informational bot post (e.g. benchmark dump with no regression)

**Never waive** `CHANGES_REQUESTED` without either implementing the change or getting explicit re-approval.

### 6. Fix Procedures (CI / merge state)

#### Branch Behind Main (`BEHIND`)
```bash
# Preferred: typed CLI (R-H4)
gh pr update-branch {n}
# Fallback if needed:
# gh api repos/{owner}/{repo}/pulls/{n}/update-branch -X PUT -f update_method=merge
```

#### Merge Conflicts (`DIRTY` / `CONFLICTING`)
```bash
gh pr checkout {n}
git merge origin/main
# resolve <<<<<<< markers
git add <files> && git commit --no-edit && git push
```

#### Cancelled CI
```bash
gh run rerun {run_id}
# or: git commit --allow-empty -m "chore: re-trigger CI" && git push
```

#### Failed CI
```bash
gh run view {run_id} --log-failed
# Fix via .agents/skills/ci-fix/SKILL.md
```

### 7. Verification After Fix

```bash
gh pr view {n} --json mergeable,mergeStateStatus,statusCheckRollup
# Re-fetch comments to ensure no new open threads
gh api "repos/${OWNER_REPO}/pulls/${n}/comments" --paginate
gh api "repos/${OWNER_REPO}/issues/${n}/comments" --paginate
```

A PR is ready to merge ONLY when ALL of:

- `mergeable` = `MERGEABLE`
- `mergeStateStatus` = `CLEAN`
- All required `statusCheckRollup` entries = `SUCCESS` or `SKIPPED`
- No stale `CANCELLED` checks that should have run
- **All actionable PR comments addressed** (fixed + pushed, or waived with evidence on the thread)
- No open `CHANGES_REQUESTED` review without follow-up approval

---

## Common Mistakes

### ❌ Wrong: Check only CI status
```
"All checks pass → ready to merge"
```

### ✅ Correct: CI + merge state + comments
```
"CI green, mergeStateStatus=CLEAN, all review/bot feedback addressed → ready to merge"
```

### ❌ Wrong: Ignore Codecov / Codacy conversation comments
```
"No human reviews → nothing to address"
```

### ✅ Correct: Bots count
```
"Codecov reports 54 missing lines on config_template.rs → add tests, push, reply on PR"
```

### ❌ Wrong: "Acknowledged" without code change
```
"Thanks for the review" (no fix, no test, no waiver evidence)
```

### ✅ Correct: Fix or evidence-based waiver
```
Implement + test + push; reply with what changed and commit SHA
```

### ❌ Wrong: Recommend merge while checks are pending
### ✅ Correct: Wait for ALL checks terminal

### ❌ Wrong: Use `gh pr merge --admin`
### ✅ Correct: Fix the underlying issue

---

## Output Format

When reporting PR readiness, always include:

```
## PR #{number}: {title}
- **URL**: {url}
- **Merge State**: {mergeStateStatus} ({mergeable})
- **CI Status**: {pass} pass, {fail} fail, {pending} pending, {cancelled} cancelled
- **Codacy**: {status}
- **Comments**:
  - Inline review threads: {n} ({open} open / {resolved} resolved)
  - Reviews: {APPROVED / CHANGES_REQUESTED / COMMENTED counts}
  - Issue comments: {n} (list bots + humans with actionable summary)
  - Actionable remaining: {list or "none"}
- **Feedback addressed this session**: {what was fixed / waived}
- **Verdict**: READY TO MERGE | NEEDS FIX: {reason} | WAITING ON CI
- **Action**: {specific next step, or "None — merge when ready"}
```

---

## Related

- **[Comments reference](comments.md)** — Fetch/classify/close-loop templates
- `.agents/skills/ci-fix/SKILL.md` — CI failure diagnosis and repair
- `.agents/skills/ci-poll/SKILL.md` — Wait for CI with backoff
- `.agents/skills/github-workflows/SKILL.md` — Workflow patterns
- `.agents/skills/code-quality/SKILL.md` — fmt/clippy/coverage gates
- `AGENTS.md` → "PR Health Check" section — Quick reference table
- `./scripts/check-pr-readiness.sh` — Automated merge/CI/comment gates