---
description: Test complete episode lifecycle with comprehensive validation
---

# Test Episode Lifecycle

Create comprehensive tests for episode creation, execution tracking, and completion.

## Test Coverage

1. **Episode Creation**
   - Unique ID generation
   - Context initialization
   - Task type validation

2. **Execution Logging**
   - Step recording
   - Tool usage tracking
   - Timestamp consistency

3. **Episode Completion**
   - Reward score calculation
   - Reflection generation
   - Storage persistence

4. **Error Handling**
   - Invalid episode IDs
   - Concurrent modifications
   - Storage failures

## Usage

```bash
# Test specific episode module
/test-episode --module memory-core::episode

# Test with verbose output
/test-episode --verbose

# Run specific test
/test-episode test_complete_episode_lifecycle
```

## Implementation

```rust
#[cfg(test)]
mod episode_lifecycle_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_episode_flow() {
        let memory = setup_test_memory().await;

        // Start episode
        let episode_id = memory.start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        ).await;

        // Log steps
        memory.log_execution_step(
            episode_id.clone(),
            ToolCall {
                tool_name: "bash".to_string(),
                input: "echo test".to_string(),
                output: Some("test".to_string()),
                duration_ms: 5,
            },
        ).await;

        // Complete episode
        memory.complete_episode(
            episode_id.clone(),
            TaskOutcome::Success,
            Some(vec!["Good work".to_string()]),
        ).await.unwrap();

        // Verify
        let episode = memory.get_episode(&episode_id).await.unwrap();
        assert_eq!(episode.outcome, TaskOutcome::Success);
    }
}
```

## Requirements

- Use `#[tokio::test]` for async tests
- Include property-based tests for ID uniqueness
- Mock storage backends for isolation
- Validate all state transitions
- Test concurrent episode operations
