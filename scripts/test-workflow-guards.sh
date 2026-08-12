#!/usr/bin/env bash
# test-workflow-guards.sh — W2.2b: cancelled required checks must not pass as success
#
# Usage:
#   ./scripts/test-workflow-guards.sh --cancelled-required
#   ./scripts/test-workflow-guards.sh --fixtures
#   ./scripts/test-workflow-guards.sh --required-aggregate
#
# Validates that pr-readiness skill and check-pr-readiness.sh treat CANCELLED
# required checks as blockers (not equivalent to SKIPPED/SUCCESS), and that the
# CI / Required aggregate evaluator (scripts/ci-required-evaluate.sh) accepts
# ONLY success results with the same-run fast-gate topology in place.

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

check_required_aggregate() {
  local ev="./scripts/ci-required-evaluate.sh"
  local ci=".github/workflows/ci.yml"
  local wf_dir=".github/workflows"
  local out rc

  [[ -f "$ev" ]] || fail "missing $ev"
  [[ -x "$ev" ]] || fail "$ev is not executable"

  # (1) All-success arguments must exit zero with exactly one success line.
  out=$("$ev" "Fast Gate=success" "Commit Message Lint=success" "Tests=success" \
    "MCP Build=success" "Multi-Platform=success" "Quality Gates=success") || \
    fail "all-success arguments to ci-required-evaluate.sh must exit 0"
  [[ "$(printf '%s\n' "$out" | rg -c 'CI / Required aggregate passed')" -eq 1 ]] || \
    fail "all-success must print exactly one success line"

  # (2) Each non-success / malformed / unknown / missing result must exit
  #     nonzero and print an ::error:: naming the culprit.
  local arg
  for arg in \
    "Fast Gate=failure" \
    "Fast Gate=cancelled" \
    "Fast Gate=timed_out" \
    "Fast Gate=skipped" \
    "Fast Gate=unknown" \
    "=success" \
    "Fast Gate=" \
    "Fast Gate" \
    "Fast Gate=success=extra" \
    ""
  do
    set +e
    out=$("$ev" "$arg" 2>&1)
    rc=$?
    set -e
    [[ "$rc" -ne 0 ]] || fail "argument '$arg' must exit nonzero"
    printf '%s\n' "$out" | rg -q '::error::' || fail "argument '$arg' must print ::error::"
  done
  # Zero arguments (missing results) must fail too.
  set +e
  out=$("$ev" 2>&1)
  rc=$?
  set -e
  [[ "$rc" -ne 0 ]] || fail "zero arguments to ci-required-evaluate.sh must exit nonzero"
  printf '%s\n' "$out" | rg -q '::error::' || fail "zero arguments must print ::error::"

  # (3) Current ci.yml references every evaluator dependency: the CI / Required
  #     needs set (same-run fast gate + substantive jobs) and the script name.
  [[ -f "$ci" ]] || fail "missing $ci"
  if ! rg -q 'needs: \[fast-gate, commitlint, test, mcp-build, multi-platform, quality-gates\]' "$ci"; then
    fail "ci.yml CI / Required aggregate needs set must be exactly [fast-gate, commitlint, test, mcp-build, multi-platform, quality-gates]"
  fi
  rg -q 'ci-required-evaluate\.sh' "$ci" || fail "ci.yml required job does not invoke scripts/ci-required-evaluate.sh"

  # (4) A restored waiter/anchor topology must NOT pass the positive guard:
  #     wait-on-check-action in ci.yml, or a quick-check.yml / pr-check-anchor.yml
  #     file, fails this guard.
  if rg -q 'wait-on-check-action' "$ci"; then
    fail "negative fixture: ci.yml must not use wait-on-check-action (restored waiter topology must fail the guard)"
  fi
  local f
  for f in "$wf_dir/quick-check.yml" "$wf_dir/pr-check-anchor.yml"; do
    if [[ -f "$f" ]]; then
      fail "negative fixture: obsolete $f must not exist (restored anchor topology must fail the guard)"
    fi
  done

  echo "OK: CI / Required aggregate evaluator + same-run fast-gate topology verified"
}

case "$MODE" in
  --cancelled-required|--fixtures)
    check_cancelled_required
    ;;
  --required-aggregate)
    check_required_aggregate
    ;;
  -h|--help)
    sed -n '2,14p' "$0"
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    exit 2
    ;;
esac
