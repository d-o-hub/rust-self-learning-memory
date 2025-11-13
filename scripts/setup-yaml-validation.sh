#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Setting up YAML validation tools..."

# Detect OS
OS="$(uname -s)"

# Install yamllint
if command -v yamllint &> /dev/null; then
    echo "✓ yamllint already installed: $(yamllint --version)"
else
    echo "📦 Installing yamllint..."
    if command -v pip3 &> /dev/null; then
        pip3 install --user yamllint
        echo "✓ yamllint installed via pip3"
    elif command -v pip &> /dev/null; then
        pip install --user yamllint
        echo "✓ yamllint installed via pip"
    elif [ "$OS" = "Darwin" ] && command -v brew &> /dev/null; then
        brew install yamllint
        echo "✓ yamllint installed via homebrew"
    else
        echo "❌ Could not install yamllint. Please install pip or homebrew first."
        exit 1
    fi
fi

# Install actionlint
if command -v actionlint &> /dev/null; then
    echo "✓ actionlint already installed: $(actionlint --version)"
else
    echo "📦 Installing actionlint..."
    if [ "$OS" = "Darwin" ] && command -v brew &> /dev/null; then
        brew install actionlint
        echo "✓ actionlint installed via homebrew"
    elif command -v go &> /dev/null; then
        go install github.com/rhysd/actionlint/cmd/actionlint@latest
        echo "✓ actionlint installed via go"
    else
        echo "⚠️  Could not install actionlint. Install manually from:"
        echo "   https://github.com/rhysd/actionlint/releases"
    fi
fi

# Verify installation
echo ""
echo "🔍 Verifying installation..."
if command -v yamllint &> /dev/null; then
    echo "✓ yamllint: $(yamllint --version)"
else
    echo "❌ yamllint not found in PATH"
fi

if command -v actionlint &> /dev/null; then
    echo "✓ actionlint: $(actionlint --version)"
else
    echo "⚠️  actionlint not found in PATH (optional)"
fi

# Test yamllint with project config
echo ""
echo "🧪 Testing YAML validation..."
if command -v yamllint &> /dev/null; then
    if yamllint .github/ .yamllint.yml 2>&1; then
        echo "✅ All YAML files passed validation!"
    else
        echo "⚠️  Some YAML files have issues. Run 'yamllint .github/' for details."
    fi
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Run 'yamllint .github/' to validate all workflows"
echo "2. Run 'actionlint' to validate GitHub Actions semantics"
echo "3. See docs/YAML_VALIDATION.md for usage guide"
