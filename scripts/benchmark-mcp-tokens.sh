#!/bin/bash
# Token usage benchmark for MCP tool listing
# Compares lazy vs non-lazy (full schema) modes

set -e

echo "=== MCP Token Usage Benchmark ==="
echo

# Build if needed
if [[ ! -x "target/release/memory-mcp-server" ]]; then
    echo "Building MCP server..."
    cargo build --release --package memory-mcp --bin memory-mcp-server
fi

# Function to send JSON-RPC request via stdio
benchmark_tools_list() {
    local lazy=$1
    local label=$2
    
    if [ "$lazy" = "true" ]; then
        REQUEST=$(cat <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"lazy":true}}
EOF
)
    else
        REQUEST=$(cat <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
EOF
)
    fi
    
    # Send request and capture response
    RESPONSE=$(echo "$REQUEST" | target/release/memory-mcp-server 2>/dev/null)
    
    # Count characters and estimate tokens
    CHAR_COUNT=$(echo "$RESPONSE" | tr -d '\n' | wc -c)
    EST_TOKENS=$((CHAR_COUNT / 4))
    
    # Count tools returned
    TOOL_COUNT=$(echo "$RESPONSE" | grep -o '"name":"[^"]*"' | wc -l)
    
    # Check if full schemas or stubs
    HAS_INPUTSCHEMA=$(echo "$RESPONSE" | grep -c '"inputSchema"' || true)
    
    echo "$label"
    echo "  Response size: $CHAR_COUNT chars"
    echo "  Est. tokens: ~$EST_TOKENS"
    echo "  Tools returned: $TOOL_COUNT"
    echo "  Has inputSchema: $([ $HAS_INPUTSCHEMA -gt 0 ] && echo "YES (full)" || echo "NO (stubs)")"
    echo
}

echo "--- Test 1: Full schemas (lazy=false, default) ---"
benchmark_tools_list "false" "Full Schemas"

echo "--- Test 2: Lazy mode (lazy=true) ---"
benchmark_tools_list "true" "Lazy Mode"

echo "=== Summary ==="
echo "Expected savings: ~92-96% token reduction"
echo "Full schemas: ~4,000-8,000 tokens (30 tools with full JSON schemas)"
echo "Lazy mode:   ~200-400 tokens (30 tool names + descriptions)"
echo ""
echo "Note: These are MCP protocol tokens, not LLM tokens."
echo "LLM token savings depend on how the client processes the response."
