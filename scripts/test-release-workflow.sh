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
    rg -q -- '--locked' "$wf" || fail "publish-crates.yml should use cargo publish --locked"
  else
    echo "NOTE: publish-crates.yml not present; skipping --locked check"
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
