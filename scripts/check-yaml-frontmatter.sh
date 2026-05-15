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
        # Check if line has a key: value pattern in YAML frontmatter
        if [[ "$line" =~ ^[a-zA-Z_-]+: ]]; then
            local key="${line%%:*}"
            local value="${line#*: }"
            
            # Skip blank values, numeric values, or list values
            [[ -z "$value" ]] && continue
            [[ "${value:0:1}" == "[" ]] && continue
            [[ "$value" =~ ^[0-9.]+$ ]] && continue
            
            local first_char="${value:0:1}"
            local last_char="${value: -1}"
            
            if [[ "$first_char" == '"' && "$last_char" == '"' ]]; then
                continue
            fi
            if [[ "$first_char" == "'" && "$last_char" == "'" ]]; then
                continue
            fi
            
            # Check if the unquoted value contains colon-space (risks YAML parsing)
            if echo "$value" | grep -q ': '; then
                if [[ "$key" == "allowed-tools" ]] && echo "$value" | grep -qE '\\([^)]*\\*:\\*[^)]*\\)'; then
                    echo "WARNING: $file: unquoted $key has fragile ':*' pattern"
                    echo "       $line"
                else
                    echo "ERROR: $file: unquoted $key has ': ' which can cause YAML parsing errors"
                    echo "       Fix: $key: \"$value\""
                    ERRORS=$((ERRORS + 1))
                fi
            elif [[ "$value" == *: ]]; then
                # Trailing colon is also fragile in YAML
                echo "WARNING: $file: unquoted $key value ends with ':' - consider quoting"
                echo "       $line"
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
