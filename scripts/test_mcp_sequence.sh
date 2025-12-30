#!/bin/bash
set -e

echo "Testing MCP server with sequential requests in single process"
echo "Starting server in background..."

# Start server as coprocess
coproc SERVER { ./target/debug/memory-mcp-server 2>/dev/null; }

# Send initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' >&${SERVER[1]}

# Read response
read -u ${SERVER[0]} -t 5 RESPONSE1
echo "Response 1 (initialize): $RESPONSE1" | jq . 2>/dev/null || echo "$RESPONSE1"

# Send tools/list request
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' >&${SERVER[1]}

# Read response
read -u ${SERVER[0]} -t 5 RESPONSE2
echo "Response 2 (tools/list): $RESPONSE2" | jq . 2>/dev/null || echo "$RESPONSE2"

# Send query_memory request
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"test","domain":"general","limit":5}}}' >&${SERVER[1]}

# Read response
read -u ${SERVER[0]} -t 5 RESPONSE3
echo "Response 3 (query_memory): $RESPONSE3" | jq . 2>/dev/null || echo "$RESPONSE3"

# Send health check request
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"health_check","arguments":{}}}' >&${SERVER[1]}

# Read response
read -u ${SERVER[0]} -t 5 RESPONSE4
echo "Response 4 (health_check): $RESPONSE4" | jq . 2>/dev/null || echo "$RESPONSE4"

# Close stdin to signal EOF
exec {SERVER[1]}>&-

# Wait for server to exit
wait $SERVER_PID 2>/dev/null || true

echo "Sequence test complete"
