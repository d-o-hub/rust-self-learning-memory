# Troubleshooting

## Common Issues

### Server Won't Start

**Symptoms**: Process exits immediately or hangs

**Checks**:
1. Binary exists: `ls -la ./target/release/memory-mcp-server`
2. Binary is executable: `chmod +x ./target/release/memory-mcp-server`
3. Database files exist: `ls -la ./data/`
4. Environment variables set: `env | grep -E '(TURSO|REDB|RUST_LOG)'`

**Solutions**:
```bash
# Rebuild
cargo build --release --bin memory-mcp-server

# Create data directory
mkdir -p ./data

# Set environment variables
export TURSO_DATABASE_URL="file:./data/memory.db"
export LOCAL_DATABASE_URL="sqlite:./data/memory.db"
export REDB_CACHE_PATH="./data/cache.redb"
```

### Tool Execution Fails

**Symptoms**: Tool returns errors or unexpected results

**Checks**:
1. Enable debug logging: `RUST_LOG=debug`
2. Validate input JSON against schema
3. Check database connectivity
4. Verify cache is accessible

**Debug Commands**:
```bash
# Run with debug logging
RUST_LOG=debug ./target/release/memory-mcp-server

# Check database
sqlite3 ./data/memory.db ".tables"

# Verify cache
ls -lh ./data/cache.redb
```

### Performance Issues

**Symptoms**: Slow responses, timeouts

**Checks**:
1. Cache size configuration
2. Database size
3. Number of cached episodes
4. Concurrent requests

**Solutions**:
```bash
# Adjust cache settings
export REDB_MAX_CACHE_SIZE="2000"
export MEMORY_MAX_EPISODES_CACHE="2000"

# Reduce cache TTL
export MEMORY_CACHE_TTL_SECONDS="900"

# Disable cache warming if startup is slow
export MCP_CACHE_WARMING_ENABLED="false"
```

### Connection Issues

**Symptoms**: Inspector can't connect, stdio communication fails

**Checks**:
1. Server process is running
2. No other process on same stdio
3. Binary path is correct
4. Shell environment is clean

**Solutions**:
```bash
# Kill existing processes
pkill memory-mcp-server

# Verify no zombie processes
ps aux | grep memory-mcp-server

# Restart inspector
npx -y @modelcontextprotocol/inspector ./target/release/memory-mcp-server
```
