#!/usr/bin/env bash
# scripts/setup-jules.sh
# Optimized setup for Jules environment to bypass network timeouts and sync issues.

set -euo pipefail

# Colors for output
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

echo -e "${GREEN}=== Configuring Jules Environment for Rust Self-Learning Memory ===${NC}"

# 1. Bypass rustup shims to avoid 'channel-rust-stable.toml.sha256' timeouts
# Every Jules VM has the toolchain pre-installed at this location
export STABLE_TOOLCHAIN_BIN="/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin"

if [ -d "$STABLE_TOOLCHAIN_BIN" ]; then
    echo -e "${GREEN}✓${NC} Found stable toolchain at $STABLE_TOOLCHAIN_BIN"
    export PATH="$STABLE_TOOLCHAIN_BIN:$PATH"
    export RUSTUP_TOOLCHAIN=stable
else
    echo -e "${YELLOW}⚠${NC} Warning: Stable toolchain bin not found at expected location."
fi

# 2. Configure Cargo to be more resilient
export CARGO_NET_RETRY=5
export CARGO_HTTP_TIMEOUT=60

# 3. Initialize local storage for development if needed
mkdir -p ./data
if [ ! -f "./data/memory.db" ]; then
    echo "Initializing empty local database..."
    touch ./data/memory.db
fi

# 4. Verify tools
echo -e "\n${GREEN}=== Environment Verification ===${NC}"
echo "rustc: $(rustc --version)"
echo "cargo: $(cargo --version)"
echo "rustfmt: $(rustfmt --version || echo 'not installed')"

echo -e "\n${GREEN}=== Setup Complete ===${NC}"
echo "You can now run quality checks using: ./scripts/code-quality.sh check"
echo "Or run tests directly: cargo test -p do-memory-core"
