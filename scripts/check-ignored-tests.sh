#!/bin/bash
# Ignored-test ceiling guard
#
# Counts #[ignore] annotations in the codebase and fails if count exceeds threshold.
# This prevents silent growth of ignored tests (ADR-041, ACT-025, WG-026).
#
# Exit codes:
#   0 - Count is at or below ceiling
#   1 - Count exceeds ceiling (prevent merging)
#
# See: plans/adr/ADR-041-Test-Health-Remediation-v0.1.20.md
# See: plans/adr/ADR-027-Ignored-Tests-Strategy.md

set -e

# Configuration
# Ceiling: Maximum allowed ignored tests
# Baseline: 119 (pre-coverage-sprint)
# Coverage sprint added 24 ignored tests (ADR-027: libsql memory corruption bug in CI)
# New baseline: 143, ceiling with buffer: 150
IGNORED_TEST_CEILING=150

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Ignored-Test Ceiling Guard                       ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Count #[ignore] annotations (excluding target/ directory)
# Note: We use '#\[ignore' to catch both #[ignore] and #[ignore = "reason"]
IGNORED_COUNT=$(grep -r '#\[ignore' --include='*.rs' | grep -v target/ | wc -l | tr -d ' ')

echo -e "${BLUE}├─ Ignored Test Statistics${NC}"
echo -e "  Current count: ${IGNORED_COUNT}"
echo -e "  Ceiling limit: ${IGNORED_TEST_CEILING}"
echo ""

# Check against ceiling
if [ "$IGNORED_COUNT" -gt "$IGNORED_TEST_CEILING" ]; then
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  FAILED: Ignored test count exceeds ceiling!                  ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}The ignored test count (${IGNORED_COUNT}) exceeds the ceiling (${IGNORED_TEST_CEILING}).${NC}"
    echo ""
    echo "This guard prevents silent accumulation of ignored tests."
    echo ""
    echo "To fix this:"
    echo "  1. Review newly added #[ignore] annotations"
    echo "  2. Either fix the underlying test issue, or"
    echo "  3. Document why the ignore is legitimate in ADR-027"
    echo "  4. If legitimate, update IGNORED_TEST_CEILING in this script"
    echo ""
    echo "See: plans/adr/ADR-041-Test-Health-Remediation-v0.1.20.md"
    echo "See: plans/adr/ADR-027-Ignored-Tests-Strategy.md"
    exit 1
fi

# Success case
if [ "$IGNORED_COUNT" -eq "$IGNORED_TEST_CEILING" ]; then
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  WARNING: At ceiling limit                                   ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Ignored test count is at the ceiling (${IGNORED_TEST_CEILING}).${NC}"
    echo "Consider reducing ignored tests before adding new ones."
else
    REMAINING=$((IGNORED_TEST_CEILING - IGNORED_COUNT))
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  PASSED: Ignored test count within limits                    ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Headroom: ${REMAINING} tests remaining before ceiling"
fi

exit 0