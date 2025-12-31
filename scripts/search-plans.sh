#!/bin/bash

# Plans Navigation Search Script
# Purpose: Search across plans/ folder with categorization
# Usage: ./scripts/search-plans.sh "keyword" [--active|--archive|--all]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default search options
SCOPE="--all"
CASE_SENSITIVE=false
SHOW_CONTEXT=2

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --active)
            SCOPE="--active"
            shift
            ;;
        --archive)
            SCOPE="--archive"
            shift
            ;;
        --case-sensitive)
            CASE_SENSITIVE=true
            shift
            ;;
        --no-context)
            SHOW_CONTEXT=0
            shift
            ;;
        --context)
            SHOW_CONTEXT="$2"
            shift 2
            ;;
        *)
            KEYWORD="$1"
            shift
            ;;
    esac
done

if [[ -z "$KEYWORD" ]]; then
    echo -e "${RED}Error: No search keyword provided${NC}"
    echo "Usage: ./scripts/search-plans.sh \"keyword\" [--active|--archive|--all] [--case-sensitive] [--context N]"
    exit 1
fi

# Determine search directories
if [[ "$SCOPE" == "--active" ]]; then
    SEARCH_DIRS="ARCHITECTURE CONFIGURATION GOAP ROADMAPS STATUS research benchmark_results *.md"
    SCOPE_NAME="Active Plans"
elif [[ "$SCOPE" == "--archive" ]]; then
    SEARCH_DIRS="archive"
    SCOPE_NAME="Archive"
else
    SEARCH_DIRS="."
    SCOPE_NAME="All Plans"
fi

# Build grep command
GREP_CMD="grep -rn"

if [[ "$CASE_SENSITIVE" == "false" ]]; then
    GREP_CMD+=" -i"
fi

if [[ "$SHOW_CONTEXT" -gt 0 ]]; then
    GREP_CMD+=" -C $SHOW_CONTEXT"
fi

GREP_CMD+=" --color=always \"$KEYWORD\" $SEARCH_DIRS 2>/dev/null || true"

echo -e "${BLUE}=== Searching in $SCOPE_NAME: \"$KEYWORD\" ===${NC}"
echo -e "${GREEN}Running: $GREP_CMD${NC}"
echo ""

# Execute search
cd /workspaces/feat-phase3/plans
eval $GREP_CMD

# Show summary
echo ""
echo -e "${BLUE}=== Search Tips ===${NC}"
echo "- Use --active to search only current planning documents"
echo "- Use --archive to search only archived documents"
echo "- Use --case-sensitive for exact matches"
echo "- Use --context N to show N lines of context (default: 2)"
echo ""
echo -e "${BLUE}=== Quick Links ===${NC}"
echo "- Navigation Guide: [PLANS_NAVIGATION_GUIDE.md](PLANS_NAVIGATION_GUIDE.md)"
echo "- Archive Index: [archive/ARCHIVE_INDEX.md](archive/ARCHIVE_INDEX.md)"
echo "- Active Roadmap: [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md)"
echo "- Project Status: [STATUS/PROJECT_STATUS_UNIFIED.md](STATUS/PROJECT_STATUS_UNIFIED.md)"
