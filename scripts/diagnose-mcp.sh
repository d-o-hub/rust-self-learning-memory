#!/bin/bash

# MCP Server Diagnostic Script
# Tests MCP server startup and identifies parsing issues

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔍 MCP Server Diagnostic Tool"
echo "=============================="
echo

# Check if the server binary exists
echo "1. Checking server binary..."
if [ ! -f "$PROJECT_ROOT/target/debug/memory-mcp-server" ]; then
    echo "❌ Server binary not found. Building..."
    cd "$PROJECT_ROOT"
    if ! cargo build --bin memory-mcp-server 2>&1; then
        echo "❌ Build failed. Compilation errors detected."
        echo
        echo "Common issues:"
        echo "- WASM sandbox API changes"
        echo "- rquickjs version compatibility"
        echo "- Missing Debug implementations"
        echo
        echo "Try running: cargo check --bin memory-mcp-server"
        exit 1
    fi
fi
echo "✅ Server binary exists"

# Test basic JSON-RPC parsing
echo
echo "2. Testing JSON-RPC parsing..."

# Create a simple test request
TEST_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

echo "Sending test request: $TEST_REQUEST"

# Try to start server and send request
timeout 10s bash -c "
cd '$PROJECT_ROOT'
echo '$TEST_REQUEST' | target/debug/memory-mcp-server 2>&1
" > /tmp/mcp_test_output 2>&1 || true

echo
echo "3. Analyzing server response..."

if [ -f /tmp/mcp_test_output ]; then
    RESPONSE=$(cat /tmp/mcp_test_output)
    echo "Raw server output:"
    echo "$RESPONSE"
    echo

    # Check if response is valid JSON
    if echo "$RESPONSE" | jq . >/dev/null 2>&1; then
        echo "✅ Response is valid JSON"
        echo "Response type: $(echo "$RESPONSE" | jq -r '.jsonrpc // "unknown"')"

        if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
            ERROR_CODE=$(echo "$RESPONSE" | jq -r '.error.code')
            ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
            echo "❌ Server returned error: $ERROR_CODE - $ERROR_MSG"
        else
            echo "✅ Server returned successful response"
        fi
    else
        echo "❌ Response is NOT valid JSON"
        echo "This explains the 'Failed to parse server response' error!"
        echo
        echo "Possible causes:"
        echo "- Server crashed during request processing"
        echo "- Server output non-JSON data (logs, panics)"
        echo "- Encoding issues (UTF-8, etc.)"
        echo "- Server not properly handling JSON-RPC protocol"
    fi
else
    echo "❌ No response captured"
fi

echo
echo "4. Checking for compilation issues..."

cd "$PROJECT_ROOT"
if cargo check --bin memory-mcp-server >/dev/null 2>&1; then
    echo "✅ Code compiles successfully"
else
    echo "❌ Code has compilation errors"
    echo "Run 'cargo check --bin memory-mcp-server' for details"
fi

echo
echo "5. Recommendations:"

if echo "$RESPONSE" | jq . >/dev/null 2>&1; then
    echo "✅ JSON parsing works - issue may be in client or specific requests"
else
    echo "❌ Fix server compilation errors first"
    echo "   Run: cargo check --bin memory-mcp-server"
    echo "   Fix the WASM sandbox API issues"
fi

echo
echo "Diagnostic complete."