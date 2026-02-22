---
name: memory-mcp-token-optimization
description: Optimize memory-mcp token usage for reduced costs and faster responses. Use lazy tool loading, field projection, and best practices to minimize token consumption.
---

# Memory MCP Token Optimization

## Overview

This skill provides guidance for minimizing token usage when using the memory-mcp server. Token optimization reduces:
- LLM context window usage
- API costs
- Response latency

## Token Usage Breakdown

| Operation | Full Schema (default) | Lazy Mode | Savings |
|-----------|---------------------|-----------|---------|
| `tools/list` (8 tools) | ~1,237 tokens | ~227 tokens | **82%** |
| `tools/list` (30 tools) | ~4,000-8,000 tokens | ~200-400 tokens | **92-96%** |
| Query response | 1,200-2,400 tokens | 600-1,200 tokens | 20-60% |

## Lazy Tool Loading

### How It Works

The MCP server supports two tool listing modes:

1. **Full Schema Mode** (`lazy=false` or default): Returns complete JSON schemas for all tools
2. **Lazy Mode** (`lazy=true`): Returns only tool names and descriptions

### Request Format

```json
// Full schema (default) - ~1,200 tokens
{"jsonrpc":"2.0","id":1,"method":"tools/list"}

// Lazy mode - ~200 tokens (82% reduction)
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"lazy":true}}
```

### When to Use Each Mode

| Mode | Use Case |
|------|----------|
| `lazy=true` | Initial tool discovery, IDE autocomplete, when you only need tool names |
| `lazy=false` | When you need inputSchema for parameter validation |

## Field Projection

Reduce response tokens by selecting only needed fields:

```json
{
  "method": "tools/call",
  "params": {
    "name": "query_memory",
    "arguments": {...},
    "fields": ["episodes.id", "episodes.task_description", "patterns.success_rate"]
  }
}
```

## Opencode Configuration

### Current Limitation

Opencode doesn't pass the `lazy` parameter to MCP servers. Current token usage:

```
tools/list: ~1,237 tokens (full schemas)
```

### Recommendations for Opencode Users

1. **Modify the MCP server to default to lazy=true** (requires code change)
2. **Request opencode add MCP parameter support** 
3. **Minimize number of MCP tools enabled** - disable unused tools

### Example Opencode Config for Token Reduction

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "memory-mcp": {
      "type": "local",
      "command": ["/path/to/scripts/preflight-memory-mcp.sh"],
      "enabled": true,
      "timeout": 3000
    }
  },
  // Disable globally
  "tools": {
    "memory-mcp_*": false
  },
  // Enable only for specific agents
  "agent": {
    "memory-agent": {
      "tools": {
        "memory-mcp_*": true
      }
    }
  }
}
```

## Implementation: Default to Lazy Mode

To reduce tokens without requiring client changes, modify the MCP server:

### File: `memory-mcp/src/bin/server_impl/core.rs`

Change the default from `lazy=false` to `lazy=true`:

```rust
// In handle_list_tools function
let lazy = params.get("lazy")
    .and_then(|v| v.as_bool())
    .unwrap_or(true);  // Changed from false to true
```

### Impact

| Metric | Before | After |
|--------|--------|-------|
| tools/list tokens | ~1,237 | ~227 |
| Reduction | - | 82% |
| Backward compatible | Yes | Yes (client can request full schemas) |

## Benchmark Script

Run the token benchmark:

```bash
./scripts/benchmark-mcp-tokens.sh
```

Sample output:
```
=== MCP Token Usage Benchmark ===

--- Test 1: Full schemas (lazy=false, default) ---
  Response size: 4949 chars
  Est. tokens: ~1237
  Tools returned: 8
  Has inputSchema: YES (full)

--- Test 2: Lazy mode (lazy=true) ---
  Response size: 910 chars
  Est. tokens: ~227
  Tools returned: 8
  Has inputSchema: NO (stubs)
```

## Best Practices Summary

1. **Use lazy mode** for tool discovery (80-85% token savings)
2. **Use field projection** for responses (20-60% token savings)
3. **Cache tool lists** - don't call tools/list repeatedly
4. **Disable unused MCPs** in opencode config
5. **Enable per-agent** instead of globally when possible
6. **Set appropriate timeout** to prevent hanging connections

## References

- ADR-024: MCP Lazy Tool Loading (`plans/adr/ADR-024-MCP-Lazy-Tool-Loading.md`)
- MCP Token Optimization Research (`plans/research/MCP_TOKEN_OPTIMIZATION_RESEARCH.md`)
- Opencode MCP Config: https://opencode.ai/docs/mcp-servers/
