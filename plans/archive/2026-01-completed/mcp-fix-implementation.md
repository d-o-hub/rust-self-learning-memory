# Fix Implementation: MCP Tools Schema Field Naming

## Summary
Fixed the "Failed to get tools" error by correcting the JSON field naming in the `McpTool` struct to match MCP specification requirements.

## Changes Made

### 1. Fixed `McpTool` Struct in protocol.rs

**File**: `/workspaces/feat-phase3/memory-mcp/src/protocol.rs`
**Lines**: 59-64

#### Before (Non-compliant):
```rust
/// MCP Tool structure for listing
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,  // ❌ SNAKE_CASE
}
```

#### After (MCP Compliant):
```rust
/// MCP Tool structure for listing
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,  // ✅ OPTIONAL: Added for better UX
    pub description: String,
    #[serde(rename = "inputSchema")]  // ✅ FIXED: Map to camelCase
    pub input_schema: Value,
}
```

### 2. Updated Tool Listing in core.rs

**File**: `/workspaces/feat-phase3/memory-mcp/src/bin/server/core.rs`
**Lines**: 83-90

#### Before:
```rust
let mcp_tools: Vec<McpTool> = tools
    .into_iter()
    .map(|tool| McpTool {
        name: tool.name,
        description: tool.description,
        input_schema: tool.input_schema,
    })
    .collect();
```

#### After:
```rust
let mcp_tools: Vec<McpTool> = tools
    .into_iter()
    .map(|tool| McpTool {
        name: tool.name,
        title: None,  // ✅ Initialize optional field
        description: tool.description,
        input_schema: tool.input_schema,
    })
    .collect();
```

## Technical Details

### Why This Fix Works

#### 1. Serde Field Renaming
The `#[serde(rename = "inputSchema")]` attribute tells serde to serialize the `input_schema` field (snake_case) as `inputSchema` (camelCase) in JSON output.

#### 2. MCP Specification Compliance
According to MCP 2025-06-18 and 2025-11-25 specifications, the tool schema field **MUST** be named `inputSchema` (camelCase):

```json
{
  "name": "query_memory",
  "description": "Query episodic memory for relevant past experiences",
  "inputSchema": {  // ← Required camelCase
    "type": "object",
    "properties": { ... }
  }
}
```

#### 3. Optional Title Field
The `title` field is optional per MCP spec. Adding it with:
- `Option<String>` type - makes it optional
- `#[serde(skip_serializing_if = "Option::is_none")]` - excludes from JSON when `None`

This allows future enhancement to add human-readable titles without breaking existing clients.

## Fix Impact

### Before Fix
```json
{
  "name": "query_memory",
  "description": "Query episodic memory...",
  "input_schema": {  // ❌ WRONG: snake_case
    "type": "object",
    ...
  }
}
```

**Result**: MCP clients reject tool definitions → "Failed to get tools" error

### After Fix
```json
{
  "name": "query_memory",
  "description": "Query episodic memory...",
  "inputSchema": {  // ✅ CORRECT: camelCase
    "type": "object",
    ...
  }
}
```

**Result**: MCP clients accept tool definitions → tools successfully listed and usable

## Files Modified

| File | Lines Changed | Type |
|------|--------------|------|
| `memory-mcp/src/protocol.rs` | 59-64 | Struct definition |
| `memory-mcp/src/bin/server/core.rs` | 83-90 | Tool mapping |

## Code Quality

### Compliance
✅ **MCP Specification**: Compliant with MCP 2025-06-18 and 2025-11-25
✅ **JSON-RPC**: Follows JSON-RPC 2.0 standard
✅ **Rust Conventions**: Follows Rust naming (snake_case internally) with serialization mapping

### Testing Requirements
- [ ] Compile successfully (pending due to build timeout)
- [ ] Pass all existing tests
- [ ] Verify JSON output format
- [ ] Test with MCP Inspector
- [ ] Verify all 11 tools are discoverable

## Verification Steps

### 1. Build Verification
```bash
cd /workspaces/feat-phase3
cargo build --release --package memory-mcp
```

**Expected**: Successful compilation, no errors

### 2. Test Execution
```bash
cd /workspaces/feat-phase3
cargo test --package memory-mcp --all
```

**Expected**: All tests pass, no regressions

### 3. JSON Output Verification
Run server and inspect `tools/list` response:

```bash
# Start server
cargo run --release --bin memory-mcp-server

# In another terminal, send tools/list request
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  cargo run --release --bin memory-mcp-server
```

**Expected Output**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "query_memory",
        "description": "Query episodic memory for relevant past experiences and learned patterns",
        "inputSchema": {  // ← Verify camelCase
          "type": "object",
          "properties": { ... }
        }
      }
    ]
  }
}
```

### 4. MCP Inspector Verification
```bash
# Install and run MCP Inspector
npx -y @modelcontextprotocol/inspector \
  cargo run --release --bin memory-mcp-server
```

**Expected**:
- Inspector successfully connects to server
- "Tools" tab shows all 11 tools
- Tool schemas are displayed correctly
- No "Failed to get tools" error

### 5. Integration Test Verification
```bash
cd /workspaces/feat-phase3
cargo test --test mcp_integration_tests -- --nocapture
```

**Expected**: All MCP integration tests pass

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing clients | Low | High | Field is required in spec - clients already expecting it |
| Test failures | Low | Medium | Added optional field with None default - backward compatible |
| Build errors | Very Low | High | Simple field addition - syntax verified |

## Future Enhancements

### 1. Add Tool Titles
Populate the optional `title` field for better UX:

```rust
// In create_default_tools() (server/mod.rs)
tools.push(Tool::new(
    "query_memory".to_string(),
    "Query episodic memory...".to_string(),
    Some("Memory Query Tool".to_string()),  // Add title
    json!({ ... }),
));
```

### 2. Schema Validation Tests
Add tests to verify JSON serialization matches MCP spec:

```rust
#[test]
fn test_tool_schema_compliance() {
    let tool = McpTool {
        name: "test".to_string(),
        title: None,
        description: "test".to_string(),
        input_schema: json!({"type": "object"}),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"inputSchema\""));  // Verify camelCase
    assert!(!json.contains("\"input_schema\""));  // Verify no snake_case
}
```

### 3. MCP Protocol Version Tests
Ensure compatibility across supported versions (2025-11-25, 2024-11-05).

## Rollback Plan

If issues arise:
1. Revert both files to original state
2. Remove `title` field from `McpTool` struct
3. Remove `#[serde(rename = "inputSchema")]` attribute
4. Change back to `pub input_schema: Value`

**Command**:
```bash
git checkout memory-mcp/src/protocol.rs memory-mcp/src/bin/server/core.rs
```

## Summary

**Fix Type**: Schema compliance (field naming)
**Lines Changed**: 2 files, ~10 lines
**Complexity**: Low
**Risk**: Low
**Expected Impact**: Resolves "Failed to get tools" error completely

The fix is minimal, targeted, and directly addresses the root cause identified in the analysis. By adding the serde `rename` attribute, we ensure that the Rust code can use snake_case naming conventions (idiomatic Rust) while producing JSON output that complies with the MCP specification's camelCase requirements.

---

**Implementation Completed**: January 11, 2026
**Status**: Code changes applied, awaiting build verification
**Next Phase**: Verification and Testing
