#!/bin/bash
# Fix File Locations - Auto-moves misplaced markdown files to plans/
#
# Usage:
#   ./scripts/fix-file-locations.sh [--dry-run]
#
# Permanent docs that stay in root:
#   README.md, AGENTS.md, CONTRIBUTING.md, SECURITY.md, DEPLOYMENT.md, TESTING.md
#   LICENSE, LICENSE-*

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DRY_RUN=false
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --help)
      echo "Usage: $0 [--dry-run]"
      echo ""
      echo "Auto-moves misplaced markdown files from root to appropriate plans/ subdirectories."
      echo ""
      echo "Options:"
      echo "  --dry-run    Show what would be moved without actually moving"
      echo "  --help       Show this help message"
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

# Permanent docs that should stay in root (from AGENTS.md line 28)
PERMANENT_DOCS=(
  "README.md"
  "AGENTS.md"
  "CONTRIBUTING.md"
  "SECURITY.md"
  "DEPLOYMENT.md"
  "TESTING.md"
  "LICENSE"
)

# Check if file is permanent doc
is_permanent_doc() {
  local file="$1"
  for perm in "${PERMANENT_DOCS[@]}"; do
    if [[ "$(basename "$file")" == "$perm" || "$(basename "$file")" == $perm-* ]]; then
      return 0
    fi
  done
  return 1
}

# Determine target directory based on file name
determine_target_dir() {
  local file="$1"
  local filename=$(basename "$file")

  case "$filename" in
    *analysis*|*dependency*|*security*|*audit*)
      echo "plans/STATUS"
      ;;
    *report*|*quality*|*gap*|*validation*|*review*)
      echo "plans/STATUS"
      ;;
    *plan*|*roadmap*|*strategy*)
      echo "plans/ROADMAPS"
      ;;
    *architecture*|*design*|*pattern*)
      echo "plans/ARCHITECTURE"
      ;;
    *config*|*configuration*)
      echo "plans/CONFIGURATION"
      ;;
    *research*|*benchmark*|*performance*)
      echo "plans/research"
      ;;
    *cleanup*|*refactor*|*fix*)
      echo "plans/archive/$(date +%Y-%m)-completed"
      ;;
    *)
      echo "plans/STATUS"
      ;;
  esac
}

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              File Location Fixer                               ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
  echo -e "${YELLOW}DRY RUN MODE - No files will be moved${NC}"
  echo ""
fi

# Find all .md files in root (excluding subdirectories)
MISPLACED_FILES=()
while IFS= read -r -d '' file; do
  if ! is_permanent_doc "$file"; then
    MISPLACED_FILES+=("$file")
  fi
done < <(find . -maxdepth 1 -name "*.md" -print0)

if [ ${#MISPLACED_FILES[@]} -eq 0 ]; then
  echo -e "${GREEN}✓${NC} No misplaced markdown files found in root"
  exit 0
fi

echo -e "${YELLOW}Found ${#MISPLACED_FILES[@]} misplaced markdown file(s) in root:${NC}"
echo ""

# Display what will be moved
for i in "${!MISPLACED_FILES[@]}"; do
  file="${MISPLACED_FILES[$i]}"
  target_dir=$(determine_target_dir "$file")
  target_path="$target_dir/$(basename "$file")"

  echo -e "${BLUE}[$((i+1))/${#MISPLACED_FILES[@]}]${NC} $(basename "$file")"
  echo "    Source: $file"
  echo "    Target: $target_path"
  echo ""
done

if [ "$DRY_RUN" = true ]; then
  echo -e "${YELLOW}Dry run complete. No files were moved.${NC}"
  echo "Run without --dry-run to apply these changes."
  exit 0
fi

# Ask for confirmation
echo -n "Move these files? [y/N] "
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
  echo "Cancelled. No files were moved."
  exit 0
fi

echo ""
echo -e "${BLUE}Moving files...${NC}"
echo ""

# Create target directories and move files
SUCCESS=0
FAILED=0
for file in "${MISPLACED_FILES[@]}"; do
  target_dir=$(determine_target_dir "$file")
  target_path="$target_dir/$(basename "$file")"

  # Create target directory if needed
  if [ ! -d "$target_dir" ]; then
    mkdir -p "$target_dir"
  fi

  # Check if target already exists
  if [ -e "$target_path" ]; then
    echo -e "${YELLOW}⚠${NC} Skipping $(basename "$file") - target already exists"
    ((FAILED++))
    continue
  fi

  # Move file
  if mv "$file" "$target_path"; then
    echo -e "${GREEN}✓${NC} Moved $(basename "$file") → $target_dir/"
    ((SUCCESS++))
  else
    echo -e "${RED}✗${NC} Failed to move $(basename "$file")"
    ((FAILED++))
  fi
done

echo ""
echo -e "${BLUE}Summary:${NC}"
echo -e "  ${GREEN}✓${NC} Successfully moved: $SUCCESS"
echo -e "  ${RED}✗${NC} Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
  echo ""
  echo -e "${GREEN}────────────────────────────────────────────────────────────────────────${NC}"
  echo -e "${GREEN}│          ✓ All files moved successfully                     │${NC}"
  echo -e "${GREEN}────────────────────────────────────────────────────────────────────────${NC}"
  exit 0
else
  exit 1
fi
