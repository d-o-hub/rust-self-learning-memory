#!/usr/bin/env bash
# scripts/build-maintenance.sh — Check and fix target/ bloat
# Verifies .cargo/config.toml optimizations are in place and target/ size is reasonable.
#
# Usage:
#   ./scripts/build-maintenance.sh          # Check only
#   ./scripts/build-maintenance.sh --fix    # Check and auto-fix (cargo clean if >5GB)

set -euo pipefail

FIX_MODE=false
THRESHOLD_KB=5242880  # 5GB in KB

for arg in "$@"; do
  case "$arg" in
    --fix) FIX_MODE=true ;;
    --help|-h)
      echo "Usage: $0 [--fix]"
      echo "  --fix    Auto-clean target/ if over 5GB threshold"
      exit 0
      ;;
  esac
done

echo "🔍 Build maintenance check..."
echo ""

ISSUES=0

# Check 1: target/ size
if [ -d "target" ]; then
    SIZE_KB=$(du -sk target/ | awk '{print $1}')
    SIZE_GB=$(awk "BEGIN {printf \"%.1f\", $SIZE_KB / 1048576}")
    if [ "$SIZE_KB" -gt "$THRESHOLD_KB" ]; then
        echo "⚠️  target/ is ${SIZE_GB}GB (threshold: 5GB)"
        ((ISSUES++))
        if [ "$FIX_MODE" = true ]; then
            echo "   🔧 Running cargo clean..."
            cargo clean
            echo "   ✅ Cleaned"
        else
            echo "   Run: cargo clean (or use --fix)"
        fi
    else
        echo "✅ target/ size OK: ${SIZE_GB}GB"
    fi
else
    echo "✅ No target/ directory"
fi

# Check 2: .cargo/config.toml dependency debug info
if [ -f ".cargo/config.toml" ]; then
    if grep -q 'profile.dev.package' .cargo/config.toml 2>/dev/null; then
        echo "✅ Dependency debug info disabled in .cargo/config.toml"
    else
        echo "⚠️  Missing [profile.dev.package.\"*\"] debug = false in .cargo/config.toml"
        ((ISSUES++))
    fi

    if grep -q 'split-debuginfo' .cargo/config.toml 2>/dev/null; then
        echo "✅ split-debuginfo configured"
    else
        echo "⚠️  Missing split-debuginfo = \"unpacked\" in .cargo/config.toml"
        ((ISSUES++))
    fi

    if grep -q 'line-tables-only' .cargo/config.toml 2>/dev/null; then
        echo "✅ dev debug = \"line-tables-only\" (compact debug info)"
    else
        echo "⚠️  Missing debug = \"line-tables-only\" for dev profile"
        ((ISSUES++))
    fi
else
    echo "⚠️  .cargo/config.toml not found"
    ((ISSUES++))
fi

# Check 3: Cargo target dir override
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    echo "ℹ️  CARGO_TARGET_DIR set: $CARGO_TARGET_DIR"
    if [ -d "$CARGO_TARGET_DIR" ]; then
        EXT_SIZE_KB=$(du -sk "$CARGO_TARGET_DIR" | awk '{print $1}')
        EXT_SIZE_GB=$(awk "BEGIN {printf \"%.1f\", $EXT_SIZE_KB / 1048576}")
        echo "   Size: ${EXT_SIZE_GB}GB"
    fi
fi

echo ""
if [ "$ISSUES" -eq 0 ]; then
    echo "✅ All build optimizations in place."
    exit 0
else
    echo "⚠️  $ISSUES issue(s) found."
    exit 1
fi
