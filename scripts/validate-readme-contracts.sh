#!/usr/bin/env bash
# validate-readme-contracts.sh — D3.2a README public-contract checks
#
# Usage:
#   ./scripts/validate-readme-contracts.sh
#   ./scripts/validate-readme-contracts.sh --fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

README="README.md"
MODE="${1:-}"

fail() {
  echo "HARNESS VIOLATION: readme-contracts — $1" >&2
  exit 1
}

check_readme_exists() {
  [[ -f "$README" ]] || fail "missing $README"
  [[ -s "$README" ]] || fail "$README is empty"
  echo "OK: README.md exists"
}

check_no_wasmtime_backend() {
  # Removed/unsupported feature must not be claimed in public README
  if rg -q 'wasmtime-backend' "$README"; then
    fail "README.md claims wasmtime-backend feature (must not)"
  fi
  echo "OK: no wasmtime-backend claim"
}

check_task_context_or_config() {
  # Public API / config surface should be discoverable
  if rg -qi 'TaskContext' "$README"; then
    echo "OK: README mentions TaskContext"
    return 0
  fi
  if rg -qi '\bconfig\b' "$README"; then
    echo "OK: README mentions config"
    return 0
  fi
  fail "README.md mentions neither TaskContext nor config"
}

run_all() {
  check_readme_exists
  check_no_wasmtime_backend
  check_task_context_or_config
  echo "OK: readme contracts validated"
}

case "$MODE" in
  ""|--fixtures)
    run_all
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
