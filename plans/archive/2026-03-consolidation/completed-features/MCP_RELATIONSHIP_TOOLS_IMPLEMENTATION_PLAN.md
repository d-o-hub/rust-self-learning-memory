# MCP Episode Relationship Tools - Implementation Plan

**Date**: 2026-02-02  
**Priority**: P0 - Critical  
**Estimated Effort**: 41-56 hours  
**Status**: NOT IMPLEMENTED  

---

## Overview

This plan implements 8 MCP tools for episode relationship management as documented in `EPISODE_RELATIONSHIPS_PHASE4_5_PLAN.md`. These tools provide the MCP interface for creating, querying, and managing relationships between episodes.

**Current Status**: The `memory-mcp/src/mcp/tools/episode_relationships/` directory exists with basic structure but the tools are NOT implemented.

---

## Tool Specifications

### 1. `add_episode_relationship` (6-8 hours)

**Purpose**: Add a relationship between two episodes with validation

**Input Schema**:
```json
{
  "from_episode_id": "string (UUID)",
  "to_episode_id": "string (UUID)", 
  "relationship_type": "follows|causes|part_of|related_to|contradicts|refines",
  "strength": "number (0.0-1.0, optional, default: 1.0)",
  "metadata": "object (optional)"
}
```

**Output Schema**:
```json
{
  "relationship_id": "string (UUID)",
  "created_at": "string (ISO 8601)",
  "message": "Relationship added successfully"
}
```

**Validation**:
- Both episodes must exist
- No duplicate relationships (same from/to/type)
- No self-references (from_episode_id != to_episode_id)
- Strength must be between 0.0 and 1.0
- Cycle detection for "follows" and "causes" types

**Implementation Location**: `memory-mcp/src/mcp/tools/episode_relationships/tool.rs`

**Code Structure**:
```rust
pub async fn add_episode_relationship(
    storage: Arc<dyn EpisodicStorage>,
    params: Value,
) -> Result<Value, McpError> {
    // 1. Parse and validate input
    // 2. Check episodes exist
    // 3. Check for cycles (if applicable)
    // 4. Check for duplicates
    // 5. Create relationship via storage.add_relationship()
    // 6. Return relationship_id and timestamp
}
```

**Audit Logging**: Yes - via `memory-mcp/src/server/audit/relationship_ops.rs`

---

### 2. `remove_episode_relationship` (4-6 hours)

**Purpose**: Remove a specific relationship by ID

**Input Schema**:
```json
{
  "relationship_id": "string (UUID)"
}
```

**Output Schema**:
```json
{
  "success": true,
  "message": "Relationship removed successfully"
}
```

**Validation**:
- Relationship must exist

**Implementation**:
```rust
pub async fn remove_episode_relationship(
    storage: Arc<dyn EpisodicStorage>,
    params: Value,
) -> Result<Value, McpError> {
    // 1. Parse relationship_id
    // 2. Verify relationship exists
    // 3. Call storage.remove_relationship()
    // 4. Return success
}
```

**Audit Logging**: Yes

---

### 3. `get_episode_relationships` (4-6 hours)

**Purpose**: Get all relationships for a specific episode

**Input Schema**:
```json
{
  "episode_id": "string (UUID)",
  "direction": "outgoing|incoming|both (optional, default: both)",
  "relationship_type": "string (optional filter)",
  "min_strength": "number (optional, default: 0.0)"
}
```

**Output Schema**:
```json
{
  "relationships": [
    {
      "id": "string (UUID)",
      "from_episode_id": "string (UUID)",
      "to_episode_id": "string (UUID)",
      "relationship_type": "string",
      "strength": "number",
      "created_at": "string (ISO 8601)",
      "metadata": "object (optional)"
    }
  ],
  "count": "number"
}
```

**Implementation**:
```rust
pub async fn get_episode_relationships(
    storage: Arc<dyn EpisodicStorage>,
    params: Value,
) -> Result<Value, McpError> {
    // 1. Parse episode_id and filters
    // 2. Call storage.get_relationships() with filters
    // 3. Apply min_strength filter
    // 4. Return relationships array
}
```

---

### 4. `find_related_episodes` (6-8 hours)

**Purpose**: Find episodes related to a given episode (transitive search)

**Input Schema**:
```json
{
  "episode_id": "string (UUID)",
  "max_depth": "number (optional, default: 2, max: 5)",
  "relationship_types": "array of strings (optional)",
  "min_strength": "number (optional, default: 0.5)"
}
```

**Output Schema**:
```json
{
  "related_episodes": [
    {
      "episode_id": "string (UUID)",
      "episode_name": "string",
      "distance": "number (1-5)",
      "path": ["array of relationship IDs"],
      "total_strength": "number (product of strengths along path)"
    }
  ],
  "count": "number"
}
```

**Algorithm**:
1. BFS traversal from starting episode
2. Track visited episodes to avoid cycles
3. Accumulate strength along paths (multiply)
4. Filter by min_strength threshold
5. Limit to max_depth hops

**Implementation**:
```rust
pub async fn find_related_episodes(
    storage: Arc<dyn EpisodicStorage>,
    params: Value,
) -> Result<Value, McpError> {
    // 1. Parse input and validate max_depth <= 5
    // 2. Initialize BFS queue with starting episode
    // 3. Traverse relationships up to max_depth
    // 4. Accumulate strength scores
    // 5. Fetch episode metadata for results
    // 6. Return sorted by total_strength
}
```

**Performance**: May be slow for deep graphs - consider caching

---

### 5. `check_relationship_exists` (3-4 hours)

**Purpose**: Check if a relationship exists between two episodes

**Input Schema**:
```json
{
  "from_episode_id": "string (UUID)",
  "to_episode_id": "string (UUID)",
  "relationship_type": "string (optional)"
}
```

**Output Schema**:
```json
{
  "exists": true,
  "relationships": [
    {
      "id": "string (UUID)",
      "relationship_type": "string",
      "strength": "number",
      "created_at": "string (ISO 8601)"
    }
  ]
}
```

**Note**: Returns all matching relationships if type not specified

---

### 6. `get_dependency_graph` (8-10 hours)

**Purpose**: Get dependency graph for visualization (GraphViz/Mermaid format)

**Input Schema**:
```json
{
  "episode_ids": "array of UUIDs (optional - if empty, all episodes)",
  "relationship_types": "array of strings (optional filter)",
  "format": "json|graphviz|mermaid (default: json)",
  "max_nodes": "number (optional, default: 100, max: 500)"
}
```

**Output Schema (JSON format)**:
```json
{
  "nodes": [
    {
      "id": "string (UUID)",
      "label": "string (episode name)",
      "created_at": "string (ISO 8601)"
    }
  ],
  "edges": [
    {
      "from": "string (UUID)",
      "to": "string (UUID)",
      "type": "string",
      "strength": "number"
    }
  ],
  "format": "json"
}
```

**Output Schema (GraphViz format)**:
```
digraph {
  node1 [label="Episode Name"];
  node1 -> node2 [label="follows (0.8)"];
}
```

**Output Schema (Mermaid format)**:
```
graph TD
  A[Episode 1] -->|follows| B[Episode 2]
```

**Implementation**:
```rust
pub async fn get_dependency_graph(
    storage: Arc<dyn EpisodicStorage>,
    params: Value,
) -> Result<Value, McpError> {
    // 1. Parse input
    // 2. Fetch episodes (filtered or all)
    // 3. Fetch relationships between episodes
    // 4. Apply max_nodes limit
    // 5. Format output based on format parameter
    // 6. Return graph
}
```

---

### 7. `validate_no_cycles` (4-6 hours)

**Purpose**: Check if adding a relationship would create a cycle

**Input Schema**:
```json
{
  "from_episode_id": "string (UUID)",
  "to_episode_id": "string (UUID)",
  "relationship_type": "follows|causes"
}
```

**Output Schema**:
```json
{
  "valid": true,
  "cycle_detected": false,
  "cycle_path": null
}
```

**Output (if cycle detected)**:
```json
{
  "valid": false,
  "cycle_detected": true,
  "cycle_path": ["uuid1", "uuid2", "uuid3", "uuid1"],
  "message": "Adding this relationship would create a cycle"
}
```

**Algorithm**:
1. Temporarily add relationship to graph
2. Run DFS from to_episode_id
3. If from_episode_id is reachable, cycle exists
4. Return path if cycle found

**Note**: Only applies to "follows" and "causes" relationship types

---

### 8. `get_topological_order` (6-8 hours)

**Purpose**: Get episodes in topological order (dependency order)

**Input Schema**:
```json
{
  "episode_ids": "array of UUIDs (optional)",
  "relationship_type": "follows|causes (default: follows)"
}
```

**Output Schema**:
```json
{
  "ordered_episodes": [
    {
      "episode_id": "string (UUID)",
      "episode_name": "string",
      "level": "number (0-based depth)",
      "dependencies": ["array of episode IDs this depends on"]
    }
  ],
  "has_cycles": false,
  "cycles": []
}
```

**Algorithm**:
1. Build directed graph from relationships
2. Run Kahn's algorithm for topological sort
3. Assign levels (0 = no dependencies, 1 = depends on level 0, etc.)
4. Detect cycles if sort fails
5. Return ordered list

**Use Cases**:
- Display episode execution order
- Identify dependency chains
- Plan episode replay order

---

## Implementation Phases

### Phase 1: Core Tools (Days 1-3)
**Effort**: 18-24 hours

1. Implement `add_episode_relationship` (6-8h)
2. Implement `remove_episode_relationship` (4-6h)
3. Implement `get_episode_relationships` (4-6h)
4. Add unit tests for core operations (4h)

**Deliverable**: Basic CRUD operations working

---

### Phase 2: Query Tools (Days 4-5)
**Effort**: 13-18 hours

1. Implement `find_related_episodes` (6-8h)
2. Implement `check_relationship_exists` (3-4h)
3. Add unit tests for queries (4-6h)

**Deliverable**: Relationship querying functional

---

### Phase 3: Advanced Tools (Days 6-7)
**Effort**: 18-24 hours

1. Implement `get_dependency_graph` (8-10h)
2. Implement `validate_no_cycles` (4-6h)
3. Implement `get_topological_order` (6-8h)

**Deliverable**: Full graph analysis capabilities

---

### Phase 4: Integration & Testing (Day 8)
**Effort**: 8-10 hours

1. Integration tests (4-6h)
2. Audit logging verification (2h)
3. Documentation (2h)

**Deliverable**: Production-ready tools

---

## File Structure

```
memory-mcp/src/mcp/tools/episode_relationships/
├── mod.rs                    (exports)
├── tool.rs                   (main implementation - ADD 8 functions)
├── types.rs                  (request/response types)
├── graph.rs                  (NEW - graph algorithms)
├── validation.rs             (NEW - cycle detection, validation)
├── formatting.rs             (NEW - GraphViz/Mermaid output)
└── tests.rs                  (unit tests)
```

---

## Integration Points

### 1. Tool Registration
**File**: `memory-mcp/src/server/tool_definitions.rs`

Add to tool list:
```rust
Tool {
    name: "add_episode_relationship".to_string(),
    description: "Add a relationship between two episodes".to_string(),
    input_schema: /* JSON schema */
}
```

Repeat for all 8 tools.

### 2. Tool Execution
**File**: `memory-mcp/src/bin/server/tools.rs`

Add match arms:
```rust
"add_episode_relationship" => {
    episode_relationships::add_episode_relationship(storage, params).await
}
```

### 3. Audit Logging
**File**: `memory-mcp/src/server/audit/relationship_ops.rs`

Already exists - verify all operations are logged.

---

## Testing Strategy

### Unit Tests (per tool)
- Valid input → success
- Invalid input → appropriate error
- Edge cases (empty results, max limits)

### Integration Tests
- End-to-end workflow (add, query, remove)
- Cycle detection scenarios
- Graph with 100+ nodes
- Concurrent relationship creation

### Performance Tests
- 1000 relationships query time < 100ms
- Graph generation for 500 nodes < 1s
- Transitive search depth=3 < 500ms

---

## Dependencies

**Storage Layer**: 
- ✅ `memory-core/src/episodic/relationships.rs` - Already implemented
- ✅ Storage traits support relationships

**No new dependencies required** - all functionality uses existing backend.

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Performance issues with large graphs | Medium | High | Add caching, limit max_nodes |
| Cycle detection bugs | Low | High | Comprehensive unit tests |
| Memory usage for graph traversal | Medium | Medium | Streaming results, pagination |
| Complex GraphViz/Mermaid formatting | Low | Low | Start with JSON, add formats later |

---

## Success Criteria

- [ ] All 8 tools implemented and tested
- [ ] 100% test coverage for relationship operations
- [ ] No cycle detection false positives/negatives
- [ ] Graph generation handles 500+ nodes
- [ ] Audit logging for all operations
- [ ] Documentation complete with examples
- [ ] Integration tests pass

---

## Next Steps

1. Review and approve this plan
2. Create feature branch: `feat/mcp-relationship-tools`
3. Start with Phase 1 (core tools)
4. Daily progress updates in plans folder

---

**Total Effort**: 41-56 hours (5-7 business days)  
**Priority**: P0 - Blocks user-facing relationship features  
**Assignee**: TBD
