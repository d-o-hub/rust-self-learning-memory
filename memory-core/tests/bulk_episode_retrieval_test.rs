//! Integration tests for bulk episode retrieval APIs
//!
//! Tests the new `get_episode()` and `get_episodes_by_ids()` methods
//! to ensure they work correctly with multiple storage backends.

use memory_core::{ComplexityLevel, ExecutionStep, SelfLearningMemory, TaskContext, TaskType};
use uuid::Uuid;

#[tokio::test]
async fn test_get_episode_single_retrieval() {
    // Create memory system
    let memory = SelfLearningMemory::new();

    // Start an episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "testing".to_string(),
        tags: vec!["bulk-ops".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Test single episode retrieval".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Add some steps
    let step1 = ExecutionStep::new(1, "tool1".to_string(), "action1".to_string());
    memory.log_step(episode_id, step1).await;

    // Flush steps to ensure they're persisted (in case batching is enabled)
    memory.flush_steps(episode_id).await.unwrap();

    // Retrieve the episode using the new API
    let retrieved = memory.get_episode(episode_id).await.unwrap();

    assert_eq!(retrieved.episode_id, episode_id);
    assert_eq!(retrieved.task_description, "Test single episode retrieval");
    assert_eq!(retrieved.steps.len(), 1);
}

#[tokio::test]
async fn test_get_episode_not_found() {
    let memory = SelfLearningMemory::new();
    let non_existent_id = Uuid::new_v4();

    // Should return NotFound error
    let result = memory.get_episode(non_existent_id).await;
    assert!(result.is_err());

    match result {
        Err(memory_core::Error::NotFound(id)) => {
            assert_eq!(id, non_existent_id);
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_episodes_by_ids_bulk_retrieval() {
    let memory = SelfLearningMemory::new();

    // Create multiple episodes
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "testing".to_string(),
        tags: vec!["bulk-ops".to_string()],
    };

    let mut episode_ids = Vec::new();
    for i in 1..=5 {
        let id = memory
            .start_episode(
                format!("Test episode {}", i),
                context.clone(),
                TaskType::Testing,
            )
            .await;
        episode_ids.push(id);
    }

    // Retrieve all episodes in bulk
    let episodes = memory.get_episodes_by_ids(&episode_ids).await.unwrap();

    assert_eq!(episodes.len(), 5);

    // Verify all episodes were retrieved
    for episode in &episodes {
        assert!(episode_ids.contains(&episode.episode_id));
    }
}

#[tokio::test]
async fn test_get_episodes_by_ids_partial_found() {
    let memory = SelfLearningMemory::new();

    // Create 3 episodes
    let context = TaskContext::default();
    let mut existing_ids = Vec::new();
    for i in 1..=3 {
        let id = memory
            .start_episode(
                format!("Existing episode {}", i),
                context.clone(),
                TaskType::Testing,
            )
            .await;
        existing_ids.push(id);
    }

    // Mix existing and non-existing IDs
    let mut mixed_ids = existing_ids.clone();
    mixed_ids.push(Uuid::new_v4()); // Non-existent
    mixed_ids.push(Uuid::new_v4()); // Non-existent

    // Should return only the existing episodes (no error)
    let episodes = memory.get_episodes_by_ids(&mixed_ids).await.unwrap();

    assert_eq!(episodes.len(), 3);

    for episode in &episodes {
        assert!(existing_ids.contains(&episode.episode_id));
    }
}

#[tokio::test]
async fn test_get_episodes_by_ids_empty_input() {
    let memory = SelfLearningMemory::new();

    // Empty input should return empty vector
    let episodes = memory.get_episodes_by_ids(&[]).await.unwrap();
    assert_eq!(episodes.len(), 0);
}

#[tokio::test]
async fn test_get_episodes_by_ids_all_missing() {
    let memory = SelfLearningMemory::new();

    // All non-existent IDs
    let non_existent_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    // Should return empty vector (no error)
    let episodes = memory.get_episodes_by_ids(&non_existent_ids).await.unwrap();
    assert_eq!(episodes.len(), 0);
}

#[tokio::test]
async fn test_get_episode_caching_behavior() {
    let memory = SelfLearningMemory::new();

    // Start an episode
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode("Cache test episode".to_string(), context, TaskType::Testing)
        .await;

    // First retrieval - will populate cache
    let episode1 = memory.get_episode(episode_id).await.unwrap();

    // Second retrieval - should come from in-memory cache
    let episode2 = memory.get_episode(episode_id).await.unwrap();

    assert_eq!(episode1.episode_id, episode2.episode_id);
    assert_eq!(episode1.task_description, episode2.task_description);
}

#[tokio::test]
async fn test_bulk_retrieval_performance() {
    let memory = SelfLearningMemory::new();

    // Create many episodes
    let context = TaskContext::default();
    let mut episode_ids = Vec::new();

    for i in 1..=50 {
        let id = memory
            .start_episode(
                format!("Performance test episode {}", i),
                context.clone(),
                TaskType::Testing,
            )
            .await;
        episode_ids.push(id);
    }

    // Bulk retrieval should be efficient
    let start = std::time::Instant::now();
    let episodes = memory.get_episodes_by_ids(&episode_ids).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(episodes.len(), 50);

    // Should complete quickly (< 100ms for in-memory)
    assert!(
        duration.as_millis() < 100,
        "Bulk retrieval took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_get_episode_with_steps() {
    let memory = SelfLearningMemory::new();

    // Start episode and add multiple steps
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode(
            "Episode with steps".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add several steps
    for i in 1..=10 {
        let step = ExecutionStep::new(i, format!("tool{}", i), format!("action{}", i));
        memory.log_step(episode_id, step).await;
    }

    // Flush steps to ensure they're persisted (in case batching is enabled)
    memory.flush_steps(episode_id).await.unwrap();

    // Retrieve and verify steps are included
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 10);

    // Verify step order
    for (idx, step) in episode.steps.iter().enumerate() {
        assert_eq!(step.step_number, idx + 1);
    }
}

#[tokio::test]
async fn test_bulk_retrieval_preserves_episode_data() {
    let memory = SelfLearningMemory::new();

    // Create episodes with different characteristics
    let contexts = vec![
        TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "async".to_string(),
            tags: vec!["tag1".to_string()],
        },
        TaskContext {
            language: Some("python".to_string()),
            framework: Some("fastapi".to_string()),
            complexity: ComplexityLevel::Complex,
            domain: "web".to_string(),
            tags: vec!["tag2".to_string()],
        },
    ];

    let mut episode_ids = Vec::new();
    for (i, context) in contexts.iter().enumerate() {
        let id = memory
            .start_episode(
                format!("Episode {}", i),
                context.clone(),
                TaskType::CodeGeneration,
            )
            .await;
        episode_ids.push(id);
    }

    // Bulk retrieve and verify all data is preserved
    let episodes = memory.get_episodes_by_ids(&episode_ids).await.unwrap();

    assert_eq!(episodes.len(), 2);

    // Verify each episode preserved its context
    for episode in episodes {
        assert!(episode.context.language.is_some());
        assert!(episode.context.framework.is_some());
        assert!(!episode.context.tags.is_empty());
    }
}
