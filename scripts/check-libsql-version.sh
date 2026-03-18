#!/bin/bash
# Track upstream libsql versions for memory corruption fixes
# ADR-027 / ADR-041

CURRENT_VERSION=$(grep "libsql =" Cargo.toml | head -n1 | cut -d'"' -f2)
LATEST_VERSION=$(curl -s -H "User-Agent: memory-system-version-checker (maintainer@example.com)" https://crates.io/api/v1/crates/libsql | jq -r '.crate.max_version')

echo "Current libsql version: $CURRENT_VERSION"
echo "Latest libsql version: $LATEST_VERSION"

if [ "$LATEST_VERSION" != "null" ] && [ "$CURRENT_VERSION" != "$LATEST_VERSION" ]; then
    echo "⚠️ A new version of libsql is available ($LATEST_VERSION)"
    echo "Check release notes for memory corruption fixes (ADR-027)."
elif [ "$LATEST_VERSION" == "null" ]; then
    echo "❌ Failed to fetch latest version from crates.io"
else
    echo "✅ libsql is up to date."
fi
