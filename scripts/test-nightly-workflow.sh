#!/usr/bin/env bash
# test-nightly-workflow.sh — W2.5b nightly workflow fixtures
#
# Usage:
#   ./scripts/test-nightly-workflow.sh --fixtures
#
# Validates:
#   - .github/workflows/nightly-tests.yml exists
#   - cargo clean does not appear BEFORE upload-artifact (W2.5b:
#     reports must be uploaded before cleanup wipes target/)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"
WF=".github/workflows/nightly-tests.yml"

fail() {
  echo "HARNESS VIOLATION: nightly-workflow — $1" >&2
  exit 1
}

check_workflow_exists() {
  [[ -f "$WF" ]] || fail "missing $WF"
  echo "OK: $WF exists"
}

# Fail if any executable 'cargo clean' appears before the first upload-artifact use.
# Ignore comments that merely mention cargo clean (W2.5b prose).
# Pass if upload appears before clean, or there is no cargo clean.
check_upload_before_clean() {
  [[ -f "$WF" ]] || fail "missing $WF"

  local first_upload first_clean
  first_upload=$(rg -n 'upload-artifact' "$WF" | head -1 | cut -d: -f1 || true)
  # Match cargo clean only on non-comment YAML/run lines
  first_clean=$(rg -n '^\s*[^#[:space:]].*cargo clean|^\s*cargo clean' "$WF" | head -1 | cut -d: -f1 || true)

  if [[ -z "${first_clean:-}" ]]; then
    echo "OK: no cargo clean in nightly workflow"
    return 0
  fi

  if [[ -z "${first_upload:-}" ]]; then
    fail "cargo clean at line $first_clean but no upload-artifact found (W2.5b)"
  fi

  if [[ "$first_clean" -lt "$first_upload" ]]; then
    fail "cargo clean (line $first_clean) appears BEFORE upload-artifact (line $first_upload) — W2.5b requires upload before cleanup"
  fi

  echo "OK: upload-artifact (line $first_upload) before cargo clean (line $first_clean)"
}

case "$MODE" in
  --fixtures)
    check_workflow_exists
    check_upload_before_clean
    echo "OK: nightly workflow fixtures passed"
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
