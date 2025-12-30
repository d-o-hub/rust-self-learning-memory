//! Integration tests for capacity-constrained storage

use memory_core::semantic::{EpisodeSummary, SemanticSummarizer};
use memory_core::{Episode, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;

/// Helper to create a test storage instance
async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await?;

    storage.initialize_schema().await?;

    Ok((storage, dir))
}

/// Helper to create a test episode with specific quality
fn create_test_episode(task_desc: &str, _quality_score: f32) -> Episode {
    let mut episode = Episode::new(
        task_desc.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    // Add a step to make it more realistic
    let mut step = ExecutionStep::new(1, "tester".to_string(), "Run tests".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Tests passed".to_string(),
    });
    episode.add_step(step);

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "Task completed".to_string(),
        artifacts: vec![format!("{}.rs", task_desc)],
    });

    episode
}

/// Helper to create episode summary
async fn create_test_summary(episode: &Episode) -> anyhow::Result<EpisodeSummary> {
    let summarizer = SemanticSummarizer::new();
    summarizer.summarize_episode(episode).await
}

#[tokio::test]
async fn test_store_and_retrieve_episode_summary() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("test_task", 0.8);
    let summary = create_test_summary(&episode).await?;

    // Store the episode first
    storage.store_episode(&episode).await?;

    // Store the summary
    storage.store_episode_summary(&summary).await?;

    // Retrieve the summary
    let retrieved = storage
        .get_episode_summary(episode.episode_id)
        .await?
        .ok_or("Summary not found")?;

    assert_eq!(retrieved.episode_id, summary.episode_id);
    assert_eq!(retrieved.summary_text, summary.summary_text);
    assert_eq!(retrieved.key_concepts, summary.key_concepts);
    assert_eq!(retrieved.key_steps, summary.key_steps);
    Ok(())
}

#[tokio::test]
async fn test_capacity_enforcement_lru() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let max_episodes = 3;

    // Store 3 episodes at capacity
    for i in 0..3 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        storage
            .store_episode_with_capacity(&episode, max_episodes)
            .await?;
    }

    // Verify we have 3 episodes
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 3);

    // Store 4th episode - should evict the oldest (task_0)
    let episode4 = create_test_episode("task_3", 0.5);
    storage
        .store_episode_with_capacity(&episode4, max_episodes)
        .await?;

    // Verify still at capacity (oldest evicted)
    let final_count = storage.get_statistics().await?.episode_count;
    assert_eq!(final_count, max_episodes);

    // Verify task_0 was evicted
    let result = storage.get_episode_by_task_desc("task_0").await?;
    assert!(result.is_none(), "Oldest episode should be evicted");

    Ok(())
}

#[tokio::test]
async fn test_capacity_enforcement_basic() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Store 5 episodes with capacity of 3
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        storage.store_episode_with_capacity(&episode, 3).await?;
    }

    // Verify only 3 episodes remain
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 3);

    Ok(())
}

#[tokio::test]
async fn test_summary_cascade_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("test_cascade", 0.7);
    let summary = create_test_summary(&episode).await?;
    let episode_id = episode.episode_id;

    // Store episode and summary
    storage.store_episode(&episode).await?;
    storage.store_episode_summary(&summary).await?;

    // Verify summary exists
    let retrieved = storage.get_episode_summary(episode_id).await?;
    assert!(retrieved.is_some());

    // Delete episode using delete_episode
    storage.delete_episode(episode_id).await?;

    // Verify summary was cascade deleted (manually check - depends on FK constraint)
    let after_delete = storage.get_episode_summary(episode_id).await?;
    assert!(after_delete.is_none(), "Summary should be deleted");
    Ok(())
}

#[tokio::test]
async fn test_capacity_count_accuracy() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Perform multiple insert/evict cycles
    for i in 0..15 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);

        storage.store_episode_with_capacity(&episode, 5).await?;

        // Check count after each operation
        let count = storage.get_statistics().await?.episode_count;

        // Should never exceed capacity
        assert!(
            count <= 5,
            "Episode count {} exceeds capacity 5 at iteration {}",
            count,
            i
        );

        // After capacity is reached, should stay at capacity
        if i >= 5 {
            assert_eq!(
                count, 5,
                "Episode count should be exactly at capacity after iteration {}",
                i
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_batch_eviction() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Fill to capacity
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        episode_ids.push(episode.episode_id);
        storage.store_episode_with_capacity(&episode, 10).await?;
    }

    // Manually evict first 3 episodes
    for id in &episode_ids[0..3] {
        storage.delete_episode(*id).await?;
    }

    // Verify count
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 2);

    // Verify evicted episodes are gone
    for id in &episode_ids[0..3] {
        let result = storage.get_episode(*id).await?;
        assert!(result.is_none(), "Episode should be deleted");
    }

    // Verify remaining episodes still exist
    for id in &episode_ids[3..5] {
        let result = storage.get_episode(*id).await?;
        assert!(result.is_some(), "Episode should still exist");
    }
    Ok(())
}

#[tokio::test]
async fn test_no_eviction_under_capacity() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Store only 5 episodes (under capacity of 10)
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);

        storage.store_episode_with_capacity(&episode, 10).await?;
    }

    // Verify all 5 episodes stored
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 5);
    Ok(())
}

#[tokio::test]
async fn test_summary_without_embedding() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("no_embedding", 0.6);
    let mut summary = create_test_summary(&episode).await?;

    // Explicitly remove embedding
    summary.summary_embedding = None;

    // Store episode and summary
    storage.store_episode(&episode).await?;
    storage.store_episode_summary(&summary).await?;

    // Retrieve and verify
    let retrieved = storage
        .get_episode_summary(episode.episode_id)
        .await?
        .ok_or("Summary not found")?;

    assert_eq!(retrieved.summary_embedding, None);
    assert_eq!(retrieved.summary_text, summary.summary_text);
    Ok(())
}
