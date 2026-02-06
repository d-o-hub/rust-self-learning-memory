# MCP Token Optimization Phase 1 (P0) - Implementation Plan

**Objective**: Achieve 57% token reduction through two key optimizations:
1. Dynamic Tool Loading (90-96% input reduction)
2. Field Selection/Projection (20-60% output reduction)

## Current State Analysis

### Architecture Overview
- **Tool Registration**: All 50+ tools loaded at server startup in `tool_definitions.rs` (225 LOC) and `tool_definitions_extended.rs` (682 LOC)
- **Tool Exposure**: `list_tools()` returns all tool schemas upfront, causing massive input token consumption
- **Query Tools**: 20+ query tools return full object structures without field filtering
- **Tool Modules**: 14 tool modules in `memory-mcp/src/server/tools/` (~1,915 LOC total)

### Token Usage Breakdown (Current)
| Component | Estimated Tokens | Percentage |
|-----------|------------------|------------|
| Tool Schemas (input) | ~8,500 | 94% |
| Query Results (output) | ~500 | 6% |
| **Total** | **~9,000** | **100%** |

## Implementation Strategy

### Phase 1: Dynamic Tool Loading (Input Token Reduction)

#### 1.1 Tool Registry System
**File**: `memory-mcp/src/server/tools/registry.rs` (NEW)

```rust
pub struct ToolRegistry {
    // Core tools always loaded (query_memory, health_check, etc.)
    core_tools: Vec<Tool>,
    // Extended tools loaded on-demand
    extended_tools: HashMap<String, Tool>,
    // Session-based cache
    loaded_tools: Arc<RwLock<HashMap<String, Tool>>>,
}
```

**Key Features**:
- Core tools (5-8 essential) loaded by default
- Extended tools (40+) loaded lazily on first use
- Session-based caching to avoid repeated loads
- Progressive disclosure based on usage patterns

#### 1.2 Lazy Loading Implementation
**File**: `memory-mcp/src/server/tools/mod.rs` (MODIFY)

**Changes**:
1. Replace static `list_tools()` with lazy-loading version
2. Add `load_tool(name: &str) -> Option<Tool>` method
3. Add `ensure_tool_loaded(name: &str)` helper
4. Track tool usage in session cache

**Benefits**:
- Input tokens: ~8,500 → ~500 (94% reduction)
- Faster initialization
- Reduced memory footprint

### Phase 2: Field Selection System (Output Token Reduction)

#### 2.1 Field Projection Module
**File**: `memory-mcp/src/server/tools/field_projection.rs` (NEW)

```rust
pub struct FieldSelector {
    allowed_fields: Option<Vec<String>>,
}

impl FieldSelector {
    pub fn apply<T: Serialize>(&self, value: &T) -> Result<serde_json::Value>
    pub fn from_request(args: &Value) -> Self
}
```

**Key Features**:
- Generic field filtering for any serializable type
- Supports nested field selection (e.g., "episode.id", "episode.task_description")
- Whitelist-based approach (only return requested fields)
- Backward compatible (no `fields` param = return all)

#### 2.2 Query Tool Modifications

**Tools Requiring Field Projection** (20 tools):
1. `query_memory` - episodes, patterns fields
2. `analyze_patterns` - patterns fields
3. `bulk_episodes` - episode fields
4. `batch_query_episodes` - multiple field groups
5. `batch_pattern_analysis` - pattern fields
6. `batch_compare_episodes` - comparison fields
7. `get_episode` - all episode fields
8. `get_episode_relationships` - relationship fields
9. `find_related_episodes` - episode + metadata
10. `get_dependency_graph` - graph fields
11. `get_topological_order` - ordering fields
12. `search_episodes_by_tags` - episode fields
13. `get_episode_tags` - tag fields
14. `get_episode_timeline` - step fields
15. `bulk_episodes` - episode fields
16. `recommend_patterns` - pattern fields
17. `search_patterns` - pattern fields
18. `get_metrics` - metric fields
19. `advanced_pattern_analysis` - analysis fields
20. `quality_metrics` - quality fields

**Implementation Pattern**:
```rust
// Before
pub async fn query_memory(...) -> Result<Value> {
    Ok(json!({"episodes": episodes, "patterns": patterns, ...}))
}

// After
pub async fn query_memory(...) -> Result<Value> {
    let result = json!({"episodes": episodes, "patterns": patterns, ...});
    let selector = FieldSelector::from_request(&args);
    selector.apply(&result)
}
```

#### 2.3 Schema Updates

Add `fields` parameter to all query tools:
```json
{
  "name": "query_memory",
  "inputSchema": {
    "properties": {
      "query": {"type": "string"},
      "fields": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Fields to return (e.g., ['episodes.id', 'episodes.task_description'])"
      }
    }
  }
}
```

**Benefits**:
- Output tokens: ~500 → ~200 (60% reduction)
- Clients only pay for what they use
- Reduced network transfer
- Faster serialization

## Implementation Steps

### Step 1: Create Tool Registry (Dynamic Loading)
1. Create `memory-mcp/src/server/tools/registry.rs`
2. Define core vs extended tool categorization
3. Implement lazy loading logic
4. Add session caching

### Step 2: Create Field Projection Module
1. Create `memory-mcp/src/server/tools/field_projection.rs`
2. Implement generic field selector
3. Add support for nested field paths
4. Handle edge cases (invalid fields, nested objects)

### Step 3: Modify Tool Registration
1. Update `memory-mcp/src/server/mod.rs`
2. Replace static tool loading with registry
3. Update `list_tools()` to use registry
4. Ensure backward compatibility

### Step 4: Update Query Tools
1. Add `fields` parameter to all 20 query tool schemas
2. Apply field projection in each tool handler
3. Add comprehensive tests
4. Document usage examples

### Step 5: Testing & Validation
1. Create token counting tests
2. Measure actual reduction percentages
3. Verify backward compatibility
4. Performance benchmarking

### Step 6: Documentation
1. Update tool documentation
2. Add usage examples
3. Document optimization strategies
4. Create migration guide

## Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Input Tokens (tools/list) | ~8,500 | ~500 | 94% ↓ |
| Output Tokens (avg query) | ~500 | ~200 | 60% ↓ |
| Total Token Usage | ~9,000 | ~700 | 92% ↓ |
| Server Init Time | 150ms | 50ms | 67% ↓ |
| Memory Footprint | 25MB | 18MB | 28% ↓ |

## File Structure

### New Files
- `memory-mcp/src/server/tools/registry.rs` (~300 LOC)
- `memory-mcp/src/server/tools/field_projection.rs` (~250 LOC)

### Modified Files
- `memory-mcp/src/server/tools/mod.rs` (add lazy loading)
- `memory-mcp/src/server/tool_definitions.rs` (add core vs extended categorization)
- `memory-mcp/src/server/tool_definitions_extended.rs` (mark as extended)
- `memory-mcp/src/server/mod.rs` (use registry instead of static tools)
- All 20 query tool files (add `fields` parameter)

### Test Files
- `memory-mcp/src/server/tools/registry_tests.rs` (NEW)
- `memory-mcp/src/server/tools/field_projection_tests.rs` (NEW)
- `memory-mcp/tests/token_optimization.rs` (NEW)

## Risk Mitigation

### Backward Compatibility
- If `fields` parameter not provided, return all fields (default behavior)
- Progressive rollout with feature flags
- Extensive testing with legacy clients

### Performance
- Cache loaded tools per session
- Use efficient field selection (not regex)
- Benchmark before/after

### Breaking Changes
- None - purely additive changes
- Existing clients work without modification

## Success Criteria

- [ ] Dynamic tool loading reduces input tokens by 90%+
- [ ] Field selection reduces output tokens by 20-60%
- [ ] All existing tests pass
- [ ] New tests demonstrate token reduction effectiveness
- [ ] Zero functionality regression
- [ ] All documentation updated
- [ ] Performance benchmarks show improvement
- [ ] Backward compatibility verified

## Timeline Estimate

- Phase 1 (Dynamic Loading): 4-6 hours
- Phase 2 (Field Selection): 6-8 hours
- Phase 3 (Testing): 4-6 hours
- Phase 4 (Documentation): 2-3 hours
- **Total**: 16-23 hours

## Next Steps

1. ✅ Analysis complete
2. ⏭️ Implement Tool Registry
3. ⏭️ Implement Field Projection
4. ⏭️ Modify Tool Registration
5. ⏭️ Update Query Tools
6. ⏭️ Testing & Validation
7. ⏭️ Documentation
