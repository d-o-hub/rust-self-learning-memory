#!/bin/bash

# MCP Tool Call Diagnostic Script
# Tests specific tool calls that might be causing parsing errors

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔧 MCP Tool Call Diagnostic"
echo "==========================="
echo

# Test different tool calls
test_tool_call() {
    local method=$1
    local params=$2
    local description=$3

    echo "Testing: $description"
    echo "Method: $method"

    REQUEST="{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}"

    echo "Request: $REQUEST"

    RESPONSE=$(timeout 15s bash -c "
        cd '$PROJECT_ROOT'
        echo '$REQUEST' | target/debug/memory-mcp-server 2>&1 | head -1
    " 2>/dev/null || echo "TIMEOUT")

    echo "Response: $RESPONSE"

    if [ "$RESPONSE" = "TIMEOUT" ]; then
        echo "❌ Request timed out"
    elif echo "$RESPONSE" | jq . >/dev/null 2>&1; then
        if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
            ERROR_CODE=$(echo "$RESPONSE" | jq -r '.error.code')
            ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
            echo "⚠️  Server returned error: $ERROR_CODE - $ERROR_MSG"
        else
            echo "✅ Tool call successful"
        fi
    else
        echo "❌ Response is not valid JSON!"
        echo "This is the source of 'Failed to parse server response'"
    fi

    echo "---"
    echo
}

# Test tools/list first
test_tool_call "tools/list" "{}" "List available tools"

# Test query_memory (should work)
test_tool_call "tools/call" "{\"name\":\"query_memory\",\"arguments\":{\"query\":\"test\",\"domain\":\"test\",\"limit\":5}}" "Query memory tool"

# Test execute_agent_code (might fail due to WASM issues)
test_tool_call "tools/call" "{\"name\":\"execute_agent_code\",\"arguments\":{\"code\":\"console.log('test');\",\"context\":{\"task\":\"test\",\"input\":{}}}}" "Execute code tool"

# Test health_check
test_tool_call "tools/call" "{\"name\":\"health_check\",\"arguments\":{}}" "Health check tool"

echo "Tool call diagnostic complete."
echo
echo "Summary:"
echo "- If query_memory works but execute_agent_code fails, it's likely WASM sandbox issues"
echo "- If all tools fail with invalid JSON, it's a server crash issue"
echo "- Check server logs for detailed error information"