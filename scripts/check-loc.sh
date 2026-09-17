#!/usr/bin/env bash
# check-loc.sh — enforces the 500 LOC per file invariant (AGENTS.md).
#
# Sensor: scripts/check-loc.sh  (do-harness sensor `loc`, pre-commit hook)
#
# Scope and classification mirror the blocking gate in
# scripts/quality-gates.sh (`run_source_file_size_gate`): every tracked or
# untracked Rust file in the workspace is considered, and files that hold
# tests (`tests/`, `*/tests/`, `*_test.rs`, `*_tests.rs`, `*/tests.rs`) are
# reported but never block — matching the gate's carve-out for test modules.
#
# Usage: check-loc.sh [MAX_LINES]   (default 500)

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
MAX="${1:-500}"
THRESHOLD=450
FAIL=0

cd "$ROOT"

oversized_source=()
oversized_tests=()
nearing=()

while IFS= read -r file; do
    [ -n "$file" ] || continue
    case "$file" in
        benches/*|target/*|.git/*) continue ;;
    esac
    [ -f "$file" ] || continue

    lines="$(wc -l < "$file" | tr -d ' ')"

    case "$file" in
        tests/*|*/tests/*|*_test.rs|*_tests.rs|*/tests.rs)
            if (( lines > MAX )); then
                oversized_tests+=("$file:$lines")
            fi
            ;;
        *)
            if (( lines > MAX )); then
                oversized_source+=("$file:$lines")
                FAIL=1
            elif (( lines >= THRESHOLD )); then
                nearing+=("$file:$lines")
            fi
            ;;
    esac
done < <(
    {
        git ls-files '*.rs'
        git ls-files --others --exclude-standard '*.rs'
    } | awk '!seen[$0]++'
)

# Keep the warning compact: a repo-wide list on every commit drowns the signal.
if (( ${#nearing[@]} > 0 )); then
    echo "WARN: ${#nearing[@]} source file(s) at or above $THRESHOLD lines (decomposition threshold, max $MAX)."
    printf '  %s\n' "${nearing[@]:0:5}"
    (( ${#nearing[@]} > 5 )) && echo "  ... and $(( ${#nearing[@]} - 5 )) more"
fi

if (( ${#oversized_tests[@]} > 0 )); then
    echo "NOTE: ${#oversized_tests[@]} oversized test file(s) (non-blocking, max $MAX)"
fi

if (( FAIL )); then
    echo "FAIL: ${#oversized_source[@]} source file(s) exceed $MAX lines:"
    printf '  %s\n' "${oversized_source[@]}"
    echo "LOC ceiling violated."
    exit 1
fi

echo "check-loc OK: all source files under $MAX lines."
