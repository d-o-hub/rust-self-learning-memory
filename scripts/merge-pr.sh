#!/usr/bin/env bash
# merge-pr.sh — the sanctioned merge path: readiness gate first, then merge.
#
# Usage:
#   ./scripts/merge-pr.sh <PR> [--method merge|squash|rebase] [--accept-codecov-waiver] [--execute]
#
# Default is a dry run: the script runs every gate, prints the verdict, and the
# exact `gh pr merge` command it would run. Pass --execute to actually merge.
#
# Gate 1 (delegated): ./scripts/check-pr-readiness.sh <PR> must exit 0. That
# script encodes merge state, cancelled/failed/pending checks, and comment
# classes. Run it with --fix to auto-repair BEHIND branches first.
#
# Gate 2 (always, independently re-checked on the live head):
#   - mergeable == MERGEABLE and mergeStateStatus == CLEAN;
#   - no FAILURE / CANCELLED / TIMED_OUT / ACTION_REQUIRED check;
#   - no pending (status != COMPLETED) check;
#   - no CHANGES_REQUESTED review.
#
# --accept-codecov-waiver relaxes only the Codecov conversation heuristic inside
# check-pr-readiness.sh, and only for the case the repository explicitly allows:
# the thread has been answered with per-file evidence. Gates 2 still apply in
# full. Use it deliberately, never as a convenience.
#
# This script NEVER uses `gh pr merge --admin` or any bypass mechanism, and never
# sends `bypass_rules=true` to the asynchronous merge API.
#
# Landing a PR here takes one of two paths, chosen by GitHub, not by the caller:
#   - a PR outside a stack merges through `gh pr merge` (GraphQL);
#   - a PR inside a stack is refused there and must use
#     `PUT /repos/{owner}/{repo}/pulls/{n}/merge-async`, which returns 202
#     "pending" *before* repository rules run — the verdict arrives on the
#     returned uuid. Both paths are gated identically (LESSON-081).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "d-o-hub/rust-self-learning-memory")"

PR_NUMBER=""
# `main` is guarded by ruleset `main-protection` with `required_linear_history`
# active, so GitHub rejects a merge commit at execution time; `squash` is the only
# method that always lands here. `rebase` is linear too, but it silently drops the
# merge commits that `check-pr-readiness.sh --fix` puts on a branch with
# `update_method=merge`, so it is safe only for branches that have none (LESSON-081).
METHOD="squash"
EXECUTE=false
ACCEPT_CODECOV_WAIVER=false

usage() {
  cat <<'EOF'
Usage: ./scripts/merge-pr.sh <PR> [options]

Options:
  --method <merge|squash|rebase>   Merge method (default: squash — `main`
                                   enforces required_linear_history, so a merge
                                   commit is rejected; rebase drops merge commits
                                   that an up-to-date branch may carry).
  --accept-codecov-waiver          Allow a Codecov conversation heuristic to be
                                   the only readiness failure (thread answered
                                   with evidence). All hard gates still apply.
  --execute                        Actually merge. Without it, dry run only.
  -h, --help                       Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    [0-9]*) PR_NUMBER="$1"; shift ;;
    --method) METHOD="${2:-}"; shift 2 ;;
    --method=*) METHOD="${1#--method=}"; shift ;;
    --accept-codecov-waiver) ACCEPT_CODECOV_WAIVER=true; shift ;;
    --execute) EXECUTE=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -n "$PR_NUMBER" ]] || { echo "ERROR: PR number is required." >&2; usage >&2; exit 2; }
case "$METHOD" in
  merge|squash|rebase) ;;
  *) echo "ERROR: --method must be merge, squash, or rebase (got: $METHOD)" >&2; exit 2 ;;
esac

fail() {
  echo "❌ merge-pr: $1" >&2
  exit 1
}

echo "═══════════════════════════════════════════════════════"
echo "  Merge gate — PR #$PR_NUMBER (method: $METHOD)"
echo "═══════════════════════════════════════════════════════"

# ── Gate 1: delegated readiness check ────────────────────────────────────────
READINESS_LOG="$(mktemp)"
trap 'rm -f "$READINESS_LOG"' EXIT

set +e
./scripts/check-pr-readiness.sh "$PR_NUMBER" >"$READINESS_LOG" 2>&1
READINESS_RC=$?
set -e
cat "$READINESS_LOG"

if [[ "$READINESS_RC" -ne 0 ]]; then
  # Gate 1 may be relaxed for exactly one failure class: the Codecov conversation
  # heuristic. Two traps decide this filter. The aggregate "N issue(s) found" line
  # is produced *by* that same Codecov finding, so reading it as an additional
  # failure makes the waiver unreachable. And the verdict is spelled uppercase
  # CODECOV while the detail line is "Codecov", so a case-sensitive filter lets the
  # verdict itself survive as a "second" failure (LESSON-082).
  OTHER_FAILURES="$(grep -E '⚠️|❌' "$READINESS_LOG" | grep -viE 'codecov' | grep -vE 'issue\(s\) found' || true)"
  if [[ "$ACCEPT_CODECOV_WAIVER" == true ]] \
    && grep -q 'Codecov reports missing coverage' "$READINESS_LOG" \
    && [[ -z "$OTHER_FAILURES" ]]; then
    echo "ℹ️  Readiness reported a Codecov coverage prompt; --accept-codecov-waiver set."
    echo "    Gates below are re-checked independently against the live head."
  else
    if [[ -n "$OTHER_FAILURES" ]]; then printf '%s\n' "$OTHER_FAILURES" >&2; fi
    fail "readiness gate failed (exit $READINESS_RC). Fix the reported items, reply on the PR threads, or pass --accept-codecov-waiver for a documented Codecov waiver."
  fi
fi

# ── Gate 2: independent live verification ───────────────────────────────────
PR_JSON="$(gh pr view "$PR_NUMBER" --repo "$REPO" --json number,title,headRefOid,mergeable,mergeStateStatus,statusCheckRollup 2>/dev/null)" \
  || fail "could not read PR #$PR_NUMBER"

HEAD_SHA="$(jq -r '.headRefOid' <<<"$PR_JSON")"
MERGEABLE="$(jq -r '.mergeable' <<<"$PR_JSON")"
MERGE_STATE="$(jq -r '.mergeStateStatus' <<<"$PR_JSON")"

[[ "$MERGEABLE" == "MERGEABLE" ]] || fail "mergeable=$MERGEABLE (conflicts?)"
[[ "$MERGE_STATE" == "CLEAN" ]] || fail "mergeStateStatus=$MERGE_STATE (need CLEAN)"

# statusCheckRollup keeps every run ever recorded for the head SHA, so a re-run
# (the human release-preparation label toggle, or a ghost commit that cleared
# `Check for unreleased changes`) leaves superseded FAILURE rows behind. GitHub
# only evaluates the newest run per name, so this gate must too — otherwise it
# refuses a CLEAN, fully green PR (LESSON-052, PR #1175). Keep it fail-closed: an
# unfinished run sorts ABOVE every finished one, so pending work can never be
# resolved by a stale row. Same reduction as check-pr-readiness.sh; if either
# changes, change both.
LATEST_RUNS="$(jq -c '
  (.statusCheckRollup // [])
  | group_by(.name // .context // "?")
  | map(sort_by(
      (if ((.status // "COMPLETED") | IN("IN_PROGRESS", "QUEUED", "PENDING")) then 1 else 0 end),
      (.completedAt // .startedAt // ""))
    | last)' <<<"$PR_JSON")"
SUPERSEDED_COUNT="$(( $(jq -r '(.statusCheckRollup // []) | length' <<<"$PR_JSON") - $(jq -r 'length' <<<"$LATEST_RUNS") ))"
if [[ "$SUPERSEDED_COUNT" -gt 0 ]]; then
  echo "  ℹ️  $SUPERSEDED_COUNT superseded check run(s) ignored — counted the latest run per check name"
fi

BAD_CHECKS="$(jq -r '[.[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED" or .conclusion == "TIMED_OUT" or .conclusion == "ACTION_REQUIRED")] | length' <<<"$LATEST_RUNS")"
PENDING_CHECKS="$(jq -r '[.[] | select(.status != "COMPLETED")] | length' <<<"$LATEST_RUNS")"
[[ "$BAD_CHECKS" == "0" ]] || fail "$BAD_CHECKS failed/cancelled check(s); re-run or fix before merging"
[[ "$PENDING_CHECKS" == "0" ]] || fail "$PENDING_CHECKS check(s) still pending; wait for a terminal state"

REVIEW_JSON="$(gh api "repos/$REPO/pulls/$PR_NUMBER/reviews" --paginate 2>/dev/null || echo '[]')"
CHANGES_REQUESTED="$(jq '[.[] | select(.state == "CHANGES_REQUESTED")] | length' <<<"$REVIEW_JSON")"
[[ "$CHANGES_REQUESTED" == "0" ]] || fail "$CHANGES_REQUESTED review(s) requesting changes; address and get re-approval"

echo ""
echo "✅ Gates passed — PR #$PR_NUMBER @ ${HEAD_SHA:0:12} (MERGEABLE, CLEAN, no failed/cancelled/pending checks)"

if [[ "$EXECUTE" != true ]]; then
  echo "   Dry run. To merge:"
  echo "     gh pr merge $PR_NUMBER --$METHOD"
  exit 0
fi

# ── Merge (standard method only; never --admin, never bypass_rules) ──────────
set +e
MERGE_OUT="$(gh pr merge "$PR_NUMBER" --repo "$REPO" --"$METHOD" 2>&1)"
MERGE_RC=$?
set -e

if [[ "$MERGE_RC" != 0 ]]; then
  # GitHub refuses the synchronous merge for every PR that belongs to a stack and
  # points at the asynchronous merge API. Same verified method, same head SHA the
  # gates checked, `bypass_rules` left at its default: only the endpoint differs.
  if grep -q "part of a stack" <<<"$MERGE_OUT"; then
    echo "  ↪ part of a stack — using the asynchronous merge API"
    REQ="$(gh api -X PUT "repos/$REPO/pulls/$PR_NUMBER/merge-async" \
      -f sha="$HEAD_SHA" -f merge_method="$METHOD" -f merge_action=direct_merge 2>&1)" \
      || { echo "$REQ" >&2; fail "asynchronous merge request rejected"; }
    UUID="$(jq -r '.details.uuid // empty' <<<"$REQ")"
    [[ -n "$UUID" ]] || { echo "$REQ" >&2; fail "asynchronous merge returned no uuid"; }
    # The request is accepted before rules are evaluated: the response only means
    # "enqueued", so the truth is on the uuid, and it is the uuid that reports a
    # ruleset rejection (LESSON-081).
    STATUS="pending"
    RESULT=""
    for _ in $(seq 1 40); do
      RESULT="$(gh api "repos/$REPO/pulls/$PR_NUMBER/merge-async/$UUID" 2>/dev/null || true)"
      [[ -n "$RESULT" ]] || { sleep 3; continue; }
      STATUS="$(jq -r '.status // "pending"' <<<"$RESULT")"
      [[ "$STATUS" == "pending" ]] || break
      sleep 3
    done
    case "$STATUS" in
      merged)   echo "✅ Asynchronous merge completed for PR #$PR_NUMBER @ ${HEAD_SHA:0:12}" ;;
      enqueued) echo "✅ PR #$PR_NUMBER added to the merge queue (not yet merged): $(jq -r '.details.message // "?"' <<<"$RESULT")" ;;
      *) echo "$RESULT" >&2; fail "asynchronous merge $STATUS: $(jq -r '.details.message // "?"' <<<"$RESULT")" ;;
    esac
  else
    echo "$MERGE_OUT" >&2
    fail "gh pr merge failed (exit $MERGE_RC)"
  fi
fi

gh pr view "$PR_NUMBER" --repo "$REPO" --json state,mergedAt,mergeCommit \
  --jq '"Merged: " + .state + " at " + (.mergedAt // "?") + " commit " + (.mergeCommit.oid // "?")'
