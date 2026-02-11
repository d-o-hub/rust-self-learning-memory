//! CLI Episode Lifecycle Workflow Tests (Day 1)
//!
//! Comprehensive E2E tests covering:
//! - Episode creation → add steps → complete episode → query results
//! - All 7 tag commands in workflow
//! - All 7 relationship commands in workflow
//! - Error recovery paths
//!
//! Target: 8+ test scenarios

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::episode::{Direction, ExecutionStep, RelationshipMetadata, RelationshipType};
use memory_core::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
use memory_core::SelfLearningMemory;
use memory_storage_redb::RedbStorage;
use serial_test::serial;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

/// Test helper to create a memory instance with storage
async fn setup_test_memory() -> (Arc<SelfLearningMemory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    // Use a lower quality threshold for tests to avoid PREMem rejections for
    // concise example episodes. See plans/ for test guidance on thresholds.
    let mut cfg = Default::default();
    cfg.quality_threshold = 0.3;

    let memory = Arc::new(SelfLearningMemory::with_storage(
        cfg,
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    (memory, dir)
}

/// Helper to create an episode with the correct API
async fn create_test_episode(
    memory: &Arc<SelfLearningMemory>,
    description: &str,
    domain: &str,
) -> Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        ..Default::default()
    };
    memory
        .start_episode(description.to_string(), context, TaskType::CodeGeneration)
        .await
}

/// Helper to log steps and complete an episode
async fn complete_episode_with_steps(
    memory: &Arc<SelfLearningMemory>,
    episode_id: Uuid,
    steps: &[(&str, &str)],
) {
    for (i, (tool, action)) in steps.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 1, tool.to_string(), action.to_string());
        step.result = Some(ExecutionResult::Success {
            output: format!("{} completed", action),
        });
        memory.log_step(episode_id, step).await;
    }

    let _ = memory.flush_steps(episode_id).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Completed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
}

// ============================================================================
// Scenario 1: Complete Episode Lifecycle
// ============================================================================

#[tokio::test]
#[serial]
async fn test_episode_lifecycle_create_complete_query() {
    let (memory, _dir) = setup_test_memory().await;

    // Create episode
    let episode_id =
        create_test_episode(&memory, "Implement user authentication with JWT", "web-api").await;

    // Query episode
    let episode = memory
        .get_episode(episode_id)
        .await
        .expect("Failed to get episode");
    assert_eq!(
        episode.task_description,
        "Implement user authentication with JWT"
    );
    assert!(!episode.is_complete());

    // Add execution steps
    let steps = [
        ("read-code", "Read existing code"),
        ("write-code", "Write authentication code"),
        ("test-code", "Run tests"),
    ];

    complete_episode_with_steps(&memory, episode_id, &steps).await;

    // Verify episode is complete
    let episode = memory
        .get_episode(episode_id)
        .await
        .expect("Failed to get episode");
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 3);

    println!("✓ Episode lifecycle test passed");
}

// ============================================================================
// Scenario 2: Tag Workflow (7 commands)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_tag_workflow_all_commands() {
    let (memory, _dir) = setup_test_memory().await;

    // Test 1: Create episode
    let episode_id = create_test_episode(&memory, "Tag workflow test", "tag-test").await;

    // Test 2: add-tags
    memory
        .add_episode_tags(
            episode_id,
            vec!["rust".to_string(), "tokio".to_string(), "async".to_string()],
        )
        .await
        .expect("Failed to add tags");

    // Test 3: get-tags
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("Failed to get tags");
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"rust".to_string()));

    // Test 4: remove-tags
    memory
        .remove_episode_tags(episode_id, vec!["async".to_string()])
        .await
        .expect("Failed to remove tags");

    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(!tags.contains(&"async".to_string()));

    // Test 5: set-tags (replace all)
    memory
        .set_episode_tags(
            episode_id,
            vec!["new-tag-1".to_string(), "new-tag-2".to_string()],
        )
        .await
        .expect("Failed to set tags");

    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"new-tag-1".to_string()));
    assert!(!tags.contains(&"rust".to_string()));

    // Test 6: list-all-tags
    let all_tags = memory.get_all_tags().await.expect("Failed to get all tags");
    assert!(all_tags.contains(&"new-tag-1".to_string()));

    // Test 7: list-episodes-by-tag
    let episodes = memory
        .list_episodes_by_tags(vec!["new-tag-1".to_string()], true, Some(10))
        .await
        .expect("Failed to list episodes by tag");
    assert!(!episodes.is_empty());
    assert!(episodes.iter().any(|e| e.episode_id == episode_id));

    println!("✓ Tag workflow test passed (7 commands tested)");
}

// ============================================================================
// Scenario 3: Relationship Workflow (7 commands)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_relationship_workflow_all_commands() {
    let (memory, _dir) = setup_test_memory().await;

    // Test 1: Create parent episode
    let parent_id = create_test_episode(&memory, "Parent task", "relationship-test").await;

    // Test 2: Create child episodes
    let child_id1 = create_test_episode(&memory, "Child task 1", "relationship-test").await;
    let child_id2 = create_test_episode(&memory, "Child task 2", "relationship-test").await;

    // Complete episodes
    complete_episode_with_steps(&memory, parent_id, &[("setup", "Setup")]).await;
    complete_episode_with_steps(&memory, child_id1, &[("impl", "Implement")]).await;
    complete_episode_with_steps(&memory, child_id2, &[("impl", "Implement")]).await;

    // Test 3: add-relationship
    let rel_id1 = memory
        .add_episode_relationship(
            parent_id,
            child_id1,
            RelationshipType::ParentChild,
            RelationshipMetadata::default(),
        )
        .await
        .expect("Failed to add relationship");

    let rel_id2 = memory
        .add_episode_relationship(
            parent_id,
            child_id2,
            RelationshipType::ParentChild,
            RelationshipMetadata::default(),
        )
        .await
        .expect("Failed to add relationship");

    assert_ne!(rel_id1, rel_id2);

    // Test 4: get-relationships
    let relationships = memory
        .get_episode_relationships(parent_id, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");

    assert_eq!(relationships.len(), 2);

    // Test 5: find-related-episodes
    let filter = memory_core::memory::relationship_query::RelationshipFilter::default();
    let related = memory
        .find_related_episodes(parent_id, filter)
        .await
        .expect("Failed to find related episodes");

    assert!(!related.is_empty());

    // Test 6: relationship-exists
    let exists = memory
        .relationship_exists(parent_id, child_id1, RelationshipType::ParentChild)
        .await
        .expect("Failed to check existence");
    assert!(exists);

    // Test 7: remove-relationship
    memory
        .remove_episode_relationship(rel_id1)
        .await
        .expect("Failed to remove relationship");

    let relationships = memory
        .get_episode_relationships(parent_id, Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(relationships.len(), 1); // Only one left

    println!("✓ Relationship workflow test passed (7 commands tested)");
}

// ============================================================================
// Scenario 4: Error Recovery Paths
// ============================================================================

#[tokio::test]
#[serial]
async fn test_error_recovery_invalid_episode_operations() {
    let (memory, _dir) = setup_test_memory().await;

    let invalid_id = Uuid::new_v4();

    // Test: Get non-existent episode
    let result = memory.get_episode(invalid_id).await;
    assert!(result.is_err(), "Should fail for non-existent episode");

    // Test: Complete non-existent episode
    let result = memory
        .complete_episode(
            invalid_id,
            TaskOutcome::Success {
                verdict: "test".to_string(),
                artifacts: vec![],
            },
        )
        .await;
    assert!(
        result.is_err(),
        "Should fail to complete non-existent episode"
    );

    // Test: Get tags from non-existent episode
    let result = memory.get_episode_tags(invalid_id).await;
    assert!(
        result.is_err(),
        "Should fail to get tags from non-existent episode"
    );

    // Test: Add tags to non-existent episode
    let result = memory
        .add_episode_tags(invalid_id, vec!["test".to_string()])
        .await;
    assert!(
        result.is_err(),
        "Should fail to add tags to non-existent episode"
    );

    // Test: Delete non-existent episode
    let result = memory.delete_episode(invalid_id).await;
    assert!(
        result.is_err(),
        "Should fail to delete non-existent episode"
    );

    println!("✓ Error recovery test passed");
}

// ============================================================================
// Scenario 5: Batch Episode Operations
// ============================================================================

#[tokio::test]
#[serial]
async fn test_batch_episode_operations() {
    let (memory, _dir) = setup_test_memory().await;

    // Create multiple episodes
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let id = create_test_episode(&memory, &format!("Batch episode {}", i), "batch-test").await;
        complete_episode_with_steps(&memory, id, &[("tool", "action")]).await;
        episode_ids.push(id);
    }

    // Get all episodes
    let all_episodes = memory
        .get_all_episodes()
        .await
        .expect("Failed to get all episodes");
    assert!(all_episodes.len() >= 5);

    // List with pagination
    let page1 = memory
        .list_episodes(Some(2), Some(0), Some(true))
        .await
        .expect("Failed to list episodes");
    assert!(page1.len() <= 2);

    // Get by IDs
    let by_ids = memory
        .get_episodes_by_ids(&episode_ids[..3])
        .await
        .expect("Failed to get by IDs");
    assert_eq!(by_ids.len(), 3);

    println!("✓ Batch episode operations test passed");
}

// ============================================================================
// Scenario 6: Episode Deletion and Cleanup
// ============================================================================

#[tokio::test]
#[serial]
async fn test_episode_deletion_and_cleanup() {
    let (memory, _dir) = setup_test_memory().await;

    // Create episode
    let episode_id = create_test_episode(&memory, "Delete test", "delete-test").await;

    // Add tags
    memory
        .add_episode_tags(episode_id, vec!["to-delete".to_string()])
        .await
        .unwrap();

    // Complete it
    complete_episode_with_steps(&memory, episode_id, &[("tool", "action")]).await;

    // Verify exists
    assert!(memory.get_episode(episode_id).await.is_ok());

    // Delete
    memory
        .delete_episode(episode_id)
        .await
        .expect("Failed to delete episode");

    // Verify deleted
    assert!(memory.get_episode(episode_id).await.is_err());

    println!("✓ Episode deletion and cleanup test passed");
}
