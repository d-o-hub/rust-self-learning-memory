#!/bin/bash
# E2E Test Runner for Embeddings
# Runs all E2E tests and generates a comprehensive report

set -e

echo "=========================================="
echo "Embeddings E2E Test Suite"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Results file
REPORT_FILE="e2e_test_results.txt"
echo "E2E Test Results - $(date)" > "$REPORT_FILE"
echo "==========================================" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Function to run a test suite
run_test_suite() {
    local test_name=$1
    local test_file=$2
    
    echo -e "${BLUE}Running: ${test_name}${NC}"
    echo "Running: ${test_name}" >> "$REPORT_FILE"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if cargo test --test "$test_file" --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
        echo -e "${GREEN}✓ ${test_name} PASSED${NC}"
        echo "Status: PASSED" >> "$REPORT_FILE"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ ${test_name} FAILED${NC}"
        echo "Status: FAILED" >> "$REPORT_FILE"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo "" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
}

# Change to the workspace root
cd "$(dirname "$0")/../.."

# Day 1: Provider E2E Tests
echo "=========================================="
echo "Day 1: Provider E2E Tests"
echo "=========================================="
echo "" >> "$REPORT_FILE"
echo "=== Day 1: Provider Tests ===" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

run_test_suite "OpenAI Provider E2E" "embeddings_openai"
run_test_suite "Local Provider E2E" "embeddings_local"

# Day 2: Integration E2E Tests
echo "=========================================="
echo "Day 2: Integration E2E Tests"
echo "=========================================="
echo "" >> "$REPORT_FILE"
echo "=== Day 2: Integration Tests ===" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

run_test_suite "CLI Integration E2E" "embeddings_cli"
run_test_suite "MCP Integration E2E" "embeddings_mcp"

# Day 3: Quality & Performance Tests
echo "=========================================="
echo "Day 3: Quality & Performance Tests"
echo "=========================================="
echo "" >> "$REPORT_FILE"
echo "=== Day 3: Quality & Performance Tests ===" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

run_test_suite "Quality Tests" "embeddings_quality"
run_test_suite "Performance Tests" "embeddings_performance"

# Print summary
echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"
echo "Skipped:       $SKIPPED_TESTS"
echo ""

# Append summary to report
echo "" >> "$REPORT_FILE"
echo "==========================================" >> "$REPORT_FILE"
echo "Summary" >> "$REPORT_FILE"
echo "==========================================" >> "$REPORT_FILE"
echo "Total Tests:  $TOTAL_TESTS" >> "$REPORT_FILE"
echo "Passed:       $PASSED_TESTS" >> "$REPORT_FILE"
echo "Failed:       $FAILED_TESTS" >> "$REPORT_FILE"
echo "Skipped:       $SKIPPED_TESTS" >> "$REPORT_FILE"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Check $REPORT_FILE for details.${NC}"
    exit 1
fi
