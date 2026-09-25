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
# This script NEVER uses `gh pr merge --admin` or any bypass mechanism.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "d-o-hub/rust-self-learning-memory")"

PR_NUMBER=""
METHOD="merge"
EXECUTE=false
ACCEPT_CODECOV_WAIVER=false

usage() {
  cat <<'EOF'
Usage: ./scripts/merge-pr.sh <PR> [options]

Options:
  --method <merge|squash|rebase>   Merge method (default: merge).
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
  if [[ "$ACCEPT_CODECOV_WAIVER" == true ]] \
    && grep -q 'Codecov reports missing coverage' "$READINESS_LOG" \
    && ! grep -qE 'issue\(s\) found' <(grep -v 'Codecov' "$READINESS_LOG" | grep -E '⚠️|❌' || true); then
    echo "ℹ️  Readiness reported a Codecov coverage prompt; --accept-codecov-waiver set."
    echo "    Gates below are re-checked independently against the live head."
  else
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

BAD_CHECKS="$(jq -r '[.statusCheckRollup[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED" or .conclusion == "TIMED_OUT" or .conclusion == "ACTION_REQUIRED")] | length' <<<"$PR_JSON")"
PENDING_CHECKS="$(jq -r '[.statusCheckRollup[] | select(.status != "COMPLETED")] | length' <<<"$PR_JSON")"
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

# ── Merge (standard method only; never --admin) ──────────────────────────────
gh pr merge "$PR_NUMBER" --repo "$REPO" --"$METHOD"
gh pr view "$PR_NUMBER" --repo "$REPO" --json state,mergedAt,mergeCommit \
  --jq '"Merged: " + .state + " at " + (.mergedAt // "?") + " commit " + (.mergeCommit.oid // "?")'
