#!/usr/bin/env bash
set -euo pipefail

echo "📝 Validating YAML files..."

# Extract file path from Claude tool input
FILE_PATH=$(echo "${CLAUDE_TOOL_INPUT:-}" | jq -r '.file_path // empty')

# Skip if not a YAML file
if [[ ! "$FILE_PATH" =~ \.(yml|yaml)$ ]]; then
    exit 0
fi

# Check if yamllint is installed
if ! command -v yamllint &> /dev/null; then
    echo "⚠️  yamllint not installed. Install with: pip install yamllint"
    echo "Skipping YAML validation for: $FILE_PATH"
    exit 0
fi

# Validate YAML syntax and style
if ! yamllint -d "{extends: default, rules: {line-length: {max: 120}, indentation: {spaces: 2}}}" "$FILE_PATH"; then
    echo "❌ YAML validation failed for: $FILE_PATH"
    echo "Common issues:"
    echo "  - Use 2 spaces for indentation (not tabs)"
    echo "  - Check for missing colons or improper list formatting"
    echo "  - Validate with: yamllint $FILE_PATH"
    exit 1
fi

echo "✓ YAML validation passed: $FILE_PATH"
exit 0
