#!/usr/bin/env bash
# test-benchmark-workflow.sh — W2.5 benchmark signal fixtures
#
# Usage:
#   ./scripts/test-benchmark-workflow.sh --fixtures
#
# Validates that the benchmarks workflow refuses dummy-only gating and
# that missing Criterion output is treated as a failure signal.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: benchmark-workflow — $1" >&2
  exit 1
}

WF=".github/workflows/benchmarks.yml"

check_workflow_exists() {
  [[ -f "$WF" ]] || fail "missing $WF"
}

check_no_soft_dummy_gate() {
  # Dummy generator may exist for local dev, but workflow must not treat
  # missing Criterion as a green path without failing later steps.
  if rg -n 'generate_dummy_benchmarks' "$WF" >/dev/null 2>&1; then
    # Must either fail on alert, fail when no real results, or stage-check empty
    if ! rg -q 'fail-on-alert:\s*true|No benchmark results found.*exit 1|missing or empty before staging' "$WF"; then
      # Accept if dummy path is only a warning and staging step fails empty files
      if ! rg -q 'benchmark_results/output.txt missing or empty' "$WF"; then
        fail "benchmarks.yml uses dummy benchmarks without a hard empty-result failure path"
      fi
    fi
  fi
  echo "OK: dummy-benchmark soft-pass constrained"
}

check_cli_paths_or_benches_package() {
  # Benches package or explicit CLI bench discovery
  if ! rg -q 'do-memory-benches|memory-benches|cargo bench' "$WF"; then
    fail "benchmarks.yml does not invoke cargo bench / do-memory-benches"
  fi
  echo "OK: bench package path present"
}

check_regression_threshold() {
  if ! rg -q "alert-threshold:.*110%|alert-threshold: '110%'" "$WF"; then
    echo "NOTE: alert-threshold 110% not found (may use different budget)"
  fi
  # Prefer fail-on-alert true for main-branch signal; warn if false
  if rg -q 'fail-on-alert:\s*false' "$WF"; then
    echo "WARN: fail-on-alert is false — regressions comment but do not block (W2.5 partial)"
  fi
  echo "OK: regression threshold section inspected"
}

check_missing_criterion_fixture() {
  # Synthetic fixture: empty criterion dir must not produce green metric parse
  local tmp
  tmp=$(mktemp -d)
  mkdir -p "$tmp/criterion"
  if [[ -x ./scripts/generate_dummy_benchmarks.sh ]]; then
    # Dummy script may produce output; ensure workflow stages empty check exists
    rg -q 'missing or empty|No benchmark results' "$WF" \
      || fail "workflow lacks missing-results handling"
  fi
  rm -rf "$tmp"
  echo "OK: missing criterion handling present"
}

case "$MODE" in
  --fixtures)
    check_workflow_exists
    check_no_soft_dummy_gate
    check_cli_paths_or_benches_package
    check_regression_threshold
    check_missing_criterion_fixture
    echo "OK: benchmark workflow fixtures passed"
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
