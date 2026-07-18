#!/usr/bin/env bash
# validate-plans.sh — D3.3 / T0 plan hygiene checks
#
# Usage:
#   ./scripts/validate-plans.sh --active-set
#   ./scripts/validate-plans.sh --version-state
#   ./scripts/validate-plans.sh --release-policy
#   ./scripts/validate-plans.sh --all

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---all}"

fail() {
  echo "HARNESS VIOLATION: plans — $1" >&2
  exit 1
}

check_active_set() {
  local required=(
    plans/GOALS.md
    plans/ACTIONS.md
    plans/GOAP_STATE.md
    plans/ROADMAPS/ROADMAP_ACTIVE.md
    plans/STATUS/CURRENT.md
    plans/GATE_CONTRACT.md
  )
  for f in "${required[@]}"; do
    [[ -f "$f" ]] || fail "missing canonical plan file: $f"
  done
  echo "OK: active-set present"
}

check_version_state() {
  local cargo_ver tag_ver
  cargo_ver=$(rg -n '^version\s*=' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
  tag_ver=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "")
  [[ -n "$cargo_ver" ]] || fail "could not parse workspace version from Cargo.toml"

  # CURRENT.md should mention workspace or released version somewhere
  if ! rg -q "$cargo_ver|0\.[0-9]+\.[0-9]+" plans/STATUS/CURRENT.md; then
    fail "plans/STATUS/CURRENT.md does not mention a semver version"
  fi

  echo "OK: version-state cargo=$cargo_ver latest_tag=${tag_ver:-none}"
}

check_release_policy() {
  # Active skills must not instruct manual gh release create without NEVER
  if [[ -f .agents/skills/release-guard/SKILL.md ]]; then
    rg -q 'release-manager.sh ship --execute' .agents/skills/release-guard/SKILL.md \
      || fail "release-guard missing canonical ship path"
    rg -q 'NEVER' .agents/skills/release-guard/SKILL.md \
      || fail "release-guard missing NEVER for manual release"
  fi
  echo "OK: release-policy"
}

case "$MODE" in
  --active-set) check_active_set ;;
  --version-state) check_version_state ;;
  --release-policy) check_release_policy ;;
  --all)
    check_active_set
    check_version_state
    check_release_policy
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
