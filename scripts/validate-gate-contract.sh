#!/usr/bin/env bash
# validate-gate-contract.sh — W2.1 gate matrix integrity checks
#
# Usage:
#   ./scripts/validate-gate-contract.sh
#   ./scripts/validate-gate-contract.sh --ci-parity   # authoritative CI surface presence
#
# Fails when:
#   - plans/GATE_CONTRACT.md is missing required sections
#   - quality-gates.sh default coverage floor disagrees with the matrix
#   - AGENTS aspirational 90% is not mentioned as target (not floor)
#   - (--ci-parity) required workflows / scripts from the gate matrix are missing

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT="$PROJECT_ROOT/plans/GATE_CONTRACT.md"
QG="$PROJECT_ROOT/scripts/quality-gates.sh"
WF_DIR="$PROJECT_ROOT/.github/workflows"
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
  # Authoritative surfaces from GATE_CONTRACT matrix (W2.1b)
  require_file() {
    local path="$1"
    local why="$2"
    [[ -f "$path" ]] || fail "CI parity missing $why: $path"
  }

  require_file "$WF_DIR/quick-check.yml" "fmt/clippy Quick PR Check"
  require_file "$WF_DIR/ci.yml" "tests / quality-gates CI"
  require_file "$WF_DIR/release-drift.yml" "release cadence"
  require_file "$PROJECT_ROOT/scripts/run-evals.sh" "skill evals runner"
  require_file "$PROJECT_ROOT/scripts/check-release-drift.sh" "release drift script"
  require_file "$PROJECT_ROOT/scripts/code-quality.sh" "local fmt/clippy entrypoint"
  require_file "$PROJECT_ROOT/scripts/quality-gates.sh" "local quality bundle"

  # Skill evals CI job (K3.1b) — dedicated workflow preferred
  if [[ -f "$WF_DIR/skill-evals.yml" ]]; then
    if ! grep -qE 'run-evals\.sh' "$WF_DIR/skill-evals.yml"; then
      fail "skill-evals.yml does not invoke run-evals.sh"
    fi
    if ! grep -qE 'validate-gate-contract\.sh' "$WF_DIR/skill-evals.yml"; then
      fail "skill-evals.yml does not invoke validate-gate-contract.sh"
    fi
  else
    # Fallback: some other workflow must run fixtures
    if ! grep -rqE 'run-evals\.sh' "$WF_DIR" --include='*.yml'; then
      fail "no workflow invokes scripts/run-evals.sh (K3.1b skill-evals CI missing)"
    fi
  fi

  # Security / supply-chain / cargo deny (at least one authoritative surface)
  has_security=false
  for candidate in security.yml supply-chain.yml; do
    if [[ -f "$WF_DIR/$candidate" ]]; then
      has_security=true
      break
    fi
  done
  [[ "$has_security" == true ]] || fail "missing security or supply-chain workflow"

  if ! grep -rqE 'cargo deny|cargo-deny' "$WF_DIR" --include='*.yml'; then
    fail "no workflow runs cargo deny / cargo-deny"
  fi

  # Quick check must mention fmt and clippy
  if ! grep -qE 'fmt|clippy' "$WF_DIR/quick-check.yml"; then
    fail "quick-check.yml does not mention fmt/clippy"
  fi

  # CI must mention tests (nextest or cargo test)
  if ! grep -qE 'nextest|cargo test' "$WF_DIR/ci.yml"; then
    fail "ci.yml does not mention nextest or cargo test"
  fi

  # Contract doc must list skill-evals CI entrypoint once wired
  if ! grep -qE 'skill-evals|run-evals\.sh --fixtures' "$CONTRACT"; then
    fail "GATE_CONTRACT.md must document skill-evals / fixtures CI entrypoint"
  fi
fi

echo -e "${GREEN}PASS${NC}: gate contract consistent (local coverage floor=${floor}%)"
exit 0
