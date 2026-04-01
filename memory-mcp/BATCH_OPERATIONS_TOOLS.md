# Batch Operations Tool Status (WG-053)

**Status Date**: 2026-03-24  
**Applies To**: MCP tool contract (`tools/list` + `tools/call`)

## Current Contract Truth

The following tool names are intentionally **not supported** at this time:

- `batch_query_episodes`
- `batch_pattern_analysis`
- `batch_compare_episodes`

They are intentionally omitted from `tools/list` and return `Tool not found` if called directly.

## What is supported

- Transport-level batching via JSON-RPC `batch/execute`
- Existing non-batch MCP tools listed by `tools/list`

## Re-enablement checklist

If these batch-analysis tools are implemented in the future, update all of the following in the same change:

1. `do-memory-mcp/src/server/tool_definitions_extended.rs` (advertise tool schemas)
2. `do-memory-mcp/src/bin/server_impl/handlers.rs` (dispatch handlers)
3. `do-memory-mcp/tests/tool_contract_parity.rs` (parity and call-behavior checks)
4. Docs in `do-memory-mcp/README.md`, `docs/API_REFERENCE.md`, and plans status files
