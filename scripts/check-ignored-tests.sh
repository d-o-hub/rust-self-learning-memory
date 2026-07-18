#!/usr/bin/env bash
# check-ignored-tests.sh — W2.5b ignored-test ceiling ratchet
#
# Usage:
#   ./scripts/check-ignored-tests.sh
#   ./scripts/check-ignored-tests.sh --ceiling 200
#   ./scripts/check-ignored-tests.sh --fixture ratchet

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CEILING="${QUALITY_GATE_IGNORED_TEST_CEILING:-200}"
MODE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ceiling)
      CEILING="$2"
      shift 2
      ;;
    --fixture)
      MODE="fixture"
      shift
      if [[ "${1:-}" == "ratchet" ]]; then shift; fi
      ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

# Count #[ignore] attributes in Rust sources (production + tests).
# Prefer ripgrep; fall back to find+grep when rg is not installed (minimal CI images).
# Never fail with exit 127 when rg is missing — that broke Quick Check (CI).
count_ignores() {
  local n=0
  if command -v rg >/dev/null 2>&1; then
    n=$(rg -c '#\[ignore' --glob '*.rs' -g '!target/**' 2>/dev/null \
      | awk -F: '{s+=$2} END {print s+0}')
  elif command -v find >/dev/null 2>&1 && command -v grep >/dev/null 2>&1; then
    n=$(find . \( -path ./target -o -path ./.git \) -prune -o -name '*.rs' -print 2>/dev/null \
      | xargs grep -c '#\[ignore' 2>/dev/null \
      | awk -F: '{s+=$2} END {print s+0}')
  else
    echo "WARN: neither rg nor find+grep available; treating ignore count as 0" >&2
    n=0
  fi
  # Normalize empty
  echo "${n:-0}"
}

COUNT=$(count_ignores)
# Bash arithmetic requires integer
COUNT=$((COUNT + 0))

echo "ignored_test_attrs=$COUNT ceiling=$CEILING"

if [[ "$MODE" == "fixture" ]]; then
  # Ratchet fixture: ceiling must be numeric and count must not exceed it
  [[ "$CEILING" =~ ^[0-9]+$ ]] || {
    echo "HARNESS VIOLATION: ignored-tests — invalid ceiling" >&2
    exit 1
  }
fi

if (( COUNT > CEILING )); then
  echo "HARNESS VIOLATION: ignored-tests — $COUNT attrs exceed ceiling $CEILING" >&2
  echo "Lower ignores or raise ceiling only with documented evidence." >&2
  exit 1
fi

echo "OK: ignored-test count within ceiling"
