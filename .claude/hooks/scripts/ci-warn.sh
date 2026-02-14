#!/bin/bash
# CI Warning Hook - Non-blocking CI checks
# Use this for PreCommit to warn without blocking

CLAUDE_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"

echo "[CI-Warn] Quick quality check..."

# Quick fmt check
if ! cargo fmt --all -- --check 2>/dev/null; then
    echo "[!] Formatting issues detected"
    echo "    Fix: cargo fmt --all"
fi

# Quick clippy summary
if command -v cargo-clippy &>/dev/null; then
    CLIPPY_OUTPUT=$(cargo clippy --all -- -D warnings 2>&1 | grep -c "warning:" || echo "0")
    if [ "$CLIPPY_OUTPUT" != "0" ]; then
        echo "[!] $CLIPPY_OUTPUT clippy warnings"
        echo "    Fix: cargo clippy --fix"
    fi
fi

echo "[CI-Warn] Run './.claude/commands/quality-gates' before merge"
