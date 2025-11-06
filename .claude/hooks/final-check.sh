#!/usr/bin/env bash
set -euo pipefail

echo "🏁 Running final session verification..."

# Check if any Rust files were modified
if git diff --name-only | grep -q '\.rs$'; then
    echo "📊 Verifying Rust code quality..."

    # Final build check
    if ! cargo build --all; then
        echo "❌ Build failed"
        exit 1
    fi

    # Final test check
    if ! cargo test --all --quiet; then
        echo "❌ Tests failed"
        exit 1
    fi
fi

# Check for uncommitted changes to Cargo.lock
if git diff --name-only | grep -q 'Cargo.lock'; then
    echo "📦 Cargo.lock was modified. Remember to commit it."
fi

echo "✅ Session verification complete"
exit 0
