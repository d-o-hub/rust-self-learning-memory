#!/bin/bash

# Test which MCP tools are actually available

echo "Building MCP server..."
cargo build --release --bin memory-mcp-server

echo ""
echo "=== Testing tool availability ==="

# Test 1: List core tools (lazy=true)
echo "1. Testing core tools listing (lazy=true)..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {"lazy": true}}' | timeout 10s ./target/release/memory-mcp-server 2>/dev/null | jq '.result.tools | length' 2>/dev/null || echo "FAILED"

# Test 2: List all tools (lazy=false)
echo "2. Testing all tools listing (lazy=false)..."  
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {"lazy": false}}' | timeout 10s ./target/release/memory-mcp-server 2>/dev/null | jq '.result.tools | length' 2>/dev/null || echo "FAILED"

# Test 3: Try to access a specific episode relationship tool
echo "3. Testing add_episode_relationship tool access..."
echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/describe", "params": {"name": "add_episode_relationship"}}' | timeout 10s ./target/release/memory-mcp-server 2>/dev/null | jq '.result.tool.name' 2>/dev/null || echo "FAILED"

# Test 4: Try to access a specific episode tagging tool
echo "4. Testing add_episode_tags tool access..."
echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/describe", "params": {"name": "add_episode_tags"}}' | timeout 10s ./target/release/memory-mcp-server 2>/dev/null | jq '.result.tool.name' 2>/dev/null || echo "FAILED"

echo ""
echo "=== Summary ==="
echo "If any tests FAILED, the tools are not properly registered"