#!/bin/bash
# Monitor upstream libsql version for new releases

CURRENT_VERSION=$(grep "libsql =" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
LATEST_VERSION=$(curl -s https://api.github.com/repos/tursodatabase/libsql/releases/latest | grep '"tag_name":' | sed 's/.*"v\(.*\)".*/\1/')

echo "Current libsql version: $CURRENT_VERSION"
echo "Latest libsql version: $LATEST_VERSION"

if [ "$CURRENT_VERSION" != "$LATEST_VERSION" ]; then
    echo "⚠️ A new version of libsql is available: $LATEST_VERSION"
else
    echo "✅ libsql is up to date."
fi
