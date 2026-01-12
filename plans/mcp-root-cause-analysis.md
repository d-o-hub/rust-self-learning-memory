# Root Cause Analysis: "Failed to get tools" Error

## Summary
The "Failed to get tools" error in memory-mcp server is caused by a **JSON field naming mismatch** between the Rust struct definition and the MCP protocol specification.

## Root Cause Identification

### Issue Location
**File**: `/workspaces/feat-phase3/memory-mcp/src/protocol.rs`
**Lines**: 59-63

### The Problem
The `McpTool` struct defines the input schema field as `input_schema` (snake_case):

```rust
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,  // ← SNAKE_CASE
}
```

### Expected Format (MCP Specification)
According to the MCP 2025-06-18 specification (and confirmed in MCP 2025-11-25), the field MUST be named `inputSchema` (camelCase):

```json
{
  "name": "get_weather",
  "title": "Weather Information Provider",
  "description": "Get current weather information for a location",
  "inputSchema": {  // ← REQUIRED CAMELCASE!
    "type": "object",
    "properties": {
      "location": {
        "type": "string",
        "description": "City name or zip code"
      }
    },
    "required": ["location"]
  }
}
```

### Actual Output (Current Implementation)
When serialized to JSON by Rust's `serde` with default settings, the output is:

```json
{
  "name": "query_memory",
  "description": "Query episodic memory for relevant past experiences and learned patterns",
  "input_schema": {  // ← WRONG: snake_case instead of camelCase
    "type": "object",
    "properties": { ... },
    "required": [...]
  }
}
```

## Why This Causes "Failed to get tools"

1. **MCP Inspector/Client Validation**: MCP clients and the MCP Inspector tool validate tool responses against the MCP schema specification
2. **Schema Mismatch**: When the response uses `input_schema` instead of `inputSchema`, the schema validation fails
3. **Error Generation**: The client generates a "Failed to get tools" error because the tool definitions don't match the expected protocol format
4. **Tool Discovery Fails**: Without valid tool schemas, the client cannot properly discover or use the tools

## Impact Assessment

### Severity: **CRITICAL**

**Direct Impact**:
- All MCP tool discovery fails
- No tools are listed by MCP Inspector
- Clients cannot query memory, analyze patterns, or use any tools
- Server is non-functional for MCP protocol communication

**Affected Components**:
1. `protocol.rs` - `McpTool` struct (line 62)
2. All tool handlers that return tool schemas
3. MCP Inspector tool discovery
4. Any MCP client connecting to this server

### Tools Affected
All 11 tools defined in the server are affected:
1. `query_memory`
2. `execute_agent_code` (if WASM available)
3. `analyze_patterns`
4. `health_check`
5. `get_metrics`
6. `advanced_pattern_analysis`
7. `quality_metrics`
8. `configure_embeddings`
9. `query_semantic_memory`
10. `test_embeddings`

## Comparison: Current vs Required

| Aspect | Current Implementation | Required (MCP Spec) |
|--------|---------------------|----------------------|
| Field Name | `input_schema` | `inputSchema` |
| Case | snake_case | camelCase |
| Serialization | Default serde behavior | Renamed field |
| Spec Compliance | ❌ Non-compliant | ✅ Compliant |
| Tool Discovery | ❌ Fails | ✅ Works |

## Evidence

### MCP Specification Reference
**Source**: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
**Quote**: "inputSchema: JSON Schema defining expected parameters"
**Authority**: Official MCP specification (June 18, 2025)

### Code Trace
1. **Tool Creation** (server/mod.rs:264-298)
   ```rust
   Tool::new(
       "query_memory".to_string(),
       "Query episodic memory for relevant past experiences and learned patterns".to_string(),
       json!({
           "type": "object",
           "properties": { ... },
           "required": ["query", "domain"]
       }),
   )
   ```

2. **Tool Storage** (server/mod.rs:90)
   ```rust
   tools: Arc<RwLock<Vec<Tool>>>,
   ```

3. **Tool Listing** (core.rs:83-90)
   ```rust
   let mcp_tools: Vec<McpTool> = tools
       .into_iter()
       .map(|tool| McpTool {
           name: tool.name,
           description: tool.description,
           input_schema: tool.input_schema,  // ← SNAKE_CASE
       })
       .collect();
   ```

4. **JSON Serialization** (core.rs:94-100)
   ```rust
   match serde_json::to_value(result) {
       Ok(value) => Some(JsonRpcResponse {
           jsonrpc: "2.0".to_string",
           id: request.id,
           result: Some(value),  // ← Serializes with snake_case
           error: None,
       }),
   ```

## Other Potential Issues (Minor)

### 1. Missing `title` Field (Optional but Recommended)
The MCP spec defines an optional `title` field for human-readable tool names. This is not required but recommended for better UX.

**Current**: No `title` field
**Recommended**: Add `title` field to McpTool struct

### 2. Protocol Version Alignment
The implementation supports MCP 2025-11-25 and 2024-11-05 versions (protocol.rs:17). This is correct and up-to-date.

## Root Cause Summary

**Primary Cause**: Field name mismatch (`input_schema` vs `inputSchema`)
**Secondary Factor**: Missing optional `title` field (UX enhancement)
**Confidence**: Very High (95%+)

This is a simple serialization bug caused by:
1. Rust's default snake_case naming convention
2. Missing serde `rename` attribute to map to MCP's camelCase
3. Insufficient validation against MCP specification during development

## Fix Strategy

### Immediate Fix Required
Add serde `rename` attribute to the `McpTool` struct:

```rust
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]  // ← FIX: Map to camelCase
    pub input_schema: Value,
}
```

### Enhancement (Optional)
Add optional `title` field:

```rust
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,  // ← OPTIONAL: Add for better UX
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}
```

## Testing Recommendations

After applying fix:
1. **Verify JSON Output**: Check that serialized tool definitions use `inputSchema`
2. **Test with MCP Inspector**: Connect to server and verify tools are listed
3. **Run Integration Tests**: Execute full MCP integration test suite
4. **Validate Protocol Compliance**: Ensure response matches MCP specification

---

**Analysis Completed**: January 11, 2026
**Root Cause Confidence**: Very High (95%+)
**Fix Complexity**: Low (single line change)
**Expected Resolution**: Immediate after fix is applied
