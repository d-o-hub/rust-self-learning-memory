# Episode Tagging Feature - Implementation Roadmap

**Feature**: Episode Tags & Labels  
**Version**: v0.1.13  
**Start Date**: 2026-01-27  
**Estimated Completion**: 2026-02-17 (3 weeks)  
**Status**: 🚀 In Progress

## Quick Reference

### Timeline Overview
```
Week 1: Phase 1-2 (Core + Storage)
Week 2: Phase 3-4 (MCP + CLI)
Week 3: Phase 5 (Docs + Testing)
```

### Effort Distribution
- **Phase 1**: 12 hours (28.6%)
- **Phase 2**: 8 hours (19.0%)
- **Phase 3**: 10 hours (23.8%)
- **Phase 4**: 8 hours (19.0%)
- **Phase 5**: 4 hours (9.5%)
- **Total**: 42 hours

### Progress Tracking
- [x] Planning & Design (100%)
- [ ] Phase 1: Core Data Model & Storage (0%)
- [ ] Phase 2: Core API Implementation (0%)
- [ ] Phase 3: MCP Server Integration (0%)
- [ ] Phase 4: CLI Integration (0%)
- [ ] Phase 5: Documentation & Examples (0%)

## Phase 1: Core Data Model & Storage

**Duration**: 12 hours  
**Status**: 🔄 Ready to Start  
**Dependencies**: None

### Objectives
- Establish data persistence layer for tags
- Support both Turso (primary) and redb (cache) backends
- Ensure atomicity and consistency of tag operations

### Tasks

#### 1.1 Update Episode Structure (2 hours)
**File**: `do-memory-core/src/episode/structs.rs`

```rust
// Add to Episode struct
#[serde(default)]
pub tags: Vec<String>,

// Add helper methods
impl Episode {
    pub fn add_tag(&mut self, tag: String) -> bool
    pub fn remove_tag(&mut self, tag: &str) -> bool
    pub fn has_tag(&self, tag: &str) -> bool
    pub fn clear_tags(&mut self)
    fn normalize_tag(tag: &str) -> String
    fn validate_tag(tag: &str) -> Result<()>
}
```

**Tests**:
- Tag normalization (lowercase, trim)
- Tag validation (alphanumeric, hyphens, underscores)
- Add/remove operations
- Duplicate handling

**Success Criteria**:
- ✅ Tags field added to Episode
- ✅ Helper methods implemented
- ✅ All unit tests pass
- ✅ No clippy warnings

#### 1.2 Database Schema Updates (2 hours)
**File**: `do-memory-storage-turso/src/schema.rs`

```sql
-- Add to schema initialization
CREATE TABLE IF NOT EXISTS episode_tags (
    episode_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (episode_id, tag),
    FOREIGN KEY (episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_episode_tags_tag ON episode_tags(tag);
CREATE INDEX IF NOT EXISTS idx_episode_tags_episode ON episode_tags(episode_id);

CREATE TABLE IF NOT EXISTS tag_metadata (
    tag TEXT PRIMARY KEY NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    first_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

**Success Criteria**:
- ✅ Tables created successfully
- ✅ Indexes created
- ✅ Foreign key constraints work
- ✅ Migration runs without errors

#### 1.3 Turso Storage Backend Implementation (4 hours)
**New File**: `do-memory-storage-turso/src/storage/tag_operations.rs`

```rust
// Core tag operations
pub async fn save_episode_tags(
    conn: &Connection,
    episode_id: &Uuid,
    tags: &[String],
) -> Result<()>

pub async fn get_episode_tags(
    conn: &Connection,
    episode_id: &Uuid,
) -> Result<Vec<String>>

pub async fn delete_episode_tags(
    conn: &Connection,
    episode_id: &Uuid,
    tags: Option<&[String]>,
) -> Result<()>

pub async fn find_episodes_by_tags(
    conn: &Connection,
    tags: &[String],
    match_all: bool,
    limit: Option<usize>,
) -> Result<Vec<Uuid>>

pub async fn get_all_tags(conn: &Connection) -> Result<Vec<String>>

pub async fn get_tag_statistics(
    conn: &Connection,
) -> Result<HashMap<String, TagStats>>
```

**File Modified**: `do-memory-storage-turso/src/trait_impls/episodes.rs`

Integrate tag operations into `StorageBackend` trait implementation.

**Success Criteria**:
- ✅ All tag operations implemented
- ✅ Transactions used for atomicity
- ✅ SQL injection protection (parameterized queries)
- ✅ Error handling complete

#### 1.4 Redb Cache Integration (2 hours)
**File**: `do-memory-storage-redb/src/storage.rs`

```rust
// Tags automatically handled via Episode serialization (postcard)
// Add cache invalidation when tags change

async fn invalidate_episode_cache(&self, episode_id: &Uuid) -> Result<()>
```

**Success Criteria**:
- ✅ Tags cached with episodes
- ✅ Cache invalidation on tag changes
- ✅ Cache consistency maintained

#### 1.5 Storage Integration Tests (2 hours)
**New File**: `do-memory-storage-turso/tests/tag_integration_test.rs`

```rust
#[tokio::test]
async fn test_add_tags_to_episode()
async fn test_remove_tags_from_episode()
async fn test_replace_all_tags()
async fn test_query_episodes_by_single_tag()
async fn test_query_episodes_by_multiple_tags_or()
async fn test_query_episodes_by_multiple_tags_and()
async fn test_tag_statistics()
async fn test_concurrent_tag_operations()
async fn test_tag_persistence_after_restart()
```

**Success Criteria**:
- ✅ All integration tests pass
- ✅ Coverage >90%
- ✅ Edge cases covered

### Deliverables
- [x] `do-memory-core/src/episode/structs.rs` (modified)
- [ ] `do-memory-storage-turso/src/schema.rs` (modified)
- [ ] `do-memory-storage-turso/src/storage/tag_operations.rs` (new)
- [ ] `do-memory-storage-turso/src/trait_impls/episodes.rs` (modified)
- [ ] `do-memory-storage-redb/src/storage.rs` (modified)
- [ ] `do-memory-storage-turso/tests/tag_integration_test.rs` (new)

### Quality Gates
- [ ] All tests pass (>95%)
- [ ] Coverage >90%
- [ ] Zero clippy warnings
- [ ] All files <500 LOC
- [ ] SQL injection tests pass

---

## Phase 2: Core API Implementation

**Duration**: 8 hours  
**Status**: ⏳ Waiting for Phase 1  
**Dependencies**: Phase 1 complete

### Objectives
- Expose tag operations through high-level API
- Integrate tags into episode lifecycle
- Enable tag-based filtering in queries

### Tasks

#### 2.1 Tag Management API (3 hours)
**File**: `do-memory-core/src/memory/management.rs`

```rust
impl SelfLearningMemory {
    pub async fn add_episode_tags(
        &self,
        episode_id: Uuid,
        tags: Vec<String>,
    ) -> Result<()>
    
    pub async fn remove_episode_tags(
        &self,
        episode_id: Uuid,
        tags: Vec<String>,
    ) -> Result<()>
    
    pub async fn set_episode_tags(
        &self,
        episode_id: Uuid,
        tags: Vec<String>,
    ) -> Result<()>
    
    pub async fn get_episode_tags(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<String>>
}
```

**Success Criteria**:
- ✅ All CRUD operations implemented
- ✅ Cache invalidation on changes
- ✅ Error handling complete

#### 2.2 Tag Query API (3 hours)
**File**: `do-memory-core/src/memory/queries.rs`

```rust
pub async fn list_episodes_by_tags(
    storage: &Arc<dyn StorageBackend>,
    tags: Vec<String>,
    match_all: bool,
    limit: Option<usize>,
) -> Result<Vec<Episode>>

pub async fn get_all_tags(
    storage: &Arc<dyn StorageBackend>,
) -> Result<Vec<String>>

pub async fn get_tag_statistics(
    storage: &Arc<dyn StorageBackend>,
) -> Result<HashMap<String, TagStats>>
```

**File**: `do-memory-core/src/types/mod.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub tag: String,
    pub usage_count: usize,
    pub first_used: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
}
```

**Success Criteria**:
- ✅ Query functions implemented
- ✅ AND/OR logic works correctly
- ✅ Pagination support

#### 2.3 Core Integration Tests (2 hours)
**New File**: `do-memory-core/tests/tag_operations_test.rs`

```rust
#[tokio::test]
async fn test_end_to_end_tag_lifecycle()
async fn test_tag_filtering_with_retrieval()
async fn test_tag_statistics_accuracy()
async fn test_bulk_tag_operations()
async fn test_concurrent_tag_modifications()
```

**Success Criteria**:
- ✅ All integration tests pass
- ✅ Coverage >90%

### Deliverables
- [ ] `do-memory-core/src/memory/management.rs` (modified)
- [ ] `do-memory-core/src/memory/queries.rs` (modified)
- [ ] `do-memory-core/src/types/mod.rs` (modified)
- [ ] `do-memory-core/tests/tag_operations_test.rs` (new)

### Quality Gates
- [ ] All tests pass (>95%)
- [ ] Coverage >90%
- [ ] Zero clippy warnings
- [ ] API documentation complete

---

## Phase 3: MCP Server Integration

**Duration**: 10 hours  
**Status**: ⏳ Waiting for Phase 2  
**Dependencies**: Phase 2 complete

### Objectives
- Expose tag operations via MCP protocol
- Provide comprehensive tool schemas
- Enable tag operations from MCP clients

### Tasks

#### 3.1 MCP Tool Implementation (5 hours)
**New File**: `do-memory-mcp/src/server/tools/episode_tags.rs`

```rust
// Tool: add_episode_tags
pub async fn add_episode_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>

// Tool: remove_episode_tags
pub async fn remove_episode_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>

// Tool: set_episode_tags
pub async fn set_episode_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>

// Tool: get_episode_tags
pub async fn get_episode_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>

// Tool: list_all_tags
pub async fn list_all_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>

// Tool: search_episodes_by_tags
pub async fn search_episodes_by_tags_tool(
    memory: Arc<SelfLearningMemory>,
    args: Value,
) -> Result<Value>
```

**File Modified**: `do-memory-mcp/src/server/tools/mod.rs`

Register all 6 tag tools.

**Success Criteria**:
- ✅ All 6 tools implemented
- ✅ Input validation complete
- ✅ Error responses well-formatted

#### 3.2 MCP Tool Schemas (2 hours)
**File**: `do-memory-mcp/src/server/tools/episode_tags.rs`

Define JSON schemas for all tool inputs/outputs.

**Success Criteria**:
- ✅ All schemas complete
- ✅ Validation enforced
- ✅ Examples provided

#### 3.3 MCP Integration Tests (2 hours)
**New File**: `do-memory-mcp/tests/tag_tools_test.rs`

```rust
#[tokio::test]
async fn test_add_episode_tags_tool()
async fn test_remove_episode_tags_tool()
async fn test_search_by_tags_tool()
async fn test_tool_input_validation()
async fn test_tool_error_handling()
```

**Success Criteria**:
- ✅ All tests pass
- ✅ Coverage >90%

#### 3.4 MCP Documentation (1 hour)
**New File**: `do-memory-mcp/EPISODE_TAGS_TOOLS.md`

Complete documentation with examples for all 6 tools.

**Success Criteria**:
- ✅ Usage examples provided
- ✅ Error cases documented
- ✅ Schema reference complete

### Deliverables
- [ ] `do-memory-mcp/src/server/tools/episode_tags.rs` (new)
- [ ] `do-memory-mcp/src/server/tools/mod.rs` (modified)
- [ ] `do-memory-mcp/tests/tag_tools_test.rs` (new)
- [ ] `do-memory-mcp/EPISODE_TAGS_TOOLS.md` (new)

### Quality Gates
- [ ] All tests pass (>95%)
- [ ] Coverage >90%
- [ ] Zero clippy warnings
- [ ] Documentation complete

---

## Phase 4: CLI Integration

**Duration**: 8 hours  
**Status**: ⏳ Waiting for Phase 2  
**Dependencies**: Phase 2 complete (can run parallel with Phase 3)

### Objectives
- Provide user-friendly CLI commands for tag operations
- Enable bulk operations and automation
- Provide excellent UX with colored output

### Tasks

#### 4.1 CLI Command Implementation (4 hours)
**New File**: `do-memory-cli/src/commands/tag.rs`

```rust
// Command: tag add
pub async fn handle_tag_add(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    tags: Vec<String>,
) -> Result<()>

// Command: tag remove
pub async fn handle_tag_remove(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    tags: Vec<String>,
) -> Result<()>

// Command: tag set
pub async fn handle_tag_set(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    tags: Vec<String>,
) -> Result<()>

// Command: tag list
pub async fn handle_tag_list(
    memory: &SelfLearningMemory,
    sort_by: SortOption,
) -> Result<()>

// Command: tag search
pub async fn handle_tag_search(
    memory: &SelfLearningMemory,
    tags: Vec<String>,
    match_all: bool,
    limit: Option<usize>,
) -> Result<()>

// Command: tag show
pub async fn handle_tag_show(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
) -> Result<()>
```

**File Modified**: `do-memory-cli/src/commands/mod.rs`

Add tag command routing.

**Success Criteria**:
- ✅ All 6 commands implemented
- ✅ Colored output (success/error)
- ✅ Table formatting for lists

#### 4.2 CLI Tests (2 hours)
**New File**: `do-memory-cli/tests/tag_commands_test.rs`

```rust
#[tokio::test]
async fn test_tag_add_command()
async fn test_tag_remove_command()
async fn test_tag_search_command()
async fn test_tag_list_formatting()
async fn test_invalid_tag_input()
```

**Success Criteria**:
- ✅ All tests pass
- ✅ Coverage >90%

#### 4.3 CLI Documentation (2 hours)
**File Modified**: `do-memory-cli/CLI_USER_GUIDE.md`

Add comprehensive tag command documentation.

**Success Criteria**:
- ✅ All commands documented
- ✅ Examples provided
- ✅ Common workflows shown

### Deliverables
- [ ] `do-memory-cli/src/commands/tag.rs` (new)
- [ ] `do-memory-cli/src/commands/mod.rs` (modified)
- [ ] `do-memory-cli/tests/tag_commands_test.rs` (new)
- [ ] `do-memory-cli/CLI_USER_GUIDE.md` (modified)

### Quality Gates
- [ ] All tests pass (>95%)
- [ ] Coverage >90%
- [ ] Zero clippy warnings
- [ ] Documentation complete

---

## Phase 5: Documentation & Examples

**Duration**: 4 hours  
**Status**: ⏳ Waiting for Phases 2-4  
**Dependencies**: Phases 2, 3, 4 complete

### Objectives
- Comprehensive user documentation
- Working examples and demos
- Performance benchmarks
- CHANGELOG updates

### Tasks

#### 5.1 Usage Guide (1 hour)
**New File**: `do-memory-core/EPISODE_TAGGING_GUIDE.md`

Complete guide covering:
- Introduction and use cases
- Tag naming conventions
- Core API usage
- MCP tool usage
- CLI command usage
- Best practices
- Performance tips

**Success Criteria**:
- ✅ Guide complete
- ✅ Examples tested

#### 5.2 Example Code (1 hour)
**New File**: `examples/episode_tagging_demo.rs`

```rust
// Comprehensive demo showing:
// - Creating episodes with tags
// - Adding/removing tags
// - Searching by tags
// - Tag statistics
// - Bulk operations
```

**Success Criteria**:
- ✅ Example runs successfully
- ✅ Comments explain each step

#### 5.3 Performance Benchmarks (1 hour)
**New File**: `benches/tag_operations.rs`

```rust
// Benchmarks:
// - Add tags to episode
// - Query by single tag
// - Query by multiple tags
// - Get all tags
// - Tag statistics
// - Bulk operations
```

**Success Criteria**:
- ✅ All benchmarks run
- ✅ Meet performance targets

#### 5.4 CHANGELOG Update (1 hour)
**File Modified**: `CHANGELOG.md`

Add v0.1.13 entry with tag feature details.

**Success Criteria**:
- ✅ CHANGELOG updated
- ✅ Breaking changes noted (none expected)

### Deliverables
- [ ] `do-memory-core/EPISODE_TAGGING_GUIDE.md` (new)
- [ ] `examples/episode_tagging_demo.rs` (new)
- [ ] `benches/tag_operations.rs` (new)
- [ ] `CHANGELOG.md` (modified)

### Quality Gates
- [ ] All examples run
- [ ] Benchmarks meet targets
- [ ] Documentation complete

---

## Performance Targets

| Operation | Target (P95) | Measurement Method |
|-----------|-------------|-------------------|
| Add tags to episode | <20ms | `benches/tag_operations.rs` |
| Query episodes by tag | <50ms | `benches/tag_operations.rs` |
| Get all tags | <30ms | `benches/tag_operations.rs` |
| Tag statistics | <100ms | `benches/tag_operations.rs` |
| Bulk operations (100) | <500ms | `benches/tag_operations.rs` |

## Testing Coverage Targets

| Component | Target | Tracking |
|-----------|--------|----------|
| do-memory-core (tags) | >90% | `cargo tarpaulin` |
| do-memory-storage-turso (tags) | >90% | `cargo tarpaulin` |
| do-memory-mcp (tag tools) | >90% | `cargo tarpaulin` |
| do-memory-cli (tag commands) | >90% | `cargo tarpaulin` |
| Overall project | >90% | Maintain current 92.5% |

## Risk Mitigation

| Risk | Mitigation Status | Notes |
|------|------------------|-------|
| Database migration | ⏳ Pending | Test on copy of production data |
| Performance impact | ⏳ Pending | Benchmark early in Phase 1 |
| API compatibility | ✅ Addressed | All existing APIs unchanged |
| User adoption | ⏳ Pending | Good docs + examples |

## Release Checklist

### Pre-Release
- [ ] All phases complete
- [ ] All tests passing (>95%)
- [ ] Coverage >90%
- [ ] Zero clippy warnings
- [ ] All files <500 LOC
- [ ] Documentation complete
- [ ] Performance targets met
- [ ] Security review complete

### Release
- [ ] Version bump to v0.1.13
- [ ] CHANGELOG.md updated
- [ ] Git tag created
- [ ] Release notes published

### Post-Release
- [ ] Monitor adoption metrics
- [ ] Gather user feedback
- [ ] Address any issues
- [ ] Plan Phase 2 enhancements

---

**Last Updated**: 2026-01-27  
**Next Update**: After Phase 1 completion
