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
6. That the **head SHA you measured is still current** (re-read `headRefOid` in the same
   breath as the verdict) and that every failing row is the **latest run for its check
   name** — both are ephemeral under `strict_required_status_checks_policy`

## Hard Blockers (NEVER BYPASS)

These rules have NO exceptions. Do not use `--admin`, `--force`, or any bypass mechanism:

1. **NEVER merge when `mergeStateStatus` is `UNSTABLE`** — Wait for all checks to complete.
2. **NEVER merge when checks are `pending`** — Wait for ALL checks to reach a terminal state.
3. **NEVER use `gh pr merge --admin` to bypass the `main` guard** — the guard is ruleset `main-protection` (branch protection 404s here); fix the code, don't bypass the gate.
4. **NEVER merge a PR you haven't personally verified** — Run the full check and confirm ALL conditions.
5. **NEVER skip loading this skill** — Load this skill before any merge recommendation.
6. **NEVER ignore PR comments** — Fetch inline reviews, review bodies, and issue
   comments; address actionable feedback before declaring ready. **Bots count as
   reviewers** (Codecov / Codacy): fix the finding, or waive it with the evidence posted
   **on the thread** — an acknowledgement is not a waiver. Nothing is skipped, waived-away,
   or bypassed to reach green.
7. **NEVER read `MERGEABLE` as "safe" on a long-lived or bot/agent branch** — GitHub
   compares the tip to the **merge base**, so `mergeable=MERGEABLE` means "no textual
   conflict", never "no lost work". Run `git diff <merge-base> <tip> --stat` and reject
   any tip that removes files it never touched (PR #1130's tip silently reverted four
   merged PRs while reporting `MERGEABLE`, LESSON-035).
   The same failure hides in **dependency manifests**: repeated
   `Merge branch 'main' into …` resolutions that keep "ours" quietly roll a
   dependency bump back. PR #1176 (a one-line `query_range` clamp) ended up with
   `sysinfo = "0.38"` in `memory-cli`/`memory-mcp`/`tests` `Cargo.toml` plus the
   `objc2` tree deleted from `Cargo.lock`, while both its merge base and `main`
   were at `0.39` — a 3-way merge takes "theirs" for a line only the branch
   changed, so merging it would have **downgraded main**. On any branch whose
   title is not a dependency change, every `Cargo.toml`/`Cargo.lock` hunk is a
   red flag; diff the tip against current main for those paths and fix the
   branch (`git checkout origin/main -- Cargo.lock '*/Cargo.toml'`) before merge.

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

**A short report is a broken report.** Run the sweep without a PR number and compare its
PR count to `gh pr list --state open | wc -l` before acting on it — the all-PRs loop
historically aborted after 2 of 9 PRs (a bare `((ISSUES_FOUND++))` under `set -e`, whose
rc=1 was indistinguishable from the script's normal "issues found" exit; LESSON-059).
Missing PRs in the output is not "nothing to report".

**Re-read the head SHA last — it is ephemeral.** Under
`strict_required_status_checks_policy`, every merge onto `main` pushes every other branch
to `BEHIND` and re-queues all of its runs, so a `headRefOid` pinned in a doc, comment, or
plan is stale after **one** merge (#1179 `3056b560`→`c36a874d`,
#1177 `479eb16d`→`826f5c7f` — both within minutes):

```bash
gh pr view {n} --json headRefOid,mergeStateStatus -q '[.headRefOid, .mergeStateStatus] | @tsv'
```

`statusCheckRollup` also keeps **superseded** runs. `check-pr-readiness.sh` reduces them
to the latest run per check name (#1177); reproduce the same reduction by hand — preferring
an unfinished run over any finished one of the same name, so a stale row never reports
running work as passed:

```bash
gh api "repos/{owner}/{repo}/commits/<headRefOid>/check-runs?per_page=100" \
  -q '[.check_runs | group_by(.name)[] | max_by(.id) | {name, status, conclusion}]'
```

Order by `.id` (monotonic per check run), **not** `.completed_at`: a queued or in-progress run
reports `completed_at = null`, which sorts first — so when one name has both an older completed
run and a re-queued pending run, `sort_by(.completed_at) | last` returns the **older** conclusion
and reports green/red for a run that has already been superseded. Measured at `dc731afb`: 22 check
names, 17 of them carrying a run with `completed_at = null`.

Only a `FAILURE` or `CANCELLED` that is still the latest run for its name is a blocker.

### 2. Interpret mergeStateStatus

| State | Meaning | Mergeable? | Fix |
|-------|---------|------------|-----|
| `CLEAN` | All clear (merge state only) | ✅ if CI + comments OK | Still verify CI + comments |
| `BEHIND` | Branch behind main, no conflicts | ⚠️ Not yet | Update branch (`gh pr update-branch {n}`) — **expected** under strict up-to-date, not a failure |
| `BLOCKED` | Required checks still pending | ❌ NO | Wait for CI |
| `UNSTABLE` | Non-required checks failing | ⚠️ Maybe | Investigate |
| `DIRTY` | Merge conflicts exist | ❌ NO | Resolve conflicts |
| `HAS_HOOKS` | Pre-receive hooks blocking | ❌ NO | Investigate hooks |
| `UNKNOWN` | GitHub hasn't computed yet | ⏳ Wait | Re-query in 30s |

`BEHIND` is the steady state here, not an alarm — but fixing it costs a full re-queue of
every run. Batch all pending doc/skill edits into **one** push; never push per file, and
never merge while another PR in the queue is still `BEHIND` (merges are serial).

`BLOCKED` while every listed row is green is normal, not suspicious: either the
`CI / Required` aggregate has not been created yet (see below), or an
unresolved review thread / extra-approval rule is holding the PR
(`required_review_thread_resolution=true`,
`require_extra_approval_for_unattributed_changes=true`).

**Which checks are required is a ruleset, not branch protection** (ruleset
`main-protection`, id 9591004; `/branches/main/protection` 404s). The endpoint
returns an **array of rule objects**, and the contexts live at
`.parameters.required_status_checks`:

```bash
gh api repos/{owner}/{repo}/rules/branches/main > /tmp/rules.json
jq '.[] | select(.type=="required_status_checks") | .parameters' /tmp/rules.json
```

Today that set is exactly `CI / Required` + `Codacy Static Code Analysis`, with
`strict_required_status_checks_policy=true` (every merge invalidates other PRs'
runs → merges are serial), `required_review_thread_resolution=true`,
`required_approving_review_count=0`, and
`dismiss_stale_reviews_on_push=true` — any push drops an existing approval, so
batch a cosmetic fix with the substantive change instead of adding it after the
review. A green row outside this list does not
unblock a merge; changing the list is a ruleset change, never a workflow edit
(LESSON-050).

**`CI / Required` will not show up in `gh pr checks` until late.** It is the
aggregate job `required` in `ci.yml` (`needs:` fast-gate, commitlint, test,
csm-tests, mcp-build, multi-platform, quality-gates; `if: always()`), which
fail-closes on *any* non-success dependency including `skipped`. So
`mergeStateStatus=BLOCKED` with 26 green per-job rows usually just means the
aggregate has not been created yet — wait for the slowest dependency (typically
`multi-platform`/macOS) instead of hunting a phantom failure. If the aggregate
ever reports while the visible rows are green, read
`scripts/ci-required-evaluate.sh` (the single result-mapping authority) before
anything else (LESSON-050, ADR-079).

### 3. Interpret CI Check Conclusions

| Conclusion | Meaning | Action |
|------------|---------|--------|
| `SUCCESS` | Check passed | ✅ None |
| `SKIPPED` | Expected skip (Release on PRs, Coverage badge on non-main). A row that skips by job guard is structural, e.g. `Maintain canonical drift issue` (`if: github.event_name != 'pull_request'`) — never a bypassed gate | ✅ None |
| `CANCELLED` | Workflow was cancelled — may be stale or dependent. `gh pr checks` prints cancelled rows as **`fail`**, so confirm with `gh api repos/{owner}/{repo}/actions/jobs/<job_id> --jq '.status, .conclusion'` | ⚠️ Investigate; re-run if stale. Never cancel a run of a PR you intend to merge |
| `FAILURE` | Check failed | ❌ Must fix — once it is the **latest run for that check name** on the current head SHA (step 1); a superseded row is not a failure |
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

- `mergeable` = `MERGEABLE` — **and**, on a long-lived or bot/agent branch,
  `git diff <merge-base> <tip> --stat` shows no removals the branch never made, and no
  `Cargo.toml`/`Cargo.lock` hunk that rolls back a bump `main` already has
- `mergeStateStatus` = `CLEAN` (`BEHIND` / `BLOCKED` are states to fix, not verdicts)
- `headRefOid` came from **this** query — never carried over from a doc, an earlier check,
  or a PR comment
- All required checks on that SHA = `SUCCESS` or `SKIPPED`, **after** reducing the runs to
  the latest per check name (raw `statusCheckRollup` keeps superseded rows)
- No stale `CANCELLED` checks that should have run (real conclusion read from the jobs API,
  not from `gh pr checks` wording)
- **All actionable comments and reviews addressed — human AND bot (Codecov / Codacy)**:
  fixed + pushed, or waived with the evidence posted **on the thread**
- No open `CHANGES_REQUESTED` review without follow-up approval
- Nothing skipped, waived-away, or bypassed; the merge itself goes through
  `./scripts/merge-pr.sh {n} --execute`

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

### ❌ Wrong: Verdict from a remembered SHA and raw rollup rows
```
"headRefOid from an earlier query + `statusCheckRollup` counted as-is → '2 fail, NOT READY'"
```

### ✅ Correct: Fresh SHA, latest run per check name
```
Re-read `headRefOid,mergeStateStatus`, reduce check-runs to the latest run per name;
a `FAILURE` superseded by a later `SUCCESS` for the same name is not a blocker
```

### ❌ Wrong: `MERGEABLE` means no work was lost
### ✅ Correct: `git diff <merge-base> <tip> --stat` on long-lived / bot branches

### ❌ Wrong: Use `gh pr merge --admin`
### ✅ Correct: Fix the underlying issue

---

## Output Format

When reporting PR readiness, always include:

```
## PR #{number}: {title}
- **URL**: {url}
- **Merge State**: {mergeStateStatus} ({mergeable})
- **Head SHA**: {headRefOid} — re-read immediately before this verdict
- **CI Status** (latest run per check name on that SHA): {pass} pass, {fail} fail, {pending} pending, {cancelled} cancelled
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

## Pipeline Traps (2026-10-11/12/15/16, LESSON-039…052)

- **Serial merges**: the strict up-to-date policy means each merge invalidates the
  other PRs' runs. Land the *structural* PR first when branches split the same
  files (`backend.rs` in #1149/#1151 vs #1152) and rebuild the rest on the
  winner's layout; never merge a stacked PR before its base.
- **Head SHAs are ephemeral**: that same policy re-queues every run on every other branch,
  so a `headRefOid` pinned in a doc, plan, or PR comment is stale **one merge** later
  (#1179 `3056b560`→`c36a874d`, #1177 `479eb16d`→`826f5c7f`). Re-read
  `gh pr view <n> --json headRefOid,mergeStateStatus` immediately before any readiness claim
  or merge; treat `BEHIND` as expected and fix it by updating the branch — which restarts the
  whole suite, so batch doc/skill edits into **one** push, never one push per file.
- **`gh pr update-branch` aborts on any conflict** — resolve locally
  (`git merge origin/main` in a worktree), re-run fmt + workspace clippy + the
  PR's own tests, then push. A rebuilt branch that skips fmt reds the fast gate,
  Storage Matrix and Coverage at once (`quality_gate_formatting`, LESSON-039).
- **Never cancel runs of a PR you intend to merge** (yours or another's): cancelled checks
  render as `fail` in `gh pr checks` and keep `mergeStateStatus` from reaching `CLEAN`
  (LESSON-041). Read the real conclusion with `gh api
  repos/{owner}/{repo}/actions/jobs/<job_id> --jq '.status, .conclusion'` before calling a
  row failed, and re-run with a fresh push instead of cancelling.
- **Drift check payload**: after the workflow auto-adds `release-preparation`, a
  re-run still fails (original payload has no label) — push a commit or
  `gh pr update-branch` so the new `synchronize` event carries it (LESSON-044).
  Prefer the cheaper human toggle when the PR is otherwise green: `gh pr edit <n>
  --remove-label release-preparation && gh pr edit <n> --add-label
  release-preparation`. The bot's own `GITHUB_TOKEN` label write cannot re-trigger
  the workflow, and `gh run rerun` only replays the stale payload (LESSON-049).
- **Advisory PRs look green locally and red in CI**: the audit gate is two tools
  (`cargo deny` + `cargo audit`) with separate DB clones, and a stale deny clone
  yields a false "advisories ok". Run both before declaring a readiness verdict on
  a dependency PR (LESSON-046/047).
- **Stale `FAILURE` rows are not current failures**: `statusCheckRollup` keeps every
  run for the head SHA, so the readiness script can report `N fail` on a PR whose
  latest run of that check passed and whose `mergeStateStatus` is `CLEAN`.
  Reduce `commits/<sha>/check-runs` to the **latest run per check name** before
  declaring a failure:

  ```bash
  gh api "repos/{owner}/{repo}/commits/<headRefOid>/check-runs?per_page=100" \
    -q '[.check_runs | group_by(.name)[] | max_by(.id) | {name, conclusion}]'
  ```

  Only the newest row counts: on #1179 two stale `Check for unreleased changes` `FAILURE` rows sat
  next to a later `SUCCESS` for the same name (run `116964525934`), so readiness reported `2 fail`
  against a gate that had already passed. Then re-trigger on a fresh SHA (`--allow-empty` commit + push).
  Do not delete runs or merge around the gate (LESSON-052); the only merge path stays
  `./scripts/merge-pr.sh <PR> --execute` — never `--admin`, never `--force`, never a bypass.
