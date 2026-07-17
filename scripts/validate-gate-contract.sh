#!/usr/bin/env bash
# validate-gate-contract.sh — W2.1 gate matrix integrity checks
#
# Usage:
#   ./scripts/validate-gate-contract.sh
#   ./scripts/validate-gate-contract.sh --ci-parity   # extra CI workflow presence checks
#
# Fails when:
#   - plans/GATE_CONTRACT.md is missing required sections
#   - quality-gates.sh default coverage floor disagrees with the matrix
#   - AGENTS aspirational 90% is not mentioned as target (not floor)

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT="$PROJECT_ROOT/plans/GATE_CONTRACT.md"
QG="$PROJECT_ROOT/scripts/quality-gates.sh"
CI_PARITY=false

if [[ "${1:-}" == "--ci-parity" ]]; then
  CI_PARITY=true
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

fail() {
  echo -e "${RED}FAIL${NC}: $*" >&2
  exit 1
}

[[ -f "$CONTRACT" ]] || fail "missing $CONTRACT"
[[ -f "$QG" ]] || fail "missing $QG"

# Required headings / anchors
for needle in \
  "Gate matrix" \
  "Coverage truth" \
  "Blocking floor" \
  "Aspirational target" \
  "Authoritative surface" \
  "Local vs CI parity" \
  "QUALITY_GATE_COVERAGE_THRESHOLD"
do
  grep -q "$needle" "$CONTRACT" || fail "GATE_CONTRACT.md missing required text: $needle"
done

# Parse default coverage floor from quality-gates.sh
# e.g. COVERAGE_THRESHOLD=${QUALITY_GATE_COVERAGE_THRESHOLD:-70}
floor=$(grep -E 'QUALITY_GATE_COVERAGE_THRESHOLD:-' "$QG" | head -1 | sed -E 's/.*:-([0-9]+).*/\1/')
[[ -n "$floor" ]] || fail "could not parse QUALITY_GATE_COVERAGE_THRESHOLD default from quality-gates.sh"

# Matrix must state this floor explicitly
grep -q "${floor}%" "$CONTRACT" || fail "GATE_CONTRACT.md does not document local blocking floor ${floor}%"

# Must not claim the script default is 90 without evidence
if grep -q 'default floor 90%' "$CONTRACT"; then
  fail "GATE_CONTRACT.md incorrectly claims default floor 90%"
fi

# AGENTS aspirational target documented
grep -q '90%' "$CONTRACT" || fail "GATE_CONTRACT.md should document 90% aspirational target"

if [[ "$CI_PARITY" == true ]]; then
  # Minimal workflow presence checks
  for wf in \
    "$PROJECT_ROOT/.github/workflows/ci.yml" \
    "$PROJECT_ROOT/.github/workflows/release-drift.yml"
  do
    [[ -f "$wf" ]] || fail "missing workflow for CI parity: $wf"
  done
  # Quick check or format/clippy mentioned in some workflow
  if ! rg -q 'clippy|fmt' "$PROJECT_ROOT/.github/workflows" -g '*.yml'; then
    fail "no workflow mentions clippy/fmt"
  fi
fi

echo -e "${GREEN}PASS${NC}: gate contract consistent (local coverage floor=${floor}%)"
exit 0
