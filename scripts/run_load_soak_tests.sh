#!/bin/bash
# Comprehensive Load and Soak Test Runner
# Runs all load and soak tests and generates a comprehensive report

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/plans/test-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create report directory if it doesn't exist
mkdir -p "${REPORT_DIR}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Load and Soak Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to print section headers
print_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Function to run test with timing
run_test() {
    local test_name=$1
    local test_file=$2
    local report_file="${REPORT_DIR}/test_${TIMESTAMP}_${test_name}.txt"

    print_section "Running: ${test_name}"
    echo "Report file: ${report_file}"
    echo ""

    local start=$(date +%s)

    if cargo test --test ${test_file} 2>&1 | tee "${report_file}"; then
        local end=$(date +%s)
        local duration=$((end - start))
        echo -e "${GREEN}✓ ${test_name} passed in ${duration}s${NC}"
        echo "${test_name},PASSED,${duration}" >> "${REPORT_DIR}/summary_${TIMESTAMP}.csv"
        return 0
    else
        local end=$(date +%s)
        local duration=$((end - start))
        echo -e "${RED}✗ ${test_name} failed in ${duration}s${NC}"
        echo "${test_name},FAILED,${duration}" >> "${REPORT_DIR}/summary_${TIMESTAMP}.csv"
        return 1
    fi
}

# Initialize CSV summary file
echo "Test Name,Status,Duration (seconds)" > "${REPORT_DIR}/summary_${TIMESTAMP}.csv"

# Overall start time
OVERALL_START=$(date +%s)

# ========================================
# Day 1: Load Tests
# ========================================
print_section "Day 1: Load Tests"

echo -e "${YELLOW}Starting load tests...${NC}"
echo ""

# Test 1: Connection Pool Load Test
run_test "connection_pool" "connection_pool_test" || true

# Test 2: Cache Load Test
run_test "cache_load" "cache_load_test" || true

# Test 3: Batch Operations Load Test
run_test "batch_operations" "batch_operations_test" || true

# ========================================
# Day 2: Soak Tests
# ========================================
print_section "Day 2: Soak Tests"

echo -e "${YELLOW}Starting soak tests...${NC}"
echo ""

# Test 4: Rate Limiter Soak Test
run_test "rate_limiter" "rate_limiter_test" || true

# Test 5: 24-Hour Stability Test (skipped by default)
print_section "Full 24-Hour Stability Test"
echo ""
echo -e "${YELLOW}WARNING: The full 24-hour stability test is not run by default.${NC}"
echo -e "${YELLOW}To run it manually, execute:${NC}"
echo -e "    cargo test --test stability_test -- --ignored --features full-soak"
echo ""
echo "stability_test,SKIPPED,N/A" >> "${REPORT_DIR}/summary_${TIMESTAMP}.csv"

# ========================================
# Day 3: Analysis
# ========================================
print_section "Day 3: Analysis"

echo -e "${YELLOW}Running analysis...${NC}"
echo ""

# Memory Leak Analysis
echo "Running memory leak analysis..."
if command -valgrind &> /dev/null; then
    print_section "Valgrind Memory Leak Detection"
    valgrind --leak-check=full --show-leak-kinds=all \
        cargo test --test connection_pool_test -- \
        2>&1 | tee "${REPORT_DIR}/valgrind_${TIMESTAMP}.txt" || true
else
    echo -e "${YELLOW}Valgrind not installed, skipping detailed memory leak detection${NC}"
fi

# Performance Regression Detection
echo "Checking for performance regressions..."
# This would compare against baseline metrics stored in a reference file
# For now, just note that this needs to be implemented with baseline data
echo -e "${YELLOW}Performance regression detection requires baseline data (see performance_baseline.txt)${NC}"

# ========================================
# Generate Final Report
# ========================================
OVERALL_END=$(date +%s)
OVERALL_DURATION=$((OVERALL_END - OVERALL_START))

print_section "Test Summary Report"

echo ""
echo "Report Files:"
echo "  - Summary: ${REPORT_DIR}/summary_${TIMESTAMP}.csv"
echo "  - Detailed reports: ${REPORT_DIR}/test_${TIMESTAMP}_*.txt"
echo ""

# Display summary
echo "Test Results Summary:"
printf "%-30s %-10s %-15s\n" "Test Name" "Status" "Duration (s)"
printf "%-30s %-10s %-15s\n" "----------" "------" "--------------"
while IFS=',' read -r name status duration; do
    if [[ "$name" != "Test Name" ]]; then
        if [[ "$status" == "PASSED" ]]; then
            printf "%-30s ${GREEN}%-10s${NC} %-15s\n" "$name" "$status" "$duration"
        elif [[ "$status" == "FAILED" ]]; then
            printf "%-30s ${RED}%-10s${NC} %-15s\n" "$name" "$status" "$duration"
        else
            printf "%-30s ${YELLOW}%-10s${NC} %-15s\n" "$name" "$status" "$duration"
        fi
    fi
done < "${REPORT_DIR}/summary_${TIMESTAMP}.csv"

echo ""
echo "Total Duration: ${OVERALL_DURATION}s"

# Count results
PASSED=$(grep -c ",PASSED," "${REPORT_DIR}/summary_${TIMESTAMP}.csv" || true)
FAILED=$(grep -c ",FAILED," "${REPORT_DIR}/summary_${TIMESTAMP}.csv" || true)
SKIPPED=$(grep -c ",SKIPPED," "${REPORT_DIR}/summary_${TIMESTAMP}.csv" || true)

echo ""
echo "Results:"
echo "  Passed:  ${PASSED}"
echo "  Failed:  ${FAILED}"
echo "  Skipped: ${SKIPPED}"

if [[ $FAILED -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}All tests passed! ✅${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed! ❌${NC}"
    exit 1
fi
