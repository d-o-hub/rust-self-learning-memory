#!/usr/bin/env bash
# Coverage Monitoring Script (ACT-037)
# Reports code coverage by crate and overall

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default threshold
THRESHOLD=${THRESHOLD:-90}

# Parse arguments
JSON_OUTPUT=false
SUMMARY_MODE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --summary-mode)
            SUMMARY_MODE=true
            shift
            ;;
        --threshold)
            THRESHOLD="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--json] [--summary-mode] [--threshold N]"
            echo "  --json       Output in JSON format"
            echo "  --summary-mode  Parse existing llvm-cov report without running tests"
            echo "  --threshold  Minimum coverage threshold (default: 90)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Check if llvm-cov is available
if ! cargo llvm-cov --version &> /dev/null; then
    echo -e "${RED}Error: cargo-llvm-cov not installed${NC}"
    echo "Install with: cargo install cargo-llvm-cov"
    exit 1
fi

# Validate threshold format (integer or decimal)
if ! [[ "$THRESHOLD" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo -e "${RED}Error: --threshold must be numeric, got '$THRESHOLD'${NC}"
    exit 1
fi

# Run coverage
if [[ "$JSON_OUTPUT" == "true" ]]; then
    cargo llvm-cov nextest --workspace \
        --exclude memory-benches \
        --exclude memory-examples \
        --exclude test-utils \
        --json
else
    echo -e "${YELLOW}Running coverage analysis...${NC}"
    echo ""

    if [[ "$SUMMARY_MODE" == "true" ]]; then
        # Fast path for CI/local validation when coverage artifacts already exist.
        if ! COVERAGE_OUTPUT=$(cargo llvm-cov report --summary-only 2>&1); then
            echo "$COVERAGE_OUTPUT"
            echo -e "${RED}Coverage report command failed (try without --summary-mode)${NC}"
            exit 1
        fi
    else
        # Full coverage execution aligned with CI scope.
        if ! COVERAGE_OUTPUT=$(cargo llvm-cov nextest --workspace \
            --exclude memory-benches \
            --exclude memory-examples \
            --exclude test-utils \
            --summary-only 2>&1); then
            echo "$COVERAGE_OUTPUT"
            echo -e "${RED}Coverage command failed${NC}"
            exit 1
        fi
    fi

    echo "$COVERAGE_OUTPUT"

    echo ""
    echo -e "${YELLOW}Per-crate coverage:${NC}"

    # Try to get per-crate breakdown for workspace members
    CRATE_LINES=$(cargo llvm-cov report --summary-only 2>&1 | awk '/^(memory-|test-utils|examples|tests|benches)/ {print}')
    if [ -n "$CRATE_LINES" ]; then
        while IFS= read -r line; do
            crate=$(echo "$line" | awk '{print $1}')
            region=$(echo "$line" | awk '{print $2}')
            function=$(echo "$line" | awk '{print $3}')

            echo "  $crate: Region=$region Functions=$function"
        done <<< "$CRATE_LINES"
    else
        echo "  (Detailed report not available)"
    fi

    echo ""
    echo -e "${YELLOW}Threshold check:${NC}"
    echo "  Target: ${THRESHOLD}%"

    # Parse TOTAL row from summary output and extract the first percentage on that line.
    # This avoids matching unrelated percentages from tool logs.
    TOTAL_LINE=$(printf "%s\n" "$COVERAGE_OUTPUT" | awk '/^TOTAL[[:space:]]+/ {line=$0} END {print line}')
    if [ -z "$TOTAL_LINE" ]; then
        TOTAL_LINE=$(printf "%s\n" "$COVERAGE_OUTPUT" | awk '/TOTAL/ {line=$0} END {print line}')
    fi

    COVERAGE_PCT=""
    if [ -n "$TOTAL_LINE" ]; then
        COVERAGE_PCT=$(printf "%s\n" "$TOTAL_LINE" | awk '{
            for (i = 1; i <= NF; i++) {
                if ($i ~ /%$/) {
                    gsub(/%/, "", $i)
                    print $i
                    exit
                }
            }
        }')
    fi

    if [ -z "$COVERAGE_PCT" ]; then
        echo -e "  ${RED}Could not parse coverage percentage from cargo llvm-cov output${NC}"
        exit 1
    fi

    echo "  Measured: ${COVERAGE_PCT}%"

    if awk -v measured="$COVERAGE_PCT" -v threshold="$THRESHOLD" 'BEGIN { exit !(measured + 0 >= threshold + 0) }'; then
        echo -e "  ${GREEN}Coverage check passed${NC}"
    else
        echo -e "  ${RED}Coverage check failed: ${COVERAGE_PCT}% < ${THRESHOLD}%${NC}"
        exit 1
    fi
fi
