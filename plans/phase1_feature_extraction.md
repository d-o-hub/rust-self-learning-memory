# Phase 1 Feature Extraction Report
## PR #265 Feature Documentation

**Extraction Date**: 2026-02-11  
**Branch**: pr-265  
**Status**: Complete

---

## Executive Summary

PR #265 introduces comprehensive relationship management and tag management features:

- **8 MCP Relationship Tools**: Full CRUD operations plus graph analysis
- **7 Standalone CLI Relationship Commands**: Top-level `relationship` subcommand
- **7 Episode CLI Relationship Commands**: Nested under `episode` subcommand
- **8 CLI Tag Commands**: Complete tag management system

Total: **30 commands/tools** across MCP and CLI interfaces.

---

## Section 1: MCP Relationship Tools (8 Tools)

### Tool Registration
**Location**: `memory-mcp/src/bin/server/handlers.rs:152-171`
**Tool Definitions**: `memory-mcp/src/server/tool_definitions_extended.rs:499-719`
**Handler Functions**: `memory-mcp/src/bin/server/tools.rs:829-1110`

### Tool 1: add_episode_relationship
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:58-108
Handler: memory-mcp/src/bin/server/tools.rs:830-869
Definition: memory-mcp/src/server/tool_definitions_extended.rs:501-539

Function Signature:
pub async fn add_relationship(
    &self,
    input: AddEpisodeRelationshipInput,
) -> Result<AddEpisodeRelationshipOutput>

Input Type: AddEpisodeRelationshipInput (types.rs:6-20)
  - from_episode_id: String (UUID)
  - to_episode_id: String (UUID)
  - relationship_type: String (enum: parent_child, depends_on, follows, related_to, blocks, duplicates, references)
  - reason: Option<String>
  - priority: Option<u8> (1-10)
  - created_by: Option<String>

Output Type: AddEpisodeRelationshipOutput (types.rs:23-37)
  - success: bool
  - relationship_id: String
  - from_episode_id: String
  - to_episode_id: String
  - relationship_type: String
  - message: String

Description: Creates a directed relationship between two episodes with validation.
Validates episode existence, prevents self-relationships, and checks for cycles.
```

### Tool 2: remove_episode_relationship
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:126-146
Handler: memory-mcp/src/bin/server/tools.rs:872-898
Definition: memory-mcp/src/server/tool_definitions_extended.rs:542-556

Function Signature:
pub async fn remove_relationship(
    &self,
    input: RemoveEpisodeRelationshipInput,
) -> Result<RemoveEpisodeRelationshipOutput>

Input Type: RemoveEpisodeRelationshipInput (types.rs:40-44)
  - relationship_id: String (UUID)

Output Type: RemoveEpisodeRelationshipOutput (types.rs:47-55)
  - success: bool
  - relationship_id: String
  - message: String

Description: Removes a relationship by its ID.
```

### Tool 3: get_episode_relationships
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:164-227
Handler: memory-mcp/src/bin/server/tools.rs:900-930
Definition: memory-mcp/src/server/tool_definitions_extended.rs:559-584

Function Signature:
pub async fn get_relationships(
    &self,
    input: GetEpisodeRelationshipsInput,
) -> Result<GetEpisodeRelationshipsOutput>

Input Type: GetEpisodeRelationshipsInput (types.rs:58-66)
  - episode_id: String (UUID)
  - direction: Option<String> ("outgoing", "incoming", "both")
  - relationship_type: Option<String> (type filter)

Output Type: GetEpisodeRelationshipsOutput (types.rs:90-104)
  - success: bool
  - episode_id: String
  - outgoing: Vec<RelationshipEdge>
  - incoming: Vec<RelationshipEdge>
  - total_count: usize
  - message: String

Description: Gets all relationships for an episode with optional direction and type filtering.
```

### Tool 4: find_related_episodes
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:241-320
Handler: memory-mcp/src/bin/server/tools.rs:932-962
Definition: memory-mcp/src/server/tool_definitions_extended.rs:587-616

Function Signature:
pub async fn find_related(
    &self,
    input: FindRelatedEpisodesInput,
) -> Result<FindRelatedEpisodesOutput>

Input Type: FindRelatedEpisodesInput (types.rs:107-117)
  - episode_id: String (UUID)
  - relationship_type: Option<String>
  - limit: Option<usize>
  - include_metadata: Option<bool>

Output Type: FindRelatedEpisodesOutput (types.rs:139-151)
  - success: bool
  - episode_id: String
  - related_episodes: Vec<RelatedEpisode>
  - count: usize
  - message: String

Description: Finds episodes related to the given episode with optional filtering.
```

### Tool 5: check_relationship_exists
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:332-369
Handler: memory-mcp/src/bin/server/tools.rs:964-996
Definition: memory-mcp/src/server/tool_definitions_extended.rs:619-642

Function Signature:
pub async fn check_exists(
    &self,
    input: CheckRelationshipExistsInput,
) -> Result<CheckRelationshipExistsOutput>

Input Type: CheckRelationshipExistsInput (types.rs:154-162)
  - from_episode_id: String (UUID)
  - to_episode_id: String (UUID)
  - relationship_type: String

Output Type: CheckRelationshipExistsOutput (types.rs:165-179)
  - success: bool
  - exists: bool
  - from_episode_id: String
  - to_episode_id: String
  - relationship_type: String
  - message: String

Description: Checks if a specific relationship exists between two episodes.
```

### Tool 6: get_dependency_graph
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:383-451
Handler: memory-mcp/src/bin/server/tools.rs:998-1029
Definition: memory-mcp/src/server/tool_definitions_extended.rs:645-672

Function Signature:
pub async fn get_dependency_graph(
    &self,
    input: DependencyGraphInput,
) -> Result<DependencyGraphOutput>

Input Type: DependencyGraphInput (types.rs:182-190)
  - episode_id: String (UUID)
  - depth: Option<usize> (1-5, default 2)
  - format: Option<String> ("json" or "dot")

Output Type: DependencyGraphOutput (types.rs:207-224)
  - success: bool
  - root: String
  - node_count: usize
  - edge_count: usize
  - nodes: Vec<RelationshipNode>
  - edges: Vec<RelationshipEdge>
  - dot: Option<String>
  - message: String

Description: Builds a relationship graph starting from an episode up to a specified depth.
```

### Tool 7: validate_no_cycles
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:463-540
Handler: memory-mcp/src/bin/server/tools.rs:1031-1072
Definition: memory-mcp/src/server/tool_definitions_extended.rs:675-698

Function Signature:
pub async fn validate_no_cycles(
    &self,
    input: ValidateNoCyclesInput,
) -> Result<ValidateNoCyclesOutput>

Input Type: ValidateNoCyclesInput (types.rs:227-235)
  - from_episode_id: String (UUID)
  - to_episode_id: String (UUID)
  - relationship_type: String

Output Type: ValidateNoCyclesOutput (types.rs:238-250)
  - success: bool
  - would_create_cycle: bool
  - is_valid: bool
  - cycle_path: Option<Vec<String>>
  - message: String

Description: Validates that adding a relationship would not create a cycle.
Only checks for acyclic relationship types.
```

### Tool 8: get_topological_order
```
Location: memory-mcp/src/mcp/tools/episode_relationships/tool.rs:554-655
Handler: memory-mcp/src/bin/server/tools.rs:1074-1110
Definition: memory-mcp/src/server/tool_definitions_extended.rs:701-719

Function Signature:
pub async fn get_topological_order(
    &self,
    input: GetTopologicalOrderInput,
) -> Result<GetTopologicalOrderOutput>

Input Type: GetTopologicalOrderInput (types.rs:253-257)
  - episode_ids: Vec<String> (UUIDs)

Output Type: GetTopologicalOrderOutput (types.rs:271-283)
  - success: bool
  - order: Vec<TopologicalEpisode>
  - count: usize
  - has_cycles: bool
  - message: String

Description: Returns episodes in topological order where dependencies come before dependents.
Only works on directed acyclic graphs (DAGs).
```

### Supporting Types (all in types.rs)
- `RelationshipEdge` (lines 69-87)
- `RelationshipNode` (lines 193-203)
- `RelatedEpisode` (lines 120-136)
- `TopologicalEpisode` (lines 260-268)

---

## Section 2: CLI Relationship Commands (7 Standalone Commands)

### Module Structure
```
memory-cli/src/commands/relationships/
├── mod.rs          # Command definitions (182 lines)
├── core.rs         # Implementation (602 lines)
└── types.rs        # Types and output (371 lines)
```

### Command Registration
**Location**: `memory-cli/src/main.rs:127-132`
```rust
#[command(alias = "rel")]
Relationship {
    #[command(subcommand)]
    command: crate::commands::relationships::StandaloneRelationshipCommands,
},
```

### Command 1: relationship add
```
Location: memory-cli/src/commands/relationships/mod.rs:23-53
Handler: memory-cli/src/commands/relationships/core.rs:14-89

Structure:
relationship add \
  --source <ID> \
  --target <ID> \
  --type <TYPE> \
  [--reason <REASON>] \
  [--priority <1-10>] \
  [--created-by <NAME>] \
  [--metadata <KEY=VALUE>...]

Aliases: create

Arguments:
  -s, --source: Source episode ID
  -t, --target: Target episode ID  
  -y, --type: Relationship type (enum)
  -r, --reason: Optional explanation
  -p, --priority: Priority 1-10
  -c, --created-by: Creator identifier
  -m, --metadata: Custom key=value pairs
```

### Command 2: relationship remove
```
Location: memory-cli/src/commands/relationships/mod.rs:55-61
Handler: memory-cli/src/commands/relationships/core.rs:91-115

Structure:
relationship remove <RELATIONSHIP_ID>

Aliases: delete

Arguments:
  relationship_id: Relationship UUID to remove
```

### Command 3: relationship list
```
Location: memory-cli/src/commands/relationships/mod.rs:63-81
Handler: memory-cli/src/commands/relationships/core.rs:117-181

Structure:
relationship list \
  --episode <ID> \
  [--direction <outgoing|incoming|both>] \
  [--type <TYPE>] \
  [--format <table|json>]

Aliases: ls

Arguments:
  -e, --episode: Episode ID to list relationships for
  -d, --direction: Direction filter (default: both)
  -t, --type: Filter by relationship type
  -f, --format: Output format (default: table)
```

### Command 4: relationship find
```
Location: memory-cli/src/commands/relationships/mod.rs:83-105
Handler: memory-cli/src/commands/relationships/core.rs:183-270

Structure:
relationship find \
  --episode <ID> \
  [--types <TYPE>...] \
  [--max-depth <N>] \
  [--limit <N>] \
  [--format <table|json>]

Aliases: search

Arguments:
  -e, --episode: Episode ID to find related episodes for
  -t, --types: Filter by relationship types (multiple)
  -m, --max-depth: Max traversal depth (default: 3)
  -l, --limit: Max results (default: 50)
  -f, --format: Output format (default: table)
```

### Command 5: relationship info
```
Location: memory-cli/src/commands/relationships/mod.rs:107-113
Handler: memory-cli/src/commands/relationships/core.rs:272-285

Structure:
relationship info <RELATIONSHIP_ID>

Aliases: show

Arguments:
  relationship_id: Relationship ID to get info for

Note: Currently not fully implemented - directs users to use list command.
```

### Command 6: relationship graph
```
Location: memory-cli/src/commands/relationships/mod.rs:115-133
Handler: memory-cli/src/commands/relationships/core.rs:287-325

Structure:
relationship graph \
  --episode <ID> \
  [--max-depth <N>] \
  [--format <dot|json|text>] \
  [--output <FILE>]

Aliases: viz

Arguments:
  -e, --episode: Root episode ID
  -m, --max-depth: Max traversal depth (default: 3)
  -f, --format: Output format (default: text)
  -o, --output: Output file (defaults to stdout)
```

### Command 7: relationship validate
```
Location: memory-cli/src/commands/relationships/mod.rs:135-146
Handler: memory-cli/src/commands/relationships/core.rs:405-453

Structure:
relationship validate \
  [--episode <ID>] \
  [--type <TYPE>]

Aliases: check

Arguments:
  -e, --episode: Episode ID to start validation (required)
  -t, --type: Relationship type to check
```

### Supporting Types (types.rs)
- `RelationshipTypeArg` (lines 11-42): CLI argument enum for relationship types
- `DirectionArg` (lines 45-64): CLI argument enum for direction
- `ListFormat` (lines 67-73): Table/JSON output format
- `GraphFormat` (lines 76-84): DOT/JSON/Text format
- Output structs: AddResult, RemoveResult, ListResult, FindResult, GraphResult, ValidateResult

---

## Section 3: CLI Episode Relationship Commands (7 Commands)

### Module Structure
```
memory-cli/src/commands/episode/relationships/
├── mod.rs          # Implementation (427 lines)
├── helpers.rs      # Helper functions
└── types.rs        # Types and output (492 lines)
```

### Command Registration
**Location**: `memory-cli/src/commands/mod.rs:173-176`
```rust
EpisodeCommands::Relationships(cmd) => {
    handle_relationships_command(cmd, memory, config, format, dry_run).await
}
```

### Commands (defined in types.rs)
All nested under `episode relationships`:

1. **add-relationship**: Add relationship between episodes
2. **remove-relationship**: Remove relationship by ID
3. **list-relationships**: List relationships for episode
4. **find-related**: Find related episodes
5. **dependency-graph**: Generate dependency graph
6. **validate-cycles**: Check for cycles
7. **topological-sort**: Get topological ordering

### Key Differences from Standalone Commands:
- Uses `kebab-case` naming (e.g., `add-relationship` vs `add`)
- Different argument styles (long args only, no short forms)
- Simpler output formats
- Part of episode command namespace

---

## Section 4: CLI Tag Commands (8 Commands)

### Module Structure
```
memory-cli/src/commands/tag/
├── mod.rs          # Module exports (20 lines)
├── core.rs         # Implementation (696 lines)
├── types.rs        # Types and enums (233 lines)
├── output.rs       # Output formatting (376 lines)
└── tests.rs        # Unit tests
```

### Command Registration
**Location**: `memory-cli/src/main.rs:121-126`
```rust
#[command(alias = "tg")]
Tag {
    #[command(subcommand)]
    command: TagCommands,
},
```

### Command 1: tag add
```
Location: memory-cli/src/commands/tag/types.rs:8-22
Handler: memory-cli/src/commands/tag/core.rs:74-136

Structure:
tag add <EPISODE_ID> <TAG>... [--color <COLOR>]

Arguments:
  episode_id: Episode to add tags to
  tags: One or more tags to add
  --color: Tag color (red, green, blue, yellow, orange, purple, pink, cyan, gray)

Output: TagAddResult (types.rs:115-121)
```

### Command 2: tag remove
```
Location: memory-cli/src/commands/tag/types.rs:24-33
Handler: memory-cli/src/commands/tag/core.rs:138-181

Structure:
tag remove <EPISODE_ID> <TAG>...

Arguments:
  episode_id: Episode to remove tags from
  tags: One or more tags to remove

Output: TagRemoveResult (types.rs:124-130)
```

### Command 3: tag set
```
Location: memory-cli/src/commands/tag/types.rs:35-44
Handler: memory-cli/src/commands/tag/core.rs:209-243

Structure:
tag set <EPISODE_ID> <TAG>...

Arguments:
  episode_id: Episode to set tags on
  tags: Tags to set (replaces all existing)

Output: TagSetResult (types.rs:133-139)
```

### Command 4: tag list
```
Location: memory-cli/src/commands/tag/types.rs:46-55
Handler: memory-cli/src/commands/tag/core.rs:183-207 (episode), 246-317 (all)

Structure:
tag list [--episode <EPISODE_ID>] [--sort-by <count|name|recent>]

Arguments:
  --episode: Specific episode (omit for all tags system-wide)
  --sort-by: Sort order (default: name)

Outputs:
  - TagListResult (types.rs:142-147) for episode
  - TagStatsResult (types.rs:149-165) for all tags
```

### Command 5: tag search
```
Location: memory-cli/src/commands/tag/types.rs:57-78
Handler: memory-cli/src/commands/tag/core.rs:320-429

Structure:
tag search <TAG>... [--all] [--partial] [--case-sensitive] [--limit <N>]

Arguments:
  tags: Tags to search for
  --all: Use AND logic (all tags must match)
  --partial: Enable partial matching (substring search)
  --case-sensitive: Enable case-sensitive matching
  -l, --limit: Max results (default: 10)

Output: TagSearchResult (types.rs:167-185)
```

### Command 6: tag show
```
Location: memory-cli/src/commands/tag/types.rs:80-85
Handler: memory-cli/src/commands/tag/core.rs:431-472

Structure:
tag show <EPISODE_ID>

Arguments:
  episode_id: Episode to show with tags

Output: TagShowResult (types.rs:188-199)
```

### Command 7: tag rename
```
Location: memory-cli/src/commands/tag/types.rs:87-100
Handler: memory-cli/src/commands/tag/core.rs:474-564

Structure:
tag rename <OLD_TAG> <NEW_TAG> [--dry-run]

Arguments:
  old_tag: Current tag name
  new_tag: New tag name
  --dry-run: Show what would change without modifying

Output: TagRenameResult (types.rs:202-208)
```

### Command 8: tag stats
```
Location: memory-cli/src/commands/tag/types.rs:102-112
Handler: memory-cli/src/commands/tag/core.rs:566-689

Structure:
tag stats [--top <N>] [--sort <count|name|recent>]

Arguments:
  -t, --top: Show only top N tags
  -s, --sort: Sort order (default: count)

Output: TagStatsDetailedResult (types.rs:211-232)
```

---

## Section 5: Dependencies and Types Required

### From memory-core

#### Types
```rust
// Episode relationship types
memory_core::episode::RelationshipType  // Enum: ParentChild, DependsOn, Follows, RelatedTo, Blocks, Duplicates, References
memory_core::episode::RelationshipMetadata  // Struct with reason, created_by, priority, custom_fields
memory_core::episode::EpisodeRelationship   // Struct representing a relationship
memory_core::episode::Direction  // Enum: Outgoing, Incoming, Both

// Graph algorithms
memory_core::episode::graph_algorithms::has_path_dfs
memory_core::episode::graph_algorithms::find_path_dfs
memory_core::episode::graph_algorithms::has_cycle
memory_core::episode::graph_algorithms::topological_sort

// Query types
memory_core::memory::relationship_query::RelationshipFilter
memory_core::memory::relationship_query::RelationshipGraph
memory_core::memory::relationship_query::RelationshipGraph::to_dot
memory_core::memory::relationship_query::RelationshipGraph::to_json

// Main memory system
memory_core::SelfLearningMemory
```

#### Memory Methods Used
```rust
// Relationship operations
memory.add_episode_relationship(from_id, to_id, rel_type, metadata) -> Result<Uuid>
memory.remove_episode_relationship(rel_id) -> Result<()>
memory.get_episode_relationships(ep_id, direction) -> Result<Vec<EpisodeRelationship>>
memory.find_related_episodes(ep_id, filter) -> Result<Vec<Uuid>>
memory.relationship_exists(from_id, to_id, rel_type) -> Result<bool>
memory.build_relationship_graph(ep_id, depth) -> Result<RelationshipGraph>

// Tag operations
memory.get_episode_tags(ep_id) -> Result<Vec<String>>
memory.add_episode_tags(ep_id, tags) -> Result<()>
memory.remove_episode_tags(ep_id, tags) -> Result<()>
memory.set_episode_tags(ep_id, tags) -> Result<()>
memory.get_tag_statistics() -> Result<HashMap<String, TagStatistics>>

// Episode operations
memory.get_episode(ep_id) -> Result<Episode>
memory.get_all_episodes() -> Result<Vec<Episode>>
```

### External Dependencies
```toml
# For MCP tools
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
anyhow = "1"

# For CLI commands
clap = { version = "4", features = ["derive", "env"] }
colored = "2"
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
```

---

## Section 6: Implementation Checklist

### MCP Relationship Tools
- [ ] `memory-mcp/src/mcp/tools/episode_relationships/mod.rs` (19 lines)
- [ ] `memory-mcp/src/mcp/tools/episode_relationships/types.rs` (284 lines)
- [ ] `memory-mcp/src/mcp/tools/episode_relationships/tool.rs` (679 lines)
- [ ] `memory-mcp/src/mcp/tools/episode_relationships/tests.rs` (existing tests)
- [ ] `memory-mcp/src/mcp/tools/mod.rs` - Add module export
- [ ] `memory-mcp/src/bin/server/tools.rs` - Add 8 handler functions (lines 829-1110)
- [ ] `memory-mcp/src/bin/server/handlers.rs` - Add tool routing (lines 152-171)
- [ ] `memory-mcp/src/server/tool_definitions_extended.rs` - Add 8 tool definitions (lines 499-719)

### CLI Standalone Relationship Commands
- [ ] `memory-cli/src/commands/relationships/mod.rs` (182 lines)
- [ ] `memory-cli/src/commands/relationships/core.rs` (602 lines)
- [ ] `memory-cli/src/commands/relationships/types.rs` (371 lines)
- [ ] `memory-cli/src/commands/mod.rs` - Add module export and handler
- [ ] `memory-cli/src/main.rs` - Add Relationship command variant (lines 127-132)

### CLI Episode Relationship Commands
- [ ] `memory-cli/src/commands/episode/relationships/mod.rs` (427 lines)
- [ ] `memory-cli/src/commands/episode/relationships/types.rs` (492 lines)
- [ ] `memory-cli/src/commands/episode/relationships/helpers.rs`
- [ ] `memory-cli/src/commands/episode/mod.rs` - Add relationships module
- [ ] `memory-cli/src/commands/mod.rs` - Add handler function

### CLI Tag Commands
- [ ] `memory-cli/src/commands/tag/mod.rs` (20 lines)
- [ ] `memory-cli/src/commands/tag/core.rs` (696 lines)
- [ ] `memory-cli/src/commands/tag/types.rs` (233 lines)
- [ ] `memory-cli/src/commands/tag/output.rs` (376 lines)
- [ ] `memory-cli/src/commands/tag/tests.rs`
- [ ] `memory-cli/src/commands/mod.rs` - Add module export
- [ ] `memory-cli/src/main.rs` - Add Tag command variant (lines 121-126)

---

## Section 7: Porting Notes for PR #272

### Structural Considerations

1. **Module Organization**: 
   - PR #265 uses flat structure for `episode_relationships` in MCP tools
   - Ensure this aligns with any module reorganization in PR #272

2. **Type Definitions**:
   - All types use `serde::{Deserialize, Serialize}` derives
   - Ensure feature flags are compatible with PR #272's feature gating

3. **Error Handling**:
   - Uses `anyhow::Result` throughout
   - Custom error types from `memory_core::error::relationship` may need updating

4. **Audit Logging**:
   - MCP tools integrate with audit logger at `memory-mcp/src/bin/server/tools.rs`
   - Ensure audit methods exist:
     - `log_add_relationship`
     - `log_remove_relationship`
     - `log_get_relationships`
     - `log_find_related`
     - `log_check_relationship`
     - `log_dependency_graph`
     - `log_validate_cycles`
     - `log_topological_order`

5. **Handler Registration**:
   - Tools are registered in `handlers.rs` match statement
   - Tool names must match definitions in `tool_definitions_extended.rs`

### Potential Conflicts

1. **Command Names**:
   - `relationship` command may conflict with existing commands in PR #272
   - Consider merging or renaming if conflicts exist

2. **Type Name Collisions**:
   - `RelationshipType` exists in `memory_core` - ensure consistent usage
   - CLI uses `RelationshipTypeArg` wrapper for clap ValueEnum

3. **Graph Algorithm APIs**:
   - Uses `memory_core::episode::graph_algorithms::*`
   - Verify these functions exist and have compatible signatures in PR #272

### Testing Requirements

1. **MCP Tool Tests**:
   - Located in `memory-mcp/src/mcp/tools/episode_relationships/tests.rs`
   - Tests use mock memory and verify tool outputs

2. **CLI Tests**:
   - Unit tests in `memory-cli/src/commands/tag/tests.rs`
   - Integration tests may be needed for relationship commands

### Build Integration

1. **Feature Flags**:
   - Ensure relationship features are properly gated if needed
   - Tag commands likely always available (no special features)

2. **Dependencies**:
   - Verify all external crates are in Cargo.toml
   - Check for version compatibility with PR #272

---

## Summary Statistics

| Category | Count | Files | Lines |
|----------|-------|-------|-------|
| MCP Relationship Tools | 8 | 4 | ~1000 |
| CLI Relationship Commands (standalone) | 7 | 3 | ~1155 |
| CLI Relationship Commands (episode) | 7 | 3 | ~925 |
| CLI Tag Commands | 8 | 5 | ~1325 |
| **Total** | **30** | **15** | **~4405** |

---

*End of Feature Extraction Report*
