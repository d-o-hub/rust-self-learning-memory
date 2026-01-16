# Episode Lifecycle Test Patterns

## Episode States

```
Created -> InProgress -> Completed
                |
                v
          Failed/Aborted
```

## Test Helper Functions

```rust
async fn create_test_episode(
    memory: &SelfLearningMemory,
    task_type: TaskType,
) -> String {
    memory.start_episode(
        format!("Test episode for {:?}", task_type),
        TaskContext::default(),
        task_type,
    ).await
}

async fn create_episode_with_steps(
    memory: &SelfLearningMemory,
    step_count: usize,
) -> String {
    let episode_id = create_test_episode(
        memory,
        TaskType::CodeGeneration,
    ).await;

    for i in 0..step_count {
        memory.log_execution_step(
            episode_id.clone(),
            ToolCall {
                tool_name: format!("tool_{}", i),
                input: format!("input_{}", i),
                output: Some(format!("output_{}", i)),
                duration_ms: (i + 1) * 10,
            },
        ).await;
    }

    episode_id
}
```

## State Transition Tests

```rust
#[tokio::test]
async fn test_episode_state_transitions() {
    let memory = setup_memory().await;

    // Initial state: Created
    let episode_id = memory.start_episode(
        "Test".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    ).await;

    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.status, EpisodeStatus::Created);

    // After first step: InProgress
    memory.log_execution_step(
        episode_id.clone(),
        ToolCall {
            tool_name: "bash".to_string(),
            input: "echo test".to_string(),
            output: Some("test".to_string()),
            duration_ms: 5,
        },
    ).await;

    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.status, EpisodeStatus::InProgress);

    // After completion: Completed
    memory.complete_episode(
        episode_id.clone(),
        TaskOutcome::Success,
        None,
    ).await.unwrap();

    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.status, EpisodeStatus::Completed);
}
```
