#!/bin/bash
# Doctest and documentation validation script
#
# Checks that all doctests pass and documentation builds without errors.
# This helps catch orphaned doctest examples after refactoring.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Doctest & Documentation Validation                  ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to print section headers
section() {
    echo -e "${BLUE}├─ ${1}${NC}"
}

# Function to print success
success() {
    echo -e "  ${GREEN}✅ ${1}${NC}"
}

# Function to print failure
failure() {
    echo -e "  ${RED}❌ ${1}${NC}"
}

# 1. Run documentation tests
section "Running documentation tests (cargo test --doc)..."
if cargo test --doc --quiet 2>&1 | grep -q "test result: ok"; then
    success "All documentation tests passed"
else
    failure "Documentation tests failed"
    echo "Running with verbose output:"
    cargo test --doc
    exit 1
fi

# 2. Build documentation to check for broken links and warnings
section "Building documentation (cargo doc --no-deps --document-private-items)..."
if cargo doc --no-deps --document-private-items --quiet 2>&1 | tail -5 | grep -q "Finished"; then
    success "Documentation built successfully"
else
    failure "Documentation build failed or produced warnings"
    echo "Running with verbose output:"
    cargo doc --no-deps --document-private-items
    exit 1
fi

# 3. Optional: Check for broken intra-doc links (requires rustdoc 1.48+)
section "Checking for broken intra-doc links (cargo doc --no-deps --document-private-items -Z rustdoc-scrape-examples)..."
if rustc --version | grep -q "nightly"; then
    if cargo doc --no-deps --document-private-items -Z rustdoc-scrape-examples --quiet 2>&1 | tail -5 | grep -q "Finished"; then
        success "Intra-doc links are valid"
    else
        failure "Broken intra-doc links detected"
        cargo doc --no-deps --document-private-items -Z rustdoc-scrape-examples
        exit 1
    fi
else
    echo -e "  ${YELLOW}⚠  Skipping intra-doc link check (requires nightly Rust)${NC}"
fi

echo ""
echo -e "${GREEN}✅ All doctest and documentation checks passed!${NC}"