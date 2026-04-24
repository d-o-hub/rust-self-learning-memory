#!/bin/bash
# Test Runner for Advanced Pattern Algorithms
# Run this after resolving compilation issues in memory-storage-turso

set -e

echo "========================================="
echo "Advanced Pattern Algorithms Test Runner"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run tests and count results
run_test_group() {
    local group_name=$1
    local test_path=$2

    echo -e "${YELLOW}Running: ${group_name}${NC}"
    echo "Command: cargo test --package memory-mcp --lib ${test_path}"

    if cargo test --package memory-mcp --lib "${test_path}" 2>&1 | tee /tmp/test_output.txt; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "${GREEN}✓ PASSED: ${group_name}${NC}"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "${RED}✗ FAILED: ${group_name}${NC}"
        echo "Check /tmp/test_output.txt for details"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
}

# Check if compilation issues exist
echo "Step 1: Checking compilation status..."
if ! cargo check --workspace 2>&1 | grep -q "error\[E"; then
    echo -e "${GREEN}✓ Workspace compiles successfully${NC}"
else
    echo -e "${RED}✗ Workspace has compilation errors${NC}"
    echo "Please fix the following errors first:"
    cargo check --workspace 2>&1 | grep "error\[E" | head -5
    echo ""
    echo "Main issues to fix:"
    echo "1. Duplicate get_patterns_batch in memory-storage-turso"
    echo "2. Missing Debug trait in memory-storage-turso/pool/adaptive.rs"
    echo "3. Async issues in memory-storage-turso/storage/batch/"
    exit 1
fi
echo ""

# Run tests
echo "Step 2: Running Pattern Algorithm Tests..."
echo ""

# Part 1: DBSCAN Tests
run_test_group "DBSCAN Unit Tests" "patterns::predictive::dbscan_tests::dbscan_unit_tests"
run_test_group "DBSCAN Integration Tests" "patterns::predictive::dbscan_tests::dbscan_integration_tests"

# Part 2: BOCPD Tests
run_test_group "BOCPD Unit Tests" "patterns::statistical::bocpd_tests::bocpd_unit_tests"
run_test_group "BOCPD Integration Tests" "patterns::statistical::bocpd_tests::bocpd_integration_tests"

# Part 3: Pattern Extraction Tests
run_test_group "Pattern Extraction Tests" "patterns::extraction"

# Part 4: Tool Compatibility Tests
run_test_group "Tool Compatibility Tests" "patterns::compatibility"

# Part 5: Performance Benchmarks
echo -e "${YELLOW}Running: Performance Benchmarks${NC}"
echo "Command: cargo test --package memory-mcp --lib patterns::benchmarks --release"
if cargo test --package memory-mcp --lib "patterns::benchmarks" --release 2>&1 | tee /tmp/benchmark_output.txt; then
    echo -e "${GREEN}✓ Benchmarks completed${NC}"
else
    echo -e "${YELLOW}⚠ Benchmarks had issues (non-critical)${NC}"
fi
echo ""

# Summary
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Total Test Groups: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"
fi
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed! 🎉${NC}"
    echo ""
    echo "Next Steps:"
    echo "1. Review test output for performance metrics"
    echo "2. Integrate pattern extraction with MCP tools"
    echo "3. Deploy tool compatibility assessment"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the output above.${NC}"
    exit 1
fi
