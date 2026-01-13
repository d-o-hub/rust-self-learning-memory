# Episode Management Guide

This guide covers episode lifecycle management operations including creation, retrieval, and deletion.

## Overview

Episodes represent complete task execution records in the memory system. Each episode captures:
- Task description and context
- Execution steps with timing and results
- Task outcome (success/failure)
- Extracted patterns and learnings

## Episode Lifecycle

### 1. Creating Episodes

```rust
use memory_core::{SelfLearningMemory, TaskContext, TaskType};

let memory = SelfLearningMemory::new();

let episode_id = memory.start_episode(
    "Implement user authentication".to_string(),
    TaskContext {
        domain: "web-api".to_string(),
        framework: Some("axum".to_string()),
        language: Some("rust".to_string()),
        tags: vec!["security".to_string(), "auth".to_string()],
        ..Default::default()
    },
    TaskType::CodeGeneration,
).await;
```

### 2. Logging Steps

```rust
use memory_core::{ExecutionStep, ExecutionResult};

let mut step = ExecutionStep::new(
    1,
    "code_generator".to_string(),
    "Generate auth middleware".to_string(),
);

step.result = Some(ExecutionResult::Success {
    output: "Created middleware.rs".to_string(),
});
step.latency_ms = 150;

memory.log_step(episode_id, step).await;
```

### 3. Completing Episodes

```rust
use memory_core::TaskOutcome;

memory.complete_episode(
    episode_id,
    TaskOutcome::Success {
        verdict: "Authentication implemented successfully".to_string(),
        artifacts: vec!["middleware.rs".to_string(), "auth_handler.rs".to_string()],
    },
).await?;
```

### 4. Retrieving Episodes

```rust
// Get a specific episode
let episode = memory.get_episode(episode_id).await?;

// List all episodes
let episodes = memory.get_all_episodes().await?;

// List with filters
let recent_episodes = memory.list_episodes(
    Some(10),    // limit
    Some(0),     // offset
    Some(true),  // completed_only
).await?;
```

### 5. Deleting Episodes

**⚠️ WARNING**: Episode deletion is permanent and cannot be undone.

```rust
// Delete an episode
memory.delete_episode(episode_id).await?;
```

**Use cases for deletion:**
- Removing test episodes
- Cleaning up failed/incomplete episodes
- GDPR/privacy compliance (data deletion requests)
- Storage space management

**What gets deleted:**
- Episode metadata and description
- All execution steps
- Associated patterns (if not referenced elsewhere)
- Episode embeddings
- Storage from all backends (cache + durable)

## CLI Usage

### Delete Episode Command

```bash
# Delete by episode ID
memory-cli episode delete <EPISODE_ID>

# Example
memory-cli episode delete 550e8400-e29b-41d4-a716-446655440000
```

**Output:**
```
✓ Successfully deleted episode: 550e8400-e29b-41d4-a716-446655440000
```

**Errors:**
- `Episode not found` - The episode doesn't exist
- `Storage error` - Failed to delete from storage backend

## MCP Tool Usage

The `delete_episode` tool is available through the MCP protocol:

```json
{
  "method": "tools/call",
  "params": {
    "name": "delete_episode",
    "arguments": {
      "episode_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"success\":true,\"episode_id\":\"550e8400-e29b-41d4-a716-446655440000\",\"message\":\"Episode deleted successfully\"}"
    }
  ]
}
```

## Storage Backend Behavior

### In-Memory Cache
- Episode removed immediately
- Step buffers cleared

### Cache Storage (redb)
- Episode deleted from cache table
- LRU cache entry removed
- Summaries deleted (if supported)

### Durable Storage (Turso)
- Episode deleted from episodes table
- Cascade deletes summaries
- FTS index updated automatically (via triggers)

## Error Handling

```rust
use memory_core::Error;

match memory.delete_episode(episode_id).await {
    Ok(()) => println!("Episode deleted successfully"),
    Err(Error::NotFound(id)) => println!("Episode {} not found", id),
    Err(Error::Storage(msg)) => println!("Storage error: {}", msg),
    Err(e) => println!("Unexpected error: {}", e),
}
```

## Future Features

### Episode Archival (Planned)
Archive episodes instead of deleting them permanently:

```rust
// Archive an episode (moves to archive storage)
memory.archive_episode(episode_id).await?;

// Restore archived episode
memory.restore_episode(episode_id).await?;

// List archived episodes
let archived = memory.list_archived_episodes().await?;
```

**Benefits:**
- Recoverable deletion
- Long-term storage in cheaper storage tier
- Compliance with retention policies
- Historical analysis without cluttering active memory

## Best Practices

1. **Think Before Deleting**: Episode deletion is permanent. Consider if you need the data for:
   - Future analysis
   - Pattern learning
   - Debugging similar tasks
   - Compliance/audit trails

2. **Batch Deletions**: For cleaning up multiple test episodes:
   ```rust
   let test_episodes = memory.list_episodes(None, None, None).await?;
   for episode in test_episodes.iter().filter(|e| e.task_description.contains("test")) {
       memory.delete_episode(episode.episode_id).await?;
   }
   ```

3. **Verify Before Deletion**: Always verify you're deleting the correct episode:
   ```rust
   let episode = memory.get_episode(episode_id).await?;
   println!("About to delete: {}", episode.task_description);
   // Confirm with user before proceeding
   memory.delete_episode(episode_id).await?;
   ```

4. **Storage Management**: Use deletion as part of capacity management:
   ```rust
   // Delete old, low-quality episodes to free space
   let old_episodes = memory.list_episodes(Some(100), None, Some(true)).await?;
   for episode in old_episodes.iter().filter(|e| should_delete(e)) {
       memory.delete_episode(episode.episode_id).await?;
   }
   ```

## Related Documentation

- [Episode Creation](memory-core/src/memory/episode.rs)
- [Pattern Extraction](memory-core/src/extraction/)
- [Storage Backends](memory-storage-redb/README.md)
- [CLI User Guide](memory-cli/CLI_USER_GUIDE.md)

## Examples

See comprehensive examples in:
- `memory-core/tests/episode_deletion_test.rs` - Integration tests
- `memory-cli/tests/command_tests.rs` - CLI command tests
