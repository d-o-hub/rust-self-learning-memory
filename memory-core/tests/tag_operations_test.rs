//! Integration tests for episode tagging operations

use memory_core::{
    ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use uuid::Uuid;

/// Create memory with lower quality threshold for testing
fn test_memory() -> SelfLearningMemory {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        ..Default::default()
    };
    SelfLearningMemory::with_config(config)
}

/// Helper to create a test episode with sufficient steps to pass quality threshold
async fn create_test_episode(memory: &SelfLearningMemory, name: &str) -> Uuid {
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode(name.to_string(), context, TaskType::CodeGeneration)
        .await;

    // Add multiple steps to meet quality threshold
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i % 5), format!("Step {i}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {i} completed successfully"),
        });
        memory.log_step(episode_id, step).await;
    }

    let outcome = TaskOutcome::Success {
        verdict: "Test completed successfully".to_string(),
        artifacts: vec!["result.rs".to_string()],
    };
    memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Failed to complete episode");

    episode_id
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_add_episode_tags() {
    let memory = test_memory();
    let episode_id = create_test_episode(&memory, "Test Episode").await;

    // Add tags
    memory
        .add_episode_tags(
            episode_id,
            vec!["bug-fix".to_string(), "critical".to_string()],
        )
        .await
        .expect("Failed to add tags");

    // Verify tags were added
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"bug-fix".to_string()));
    assert!(tags.contains(&"critical".to_string()));
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_add_duplicate_tags() {
    let memory = test_memory();
    let episode_id = create_test_episode(&memory, "Test Episode").await;

    // Add tags
    memory
        .add_episode_tags(episode_id, vec!["feature".to_string()])
        .await
        .expect("Failed to add tags");

    // Try to add duplicate (should be ignored)
    memory
        .add_episode_tags(
            episode_id,
            vec!["feature".to_string(), "FEATURE".to_string()],
        )
        .await
        .expect("Failed to add tags");

    // Verify only one tag exists (case-insensitive)
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0], "feature");
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_remove_episode_tags() {
    let memory = test_memory();
    let episode_id = create_test_episode(&memory, "Test Episode").await;

    // Add tags
    memory
        .add_episode_tags(
            episode_id,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        )
        .await
        .expect("Failed to add tags");

    // Remove one tag
    memory
        .remove_episode_tags(episode_id, vec!["tag2".to_string()])
        .await
        .expect("Failed to remove tags");

    // Verify tag was removed
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(tags.len(), 2);
    assert!(!tags.contains(&"tag2".to_string()));
    assert!(tags.contains(&"tag1".to_string()));
    assert!(tags.contains(&"tag3".to_string()));
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_set_episode_tags() {
    let memory = test_memory();
    let episode_id = create_test_episode(&memory, "Test Episode").await;

    // Add initial tags
    memory
        .add_episode_tags(episode_id, vec!["old1".to_string(), "old2".to_string()])
        .await
        .expect("Failed to add tags");

    // Replace with new tags
    memory
        .set_episode_tags(episode_id, vec!["new1".to_string(), "new2".to_string()])
        .await
        .expect("Failed to set tags");

    // Verify old tags are gone and new tags exist
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"new1".to_string()));
    assert!(tags.contains(&"new2".to_string()));
    assert!(!tags.contains(&"old1".to_string()));
    assert!(!tags.contains(&"old2".to_string()));
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_list_episodes_by_tags_or() {
    let memory = test_memory();

    // Create episodes with different tags
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    memory
        .add_episode_tags(ep1, vec!["tag1".to_string(), "tag2".to_string()])
        .await
        .expect("Failed to add tags");

    let ep2 = create_test_episode(&memory, "Episode 2").await;
    memory
        .add_episode_tags(ep2, vec!["tag2".to_string(), "tag3".to_string()])
        .await
        .expect("Failed to add tags");

    let ep3 = create_test_episode(&memory, "Episode 3").await;
    memory
        .add_episode_tags(ep3, vec!["tag3".to_string(), "tag4".to_string()])
        .await
        .expect("Failed to add tags");

    // Query with OR logic (tag1 OR tag2)
    let results = memory
        .list_episodes_by_tags(vec!["tag1".to_string(), "tag2".to_string()], false, None)
        .await
        .expect("Failed to query tags");

    // Should find ep1 and ep2
    assert_eq!(results.len(), 2);
    let result_ids: Vec<Uuid> = results.iter().map(|e| e.episode_id).collect();
    assert!(result_ids.contains(&ep1));
    assert!(result_ids.contains(&ep2));
    assert!(!result_ids.contains(&ep3));
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_list_episodes_by_tags_and() {
    let memory = test_memory();

    // Create episodes with different tags
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    memory
        .add_episode_tags(ep1, vec!["tag1".to_string(), "tag2".to_string()])
        .await
        .expect("Failed to add tags");

    let ep2 = create_test_episode(&memory, "Episode 2").await;
    memory
        .add_episode_tags(ep2, vec!["tag2".to_string(), "tag3".to_string()])
        .await
        .expect("Failed to add tags");

    let ep3 = create_test_episode(&memory, "Episode 3").await;
    memory
        .add_episode_tags(
            ep3,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        )
        .await
        .expect("Failed to add tags");

    // Query with AND logic (tag1 AND tag2)
    let results = memory
        .list_episodes_by_tags(vec!["tag1".to_string(), "tag2".to_string()], true, None)
        .await
        .expect("Failed to query tags");

    // Should find ep1 and ep3 (both have tag1 AND tag2)
    assert_eq!(results.len(), 2);
    let result_ids: Vec<Uuid> = results.iter().map(|e| e.episode_id).collect();
    assert!(result_ids.contains(&ep1));
    assert!(!result_ids.contains(&ep2));
    assert!(result_ids.contains(&ep3));
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_get_all_tags() {
    let memory = test_memory();

    // Create episodes with various tags
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    memory
        .add_episode_tags(ep1, vec!["alpha".to_string(), "beta".to_string()])
        .await
        .expect("Failed to add tags");

    let ep2 = create_test_episode(&memory, "Episode 2").await;
    memory
        .add_episode_tags(ep2, vec!["beta".to_string(), "gamma".to_string()])
        .await
        .expect("Failed to add tags");

    // Get all unique tags
    let all_tags = memory.get_all_tags().await.expect("Failed to get all tags");

    // Should have 3 unique tags (alpha, beta, gamma)
    assert_eq!(all_tags.len(), 3);
    assert!(all_tags.contains(&"alpha".to_string()));
    assert!(all_tags.contains(&"beta".to_string()));
    assert!(all_tags.contains(&"gamma".to_string()));

    // Should be sorted
    assert_eq!(all_tags[0], "alpha");
    assert_eq!(all_tags[1], "beta");
    assert_eq!(all_tags[2], "gamma");
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_tag_statistics() {
    let memory = test_memory();

    // Create episodes with tags
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    memory
        .add_episode_tags(ep1, vec!["common".to_string(), "unique1".to_string()])
        .await
        .expect("Failed to add tags");

    let ep2 = create_test_episode(&memory, "Episode 2").await;
    memory
        .add_episode_tags(ep2, vec!["common".to_string(), "unique2".to_string()])
        .await
        .expect("Failed to add tags");

    let ep3 = create_test_episode(&memory, "Episode 3").await;
    memory
        .add_episode_tags(ep3, vec!["common".to_string()])
        .await
        .expect("Failed to add tags");

    // Get tag statistics
    let stats = memory
        .get_tag_statistics()
        .await
        .expect("Failed to get tag statistics");

    // Should have 3 tags
    assert_eq!(stats.len(), 3);

    // "common" should have usage_count of 3
    let common_stats = stats.get("common").expect("Missing 'common' tag");
    assert_eq!(common_stats.usage_count, 3);

    // "unique1" and "unique2" should have usage_count of 1
    let unique1_stats = stats.get("unique1").expect("Missing 'unique1' tag");
    assert_eq!(unique1_stats.usage_count, 1);

    let unique2_stats = stats.get("unique2").expect("Missing 'unique2' tag");
    assert_eq!(unique2_stats.usage_count, 1);
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored"]
async fn test_tag_validation() {
    let memory = test_memory();
    let episode_id = create_test_episode(&memory, "Test Episode").await;

    // Invalid tag with spaces - should fail
    let result = memory
        .add_episode_tags(episode_id, vec!["invalid tag".to_string()])
        .await;
    assert!(result.is_err());

    // Invalid tag with special chars - should fail
    let result = memory
        .add_episode_tags(episode_id, vec!["invalid@tag".to_string()])
        .await;
    assert!(result.is_err());

    // Valid tags - should succeed
    let result = memory
        .add_episode_tags(
            episode_id,
            vec![
                "valid-tag".to_string(),
                "valid_tag".to_string(),
                "validtag123".to_string(),
            ],
        )
        .await;
    assert!(result.is_ok());

    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");
    assert_eq!(tags.len(), 3);
}
