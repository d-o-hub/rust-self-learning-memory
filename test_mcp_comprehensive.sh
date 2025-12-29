#!/bin/bash
set -e

echo "=== Comprehensive MCP Server Test ==="
echo "Testing memory-mcp-server with STDIO JSON-RPC protocol"
echo ""

# Test 1: Initialize
echo "Test 1: Initialize server"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 2: List tools
echo "Test 2: List available tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 3: Health check
echo "Test 3: Health check"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"health_check","arguments":{}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 4: Get metrics
echo "Test 4: Get metrics"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_metrics","arguments":{"metric_type":"all"}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 5: Query memory with general domain
echo "Test 5: Query memory (domain: general)"
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"test query","domain":"general","limit":10}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 6: Analyze patterns
echo "Test 6: Analyze patterns"
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"analyze_patterns","arguments":{"task_type":"code_generation","min_success_rate":0.7,"limit":10}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 7: Quality metrics
echo "Test 7: Quality metrics"
echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"quality_metrics","arguments":{"time_range":"7d"}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 8: Execute agent code (simple success)
echo "Test 8: Execute agent code (simple arithmetic)"
echo '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"execute_agent_code","arguments":{"code":"return {result: 42};","context":{"task":"test calculation","input":{}}}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 9: Advanced pattern analysis
echo "Test 9: Advanced pattern analysis"
echo '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"advanced_pattern_analysis","arguments":{"analysis_type":"statistical","time_series_data":{"success_rate":[0.8,0.85,0.82,0.88,0.9,0.87]}}}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 10: Invalid method (error handling)
echo "Test 10: Invalid method (error handling)"
echo '{"jsonrpc":"2.0","id":10,"method":"invalid_method","params":{}}' | \
  ./target/debug/memory-mcp-server 2>/dev/null | grep -v "^\[.*\]" | jq .
echo ""

# Test 11: Malformed JSON (should show error or exit)
echo "Test 11: Malformed JSON"
echo '{invalid json}' | ./target/debug/memory-mcp-server 2>&1 | head -5
echo ""

echo "=== Database Files Check ==="
ls -la data/ 2>/dev/null || echo "data directory not found"
ls -la memory-mcp/data/ 2>/dev/null || echo "memory-mcp/data directory not found"
echo ""

echo "=== Test Complete ==="
