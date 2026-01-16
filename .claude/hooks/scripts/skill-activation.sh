#!/bin/bash
# Skill activation script based on context

set -e

# Read last user message from stdin
MESSAGE=$(cat)

# Check for skill triggers
if echo "$MESSAGE" | grep -qiE '(async|tokio|test.*await)'; then
    echo "Activating: rust-async-testing skill"
elif echo "$MESSAGE" | grep -qiE '(episode|pattern|memory|extraction)'; then
    echo "Activating: episodic-memory-testing skill"
elif echo "$MESSAGE" | grep -qiE '(nextest|benchmark|proptest|optimize|performance)'; then
    echo "Activating: test-optimization skill"
fi
