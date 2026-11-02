#!/usr/bin/env bash
# check-shell-counters.sh — post-increment statements must not decide control flow
#
# `((i++))` evaluates to the *pre-increment* value, so when that value is 0 the
# arithmetic command exits 1 and `set -e` aborts the script one iteration early.
# The report then looks complete but is truncated with no error (LESSON-059;
# same vacuous-green class as LESSON-026). Write `i=$((i + 1))`, which always
# exits 0, or make the test explicit with `(( ++i ))` / `|| true`.
#
# This is a ratchet: the recorded baseline is existing debt, and any new
# occurrence fails the gate.
#
# Usage:
#   ./scripts/check-shell-counters.sh              # verify against baseline
#   ./scripts/check-shell-counters.sh --update     # rewrite baseline from tree
#   ./scripts/check-shell-counters.sh --list       # show every occurrence

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

readonly DEFAULT_BASELINE="scripts/shell-counters-baseline.txt"
BASELINE_FILE="$DEFAULT_BASELINE"
UPDATE=0
LIST=0

readonly PATTERN='\(\([[:space:]]*[A-Za-z_][A-Za-z0-9_]*[[:space:]]*(\+\+|--)[[:space:]]*\)\)'

while [[ $# -gt 0 ]]; do
  case "$1" in
    --update) UPDATE=1; shift ;;
    --list)   LIST=1; shift ;;
    --baseline) BASELINE_FILE="${2:?baseline file required}"; shift 2 ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
done

matches() {
  git ls-files -z '*.sh' | xargs -0 grep -nEH "$PATTERN" 2>/dev/null || true
}

ALL="$(matches)"
COUNT=0
if [[ -n "$ALL" ]]; then
  COUNT=$(printf '%s\n' "$ALL" | wc -l)
fi

if (( LIST )); then
  printf '%s\n' "$ALL"
  echo "total=$COUNT"
  exit 0
fi

if (( UPDATE )); then
  {
    echo "# Bare ((var++)) / ((var--)) statements in tracked *.sh for"
    echo "#   scripts/check-shell-counters.sh — ratchet ceiling, never raise it"
    echo "# Regenerate with: ./scripts/check-shell-counters.sh --update"
    echo "$COUNT"
  } > "$BASELINE_FILE"
  echo "Baseline updated: $BASELINE_FILE = $COUNT"
  exit 0
fi

if [[ ! -f "$BASELINE_FILE" ]]; then
  echo "ERROR: baseline file '$BASELINE_FILE' not found; run with --update" >&2
  exit 1
fi

BASELINE=$(grep -vE '^[[:space:]]*#' "$BASELINE_FILE" | tr -d '[:space:]')
if [[ ! "$BASELINE" =~ ^[0-9]+$ ]]; then
  echo "ERROR: invalid baseline in '$BASELINE_FILE' (expected an integer)" >&2
  exit 1
fi

echo "bare_post_increments=$COUNT baseline=$BASELINE"

if (( COUNT > BASELINE )); then
  echo "HARNESS VIOLATION: shell-counters — $COUNT bare post-increment statements exceed baseline $BASELINE" >&2
  printf '%s\n' "$ALL" >&2
  echo "A value-0 increment under 'set -e' exits 1 and truncates the script." >&2
  echo "Use 'i=\$((i + 1))' in new code; run --list to see the recorded debt." >&2
  exit 1
fi

if (( COUNT < BASELINE )); then
  echo "NOTE: $((BASELINE - COUNT)) fewer than baseline; run --update to ratchet down." >&2
fi

echo "OK: bare post-increment count within baseline"
