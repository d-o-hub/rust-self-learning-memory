#!/usr/bin/env bash
# validate-yaml-frontmatter: Check that YAML frontmatter in SKILL.md and agent .md files
# has properly quoted description values to prevent YAML parsing errors.
# Also checks for fragile colon patterns in other YAML fields like allowed-tools.
set -euo pipefail

ERRORS=0

check_file() {
    local file="$1"
    local in_frontmatter=0
    
    while IFS= read -r line; do
        # Track YAML frontmatter boundaries
        if [[ "$line" == "---" ]]; then
            if [[ $in_frontmatter -eq 0 ]]; then
                in_frontmatter=1
            else
                break  # End of frontmatter
            fi
            continue
        fi
        
        [[ $in_frontmatter -eq 0 ]] && continue
        
        # Check description field for unquoted colon-space
        if [[ "$line" =~ ^description: ]]; then
            local prefix="description: "
            local value="${line#$prefix}"
            local first_char="${value:0:1}"
            if [[ "$first_char" != '"' && "$first_char" != "'" ]]; then
                if echo "$value" | grep -q ': '; then
                    echo "ERROR: $file: unquoted description contains ': ' which can cause YAML parsing errors"
                    echo "       Fix: wrap the description value in quotes:\n       description: \"your text here\""
                    ERRORS=$((ERRORS + 1))
                fi
            fi
            continue
        fi
        
        # Check allowed-tools field for fragile colon patterns like Bash(gh *:*)
        if [[ "$line" =~ ^allowed-tools: ]]; then
            local at_value="${line#allowed-tools: }"
            # Check for colons inside parentheses that are NOT followed by a space
            # Pattern: something(*:*) where colon is not part of a space-separated list
            if echo "$at_value" | grep -qP '\([^)]*\*:\*[^)]*\)'; then
                echo "WARNING: $file: allowed-tools contains ':*' pattern that may break if spaces are added"
                echo "       File: $line"
                # Warn only, don't error - these are valid but fragile
            fi
            continue
        fi
    done < "$file"
}

# Check all SKILL.md files
while IFS= read -r -d '' file; do
    check_file "$file"
done < <(find .agents/skills -name 'SKILL.md' -type f -print0 2>/dev/null)

# Check all .claude/agents .md files
while IFS= read -r -d '' file; do
    check_file "$file"
done < <(find .claude/agents -name '*.md' -type f -print0 2>/dev/null)

if [ "$ERRORS" -gt 0 ]; then
    echo "Found $ERRORS file(s) with invalid YAML frontmatter."
    exit 1
else
    echo "All YAML frontmatter validated successfully."
fi
