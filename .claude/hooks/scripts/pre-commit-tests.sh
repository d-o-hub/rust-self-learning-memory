#!/bin/bash
# Fast pre-commit test suite for rapid feedback

set -e

echo "Running pre-commit tests..."

# Only test changed crates
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)
CHANGED_CRATES=$(echo "$CHANGED_FILES" | \
    grep -E '^(memory-[^/]+)/' | \
    cut -d/ -f1 | \
    sort -u)

if [ -z "$CHANGED_CRATES" ]; then
    echo "No Rust crates modified"
    exit 0
fi

echo "Changed crates: $CHANGED_CRATES"

# Run fast unit tests only
for crate in $CHANGED_CRATES; do
    echo "Testing $crate..."
    if cargo nextest run -p "$crate" \
        --profile quick \
        -E 'kind(lib)' \
        --no-fail-fast 2>&1; then
        echo "Tests passed for $crate"
    else
        echo "Tests failed for $crate"
        exit 1
    fi
done

echo "Pre-commit tests passed"
