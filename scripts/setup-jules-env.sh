#!/bin/bash
# scripts/setup-jules-env.sh
# Optimized setup for Jules VM to bypass network timeouts and sync issues.

set -e

echo "=== Configuring Jules Environment for Rust Self-Learning Memory ==="

# 1. Bypass rustup shims to avoid 'channel-rust-stable.toml.sha256' timeouts
export STABLE_TOOLCHAIN_BIN="/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin"

if [ -d "$STABLE_TOOLCHAIN_BIN" ]; then
    echo "Found stable toolchain at $STABLE_TOOLCHAIN_BIN"
    export PATH="$STABLE_TOOLCHAIN_BIN:$PATH"
    export RUSTUP_TOOLCHAIN=stable
else
    echo "Warning: Stable toolchain bin not found at expected location."
fi

# 2. Configure Cargo to be more resilient
export CARGO_NET_RETRY=5
export CARGO_HTTP_TIMEOUT=60

# 3. Set up aliases for common developer tasks
# Note: aliases won't persist in subshells but are useful for the user
alias build-check="./scripts/build-rust.sh check"
alias quality="./scripts/code-quality.sh fmt && ./scripts/code-quality.sh clippy"
alias run-tests="cargo test -p do-memory-core"

# 4. Initialize local storage for development
mkdir -p ./data
if [ ! -f "./data/memory.db" ]; then
    echo "Initializing empty local database..."
    touch ./data/memory.db
fi

echo "=== Setup Complete ==="
echo "Using rustc: $(rustc --version)"
echo "Using cargo: $(cargo --version)"
