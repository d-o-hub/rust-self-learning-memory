#!/usr/bin/env bash
set -euo pipefail

echo "🔒 Running Zero-Trust pre-commit security checks..."

# 1. Format check
echo "📝 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting issues found. Running auto-format..."
    cargo fmt --all
    echo "✓ Code formatted. Please review changes."
fi

# 2. Clippy lints
echo "🔍 Running Clippy lints..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues. Fix them before committing."
    exit 1
fi

# 3. Security audit (skip if cargo-audit not installed)
if command -v cargo-audit &> /dev/null; then
    echo "🛡️  Auditing dependencies..."
    if ! cargo audit; then
        echo "⚠️  Security vulnerabilities found in dependencies!"
        echo "Run 'cargo audit fix' or update manually"
        exit 1
    fi
else
    echo "⚠️  cargo-audit not installed. Run: cargo install cargo-audit --locked"
fi

# 4. Deny check (skip if cargo-deny not installed)
if command -v cargo-deny &> /dev/null; then
    echo "📋 Running cargo-deny checks..."
    if ! cargo deny check; then
        echo "❌ cargo-deny found policy violations"
        exit 1
    fi
else
    echo "⚠️  cargo-deny not installed. Run: cargo install cargo-deny --locked"
fi

# 5. Test execution
echo "🧪 Running tests..."
if ! cargo test --all; then
    echo "❌ Tests failed. Fix them before committing."
    exit 1
fi

# 6. Secret scanning
echo "🔐 Scanning for secrets..."
if git diff --cached --name-only | xargs grep -inE '(api[_-]?key|password|secret|token|credential)["\']?\s*[:=]' 2>/dev/null; then
    echo "❌ Potential secrets detected in staged files!"
    echo "Remove hardcoded secrets and use environment variables"
    exit 1
fi

echo "✅ All security checks passed!"
exit 0
