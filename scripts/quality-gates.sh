#!/bin/bash
# Quality Gates Runner
#
# Run quality gates locally before committing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Quality Gates - Local Runner                        ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuration
COVERAGE_THRESHOLD=${QUALITY_GATE_COVERAGE_THRESHOLD:-90}
PATTERN_THRESHOLD=${QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD:-70}
COMPLEXITY_THRESHOLD=${QUALITY_GATE_COMPLEXITY_THRESHOLD:-10}
SECURITY_THRESHOLD=${QUALITY_GATE_SECURITY_THRESHOLD:-0}
SKIP_OPTIONAL=${QUALITY_GATE_SKIP_OPTIONAL:-false}

echo -e "${BLUE}Configuration:${NC}"
echo "  Coverage Threshold: ${COVERAGE_THRESHOLD}%"
echo "  Pattern Accuracy Threshold: ${PATTERN_THRESHOLD}%"
echo "  Complexity Threshold: ${COMPLEXITY_THRESHOLD}"
echo "  Security Vuln Threshold: ${SECURITY_THRESHOLD}"
echo "  Skip Optional: ${SKIP_OPTIONAL}"
echo ""

# Check required tools
echo -e "${BLUE}Checking required tools...${NC}"
MISSING_TOOLS=()

if ! command -v cargo &> /dev/null; then
    MISSING_TOOLS+=("cargo")
fi

if ! cargo llvm-cov --version &> /dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠${NC}  cargo-llvm-cov not installed"
    if [ "$SKIP_OPTIONAL" != "true" ]; then
        MISSING_TOOLS+=("cargo-llvm-cov")
    fi
fi

if ! cargo audit --version &> /dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠${NC}  cargo-audit not installed"
    if [ "$SKIP_OPTIONAL" != "true" ]; then
        MISSING_TOOLS+=("cargo-audit")
    fi
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo -e "${RED}Missing required tools: ${MISSING_TOOLS[*]}${NC}"
    echo ""
    echo "Install with:"
    for tool in "${MISSING_TOOLS[@]}"; do
        if [ "$tool" = "cargo-llvm-cov" ]; then
            echo "  cargo install cargo-llvm-cov"
        elif [ "$tool" = "cargo-audit" ]; then
            echo "  cargo install cargo-audit"
        fi
    done
    echo ""
    echo "Or set QUALITY_GATE_SKIP_OPTIONAL=true to skip optional tools"
    exit 1
fi

echo -e "${GREEN}✓${NC} All required tools available"
echo ""

# Run quality gates
echo -e "${BLUE}Running quality gates...${NC}"
echo ""

export QUALITY_GATE_COVERAGE_THRESHOLD=$COVERAGE_THRESHOLD
export QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD=$PATTERN_THRESHOLD
export QUALITY_GATE_COMPLEXITY_THRESHOLD=$COMPLEXITY_THRESHOLD
export QUALITY_GATE_SECURITY_THRESHOLD=$SECURITY_THRESHOLD
export QUALITY_GATE_SKIP_OPTIONAL=$SKIP_OPTIONAL

if cargo test --test quality_gates -- --nocapture; then
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          ✓ All Quality Gates PASSED                          ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║          ✗ Quality Gates FAILED                               ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Review the output above to identify which gates failed."
    echo "See docs/QUALITY_GATES.md for troubleshooting guidance."
    exit 1
fi
