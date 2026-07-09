#!/usr/bin/env bash
# scripts/check-pr-readiness.sh — Check all open PRs for merge readiness
# Verifies merge state, CI status, conflicts, and cancelled checks.
#
# Usage:
#   ./scripts/check-pr-readiness.sh          # Check all open PRs
#   ./scripts/check-pr-readiness.sh 796      # Check specific PR
#   ./scripts/check-pr-readiness.sh --fix    # Check and attempt auto-fix (update behind branches)

set -euo pipefail

REPO="d-o-hub/rust-self-learning-memory"
FIX_MODE=false
PR_NUMBER=""

for arg in "$@"; do
  case "$arg" in
    --fix) FIX_MODE=true ;;
    [0-9]*) PR_NUMBER="$arg" ;;
    --help|-h)
      echo "Usage: $0 [--fix] [PR_NUMBER]"
      echo "  --fix    Auto-fix BEHIND branches (update from main)"
      echo "  PR_NUM   Check a specific PR only"
      exit 0
      ;;
  esac
done

echo "═══════════════════════════════════════════════════════"
echo "  PR Readiness Check — $(date -u '+%Y-%m-%d %H:%M UTC')"
echo "═══════════════════════════════════════════════════════"
echo ""

# Build jq query
if [ -n "$PR_NUMBER" ]; then
  PRS=$(gh pr view "$PR_NUMBER" --json number,title,mergeable,mergeStateStatus,headRefName,statusCheckRollup 2>&1)
  PRS="[$PRS]"
else
  PRS=$(gh pr list --state open --json number,title,mergeable,mergeStateStatus,headRefName,statusCheckRollup 2>&1)
fi

PR_COUNT=$(echo "$PRS" | jq 'length')

if [ "$PR_COUNT" -eq 0 ]; then
  echo "✅ No open PRs."
  exit 0
fi

ALL_READY=true
ISSUES_FOUND=0

for i in $(seq 0 $((PR_COUNT - 1))); do
  NUMBER=$(echo "$PRS" | jq -r ".[$i].number")
  TITLE=$(echo "$PRS" | jq -r ".[$i].title")
  MERGEABLE=$(echo "$PRS" | jq -r ".[$i].mergeable")
  MERGE_STATE=$(echo "$PRS" | jq -r ".[$i].mergeStateStatus")
  BRANCH=$(echo "$PRS" | jq -r ".[$i].headRefName")

  # Count check statuses
  SUCCESS=$(echo "$PRS" | jq "[.[$i].statusCheckRollup[] | select(.conclusion == \"SUCCESS\")] | length")
  FAILURE=$(echo "$PRS" | jq "[.[$i].statusCheckRollup[] | select(.conclusion == \"FAILURE\")] | length")
  CANCELLED=$(echo "$PRS" | jq "[.[$i].statusCheckRollup[] | select(.conclusion == \"CANCELLED\")] | length")
  PENDING=$(echo "$PRS" | jq "[.[$i].statusCheckRollup[] | select(.status == \"IN_PROGRESS\" or .status == \"QUEUED\" or .status == \"PENDING\")] | length")
  SKIPPED=$(echo "$PRS" | jq "[.[$i].statusCheckRollup[] | select(.conclusion == \"SKIPPED\")] | length")
  TOTAL=$(echo "$PRS" | jq ".[$i].statusCheckRollup | length")

  # Determine Codacy status
  CODACY=$(echo "$PRS" | jq -r "[.[$i].statusCheckRollup[] | select(.name == \"Codacy Static Code Analysis\")] | .[0].conclusion // \"NOT_FOUND\"")

  echo "───────────────────────────────────────────────────────"
  echo "PR #$NUMBER: $TITLE"
  echo "  Branch: $BRANCH"
  echo "  Merge State: $MERGE_STATE ($MERGEABLE)"
  echo "  CI: $SUCCESS pass, $FAILURE fail, $CANCELLED cancelled, $PENDING pending, $SKIPPED skipped (of $TOTAL)"
  echo "  Codacy: $CODACY"

  # Determine verdict
  VERDICT="✅ READY"
  ACTIONS=""

  if [ "$MERGEABLE" = "CONFLICTING" ]; then
    VERDICT="❌ MERGE CONFLICTS"
    ACTIONS="Checkout branch, merge main, resolve conflicts, push"
    ALL_READY=false
    ((ISSUES_FOUND++))
  elif [ "$MERGE_STATE" = "DIRTY" ]; then
    VERDICT="❌ DIRTY (conflicts)"
    ACTIONS="Checkout branch, merge main, resolve conflicts, push"
    ALL_READY=false
    ((ISSUES_FOUND++))
  elif [ "$MERGE_STATE" = "BEHIND" ]; then
    VERDICT="⚠️ BEHIND MAIN"
    ACTIONS="Update branch: gh api repos/$REPO/pulls/$NUMBER/update-branch -X PUT -f update_method=merge"
    ALL_READY=false
    ((ISSUES_FOUND++))
    if [ "$FIX_MODE" = true ]; then
      echo "  🔧 Auto-fixing: updating branch..."
      gh api "repos/$REPO/pulls/$NUMBER/update-branch" -X PUT -f update_method=merge 2>&1 | jq -r '.message // "Updated"'
    fi
  elif [ "$MERGE_STATE" = "BLOCKED" ]; then
    if [ "$PENDING" -gt 0 ]; then
      VERDICT="⏳ BLOCKED (CI pending)"
      ACTIONS="Wait for $PENDING pending check(s) to complete"
    elif [ "$FAILURE" -gt 0 ]; then
      VERDICT="❌ BLOCKED (CI failing)"
      ACTIONS="Fix $FAILURE failing check(s)"
      ((ISSUES_FOUND++))
    else
      VERDICT="⏳ BLOCKED (required checks)"
      ACTIONS="Wait for required checks"
    fi
    ALL_READY=false
  elif [ "$MERGE_STATE" = "UNSTABLE" ]; then
    VERDICT="⚠️ UNSTABLE (non-required checks failing)"
    ACTIONS="Check if failing checks are pre-existing or non-blocking"
    ALL_READY=false
    ((ISSUES_FOUND++))
  fi

  if [ "$FAILURE" -gt 0 ] && [ "$VERDICT" = "✅ READY" ]; then
    VERDICT="❌ CI FAILURE"
    ACTIONS="Fix $FAILURE failing check(s)"
    ALL_READY=false
    ((ISSUES_FOUND++))
  fi

  if [ "$CANCELLED" -gt 0 ]; then
    # Check if cancelled checks are just dependent on pending quick check
    if [ "$PENDING" -gt 0 ]; then
      echo "  ℹ️  $CANCELLED cancelled check(s) — likely waiting on pending prerequisite"
    else
      echo "  ⚠️  $CANCELLED cancelled check(s) — investigate: may need re-run"
      if [ "$VERDICT" = "✅ READY" ]; then
        VERDICT="⚠️ CANCELLED CHECKS"
        ACTIONS="Re-run cancelled workflows: gh run rerun {run_id}"
        ALL_READY=false
        ((ISSUES_FOUND++))
      fi
    fi
  fi

  if [ "$PENDING" -gt 0 ] && [ "$VERDICT" = "✅ READY" ]; then
    VERDICT="⏳ CI PENDING"
    ACTIONS="Wait for $PENDING check(s) to complete"
    ALL_READY=false
  fi

  echo "  Verdict: $VERDICT"
  if [ -n "$ACTIONS" ]; then
    echo "  Action: $ACTIONS"
  fi
  echo ""
done

echo "═══════════════════════════════════════════════════════"
if [ "$ALL_READY" = true ]; then
  echo "✅ All $PR_COUNT PR(s) are ready to merge."
  exit 0
else
  echo "⚠️  $ISSUES_FOUND issue(s) found across $PR_COUNT PR(s)."
  echo "   Run with --fix to auto-fix BEHIND branches."
  exit 1
fi
