#!/usr/bin/env bash
# scripts/check-pr-readiness.sh — Check all open PRs for merge readiness
# Verifies merge state, CI status, conflicts, cancelled checks, AND PR comments.
# See .agents/skills/pr-readiness/SKILL.md for the full agent procedure.
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

  # ── PR comments / reviews (human + bots) ───────────────────
  INLINE_COUNT=$(gh api "repos/$REPO/pulls/$NUMBER/comments" --paginate 2>/dev/null | jq 'length' 2>/dev/null || echo 0)
  REVIEWS_JSON=$(gh api "repos/$REPO/pulls/$NUMBER/reviews" --paginate 2>/dev/null || echo '[]')
  REVIEW_CHANGES=$(echo "$REVIEWS_JSON" | jq '[.[] | select(.state == "CHANGES_REQUESTED")] | length' 2>/dev/null || echo 0)
  REVIEW_APPROVED=$(echo "$REVIEWS_JSON" | jq '[.[] | select(.state == "APPROVED")] | length' 2>/dev/null || echo 0)
  REVIEW_COMMENTED=$(echo "$REVIEWS_JSON" | jq '[.[] | select(.state == "COMMENTED")] | length' 2>/dev/null || echo 0)
  ISSUE_JSON=$(gh api "repos/$REPO/issues/$NUMBER/comments" --paginate 2>/dev/null || echo '[]')
  ISSUE_COUNT=$(echo "$ISSUE_JSON" | jq 'length' 2>/dev/null || echo 0)
  # Flag known bot feedback that is often actionable
  HAS_CODECOV=$(echo "$ISSUE_JSON" | jq '[.[] | select(.user.login | test("codecov"; "i"))] | length' 2>/dev/null || echo 0)
  CODECOV_FAIL=$(echo "$ISSUE_JSON" | jq -r '[.[] | select(.user.login | test("codecov"; "i")) | .body] | join("\n")' 2>/dev/null | grep -cE 'Patch coverage is|:x: Patch|Missing :warning:' || true)
  CODECOV_FAIL=${CODECOV_FAIL:-0}
  INLINE_COUNT=${INLINE_COUNT:-0}
  REVIEW_CHANGES=${REVIEW_CHANGES:-0}
  REVIEW_APPROVED=${REVIEW_APPROVED:-0}
  REVIEW_COMMENTED=${REVIEW_COMMENTED:-0}
  ISSUE_COUNT=${ISSUE_COUNT:-0}
  HAS_CODECOV=${HAS_CODECOV:-0}

  echo "  Comments: inline=$INLINE_COUNT  reviews(approved=$REVIEW_APPROVED changes_requested=$REVIEW_CHANGES commented=$REVIEW_COMMENTED)  conversation=$ISSUE_COUNT"
  if [ "$HAS_CODECOV" -gt 0 ]; then
    echo "  ℹ️  Codecov conversation comment present (check for missing-lines / low patch %)"
  fi
  if [ "$CODECOV_FAIL" -gt 0 ]; then
    echo "  ⚠️  Codecov reports missing coverage / low patch % — address before merge (see skill)"
  fi

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

  # Review feedback gates (script cannot auto-fix code; flags for agents)
  if [ "${REVIEW_CHANGES:-0}" -gt 0 ]; then
    VERDICT="❌ CHANGES REQUESTED"
    ACTIONS="Address review feedback (gh api repos/$REPO/pulls/$NUMBER/comments + reviews); fix, push, request re-review"
    ALL_READY=false
    ((ISSUES_FOUND++))
  fi

  if [ "${INLINE_COUNT:-0}" -gt 0 ] && [ "$VERDICT" = "✅ READY" ]; then
    echo "  ℹ️  $INLINE_COUNT inline review comment(s) — agent must verify each is resolved"
  fi

  if [ "${CODECOV_FAIL:-0}" -gt 0 ] && [ "$VERDICT" = "✅ READY" ]; then
    VERDICT="⚠️ CODECOV FEEDBACK"
    ACTIONS="Address Codecov missing lines / patch coverage (add tests); see .agents/skills/pr-readiness/SKILL.md"
    ALL_READY=false
    ((ISSUES_FOUND++))
  fi

  echo "  Verdict: $VERDICT"
  if [ -n "$ACTIONS" ]; then
    echo "  Action: $ACTIONS"
  fi
  echo ""
done

echo "═══════════════════════════════════════════════════════"
echo "Note: Agents must still load .agents/skills/pr-readiness/SKILL.md,"
echo "      read full comment bodies, implement fixes, and reply on the PR."
if [ "$ALL_READY" = true ]; then
  echo "✅ All $PR_COUNT PR(s) pass automated readiness gates."
  exit 0
else
  echo "⚠️  $ISSUES_FOUND issue(s) found across $PR_COUNT PR(s)."
  echo "   Run with --fix to auto-fix BEHIND branches."
  exit 1
fi
