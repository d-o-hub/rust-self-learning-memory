#!/bin/bash
# Skill activation script based on context

set -e

# Read last user message from stdin
MESSAGE=$(cat)

# Check for skill triggers (all consolidated into test-runner)
if echo "$MESSAGE" | grep -qiE '(async|tokio|test|episode|pattern|memory|nextest|benchmark|proptest|optimize)'; then
    echo "Activating: test-runner skill (consolidated testing patterns)"
fi
