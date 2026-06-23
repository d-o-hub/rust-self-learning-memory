#!/bin/bash
# scripts/audit-msrv.sh
# Verifies that Cargo.toml, .clippy.toml, and rust-toolchain.toml agree on MSRV/Toolchain.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Auditing MSRV and Toolchain consistency..."
echo "=========================================="

# Extract versions
CARGO_MSRV=$(grep "rust-version" Cargo.toml | head -n 1 | cut -d'"' -f2 || echo "NOT FOUND")
CLIPPY_MSRV=$(grep "msrv" .clippy.toml | head -n 1 | cut -d'"' -f2 || echo "NOT FOUND")
TOOLCHAIN_VER=$(grep "channel" rust-toolchain.toml | head -n 1 | cut -d'"' -f2 || echo "NOT FOUND")

echo "Cargo.toml rust-version:    $CARGO_MSRV"
echo ".clippy.toml msrv:          $CLIPPY_MSRV"
echo "rust-toolchain.toml channel: $TOOLCHAIN_VER"

errors=0

if [ "$CARGO_MSRV" != "$TOOLCHAIN_VER" ]; then
    echo -e "${RED}✗ Mismatch: Cargo.toml rust-version ($CARGO_MSRV) != rust-toolchain.toml channel ($TOOLCHAIN_VER)${NC}"
    errors=$((errors + 1))
else
    echo -e "${GREEN}✓ Cargo.toml and rust-toolchain.toml match${NC}"
fi

if [ "$CARGO_MSRV" != "$CLIPPY_MSRV" ]; then
    echo -e "${RED}✗ Mismatch: Cargo.toml rust-version ($CARGO_MSRV) != .clippy.toml msrv ($CLIPPY_MSRV)${NC}"
    errors=$((errors + 1))
else
    echo -e "${GREEN}✓ Cargo.toml and .clippy.toml match${NC}"
fi

# Check individual crates (they should inherit or match)
while read -r crate_cargo; do
    crate_name=$(grep "^name =" "$crate_cargo" | head -n 1 | cut -d'"' -f2)
    # If they use workspace inheritance, they are fine.
    # If they define it explicitly, it must match.
    if grep -q "rust-version.workspace = true" "$crate_cargo"; then
         echo -e "${GREEN}✓ $crate_name inherits rust-version from workspace${NC}"
    elif grep -q "rust-version =" "$crate_cargo"; then
         CRATE_MSRV=$(grep "rust-version =" "$crate_cargo" | cut -d'"' -f2)
         if [ "$CRATE_MSRV" != "$CARGO_MSRV" ]; then
             echo -e "${RED}✗ Mismatch: $crate_name rust-version ($CRATE_MSRV) != workspace ($CARGO_MSRV)${NC}"
             errors=$((errors + 1))
         else
             echo -e "${GREEN}✓ $crate_name matches workspace rust-version${NC}"
         fi
    fi
done < <(find . -maxdepth 2 -name "Cargo.toml" -not -path "./Cargo.toml")

echo "=========================================="
if [ $errors -gt 0 ]; then
    echo -e "${RED}FAILED: $errors mismatch(es) found${NC}"
    exit 1
else
    echo -e "${GREEN}SUCCESS: All MSRV and toolchain settings are consistent${NC}"
    exit 0
fi
