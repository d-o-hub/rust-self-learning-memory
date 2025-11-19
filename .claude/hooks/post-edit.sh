#!/usr/bin/env bash
# Cross-platform post-edit hook for formatting, linting, and testing
# Works on both Linux and Windows (MINGW64/Git Bash)

# Exit 0 by default (non-blocking)
set +e

# Try to extract file_path from CLAUDE_TOOL_INPUT
FILE_PATH=""

# Method 1: Try jq if available
if command -v jq &> /dev/null && [ -n "$CLAUDE_TOOL_INPUT" ]; then
    FILE_PATH=$(echo "$CLAUDE_TOOL_INPUT" | jq -r '.file_path // empty' 2>/dev/null)
fi

# Method 2: If jq failed or not available, try simple grep
if [ -z "$FILE_PATH" ] && [ -n "$CLAUDE_TOOL_INPUT" ]; then
    if echo "$CLAUDE_TOOL_INPUT" | grep -q '"file_path"'; then
        FILE_PATH=$(echo "$CLAUDE_TOOL_INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"$//')
    fi
fi

# Only process if we have a file path and it's a Rust file
if [ -n "$FILE_PATH" ] && echo "$FILE_PATH" | grep -q '\.rs$'; then
    # Check if file exists
    if [ -f "$FILE_PATH" ]; then
        echo "📝 Formatting Rust file: $FILE_PATH"
        cargo fmt -- "$FILE_PATH" 2>&1 || true
    fi
fi

# Always exit 0 (non-blocking)
exit 0
