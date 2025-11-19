#!/usr/bin/env bash
# Cross-platform post-bash hook for detecting git commits
# Works on both Linux and Windows (MINGW64/Git Bash)

# Exit 0 by default (non-blocking)
set +e

# Try to extract command from CLAUDE_TOOL_INPUT
# Use multiple methods for cross-platform compatibility
BASH_CMD=""

# Method 1: Try jq if available
if command -v jq &> /dev/null && [ -n "$CLAUDE_TOOL_INPUT" ]; then
    BASH_CMD=$(echo "$CLAUDE_TOOL_INPUT" | jq -r '.command // empty' 2>/dev/null)
fi

# Method 2: If jq failed or not available, try simple grep
if [ -z "$BASH_CMD" ] && [ -n "$CLAUDE_TOOL_INPUT" ]; then
    if echo "$CLAUDE_TOOL_INPUT" | grep -q '"command"'; then
        BASH_CMD=$(echo "$CLAUDE_TOOL_INPUT" | grep -o '"command":"[^"]*"' | sed 's/"command":"//;s/"$//')
    fi
fi

# Check if this is a git commit command
if [ -n "$BASH_CMD" ] && echo "$BASH_CMD" | grep -q 'git commit'; then
    echo "🔒 Running post-commit security checks..."

    # Run the security check script
    if [ -f ".claude/hooks/pre-commit-security.sh" ]; then
        bash .claude/hooks/pre-commit-security.sh 2>&1 || {
            echo "⚠️  Security check warnings (non-blocking)"
        }
    else
        echo "⚠️  Security check script not found"
    fi
fi

# Always exit 0 (non-blocking)
exit 0
