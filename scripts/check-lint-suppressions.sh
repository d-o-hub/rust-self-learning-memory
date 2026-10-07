#!/usr/bin/env bash
# check-lint-suppressions.sh — crate-root #![allow] ceiling ratchet
#
# Counts crate-root `#![allow(...)]` inner attributes across the workspace
# (*/src/lib.rs and */src/main.rs) and fails when the total exceeds the recorded
# baseline. This is the real gate preventing new blanket suppressions from
# landing silently — the workspace `allow_attributes = "deny"` lint is currently
# inert and does not catch crate-root allows.
#
# Usage:
#   ./scripts/check-lint-suppressions.sh              # verify against baseline
#   ./scripts/check-lint-suppressions.sh --update     # rewrite baseline from tree
#   ./scripts/check-lint-suppressions.sh --baseline <file>

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

readonly DEFAULT_BASELINE="scripts/lint-suppressions-baseline.txt"
BASELINE_FILE="$DEFAULT_BASELINE"
UPDATE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --update)
      UPDATE=1
      shift
      ;;
    --baseline)
      BASELINE_FILE="${2:?baseline file required}"
      shift 2
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

# Count crate-root `#![allow` inner attributes in */src/lib.rs and */src/main.rs.
count_suppressions() {
  shopt -s nullglob
  local files=(*/src/lib.rs */src/main.rs)
  shopt -u nullglob
  if (( ${#files[@]} == 0 )); then
    echo 0
    return
  fi
  grep -chE '^[[:space:]]*#!\[allow' "${files[@]}" 2>/dev/null | awk '{s+=$1} END {print s+0}' || true
}

COUNT=$(count_suppressions)
COUNT=$((COUNT + 0))

if (( UPDATE )); then
  {
    echo "# Crate-root #![allow(...)] ceiling baseline for scripts/check-lint-suppressions.sh"
    echo "# Regenerate with: ./scripts/check-lint-suppressions.sh --update"
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

echo "crate_root_allows=$COUNT baseline=$BASELINE"

if (( COUNT > BASELINE )); then
  echo "HARNESS VIOLATION: lint-suppressions — $COUNT crate-root #![allow] exceeds baseline $BASELINE" >&2
  echo "Narrow or remove the new suppression (prefer per-site #[expect] with a reason);" >&2
  echo "only run --update after review and record the justification." >&2
  exit 1
fi

if (( COUNT < BASELINE )); then
  echo "NOTE: $((BASELINE - COUNT)) fewer crate-root suppressions than baseline; run --update to ratchet down." >&2
fi

echo "OK: crate-root suppression count within baseline"
