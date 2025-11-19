#!/usr/bin/env bash
set -euo pipefail

# Extract file path from Claude tool input
FILE_PATH=$(echo "${CLAUDE_TOOL_INPUT:-}" | jq -r '.file_path // empty')

if [[ -z "$FILE_PATH" ]]; then
    exit 0
fi

# Block editing of sensitive files
if [[ "$FILE_PATH" =~ \.(env|secret|key)$ ]] || [[ "$FILE_PATH" == *".turso"* ]]; then
    echo "🚫 BLOCKED: Cannot edit sensitive file: $FILE_PATH"
    echo "Reason: Zero-Trust policy prohibits LLM access to credential files"
    exit 1
fi

# Check for hardcoded secrets in new content
if echo "${CLAUDE_TOOL_INPUT:-}" | jq -r '.content // empty' | grep -qiE "(api[_-]?key|password|secret|token|credential)[\"']?\s*[:=]"; then
    echo "🚫 BLOCKED: Potential hardcoded secret detected"
    echo "Use environment variables instead"
    exit 1
fi

echo "✓ File access validated"
exit 0
