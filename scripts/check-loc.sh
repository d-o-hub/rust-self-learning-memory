#!/usr/bin/env bash
# check-loc.sh — enforces the 500 LOC per file invariant.
#
# Sensor: scripts/check-loc.sh
# Fails if any .rs file under src/ or crates/ exceeds MAX lines.
# Writes a warning when a file is at or above the decomposition threshold.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
MAX="${1:-500}"
THRESHOLD=450
FAIL=0

while IFS= read -r file; do
    lines="$(wc -l < "$file")"
    if (( lines > MAX )); then
        echo "FAIL: $file has $lines lines (max $MAX)"
        FAIL=1
    elif (( lines >= THRESHOLD )); then
        echo "WARN: $file is nearing the limit: $lines lines"
    fi
done < <(find "$ROOT/src" "$ROOT/crates" -name '*.rs' -not -path '*/target/*' 2>/dev/null)

if (( FAIL )); then
    echo "LOC ceiling violated."
    exit 1
fi

echo "check-loc OK: all source files under $MAX lines."