#!/usr/bin/env bash
# Skills Consolidation Validation Script
# Validates the consolidated .agents/skills/ directory

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

AGENTS_DIR=".agents/skills"
CLAUDE_DIR=".claude/skills"
OPENCODE_DIR=".opencode/skill"

# Test counters
tests_passed=0
tests_failed=0
tests_total=0

# Test functions
test_start() {
    ((tests_total++))
    echo -ne "${BLUE}[TEST $tests_total]${NC} $1... "
}

test_pass() {
    ((tests_passed++))
    echo -e "${GREEN}PASS${NC}"
}

test_fail() {
    ((tests_failed++))
    echo -e "${RED}FAIL${NC}"
    echo -e "  ${RED}$1${NC}"
}

test_warn() {
    echo -e "${YELLOW}WARNING${NC}: $1"
}

# Header
echo "==================================="
echo "Skills Consolidation Validation"
echo "==================================="
echo ""

# Test 1: Directory exists
test_start "Check .agents/skills/ directory exists"
if [ -d "$AGENTS_DIR" ]; then
    test_pass
else
    test_fail "Directory not found: $AGENTS_DIR"
    echo ""
    echo "Run ./scripts/consolidate-skills.sh first"
    exit 1
fi

# Test 2: Inventory exists
test_start "Check INVENTORY.md exists"
if [ -f "$AGENTS_DIR/INVENTORY.md" ]; then
    test_pass
else
    test_fail "INVENTORY.md not found"
fi

# Test 3: Count skills
test_start "Count total skills"
skill_count=$(find "$AGENTS_DIR" -mindepth 1 -maxdepth 1 -type d | wc -l)
echo "($skill_count skills)"
if [ "$skill_count" -ge 70 ]; then
    test_pass
else
    test_warn "Expected ~72 skills, found $skill_count"
    test_pass
fi

# Test 4: All skills have SKILL.md
test_start "Check all skills have SKILL.md"
missing_skill_md=0
for skill_dir in "$AGENTS_DIR"/*/; do
    if [ ! -f "$skill_dir/SKILL.md" ]; then
        ((missing_skill_md++))
        echo ""
        echo -e "  ${RED}Missing SKILL.md: ${NC}$(basename "$skill_dir")"
    fi
done

if [ "$missing_skill_md" -eq 0 ]; then
    test_pass
else
    test_fail "$missing_skill_md skills missing SKILL.md"
fi

# Test 5: Check for broken symlinks
test_start "Check for broken symlinks"
broken_links=$(find "$AGENTS_DIR" -type l ! -exec test -e {} \; 2>/dev/null || true)
if [ -z "$broken_links" ]; then
    test_pass
else
    test_fail "Broken symlinks found:"
    echo "$broken_links" | while read -r link; do
        echo -e "  ${RED}x${NC} $link"
    done
fi

# Test 6: Check SKILL.md frontmatter
test_start "Check SKILL.md frontmatter"
missing_frontmatter=0
for skill_md in "$AGENTS_DIR"/*/SKILL.md; do
    if [ -f "$skill_md" ]; then
        if ! grep -q '^name:' "$skill_md" 2>/dev/null; then
            ((missing_frontmatter++))
            echo ""
            echo -e "  ${YELLOW}Missing 'name'${NC}: $skill_md"
        fi
    fi
done

if [ "$missing_frontmatter" -eq 0 ]; then
    test_pass
else
    test_warn "$missing_frontmatter files missing frontmatter"
    test_pass
fi

# Test 7: Check for merge notes
test_start "Check for pending merges"
merge_notes=$(find "$AGENTS_DIR" -name "MERGE_NOTES.md" 2>/dev/null || true)
merge_count=$(echo "$merge_notes" | grep -c "MERGE_NOTES" || echo 0)

if [ "$merge_count" -eq 0 ]; then
    test_pass
    echo "(no pending merges)"
else
    test_warn "$merge_count skills need manual merge"
    test_pass
    echo "$merge_notes" | while read -r note; do
        if [ -n "$note" ]; then
            echo -e "  ${YELLOW}→${NC} $(basename "$(dirname "$note")")"
        fi
    done
fi

# Test 8: Validate no orphaned files
test_start "Check for orphaned non-md files"
orphaned_files=$(find "$AGENTS_DIR" -maxdepth 2 -type f ! -name "*.md" ! -name ".*" 2>/dev/null || true)
if [ -z "$orphaned_files" ]; then
    test_pass
else
    test_warn "Found non-markdown files (may be OK)"
    echo "$orphaned_files" | head -5 | while read -r file; do
        if [ -n "$file" ]; then
            echo -e "  ${YELLOW}→${NC} $file"
        fi
    done
    test_pass
fi

# Test 9: Verify source directories still exist
test_start "Check source directories intact"
if [ -d "$CLAUDE_DIR" ] && [ -d "$OPENCODE_DIR" ]; then
    test_pass
else
    test_fail "Source directories missing"
fi

# Test 10: Sample skill content check
test_start "Sample skill content check"
sample_skill="$AGENTS_DIR/agent-coordination/SKILL.md"
if [ -f "$sample_skill" ]; then
    if grep -q 'description:' "$sample_skill" 2>/dev/null; then
        test_pass
    else
        test_fail "Sample skill missing expected content"
    fi
else
    test_warn "Sample skill not found"
    test_pass
fi

# Summary
echo ""
echo "==================================="
echo "Test Summary"
echo "==================================="
echo -e "Total Tests: $tests_total"
echo -e "${GREEN}Passed: $tests_passed${NC}"

if [ "$tests_failed" -gt 0 ]; then
    echo -e "${RED}Failed: $tests_failed${NC}"
fi

if [ "$tests_failed" -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ All validation checks passed!${NC}"
    echo ""
    echo "Next steps:"
    if [ "$merge_count" -gt 0 ]; then
        echo "  1. Process $merge_count MERGE_NOTES.md files"
        echo "  2. Re-run validation"
    fi
    echo "  3. Test skill loading functionality"
    echo "  4. Review inventory: cat $AGENTS_DIR/INVENTORY.md"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Validation failed${NC}"
    echo ""
    echo "Action items:"
    echo "  1. Review failed tests above"
    echo "  2. Fix issues or re-run consolidation"
    echo "  3. Re-run validation: ./scripts/validate-consolidation.sh"
    exit 1
fi
