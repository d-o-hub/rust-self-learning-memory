# MCP Configuration

## .mcp.json Structure

```json
{
  "mcpServers": {
    "memory-mcp": {
      "type": "stdio",
      "command": "./target/release/memory-mcp-server",
      "args": [],
      "env": {
        "TURSO_DATABASE_URL": "file:/workspaces/feat-phase3/data/memory.db",
        "LOCAL_DATABASE_URL": "sqlite:/workspaces/feat-phase3/data/memory.db",
        "REDB_CACHE_PATH": "/workspaces/feat-phase3/data/cache.redb",
        "REDB_MAX_CACHE_SIZE": "1000",
        "MCP_CACHE_WARMING_ENABLED": "true",
        "MEMORY_MAX_EPISODES_CACHE": "1000",
        "MEMORY_CACHE_TTL_SECONDS": "1800",
        "RUST_LOG": "off"
      }
    }
  }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TURSO_DATABASE_URL` | Primary database URL (file:// for local) |
| `LOCAL_DATABASE_URL` | Local SQLite database URL |
| `REDB_CACHE_PATH` | Path to redb cache file |
| `REDB_MAX_CACHE_SIZE` | Maximum cache entries (default: 1000) |
| `MCP_CACHE_WARMING_ENABLED` | Enable cache warming on startup |
| `MEMORY_MAX_EPISODES_CACHE` | Maximum episodes in cache |
| `MEMORY_CACHE_TTL_SECONDS` | Cache time-to-live in seconds |
| `RUST_LOG` | Logging level (off, error, warn, info, debug, trace) |

## Starting the MCP Server

### Build the Server

```bash
cargo build --release --bin memory-mcp-server
```

### Run Directly

```bash
# With environment variables
export TURSO_DATABASE_URL="file:./data/memory.db"
export LOCAL_DATABASE_URL="sqlite:./data/memory.db"
export REDB_CACHE_PATH="./data/cache.redb"
export RUST_LOG=info

./target/release/memory-mcp-server
```

### Run via MCP Inspector

```bash
npx -y @modelcontextprotocol/inspector ./target/release/memory-mcp-server
```

This opens a web interface at `http://localhost:5173` where you can:
- List available tools
- Test tool execution
- View request/response JSON
- Debug connection issues
- Validate tool schemas

See: https://modelcontextprotocol.io/docs/tools/inspector
