# Episode Relationships Phase 4-5: MCP Tools & CLI Commands

**Phase**: 4 & 5 of 6 (Combined)  
**Status**: ⏳ PENDING - Ready for Implementation  
**Estimated Effort**: 4-5 days (can be parallelized)  
**Estimated LOC**: ~1,100 (600 MCP + 500 CLI)  
**Estimated Tests**: 30+  
**Dependencies**: Phase 1 (✅), Phase 2 (✅), Phase 3 (✅ Complete 2026-02-01)

---

## Current Status Summary

| Phase | Status | Completion Date |
|-------|--------|-----------------|
| Phase 1: Storage Foundation | ✅ Complete | 2026-01-31 |
| Phase 2: Core API & Business Logic | ✅ Complete | 2026-01-31 |
| Phase 3: Memory Layer Integration | ✅ Complete | 2026-02-01 |
| **Phase 4: MCP Tools** | ⏳ **PENDING** | 8 tools to implement |
| **Phase 5: CLI Commands** | ⏳ **PENDING** | 7 commands to implement |
| Phase 6: Testing & Documentation | ⏳ Planned | After Phase 4-5 |

**Overall Progress**: 50% Complete (3 of 6 phases)

---

## Overview

Phases 4 and 5 expose the relationship functionality to users through:
- **Phase 4**: 8 MCP server tools (JSON-RPC API)
- **Phase 5**: 7 CLI commands (terminal interface)

These can be implemented in parallel by different developers.

---

## What's Pending

### Phase 4: MCP Tools (8 tools to implement)
1. ❌ `add_episode_relationship` - Create relationship with validation
2. ❌ `remove_episode_relationship` - Delete relationship by ID
3. ❌ `get_episode_relationships` - Query relationships for an episode
4. ❌ `find_related_episodes` - Find episodes related to a given episode
5. ❌ `check_relationship_exists` - Check if a specific relationship exists
6. ❌ `get_dependency_graph` - Get relationship graph for visualization
7. ❌ `validate_no_cycles` - Check if adding relationship would create cycle
8. ❌ `get_topological_order` - Get topological ordering of episodes

### Phase 5: CLI Commands (7 commands to implement)
1. ❌ `episode add-relationship` - Create relationship
2. ❌ `episode remove-relationship` - Delete relationship
3. ❌ `episode list-relationships` - List with filters
4. ❌ `episode find-related` - Find related episodes
5. ❌ `episode dependency-graph` - Export graph (DOT/JSON/ASCII)
6. ❌ `episode validate-cycles` - Check for cycles
7. ❌ `episode topological-sort` - Sort episodes

---

## Phase 4: MCP Server Tools

### Tool 1: add_episode_relationship

**JSON-RPC Schema**:
```json
{
  "name": "add_episode_relationship",
  "description": "Add a relationship between two episodes with validation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_episode_id": {
        "type": "string",
        "description": "Source episode UUID",
        "format": "uuid"
      },
      "to_episode_id": {
        "type": "string",
        "description": "Target episode UUID",
        "format": "uuid"
      },
      "relationship_type": {
        "type": "string",
        "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"],
        "description": "Type of relationship"
      },
      "reason": {
        "type": "string",
        "description": "Optional explanation"
      },
      "priority": {
        "type": "integer",
        "minimum": 1,
        "maximum": 10,
        "description": "Optional priority (1-10)"
      },
      "created_by": {
        "type": "string",
        "description": "Optional creator identifier"
      }
    },
    "required": ["from_episode_id", "to_episode_id", "relationship_type"]
  }
}
```

**Implementation** (`memory-mcp/src/bin/server/handlers.rs`):
```rust
async fn handle_add_episode_relationship(
    memory_manager: &Arc<Mutex<MemoryManager>>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let from_id = Uuid::parse_str(params["from_episode_id"].as_str().ok_or("Missing from_episode_id")?)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let to_id = Uuid::parse_str(params["to_episode_id"].as_str().ok_or("Missing to_episode_id")?)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let rel_type = RelationshipType::from_str(params["relationship_type"].as_str().ok_or("Missing relationship_type")?)
        .ok_or("Invalid relationship type")?;
    
    let metadata = RelationshipMetadata {
        reason: params["reason"].as_str().map(|s| s.to_string()),
        created_by: params["created_by"].as_str().map(|s| s.to_string()),
        priority: params["priority"].as_u64().map(|p| p as u8),
        custom_fields: HashMap::new(),
    };
    
    let mut manager = memory_manager.lock().await;
    let relationship_id = manager.add_episode_relationship(from_id, to_id, rel_type, metadata)
        .await
        .map_err(|e| format!("Failed to add relationship: {}", e))?;
    
    Ok(json!({
        "relationship_id": relationship_id.to_string(),
        "success": true
    }))
}
```

---

### Tool 2: remove_episode_relationship

**JSON-RPC Schema**:
```json
{
  "name": "remove_episode_relationship",
  "description": "Remove a relationship by ID",
  "inputSchema": {
    "type": "object",
    "properties": {
      "relationship_id": {
        "type": "string",
        "format": "uuid",
        "description": "Relationship UUID to remove"
      }
    },
    "required": ["relationship_id"]
  }
}
```

**Implementation**:
```rust
async fn handle_remove_episode_relationship(
    memory_manager: &Arc<Mutex<MemoryManager>>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let rel_id = Uuid::parse_str(params["relationship_id"].as_str().ok_or("Missing relationship_id")?)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let mut manager = memory_manager.lock().await;
    manager.remove_episode_relationship(rel_id)
        .await
        .map_err(|e| format!("Failed to remove relationship: {}", e))?;
    
    Ok(json!({"success": true}))
}
```

---

### Tool 3: get_episode_relationships

**JSON-RPC Schema**:
```json
{
  "name": "get_episode_relationships",
  "description": "Get relationships for an episode",
  "inputSchema": {
    "type": "object",
    "properties": {
      "episode_id": {"type": "string", "format": "uuid"},
      "direction": {
        "type": "string",
        "enum": ["outgoing", "incoming", "both"],
        "default": "both"
      },
      "relationship_type": {
        "type": "string",
        "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"]
      }
    },
    "required": ["episode_id"]
  }
}
```

---

### Tool 4: find_related_episodes

**JSON-RPC Schema**:
```json
{
  "name": "find_related_episodes",
  "description": "Find episodes related to a given episode",
  "inputSchema": {
    "type": "object",
    "properties": {
      "episode_id": {"type": "string", "format": "uuid"},
      "relationship_type": {"type": "string"},
      "limit": {"type": "integer", "minimum": 1, "default": 10},
      "include_metadata": {"type": "boolean", "default": false}
    },
    "required": ["episode_id"]
  }
}
```

---

### Tool 5: check_relationship_exists

**JSON-RPC Schema**:
```json
{
  "name": "check_relationship_exists",
  "description": "Check if a specific relationship exists",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_episode_id": {"type": "string", "format": "uuid"},
      "to_episode_id": {"type": "string", "format": "uuid"},
      "relationship_type": {"type": "string"}
    },
    "required": ["from_episode_id", "to_episode_id", "relationship_type"]
  }
}
```

---

### Tool 6: get_dependency_graph

**JSON-RPC Schema**:
```json
{
  "name": "get_dependency_graph",
  "description": "Get relationship graph for visualization",
  "inputSchema": {
    "type": "object",
    "properties": {
      "episode_id": {"type": "string", "format": "uuid"},
      "depth": {"type": "integer", "minimum": 1, "maximum": 5, "default": 2},
      "format": {"type": "string", "enum": ["json", "dot"], "default": "json"}
    },
    "required": ["episode_id"]
  }
}
```

---

### Tool 7: validate_no_cycles

**JSON-RPC Schema**:
```json
{
  "name": "validate_no_cycles",
  "description": "Check if adding a relationship would create a cycle",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_episode_id": {"type": "string", "format": "uuid"},
      "to_episode_id": {"type": "string", "format": "uuid"},
      "relationship_type": {"type": "string"}
    },
    "required": ["from_episode_id", "to_episode_id", "relationship_type"]
  }
}
```

---

### Tool 8: get_topological_order

**JSON-RPC Schema**:
```json
{
  "name": "get_topological_order",
  "description": "Get topological ordering of episodes",
  "inputSchema": {
    "type": "object",
    "properties": {
      "episode_ids": {
        "type": "array",
        "items": {"type": "string", "format": "uuid"},
        "minItems": 1
      }
    },
    "required": ["episode_ids"]
  }
}
```

---

## Phase 5: CLI Commands

### Command 1: episode add-relationship

**Signature**:
```bash
memory-cli episode add-relationship <FROM_ID> \
  --to <TO_ID> \
  --type <TYPE> \
  [--reason <REASON>] \
  [--priority <1-10>] \
  [--created-by <NAME>]
```

**Example**:
```bash
memory-cli episode add-relationship abc-123 \
  --to def-456 \
  --type depends_on \
  --reason "Prerequisite for advanced feature" \
  --priority 8
```

**Implementation** (`memory-cli/src/commands/episode_v2/relationships.rs`):
```rust
#[derive(Parser)]
pub struct AddRelationshipCommand {
    /// Source episode ID
    from_episode_id: String,
    
    /// Target episode ID
    #[arg(long)]
    to: String,
    
    /// Relationship type
    #[arg(long, value_parser = parse_relationship_type)]
    r#type: RelationshipType,
    
    /// Optional reason
    #[arg(long)]
    reason: Option<String>,
    
    /// Priority (1-10)
    #[arg(long)]
    priority: Option<u8>,
    
    /// Creator
    #[arg(long)]
    created_by: Option<String>,
}
```

---

### Command 2: episode remove-relationship

**Signature**:
```bash
memory-cli episode remove-relationship <RELATIONSHIP_ID>
```

---

### Command 3: episode list-relationships

**Signature**:
```bash
memory-cli episode list-relationships <EPISODE_ID> \
  [--direction <outgoing|incoming|both>] \
  [--type <TYPE>] \
  [--format <table|json>]
```

**Output (table format)**:
```
ID                                   Type        From      To        Priority  Reason
------------------------------------ ----------- --------- --------- --------- -------------------------
abc-123                              depends_on  ep-001    ep-002    8         Prerequisite for feature
def-456                              follows     ep-002    ep-003    5         Sequential workflow
```

---

### Command 4: episode find-related

**Signature**:
```bash
memory-cli episode find-related <EPISODE_ID> \
  [--type <TYPE>] \
  [--limit <N>] \
  [--format <table|json>]
```

---

### Command 5: episode dependency-graph

**Signature**:
```bash
memory-cli episode dependency-graph <EPISODE_ID> \
  [--depth <N>] \
  [--format <dot|json|ascii>] \
  [--output <FILE>]
```

**ASCII Output Example**:
```
Episode A
├── depends_on → Episode B
│   └── depends_on → Episode C
└── follows → Episode D
```

---

### Command 6: episode validate-cycles

**Signature**:
```bash
memory-cli episode validate-cycles <EPISODE_ID> \
  [--type <TYPE>]
```

**Output**:
```
✓ No cycles detected for episode abc-123
```

or

```
✗ Cycle detected: abc-123 → def-456 → ghi-789 → abc-123
```

---

### Command 7: episode topological-sort

**Signature**:
```bash
memory-cli episode topological-sort <EPISODE_ID_1> <EPISODE_ID_2> ...
```

**Output**:
```
Topological order:
1. Episode A (abc-123)
2. Episode B (def-456)
3. Episode C (ghi-789)
```

---

## Implementation Checklist

### Phase 4: MCP Tools
- [ ] Add 8 tool handlers to `memory-mcp/src/bin/server/handlers.rs`
- [ ] Define JSON-RPC schemas
- [ ] Implement error handling
- [ ] Add tool tests (16 tests: 2 per tool)
- [ ] Update MCP tool registry

### Phase 5: CLI Commands
- [ ] Create `memory-cli/src/commands/episode_v2/relationships.rs`
- [ ] Implement 7 command structs
- [ ] Add table formatting for list commands
- [ ] Add ASCII graph rendering
- [ ] Add DOT export support
- [ ] Write CLI tests (14 tests: 2 per command)
- [ ] Update CLI help documentation

### Documentation
- [ ] Add MCP tool examples to docs
- [ ] Add CLI command examples to docs
- [ ] Create user guide section

---

## Testing Strategy

### MCP Tool Tests
```rust
#[tokio::test]
async fn test_mcp_add_relationship() {
    let memory_manager = create_test_manager().await;
    
    let params = json!({
        "from_episode_id": "abc-123",
        "to_episode_id": "def-456",
        "relationship_type": "depends_on"
    });
    
    let result = handle_add_episode_relationship(&memory_manager, params).await;
    assert!(result.is_ok());
}
```

### CLI Integration Tests
```rust
#[test]
fn test_cli_add_relationship() {
    let output = Command::new("memory-cli")
        .args(&["episode", "add-relationship", "abc-123", "--to", "def-456", "--type", "depends_on"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
}
```

---

## Performance Targets

| Operation | Target (P95) |
|-----------|-------------|
| MCP tool call (add) | <100ms |
| MCP tool call (get) | <50ms |
| CLI command (add) | <200ms |
| CLI command (list) | <150ms |
| Graph export (DOT) | <500ms |

---

## Dependencies

- Phase 1: Complete ✅ (Storage foundation)
- Phase 2: Complete ✅ (Graph algorithms, validation, RelationshipManager)
- Phase 3: Complete ✅ (MemoryManager API with relationship methods)

**All dependencies satisfied - ready to begin implementation**

---

## Implementation Priority

### Recommended Order:
1. **Phase 4 MCP Tools** (2-3 days)
   - Can be developed independently
   - Follow existing MCP tool patterns in `memory-mcp/src/bin/server/handlers.rs`
   
2. **Phase 5 CLI Commands** (2 days)
   - Can be developed in parallel with Phase 4 by different developer
   - Follow existing CLI patterns in `memory-cli/src/commands/episode_v2/`

**Status**: ✅ Ready for implementation NOW
