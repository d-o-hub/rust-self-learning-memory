# Episode Tags & Labels Feature Specification

**Feature ID**: FEAT-2026-001  
**Status**: Planning → Implementation  
**Priority**: High Impact  
**Target Version**: v0.1.14  
**Created**: 2026-01-27  
**Owner**: Development Team

## Executive Summary

Implement a comprehensive tagging system for episodes that enables better organization, filtering, and retrieval across all system components (memory-core, memory-mcp, memory-cli, and storage backends).

### Business Value
- **Organization**: Better categorization of episodes by type, priority, or domain
- **Discovery**: Fast tag-based search and filtering
- **Analytics**: Track patterns and success rates by category
- **Collaboration**: Shared taxonomies for team-based memory systems

### Key Metrics
- **Adoption Target**: 50%+ of episodes tagged within first month
- **Performance**: All operations <50ms (P95)
- **Quality**: >90% test coverage, zero critical bugs in first 2 weeks

## Feature Requirements

### Functional Requirements

#### FR-1: Tag Management
- **FR-1.1**: Users can add one or more tags to any episode
- **FR-1.2**: Users can remove tags from episodes
- **FR-1.3**: Users can replace all tags on an episode
- **FR-1.4**: Tags are case-insensitive and normalized
- **FR-1.5**: Tag names support alphanumeric, hyphens, and underscores

#### FR-2: Tag Queries
- **FR-2.1**: Search episodes by single tag
- **FR-2.2**: Search episodes by multiple tags (AND/OR logic)
- **FR-2.3**: List all available tags in the system
- **FR-2.4**: Get tag usage statistics (count, first used, last used)
- **FR-2.5**: Get all tags for a specific episode

#### FR-3: Integration Points
- **FR-3.1**: Core API methods for tag operations
- **FR-3.2**: MCP tools for tag management
- **FR-3.3**: CLI commands for tag operations
- **FR-3.4**: Storage backend support (Turso + redb)

#### FR-4: User Experience
- **FR-4.1**: Tags display in episode lists and details
- **FR-4.2**: Tag autocomplete suggestions (CLI)
- **FR-4.3**: Tag validation with helpful error messages
- **FR-4.4**: Bulk tag operations for efficiency

### Non-Functional Requirements

#### NFR-1: Performance
- **NFR-1.1**: Add tags to episode: <20ms (P95)
- **NFR-1.2**: Query episodes by tag: <50ms (P95)
- **NFR-1.3**: Get all tags: <30ms (P95)
- **NFR-1.4**: Tag statistics: <100ms (P95)
- **NFR-1.5**: Bulk operations (100 episodes): <500ms (P95)

#### NFR-2: Scalability
- **NFR-2.1**: Support 10,000+ unique tags
- **NFR-2.2**: Support 100,000+ tag associations
- **NFR-2.3**: No performance degradation with large tag sets

#### NFR-3: Reliability
- **NFR-3.1**: Tag operations are atomic (all-or-nothing)
- **NFR-3.2**: Cache consistency between Turso and redb
- **NFR-3.3**: Graceful degradation if tag system unavailable
- **NFR-3.4**: No data loss during tag operations

#### NFR-4: Maintainability
- **NFR-4.1**: >90% test coverage for new code
- **NFR-4.2**: All files <500 LOC
- **NFR-4.3**: Comprehensive documentation
- **NFR-4.4**: Zero clippy warnings

## Technical Design

### Data Model

#### Episode Structure
```rust
pub struct Episode {
    pub episode_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: EpisodeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    
    // NEW: Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}
```

#### Database Schema (Turso)
```sql
-- Episode tags (many-to-many relationship)
CREATE TABLE IF NOT EXISTS episode_tags (
    episode_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (episode_id, tag),
    FOREIGN KEY (episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_episode_tags_tag 
ON episode_tags(tag);

CREATE INDEX IF NOT EXISTS idx_episode_tags_episode 
ON episode_tags(episode_id);

-- Tag metadata and statistics
CREATE TABLE IF NOT EXISTS tag_metadata (
    tag TEXT PRIMARY KEY NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    first_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

### API Design

#### Core API (memory-core)
```rust
// Tag management
pub async fn add_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()>
pub async fn remove_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()>
pub async fn set_episode_tags(&self, episode_id: Uuid, tags: Vec<String>) -> Result<()>
pub async fn get_episode_tags(&self, episode_id: Uuid) -> Result<Vec<String>>

// Tag queries
pub async fn list_episodes_by_tags(
    &self,
    tags: Vec<String>,
    match_all: bool,
    limit: Option<usize>,
) -> Result<Vec<Episode>>

pub async fn get_all_tags(&self) -> Result<Vec<String>>
pub async fn get_tag_statistics(&self) -> Result<HashMap<String, TagStats>>
```

#### MCP Tools
```json
{
  "name": "add_episode_tags",
  "description": "Add one or more tags to an episode",
  "inputSchema": {
    "type": "object",
    "properties": {
      "episode_id": {"type": "string", "format": "uuid"},
      "tags": {"type": "array", "items": {"type": "string"}}
    },
    "required": ["episode_id", "tags"]
  }
}
```

#### CLI Commands
```bash
# Add tags
memory-cli tag add <episode-id> <tag1> [tag2...]

# Remove tags
memory-cli tag remove <episode-id> <tag1> [tag2...]

# Set tags (replace all)
memory-cli tag set <episode-id> <tag1> [tag2...]

# List all tags with statistics
memory-cli tag list [--sort-by count|name|recent]

# Search episodes by tags
memory-cli tag search <tag1> [tag2...] [--match-all] [--limit N]

# Show tags for episode
memory-cli tag show <episode-id>
```

## Implementation Phases

### Phase 1: Core Data Model & Storage (12 hours)
**Goal**: Establish data persistence layer

**Tasks**:
1. Update Episode struct with tags field
2. Add database schema migrations
3. Implement storage backend methods (Turso)
4. Update redb cache handling
5. Write storage unit tests
6. Write integration tests

**Deliverables**:
- Modified: `memory-core/src/episode/structs.rs`
- Modified: `memory-storage-turso/src/schema.rs`
- New: `memory-storage-turso/src/storage/tag_operations.rs`
- New: `memory-storage-turso/tests/tag_integration_test.rs`

**Success Criteria**:
- ✅ Tags persist in Turso database
- ✅ Tags cached in redb
- ✅ All storage tests pass (>95%)

### Phase 2: Core API Implementation (8 hours)
**Goal**: Expose tag operations through core API

**Tasks**:
1. Add tag management methods to SelfLearningMemory
2. Add tag query functions
3. Integrate tags into episode lifecycle
4. Update episode filtering
5. Write API integration tests

**Deliverables**:
- Modified: `memory-core/src/memory/management.rs`
- Modified: `memory-core/src/memory/queries.rs`
- New: `memory-core/tests/tag_operations_test.rs`

**Success Criteria**:
- ✅ Tag CRUD operations work
- ✅ Tag-based filtering works
- ✅ All core tests pass (>95%)

### Phase 3: MCP Server Integration (10 hours)
**Goal**: Enable tag operations via MCP protocol

**Tasks**:
1. Create episode_tags tool module
2. Implement 6 MCP tag tools
3. Add tool schemas and validation
4. Write MCP integration tests
5. Update MCP documentation

**Deliverables**:
- New: `memory-mcp/src/server/tools/episode_tags.rs`
- Modified: `memory-mcp/src/server/tools/mod.rs`
- New: `memory-mcp/tests/tag_tools_test.rs`
- New: `memory-mcp/EPISODE_TAGS_TOOLS.md`

**Success Criteria**:
- ✅ All 6 MCP tools operational
- ✅ All MCP tests pass (>95%)
- ✅ Documentation complete

### Phase 4: CLI Integration (8 hours)
**Goal**: Provide user-friendly CLI commands

**Tasks**:
1. Create tag command module
2. Implement 6 CLI tag commands
3. Add command parsing and validation
4. Add colored output and formatting
5. Write CLI tests
6. Update CLI documentation

**Deliverables**:
- New: `memory-cli/src/commands/tag.rs`
- Modified: `memory-cli/src/commands/mod.rs`
- New: `memory-cli/tests/tag_commands_test.rs`
- Modified: `memory-cli/CLI_USER_GUIDE.md`

**Success Criteria**:
- ✅ All CLI commands work
- ✅ User-friendly output
- ✅ All CLI tests pass (>95%)

### Phase 5: Documentation & Examples (4 hours)
**Goal**: Comprehensive user and developer documentation

**Tasks**:
1. Write usage guide
2. Create example workflows
3. Add API documentation
4. Update CHANGELOG
5. Add performance benchmarks

**Deliverables**:
- New: `memory-core/EPISODE_TAGGING_GUIDE.md`
- New: `examples/episode_tagging_demo.rs`
- Modified: `CHANGELOG.md`
- New: `benches/tag_operations.rs`

**Success Criteria**:
- ✅ Complete documentation
- ✅ Working examples
- ✅ Performance benchmarks run

## Testing Strategy

### Unit Tests
- Episode struct tag methods
- Tag normalization and validation
- Storage backend operations
- Cache invalidation logic

### Integration Tests
- End-to-end tag lifecycle
- Tag-based episode filtering
- Tag statistics and aggregation
- Multi-backend consistency (Turso + redb)
- MCP tool operations
- CLI command execution

### Performance Tests
- Tag query performance (<50ms target)
- Bulk tag operations (1000 tags/sec)
- Large dataset filtering (10k+ episodes)
- Cache hit/miss ratios

### Edge Cases
- Empty tag lists
- Duplicate tags
- Invalid tag characters
- Unicode tag names
- Very long tag names (>100 chars)
- Concurrent tag modifications

## Migration Strategy

### Backwards Compatibility
1. **Existing Episodes**: Auto-initialize with empty tags array
2. **Storage Migration**: Add tables without breaking existing data
3. **API Compatibility**: All existing APIs remain unchanged
4. **Version Support**: Feature detection for older clients

### Migration Steps
1. Deploy database schema changes (backwards compatible)
2. Deploy core API with tags support
3. Deploy MCP and CLI updates
4. Monitor adoption and performance
5. Optimize based on usage patterns

## Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| Database migration fails | High | Low | Thorough testing, rollback plan |
| Performance degradation | Medium | Low | Early benchmarking, proper indexes |
| Tag validation too strict | Low | Medium | User feedback, iterative refinement |
| Storage backend inconsistency | Medium | Low | Comprehensive integration tests |
| User adoption low | Low | Medium | Good UX, examples, documentation |

## Success Metrics

### Adoption Metrics
- 50%+ of episodes tagged within first month
- Average 2-3 tags per episode
- <5% of episodes with validation errors

### Performance Metrics
- All operations meet P95 targets
- <1% cache miss rate
- Zero performance regressions in existing operations

### Quality Metrics
- >90% test coverage maintained
- Zero critical bugs in first 2 weeks
- Zero clippy warnings
- All files <500 LOC

### User Satisfaction
- Positive feedback on CLI/MCP UX
- <10 support tickets in first month
- Feature requests indicate engagement

## Future Enhancements

### Phase 2 Features (v0.2.x)
1. **Tag Hierarchies**: Support categories (e.g., `type:bug-fix`, `priority:high`)
2. **Tag Aliases**: Synonym support and normalization
3. **Tag Colors**: Visual categorization in CLI/UI
4. **Tag Templates**: Pre-defined tag sets for common workflows

### Phase 3 Features (v0.3.x)
1. **Smart Tag Suggestions**: ML-based recommendations
2. **Tag-based Analytics**: Success rates and patterns by tag
3. **Tag Collaboration**: Share taxonomies across teams
4. **Tag History**: Track tag changes over time

## Dependencies

### Internal
- memory-core: Episode struct, storage traits
- memory-storage-turso: Database operations
- memory-storage-redb: Cache operations
- memory-mcp: MCP protocol handlers
- memory-cli: Command parsing and execution

### External
- sqlx: Database queries
- serde: Serialization
- uuid: Episode identifiers
- chrono: Timestamps
- clap: CLI argument parsing

## Rollout Plan

### Week 1
- Complete Phase 1: Core Data Model & Storage
- Complete Phase 2: Core API Implementation
- Internal testing and validation

### Week 2
- Complete Phase 3: MCP Server Integration
- Complete Phase 4: CLI Integration
- Integration testing across all components

### Week 3
- Complete Phase 5: Documentation & Examples
- Performance benchmarking
- Security review
- Release v0.1.13 with tagging feature

### Week 4
- Monitor adoption and performance
- Gather user feedback
- Address any issues
- Plan Phase 2 enhancements

## Approval & Sign-off

**Approved By**: Development Team  
**Date**: 2026-01-27  
**Version**: 1.0

---

**Document Status**: ✅ Approved - Ready for Implementation  
**Next Steps**: Begin Phase 1 implementation
