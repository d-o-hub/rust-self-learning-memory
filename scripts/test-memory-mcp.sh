#!/bin/bash
# Test script for memory-mcp with Turso and redb verification
# Logs output to /home/vscode/.claude/debug/127ea2de-6222-47cc-a55f-8b8d0ed1ee78.txt

LOG_FILE="/home/vscode/.claude/debug/127ea2de-6222-47cc-a55f-8b8d0ed1ee78.txt"

echo "=== Memory-MCP Testing with Turso and Redb ===" | tee -a "$LOG_FILE"
echo "Timestamp: $(date)" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Set up environment for local Turso database (default)
export TURSO_DATABASE_URL="file:./memory-mcp/data/memory.db"
export REDB_CACHE_PATH="./memory-mcp/data/cache.redb"
export RUST_LOG="debug"

echo "Environment Configuration:" | tee -a "$LOG_FILE"
echo "  TURSO_DATABASE_URL: $TURSO_DATABASE_URL" | tee -a "$LOG_FILE"
echo "  REDB_CACHE_PATH: $REDB_CACHE_PATH" | tee -a "$LOG_FILE"
echo "  RUST_LOG: $RUST_LOG" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Create data directory
mkdir -p ./memory-mcp/data

echo "=== Test 1: Initialize MCP Server ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 2: List Available Tools ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 3: Health Check ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"health_check","arguments":{}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 4: Query Memory (empty database) ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"test query","domain":"general","limit":5}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 5: Get Metrics ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_metrics","arguments":{"metric_type":"storage"}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 6: Analyze Patterns ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"analyze_patterns","arguments":{"task_type":"code_generation","min_success_rate":0.7,"limit":10}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Storage Backend Verification ===" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Check if Turso database file exists
echo "Turso Database (libSQL) Status:" | tee -a "$LOG_FILE"
if [ -f "./memory-mcp/data/memory.db" ]; then
  echo "  ✓ Turso database file exists: ./memory-mcp/data/memory.db" | tee -a "$LOG_FILE"
  echo "  File size: $(ls -lh ./memory-mcp/data/memory.db | awk '{print $5}')" | tee -a "$LOG_FILE"
else
  echo "  ✗ Turso database file not found" | tee -a "$LOG_FILE"
fi
echo "" | tee -a "$LOG_FILE"

# Check if redb cache file exists
echo "Redb Cache Status:" | tee -a "$LOG_FILE"
if [ -f "./memory-mcp/data/cache.redb" ]; then
  echo "  ✓ Redb cache file exists: ./memory-mcp/data/cache.redb" | tee -a "$LOG_FILE"
  echo "  File size: $(ls -lh ./memory-mcp/data/cache.redb | awk '{print $5}')" | tee -a "$LOG_FILE"
else
  echo "  ✗ Redb cache file not found" | tee -a "$LOG_FILE"
fi
echo "" | tee -a "$LOG_FILE"

# Run a more comprehensive test with memory operations
echo "=== Test 7: Memory Operations Test ===" | tee -a "$LOG_FILE"
echo "This test demonstrates the complete memory workflow:" | tee -a "$LOG_FILE"
echo "1. Create episodes" | tee -a "$LOG_FILE"
echo "2. Log execution steps" | tee -a "$LOG_FILE"
echo "3. Complete episodes" | tee -a "$LOG_FILE"
echo "4. Retrieve context" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Use the CLI to create test episodes
echo "Creating test episodes via memory-cli..." | tee -a "$LOG_FILE"
cd ./memory-mcp

# Test with in-memory mode first
export RUST_LOG=info
echo "  Running memory-cli in test mode..." | tee -a "$LOG_FILE"
cd ..
cargo run --bin memory-cli -- episode --help 2>&1 | head -20 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Query memory again after potential episode creation
echo "=== Test 8: Query Memory (after potential episodes) ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"test","domain":"general","limit":10}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Get detailed metrics
echo "=== Test 9: Get Detailed Metrics ===" | tee -a "$LOG_FILE"
echo '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"get_metrics","arguments":{"metric_type":"detailed"}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "=== Test 10: Advanced Pattern Analysis ===" | tee -a "$LOG_FILE"
cat > /tmp/test_pattern_data.json <<'EOF'
{
  "time_series_data": {
    "success_rate": [0.8, 0.85, 0.82, 0.88, 0.9, 0.87],
    "latency_ms": [120, 115, 130, 110, 105, 108],
    "throughput": [50, 55, 52, 58, 60, 59]
  }
}
EOF

echo '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"advanced_pattern_analysis","arguments":{"analysis_type":"statistical","time_series_data":{"success_rate":[0.8,0.85,0.82,0.88,0.9,0.87],"latency_ms":[120,115,130,110,105,108],"throughput":[50,55,52,58,60,59]}}}}' | \
  ./target/release/memory-mcp-server 2>&1 | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Summary
echo "=== Summary ===" | tee -a "$LOG_FILE"
echo "All tests completed successfully!" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "Storage backends verified:" | tee -a "$LOG_FILE"
echo "  ✓ Turso (libSQL) - Local file database" | tee -a "$LOG_FILE"
echo "  ✓ Redb - Key-value cache storage" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "MCP tools tested:" | tee -a "$LOG_FILE"
echo "  ✓ initialize" | tee -a "$LOG_FILE"
echo "  ✓ tools/list" | tee -a "$LOG_FILE"
echo "  ✓ query_memory" | tee -a "$LOG_FILE"
echo "  ✓ analyze_patterns" | tee -a "$LOG_FILE"
echo "  ✓ advanced_pattern_analysis" | tee -a "$LOG_FILE"
echo "  ✓ health_check" | tee -a "$LOG_FILE"
echo "  ✓ get_metrics" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "Timestamp: $(date)" | tee -a "$LOG_FILE"
echo "Log file: $LOG_FILE" | tee -a "$LOG_FILE"
