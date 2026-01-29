# Episode Tagging MCP Tools

This document describes the MCP tools available for managing episode tags in the memory system.

## Overview

Episode tagging allows you to categorize and organize episodes using custom labels (tags). Tags are normalized strings that follow specific validation rules and enable efficient searching and filtering of episodes.

## Tag Validation Rules

- **Length**: 2-100 characters
- **Characters**: Alphanumeric, hyphens (-), and underscores (_) only
- **Normalization**: Automatically converted to lowercase and trimmed
- **Uniqueness**: Duplicate tags are automatically prevented
- **Case-Insensitive**: Tags are matched case-insensitively

### Valid Tag Examples
- `bug-fix`
- `critical`
- `feature-123`
- `performance_optimization`

### Invalid Tag Examples
- `a` (too short)
- `bug fix` (contains space)
- `bug@fix` (invalid character)

## Available Tools

### 1. Add Episode Tags

Add one or more tags to an episode without removing existing tags.

**Tool**: `add_episode_tags`

**Input**:
```json
{
  "episode_id": "uuid-string",
  "tags": ["tag1", "tag2", "tag3"]
}
```

**Output**:
```json
{
  "success": true,
  "episode_id": "uuid-string",
  "tags_added": 2,
  "current_tags": ["tag1", "tag2", "existing-tag"],
  "message": "Added 2 tag(s) to episode"
}
```

**Example Usage**:
```rust
let tools = EpisodeTagTools::new(memory);
let input = AddEpisodeTagsInput {
    episode_id: episode_id.to_string(),
    tags: vec!["bug-fix".to_string(), "critical".to_string()],
};
let output = tools.add_tags(input).await?;
```

**Notes**:
- Duplicate tags are silently ignored
- Invalid tags will cause an error
- Tags are normalized before adding

---

### 2. Remove Episode Tags

Remove one or more tags from an episode.

**Tool**: `remove_episode_tags`

**Input**:
```json
{
  "episode_id": "uuid-string",
  "tags": ["tag1", "tag2"]
}
```

**Output**:
```json
{
  "success": true,
  "episode_id": "uuid-string",
  "tags_removed": 2,
  "current_tags": ["remaining-tag"],
  "message": "Removed 2 tag(s) from episode"
}
```

**Example Usage**:
```rust
let input = RemoveEpisodeTagsInput {
    episode_id: episode_id.to_string(),
    tags: vec!["obsolete-tag".to_string()],
};
let output = tools.remove_tags(input).await?;
```

**Notes**:
- Non-existent tags are silently ignored
- Case-insensitive removal
- Returns count of actually removed tags

---

### 3. Set Episode Tags

Replace all existing tags with a new set of tags.

**Tool**: `set_episode_tags`

**Input**:
```json
{
  "episode_id": "uuid-string",
  "tags": ["new-tag1", "new-tag2"]
}
```

**Output**:
```json
{
  "success": true,
  "episode_id": "uuid-string",
  "tags_set": 2,
  "current_tags": ["new-tag1", "new-tag2"],
  "message": "Set 2 tag(s) on episode"
}
```

**Example Usage**:
```rust
let input = SetEpisodeTagsInput {
    episode_id: episode_id.to_string(),
    tags: vec!["refactor".to_string(), "performance".to_string()],
};
let output = tools.set_tags(input).await?;
```

**Notes**:
- All existing tags are removed first
- Useful for complete tag reorganization
- Empty tag list will clear all tags

---

### 4. Get Episode Tags

Retrieve all tags currently assigned to an episode.

**Tool**: `get_episode_tags`

**Input**:
```json
{
  "episode_id": "uuid-string"
}
```

**Output**:
```json
{
  "success": true,
  "episode_id": "uuid-string",
  "tags": ["tag1", "tag2", "tag3"],
  "message": "Found 3 tag(s)"
}
```

**Example Usage**:
```rust
let input = GetEpisodeTagsInput {
    episode_id: episode_id.to_string(),
};
let output = tools.get_tags(input).await?;
println!("Tags: {:?}", output.tags);
```

**Notes**:
- Returns empty array if no tags
- Tags are returned in the order they were added

---

### 5. Search Episodes by Tags

Find episodes that match specific tag criteria using AND or OR logic.

**Tool**: `search_episodes_by_tags`

**Input**:
```json
{
  "tags": ["tag1", "tag2"],
  "require_all": false,
  "limit": 10
}
```

**Parameters**:
- `tags`: Array of tags to search for (required)
- `require_all`: `true` for AND search, `false` for OR search (default: `false`)
- `limit`: Maximum number of results to return (default: `100`)

**Output**:
```json
{
  "success": true,
  "count": 2,
  "episodes": [
    {
      "episode_id": "uuid-1",
      "task_description": "Fix authentication bug",
      "task_type": "BugFix",
      "tags": ["bug-fix", "critical"],
      "start_time": 1706544000,
      "end_time": 1706547600,
      "outcome": "Success"
    }
  ],
  "search_criteria": "Any of: [bug-fix, critical]",
  "message": "Found 2 episode(s) matching tags"
}
```

**Example Usage - OR Search**:
```rust
// Find episodes with "bug-fix" OR "feature"
let input = SearchEpisodesByTagsInput {
    tags: vec!["bug-fix".to_string(), "feature".to_string()],
    require_all: Some(false),
    limit: Some(20),
};
let output = tools.search_by_tags(input).await?;
```

**Example Usage - AND Search**:
```rust
// Find episodes with both "bug-fix" AND "critical"
let input = SearchEpisodesByTagsInput {
    tags: vec!["bug-fix".to_string(), "critical".to_string()],
    require_all: Some(true),
    limit: Some(10),
};
let output = tools.search_by_tags(input).await?;
```

**Notes**:
- OR search (`require_all: false`): Returns episodes with ANY of the tags
- AND search (`require_all: true`): Returns episodes with ALL of the tags
- Case-insensitive matching
- Results are limited to prevent performance issues

---

## Common Use Cases

### Categorizing Episodes

```rust
// Tag a bug fix episode
memory.add_episode_tags(
    episode_id,
    vec!["bug-fix".to_string(), "critical".to_string(), "authentication".to_string()]
).await?;

// Tag a feature episode
memory.add_episode_tags(
    episode_id,
    vec!["feature".to_string(), "user-profile".to_string()]
).await?;
```

### Finding Related Episodes

```rust
// Find all critical bug fixes
let tools = EpisodeTagTools::new(memory);
let results = tools.search_by_tags(SearchEpisodesByTagsInput {
    tags: vec!["bug-fix".to_string(), "critical".to_string()],
    require_all: Some(true),
    limit: Some(50),
}).await?;

for episode in results.episodes {
    println!("Episode: {} - {}", episode.episode_id, episode.task_description);
}
```

### Reorganizing Tags

```rust
// Update episode categorization
tools.set_tags(SetEpisodeTagsInput {
    episode_id: episode_id.to_string(),
    tags: vec![
        "refactored".to_string(),
        "performance-improved".to_string(),
        "documented".to_string()
    ],
}).await?;
```

---

## Error Handling

All tools return a `Result` type and may fail in the following scenarios:

### Invalid Episode ID
```rust
let result = tools.add_tags(AddEpisodeTagsInput {
    episode_id: "not-a-uuid".to_string(),
    tags: vec!["test".to_string()],
}).await;

// Error: "Invalid episode ID: invalid character"
assert!(result.is_err());
```

### Episode Not Found
```rust
let result = tools.add_tags(AddEpisodeTagsInput {
    episode_id: Uuid::new_v4().to_string(),
    tags: vec!["test".to_string()],
}).await;

// Error: Episode not found in storage
assert!(result.is_err());
```

### Invalid Tag Format
```rust
let result = memory.add_episode_tags(
    episode_id,
    vec!["invalid tag".to_string()] // Space not allowed
).await;

// Error: Tag validation failed
assert!(result.is_err());
```

---

## Performance Considerations

### Search Performance
- Tag searches iterate through all episodes in memory
- For large datasets, consider adding database indexes on tags
- Use the `limit` parameter to control result size

### Tag Storage
- Tags are stored as part of the episode JSON
- Tags are persisted in both cache (redb) and durable storage (Turso)
- Tag updates trigger full episode serialization

### Best Practices
1. Keep tag counts reasonable (< 20 tags per episode)
2. Use consistent naming conventions for tags
3. Avoid very long tag names (keep under 50 characters)
4. Use hyphens or underscores instead of spaces
5. Consider using a controlled vocabulary for important tags

---

## Integration Examples

### CLI Integration

```rust
// Add tags via CLI
memory-cli tag add <episode-id> bug-fix critical

// Search by tags
memory-cli tag search --tags bug-fix,critical --all

// List episode tags
memory-cli tag list <episode-id>
```

### MCP Client Integration

```json
// JSON-RPC request to add tags
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "add_episode_tags",
    "arguments": {
      "episode_id": "550e8400-e29b-41d4-a716-446655440000",
      "tags": ["bug-fix", "critical"]
    }
  }
}
```

---

## Schema Reference

### EpisodeTagResult

Complete episode information returned from tag searches:

```rust
pub struct EpisodeTagResult {
    pub episode_id: String,
    pub task_description: String,
    pub task_type: String,
    pub tags: Vec<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub outcome: Option<String>,
}
```

---

## Testing

The episode tagging tools include comprehensive test coverage:

- `test_add_episode_tags` - Basic tag addition
- `test_add_duplicate_tags` - Duplicate prevention
- `test_remove_episode_tags` - Tag removal
- `test_set_episode_tags` - Tag replacement
- `test_get_episode_tags` - Tag retrieval
- `test_search_episodes_by_tags_any` - OR search logic
- `test_search_episodes_by_tags_all` - AND search logic
- `test_invalid_episode_id` - Error handling
- `test_empty_tags` - Edge cases

Run tests:
```bash
cargo test --package memory-mcp --lib episode_tags
```

---

## Future Enhancements

Potential improvements for future versions:

1. **Tag Analytics**
   - Most frequently used tags
   - Tag co-occurrence analysis
   - Tag usage over time

2. **Tag Suggestions**
   - Auto-suggest tags based on task description
   - Similar episode recommendations

3. **Tag Hierarchies**
   - Parent/child tag relationships
   - Tag namespaces (e.g., `project:auth`, `type:bug`)

4. **Tag Aliases**
   - Alternative names for the same concept
   - Deprecation warnings for old tags

5. **Bulk Operations**
   - Tag multiple episodes at once
   - Rename tags across all episodes
   - Merge similar tags

---

## See Also

- [Episode Management API](../memory-core/EPISODE_MANAGEMENT.md)
- [Memory Core Documentation](../memory-core/README.md)
- [MCP Protocol Specification](./README.md)
