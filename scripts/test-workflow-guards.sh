#!/usr/bin/env bash
# test-workflow-guards.sh — W2.2b: cancelled required checks must not pass as success
#
# Usage:
#   ./scripts/test-workflow-guards.sh --cancelled-required
#   ./scripts/test-workflow-guards.sh --fixtures
#
# Validates that pr-readiness skill and check-pr-readiness.sh treat CANCELLED
# required checks as blockers (not equivalent to SKIPPED/SUCCESS).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: workflow-guards — $1" >&2
  exit 1
}

check_cancelled_required() {
  local skill=".agents/skills/pr-readiness/SKILL.md"
  local script="./scripts/check-pr-readiness.sh"

  [[ -f "$skill" ]] || fail "missing $skill"
  [[ -f "$script" ]] || fail "missing $script"

  # Skill must document CANCELLED as non-success
  rg -q 'CANCELLED' "$skill" || fail "pr-readiness skill does not mention CANCELLED"

  # Must not claim CANCELLED is OK/skipped
  if rg -qi 'CANCELLED.*(success|ok|ignore|skip)' "$skill" | head -1 | rg -q .; then
    # Allow "not skip" / "not success" phrasing; reject "CANCELLED is success"
    if rg -qi 'CANCELLED is (success|ok|fine)' "$skill"; then
      fail "pr-readiness treats CANCELLED as success"
    fi
  fi

  # Script should surface cancelled / cancelled checks when present
  if [[ -f "$script" ]]; then
    if ! rg -q 'CANCELLED|cancelled' "$script"; then
      fail "check-pr-readiness.sh does not handle CANCELLED status"
    fi
  fi

  echo "OK: cancelled-required guard documented in pr-readiness + script"
}

case "$MODE" in
  --cancelled-required|--fixtures)
    check_cancelled_required
    ;;
  -h|--help)
    sed -n '2,12p' "$0"
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    exit 2
    ;;
esac
