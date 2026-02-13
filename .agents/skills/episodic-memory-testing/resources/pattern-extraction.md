# Pattern Extraction Testing Strategies

## Pattern Extraction Pipeline

1. **Episode Completion** -> Triggers extraction
2. **Worker Queue** -> Async pattern workers
3. **Embedding Generation** -> Vector representation
4. **Clustering** -> Similar pattern grouping
5. **Storage** -> Persist extracted patterns

## Test Patterns

```rust
#[tokio::test]
async fn test_pattern_extraction_pipeline() {
    let memory = setup_memory().await;

    // Create episode with rich tool usage
    let episode_id = create_episode_with_diverse_tools(&memory).await;

    // Complete triggers extraction
    memory.complete_episode(
        episode_id.clone(),
        TaskOutcome::Success,
        Some(vec!["Good tool usage".to_string()]),
    ).await.unwrap();

    // Wait for async extraction
    let patterns = wait_for_patterns(&memory, &episode_id, 10).await;

    // Verify extraction
    assert!(!patterns.is_empty());
    assert!(patterns.len() <= 10);

    // Verify pattern structure
    for pattern in &patterns {
        assert!(!pattern.id.is_empty());
        assert!(pattern.confidence > 0.0);
        assert!(pattern.confidence <= 1.0);
    }
}

async fn wait_for_patterns(
    memory: &SelfLearningMemory,
    episode_id: &str,
    max_wait_ms: u64,
) -> Vec<Pattern> {
    let start = tokio::time::Instant::now();
    let sleep_duration = Duration::from_millis(100);

    while start.elapsed().as_millis() < max_wait_ms as i64 {
        if let Some(patterns) = memory.get_patterns_for_episode(episode_id).await {
            if !patterns.is_empty() {
                return patterns;
            }
        }
        tokio::time::sleep(sleep_duration).await;
    }

    Vec::new()
}
```

## Pattern Quality Metrics

```rust
#[tokio::test]
async fn test_pattern_quality_metrics() {
    let memory = setup_memory().await;

    // Create episodes with consistent patterns
    for i in 0..20 {
        let episode_id = create_test_episode(&memory, TaskType::CodeGeneration).await;

        // Always use cargo build
        memory.log_execution_step(
            episode_id.clone(),
            ToolCall {
                tool_name: "bash".to_string(),
                input: "cargo build".to_string(),
                output: Some("Finished".to_string()),
                duration_ms: 100,
            },
        ).await;

        memory.complete_episode(
            episode_id,
            TaskOutcome::Success,
            None,
        ).await.unwrap();
    }

    // Extract and verify frequency
    let patterns = memory.extract_patterns_for_task_type(TaskType::CodeGeneration)
        .await
        .unwrap();

    let cargo_build = patterns.iter()
        .find(|p| p.name.contains("cargo build"))
        .expect("Should find cargo build pattern");

    // High frequency for consistent pattern
    assert!(cargo_build.frequency >= 0.9);
    assert!(cargo_build.confidence >= 0.8);
}
```
