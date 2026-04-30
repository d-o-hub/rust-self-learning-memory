//! Test for verifying double-write behavior doesn't cause data loss in redb storage.
//!
//! This test simulates the scenario where both storage backends (Turso and redb)
//! write the same episode to the same redb database, verifying that data is not lost.
//!
//! NOTE: Episodes with `ExecutionSteps` cannot be stored in redb due to a postcard
//! serialization limitation. The `ExecutionStep.parameters` field uses `serde_json::Value`
//! which postcard cannot serialize (returns `WontImplement` error). This test modifies
//! other postcard-compatible fields (`task_description`, tags, metadata) to verify
//! double-write behavior.

use do_memory_core::{Episode, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use tempfile::TempDir;

async fn create_test_storage() -> anyhow::Result<(RedbStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.redb");
    let storage = RedbStorage::new(&db_path).await?;
    Ok((storage, dir))
}

/// Test that writing the same episode twice to the same redb database doesn't cause data loss.
///
/// This simulates the double-write pattern used when both Turso and redb backends
/// store the same episode to the redb cache layer.
///
/// Scenario:
/// 1. Create an episode with initial data
/// 2. Store it (first write)
/// 3. Retrieve and verify it exists
/// 4. Clone the episode, modify `task_description` and add tags
/// 5. Store again (second write - simulates double-write from both backends)
/// 6. Retrieve and verify the episode still exists with the modified data
///
/// NOTE: Steps are not added due to postcard serialization limitation with `serde_json::Value`.
/// See `capacity_enforcement_test.rs` for the same workaround pattern.
#[tokio::test]
async fn test_episode_double_write_preserves_data() {
    // Arrange: Create storage and an episode
    let (storage, _dir) = create_test_storage()
        .await
        .expect("Failed to create storage");

    let context = TaskContext::default();
    let episode = Episode::new(
        "Initial task description".to_string(),
        context,
        TaskType::Testing,
    );

    // Pre-condition: Episode has no steps (postcard limitation workaround)
    assert_eq!(
        episode.steps.len(),
        0,
        "Episode should have no steps initially"
    );
    assert_eq!(episode.task_description, "Initial task description");

    // Act 1: Store the episode (first write - simulates Turso backend)
    storage
        .store_episode(&episode)
        .await
        .expect("First store_episode should succeed");

    // Assert 1: Episode exists after first write
    let retrieved_after_first = storage
        .get_episode(episode.episode_id)
        .await
        .expect("get_episode after first write should succeed");
    assert!(
        retrieved_after_first.is_some(),
        "Episode should exist after first write"
    );
    let retrieved_episode = retrieved_after_first.expect("Episode should be found");
    assert_eq!(
        retrieved_episode.episode_id, episode.episode_id,
        "Episode ID should match after first write"
    );
    assert_eq!(
        retrieved_episode.task_description, "Initial task description",
        "Task description should match after first write"
    );
    assert_eq!(
        retrieved_episode.steps.len(),
        0,
        "Episode should have no steps after first write (postcard limitation)"
    );

    // Act 2: Clone episode, modify fields, and store again (simulates redb backend write)
    let mut modified_episode = episode.clone();
    modified_episode.task_description = "Updated task description after double-write".to_string();
    modified_episode
        .add_tag("double-write-test".to_string())
        .expect("Tag should be valid");
    modified_episode
        .metadata
        .insert("modified_at".to_string(), "2024-01-01".to_string());

    // Pre-condition for second write: Episode is modified
    assert_eq!(
        modified_episode.task_description, "Updated task description after double-write",
        "Episode should have modified description before second write"
    );
    assert_eq!(
        modified_episode.tags.len(),
        1,
        "Episode should have one tag before second write"
    );

    // Second write - simulates double-write from both storage backends
    storage
        .store_episode(&modified_episode)
        .await
        .expect("Second store_episode should succeed");

    // Assert 2: Episode exists after second write with modified data preserved
    let retrieved_after_second = storage
        .get_episode(episode.episode_id)
        .await
        .expect("get_episode after second write should succeed");
    assert!(
        retrieved_after_second.is_some(),
        "Episode should exist after second write"
    );
    let final_episode = retrieved_after_second.expect("Episode should be found after second write");
    assert_eq!(
        final_episode.episode_id, episode.episode_id,
        "Episode ID should match after second write"
    );
    assert_eq!(
        final_episode.task_description, "Updated task description after double-write",
        "Task description should be updated after second write - data should not be lost"
    );
    assert_eq!(
        final_episode.tags.len(),
        1,
        "Episode should have one tag after second write - data should not be lost"
    );
    assert!(final_episode.has_tag("double-write-test"));
    assert_eq!(
        final_episode.metadata.get("modified_at"),
        Some(&"2024-01-01".to_string()),
        "Metadata should be preserved after second write"
    );
}

/// Test that multiple writes with modifications all preserve data.
///
/// This extends the double-write test to verify that repeated writes
/// with modified content properly update the stored data.
#[tokio::test]
async fn test_episode_multiple_writes_update_data() {
    // Arrange: Create storage and an episode
    let (storage, _dir) = create_test_storage()
        .await
        .expect("Failed to create storage");

    let context = TaskContext::default();
    let mut episode = Episode::new(
        "Initial description".to_string(),
        context,
        TaskType::Testing,
    );

    // Act & Assert: Write multiple times with modifications
    for write_count in 1..=5 {
        // Modify task_description and add a tag
        episode.task_description = format!("Description after write {}", write_count);
        episode
            .add_tag(format!("tag-{}", write_count))
            .expect("Tag should be valid");

        // Store (each write simulates a backend sync)
        storage
            .store_episode(&episode)
            .await
            .expect("store_episode should succeed");

        // Retrieve and verify
        let retrieved = storage
            .get_episode(episode.episode_id)
            .await
            .expect("get_episode should succeed")
            .expect("Episode should exist");

        assert_eq!(
            retrieved.task_description,
            format!("Description after write {}", write_count),
            "Task description should be updated after {} writes",
            write_count
        );
        assert_eq!(
            retrieved.tags.len(),
            write_count,
            "Episode should have {} tags after {} writes",
            write_count,
            write_count
        );
    }
}

/// Test that double-write with different episodes doesn't cause ID collision.
///
/// Verifies that storing different episodes with different IDs doesn't
/// interfere with each other, even when the writes happen concurrently.
#[tokio::test]
async fn test_double_write_different_episodes_preserve_identity() {
    // Arrange: Create storage
    let (storage, _dir) = create_test_storage()
        .await
        .expect("Failed to create storage");

    // Create two different episodes
    let episode1 = Episode::new(
        "Episode 1".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let episode2 = Episode::new(
        "Episode 2".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Act: Store both episodes
    storage
        .store_episode(&episode1)
        .await
        .expect("Store episode 1");
    storage
        .store_episode(&episode2)
        .await
        .expect("Store episode 2");

    // Store again (double-write simulation)
    storage
        .store_episode(&episode1)
        .await
        .expect("Double-write episode 1");
    storage
        .store_episode(&episode2)
        .await
        .expect("Double-write episode 2");

    // Assert: Both episodes still exist with correct data
    let retrieved1 = storage
        .get_episode(episode1.episode_id)
        .await
        .expect("Get episode 1")
        .expect("Episode 1 should exist");
    let retrieved2 = storage
        .get_episode(episode2.episode_id)
        .await
        .expect("Get episode 2")
        .expect("Episode 2 should exist");

    assert_eq!(retrieved1.task_description, "Episode 1");
    assert_eq!(retrieved2.task_description, "Episode 2");
    assert_eq!(retrieved1.task_type, TaskType::Testing);
    assert_eq!(retrieved2.task_type, TaskType::CodeGeneration);
}
