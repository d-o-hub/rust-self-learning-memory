#!/bin/bash
# verify-crate-metadata.sh
# Verifies all required metadata before publishing to crates.io

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Crates to verify
CRATES=(
    "memory-core"
    "memory-storage-turso"
    "memory-storage-redb"
    "memory-mcp"
)

# Required fields
REQUIRED_FIELDS=(
    "name"
    "version"
    "description"
    "license"
    "repository"
    "documentation"
    "readme"
)

echo "Verifying crate metadata for publishing..."
echo "=========================================="

errors=0

for crate in "${CRATES[@]}"; do
    echo ""
    echo "Checking $crate..."

    metadata=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | jq -r ".packages[] | select(.name == \"$crate\")" 2>/dev/null || echo "")

    if [ -z "$metadata" ]; then
        echo -e "${RED}ERROR: Could not find crate $crate${NC}"
        ((errors++))
        continue
    fi

    for field in "${REQUIRED_FIELDS[@]}"; do
        value=$(echo "$metadata" | jq -r ".$field" 2>/dev/null || echo "null")

        if [ "$value" == "null" ] || [ -z "$value" ]; then
            echo -e "${RED}  ✗ Missing: $field${NC}"
            ((errors++))
        else
            echo -e "${GREEN}  ✓ $field: $value${NC}"
        fi
    done

    # Check keywords count
    keywords_count=$(echo "$metadata" | jq -r '.keywords | length' 2>/dev/null || echo "0")
    if [ "$keywords_count" -lt 3 ]; then
        echo -e "${YELLOW}  ⚠ Only $keywords_count keywords (recommended: ≥3)${NC}"
    else
        echo -e "${GREEN}  ✓ keywords: $keywords_count${NC}"
    fi

    # Check categories count
    categories_count=$(echo "$metadata" | jq -r '.categories | length' 2>/dev/null || echo "0")
    if [ "$categories_count" -lt 1 ]; then
        echo -e "${YELLOW}  ⚠ No categories specified (recommended: ≥1)${NC}"
    else
        echo -e "${GREEN}  ✓ categories: $categories_count${NC}"
    fi
done

echo ""
echo "=========================================="
if [ $errors -gt 0 ]; then
    echo -e "${RED}FAILED: $errors error(s) found${NC}"
    exit 1
else
    echo -e "${GREEN}SUCCESS: All crates have required metadata${NC}"
    exit 0
fi