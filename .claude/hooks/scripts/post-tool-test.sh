#!/bin/bash
# Auto-test after Write/Edit operations

set -e

# Parse stdin JSON to get file paths
FILES=$(jq -r '.tool_use.parameters.path // .tool_use.parameters.paths[]?' 2>/dev/null)

if [ -z "$FILES" ]; then
    exit 0
fi

# Determine affected crate
for file in $FILES; do
    if [[ $file =~ ^(memory-[^/]+)/ ]]; then
        CRATE="${BASH_REMATCH[1]}"

        echo "Testing $CRATE due to changes in $file"

        # Run quick tests for affected crate
        if cargo nextest run -p "$CRATE" \
            --profile quick \
            --no-fail-fast 2>&1; then
            echo "Tests passed for $CRATE"
        else
            echo "Tests failed for $CRATE"
            exit 1
        fi
    fi
done
