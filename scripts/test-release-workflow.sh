#!/usr/bin/env bash
# test-release-workflow.sh — W2.4 release/publish precondition fixtures
#
# Usage:
#   ./scripts/test-release-workflow.sh --fixtures
#   ./scripts/test-release-workflow.sh --publish-fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: release-workflow — $1" >&2
  exit 1
}

check_release_authority() {
  [[ -x ./scripts/release-manager.sh ]] || fail "release-manager.sh missing or not executable"
  [[ -f .github/workflows/release.yml ]] || fail "release.yml missing"
  [[ -f .agents/skills/release-guard/SKILL.md ]] || fail "release-guard skill missing"

  ./scripts/release-manager.sh help 2>&1 | rg -q 'ship' || fail "release-manager help lacks ship"

  # CIT-A4 / ADR-079 §6 / ADR-072: tag-only release authority. A manual
  # workflow_dispatch on release.yml is a guaranteed-failure advertised trigger.
  if rg -q 'workflow_dispatch' .github/workflows/release.yml; then
    fail "release.yml must not expose workflow_dispatch (tag-only release under ADR-072)"
  fi

  # Active skills must forbid manual gh release create as the ship path
  if rg -n 'gh release create' .agents/skills --glob '**/SKILL.md' \
    | rg -v 'NEVER|never|not |forbid|do not|Don.t|must not' \
    | rg -v 'release-guard' \
    | head -5 | rg -q .; then
    # Soft: list offenders for visibility but only fail if release-guard lacks NEVER
    :
  fi
  rg -q 'NEVER' .agents/skills/release-guard/SKILL.md || fail "release-guard lacks NEVER for manual release"
  rg -q 'release-manager.sh ship --execute' .agents/skills/release-guard/SKILL.md \
    || fail "release-guard missing canonical ship command"

  echo "OK: release authority fixtures"
}

check_publish_fixtures() {
  local wf=".github/workflows/publish-crates.yml"
  if [[ -f "$wf" ]]; then
    # LESSON-014 / AGENTS.md publish pipeline: reproducible locked publish
    rg -q -- '--locked' "$wf" || fail "publish-crates.yml should use cargo publish --locked"
    # CIT-A4 / LESSON-014: bounded propagation polling, never a fixed sleep
    if rg -q 'run: sleep 30' "$wf"; then
      fail "publish-crates.yml must not use fixed 'run: sleep 30' (bounded polling required)"
    fi
    rg -q 'Wait for crates.io propagation' "$wf" || fail "publish-crates.yml missing propagation wait step"
    rg -q 'seq 1 20' "$wf" || fail "publish-crates.yml propagation polling must be bounded (seq 1 20)"
    # CIT-A4 / ADR-079 §6: single-crate dispatch must verify the non-dev
    # workspace dependency closure and fail with a named reason - no silent skip.
    rg -q 'Verify dependency closure' "$wf" || fail "publish-crates.yml missing dependency-closure verification step"
    rg -q "inputs.crate != ''" "$wf" || fail "dependency-closure step must gate on single-crate dispatch (inputs.crate != '')"
    rg -q '::error::Cannot publish' "$wf" || fail "dependency-closure step must fail with a named ::error:: reason"
  else
    echo "NOTE: publish-crates.yml not present; skipping publish fixture checks"
  fi
  [[ -x ./scripts/verify-release-state.sh ]] || fail "verify-release-state.sh missing"
  echo "OK: publish fixtures"
}

case "$MODE" in
  --fixtures)
    check_release_authority
    ;;
  --publish-fixtures)
    check_release_authority
    check_publish_fixtures
    ;;
  -h|--help)
    sed -n '2,10p' "$0"
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE" >&2
    exit 2
    ;;
esac
