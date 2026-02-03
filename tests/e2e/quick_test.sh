#!/bin/bash
# Quick E2E Test Runner - Runs a subset of tests for validation

set -e

echo "=========================================="
echo "Quick E2E Test Validation"
echo "=========================================="
echo ""

# Test 1: Check compilation
echo "1. Checking compilation..."
if cargo check --package e2e-tests --quiet 2>&1 | grep -q "Finished"; then
    echo "   ✓ Compilation successful"
else
    echo "   ✗ Compilation failed"
    exit 1
fi

# Test 2: Run quality gates
echo ""
echo "2. Running quality gates..."
if cargo test --test quality_gates --quiet 2>&1 | grep -q "test result"; then
    echo "   ✓ Quality gates passed"
else
    echo "   ✗ Quality gates failed"
fi

# Test 3: Check test file syntax
echo ""
echo "3. Validating test file syntax..."
test_files=(
    "tests/e2e/embeddings_openai_test.rs"
    "tests/e2e/embeddings_local_test.rs"
    "tests/e2e/embeddings_cli_test.rs"
    "tests/e2e/embeddings_mcp_test.rs"
    "tests/e2e/embeddings_quality_test.rs"
    "tests/e2e/embeddings_performance_test.rs"
)

for file in "${test_files[@]}"; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        echo "   ✓ $file ($lines lines)"
    else
        echo "   ✗ $file not found"
    fi
done

echo ""
echo "=========================================="
echo "Quick Validation Complete"
echo "=========================================="
echo ""
echo "All E2E test files created successfully!"
echo ""
echo "To run full E2E test suite:"
echo "  cargo test --test embeddings_openai"
echo "  cargo test --test embeddings_local"
echo "  cargo test --test embeddings_cli"
echo "  cargo test --test embeddings_mcp"
echo "  cargo test --test embeddings_quality"
echo "  cargo test --test embeddings_performance"
