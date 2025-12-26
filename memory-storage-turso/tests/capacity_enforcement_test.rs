//! Integration tests for Phase 2 (GENESIS) capacity-constrained storage

use memory_core::episodic::{CapacityManager, EvictionPolicy};
use memory_core::semantic::{EpisodeSummary, SemanticSummarizer};
use memory_core::{Episode, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;

/// Helper to create a test storage instance
async fn create_test_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "")
        .await
        .expect("Failed to create storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    (storage, dir)
}

/// Helper to create a test episode with specific quality
fn create_test_episode(task_desc: &str, quality_score: f32) -> Episode {
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

    // Complete the episode with a reward based on quality score
    episode.complete(TaskOutcome::Success {
        verdict: format!("Task completed with quality: {}", quality_score),
        artifacts: vec![format!("{}.rs", task_desc)],
    });

    // Set reward to simulate quality
    episode.reward = Some(memory_core::RewardScore {
        total: quality_score * 2.0, // Scale to 0-2 range
        base: quality_score,
        efficiency: quality_score * 0.8,
        complexity_bonus: quality_score * 0.5,
        quality_multiplier: quality_score,
        learning_bonus: quality_score * 0.6,
    });

    episode
}

/// Helper to create episode summary
async fn create_test_summary(episode: &Episode) -> EpisodeSummary {
    let summarizer = SemanticSummarizer::new();
    summarizer
        .summarize_episode(episode)
        .await
        .expect("Failed to create summary")
}

#[tokio::test]
async fn test_store_and_retrieve_episode_summary() {
    let (storage, _dir) = create_test_storage().await;

    let episode = create_test_episode("test_task", 0.8);
    let summary = create_test_summary(&episode).await;

    // Store the episode first
    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");

    // Store the summary
    storage
        .store_episode_summary(&summary)
        .await
        .expect("Failed to store summary");

    // Retrieve the summary
    let retrieved = storage
        .get_episode_summary(episode.episode_id)
        .await
        .expect("Failed to retrieve summary")
        .expect("Summary not found");

    assert_eq!(retrieved.episode_id, summary.episode_id);
    assert_eq!(retrieved.summary_text, summary.summary_text);
    assert_eq!(retrieved.key_concepts, summary.key_concepts);
    assert_eq!(retrieved.key_steps, summary.key_steps);
}

#[tokio::test]
async fn test_capacity_enforcement_lru() {
    let (storage, _dir) = create_test_storage().await;

    let capacity_manager = CapacityManager::new(3, EvictionPolicy::LRU);

    // Store 3 episodes at capacity
    for i in 0..3 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        let summary = create_test_summary(&episode).await;

        let evicted = storage
            .store_episode_with_capacity(&episode, Some(&summary), &capacity_manager)
            .await
            .expect("Failed to store episode");

        assert!(evicted.is_none(), "No eviction should occur yet");
    }

    // Verify we have 3 episodes
    let count = storage
        .get_statistics()
        .await
        .expect("Failed to get stats")
        .episode_count;
    assert_eq!(count, 3);

    // Store 4th episode - should evict the oldest (task_0)
    let episode4 = create_test_episode("task_3", 0.5);
    let summary4 = create_test_summary(&episode4).await;

    let evicted = storage
        .store_episode_with_capacity(&episode4, Some(&summary4), &capacity_manager)
        .await
        .expect("Failed to store episode");

    assert!(evicted.is_some(), "Should have evicted episodes");
    assert_eq!(evicted.unwrap().len(), 1);

    // Verify still at capacity
    let final_count = storage
        .get_statistics()
        .await
        .expect("Failed to get stats")
        .episode_count;
    assert_eq!(final_count, 3);
}

#[tokio::test]
async fn test_capacity_enforcement_relevance_weighted() {
    let (storage, _dir) = create_test_storage().await;

    let capacity_manager = CapacityManager::new(3, EvictionPolicy::RelevanceWeighted);

    // Store 3 episodes with different quality scores
    let episodes = vec![
        create_test_episode("low_quality", 0.2),    // Low quality
        create_test_episode("high_quality", 0.9),   // High quality
        create_test_episode("medium_quality", 0.5), // Medium quality
    ];

    for episode in &episodes {
        let summary = create_test_summary(episode).await;
        storage
            .store_episode_with_capacity(&episode, Some(&summary), &capacity_manager)
            .await
            .expect("Failed to store episode");
    }

    // Store new high-quality episode - should evict low_quality
    let new_episode = create_test_episode("new_high_quality", 0.85);
    let new_summary = create_test_summary(&new_episode).await;

    let evicted = storage
        .store_episode_with_capacity(&new_episode, Some(&new_summary), &capacity_manager)
        .await
        .expect("Failed to store episode");

    assert!(evicted.is_some(), "Should have evicted episodes");

    // Verify the low quality episode was evicted
    let evicted_ids = evicted.unwrap();
    assert_eq!(evicted_ids.len(), 1);

    // The evicted episode should be the low quality one
    let evicted_episode = episodes[0].episode_id; // low_quality
    assert!(
        evicted_ids.contains(&evicted_episode),
        "Low quality episode should be evicted"
    );

    // Verify capacity maintained
    let final_count = storage
        .get_statistics()
        .await
        .expect("Failed to get stats")
        .episode_count;
    assert_eq!(final_count, 3);
}

#[tokio::test]
async fn test_summary_cascade_deletion() {
    let (storage, _dir) = create_test_storage().await;

    let episode = create_test_episode("test_cascade", 0.7);
    let summary = create_test_summary(&episode).await;
    let episode_id = episode.episode_id;

    // Store episode and summary
    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");
    storage
        .store_episode_summary(&summary)
        .await
        .expect("Failed to store summary");

    // Verify summary exists
    let retrieved = storage
        .get_episode_summary(episode_id)
        .await
        .expect("Failed to query summary");
    assert!(retrieved.is_some());

    // Delete episode (using batch eviction)
    storage
        .batch_evict_episodes(&[episode_id])
        .await
        .expect("Failed to evict episode");

    // Verify summary was cascade deleted
    let after_delete = storage
        .get_episode_summary(episode_id)
        .await
        .expect("Failed to query summary");
    assert!(after_delete.is_none(), "Summary should be cascade deleted");
}

#[tokio::test]
async fn test_capacity_count_accuracy() {
    let (storage, _dir) = create_test_storage().await;

    let capacity_manager = CapacityManager::new(5, EvictionPolicy::LRU);

    // Perform multiple insert/evict cycles
    for i in 0..15 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        let summary = create_test_summary(&episode).await;

        storage
            .store_episode_with_capacity(&episode, Some(&summary), &capacity_manager)
            .await
            .expect("Failed to store episode");

        // Check count after each operation
        let count = storage
            .get_statistics()
            .await
            .expect("Failed to get stats")
            .episode_count;

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
}

#[tokio::test]
async fn test_batch_eviction() {
    let (storage, _dir) = create_test_storage().await;

    let capacity_manager = CapacityManager::new(5, EvictionPolicy::LRU);

    // Fill to capacity
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        let summary = create_test_summary(&episode).await;
        episode_ids.push(episode.episode_id);

        storage
            .store_episode_with_capacity(&episode, Some(&summary), &capacity_manager)
            .await
            .expect("Failed to store episode");
    }

    // Batch evict first 3 episodes
    let to_evict = &episode_ids[0..3];
    storage
        .batch_evict_episodes(to_evict)
        .await
        .expect("Failed to batch evict");

    // Verify count
    let count = storage
        .get_statistics()
        .await
        .expect("Failed to get stats")
        .episode_count;
    assert_eq!(count, 2);

    // Verify evicted episodes are gone
    for id in to_evict {
        let result = storage
            .get_episode(*id)
            .await
            .expect("Failed to query episode");
        assert!(result.is_none(), "Episode should be deleted");
    }

    // Verify remaining episodes still exist
    for id in &episode_ids[3..5] {
        let result = storage
            .get_episode(*id)
            .await
            .expect("Failed to query episode");
        assert!(result.is_some(), "Episode should still exist");
    }
}

#[tokio::test]
async fn test_no_eviction_under_capacity() {
    let (storage, _dir) = create_test_storage().await;

    let capacity_manager = CapacityManager::new(10, EvictionPolicy::LRU);

    // Store only 5 episodes (under capacity)
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        let summary = create_test_summary(&episode).await;

        let evicted = storage
            .store_episode_with_capacity(&episode, Some(&summary), &capacity_manager)
            .await
            .expect("Failed to store episode");

        assert!(
            evicted.is_none(),
            "No eviction should occur when under capacity"
        );
    }

    // Verify all 5 episodes stored
    let count = storage
        .get_statistics()
        .await
        .expect("Failed to get stats")
        .episode_count;
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_summary_without_embedding() {
    let (storage, _dir) = create_test_storage().await;

    let episode = create_test_episode("no_embedding", 0.6);
    let mut summary = create_test_summary(&episode).await;

    // Explicitly remove embedding
    summary.summary_embedding = None;

    // Store episode and summary
    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");
    storage
        .store_episode_summary(&summary)
        .await
        .expect("Failed to store summary");

    // Retrieve and verify
    let retrieved = storage
        .get_episode_summary(episode.episode_id)
        .await
        .expect("Failed to retrieve summary")
        .expect("Summary not found");

    assert_eq!(retrieved.summary_embedding, None);
    assert_eq!(retrieved.summary_text, summary.summary_text);
}
