//! Lazy loading tests for `SelfLearningMemory`.

use crate::SelfLearningMemory;
use crate::episode::ExecutionStep;
use crate::patterns::{Pattern, PatternEffectiveness};
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Test `get_all_episodes` with lazy loading.
#[tokio::test(flavor = "multi_thread")]
pub async fn test_get_all_episodes_lazy_loading() {
    // Optimized config for fast test execution
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.5,
        pattern_extraction_threshold: 1.0, // Skip pattern extraction
        enable_summarization: false,       // Skip semantic summarization
        enable_embeddings: false,          // Skip embedding generation
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    // Create a few episodes
    let episode_id1 = memory
        .start_episode(
            "Test task 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let _episode_id2 = memory
        .start_episode(
            "Test task 2".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps to meet quality threshold
    for i in 0..20 {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Test step {i}"));
        step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        memory.log_step(episode_id1, step).await;
    }

    // Complete one episode
    memory
        .complete_episode(
            episode_id1,
            TaskOutcome::Success {
                verdict: "Task completed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Test get_all_episodes
    let all_episodes = memory.get_all_episodes().await.unwrap();
    assert_eq!(all_episodes.len(), 2, "Should return all episodes");

    // Test list_episodes with filters
    let all_episodes_list = memory.list_episodes(None, None, None).await.unwrap();
    assert_eq!(all_episodes_list.len(), 2, "Should list all episodes");

    let completed_episodes = memory.list_episodes(None, None, Some(true)).await.unwrap();
    assert_eq!(
        completed_episodes.len(),
        1,
        "Should return only completed episodes"
    );

    let limited_episodes = memory.list_episodes(Some(1), None, None).await.unwrap();
    assert_eq!(limited_episodes.len(), 1, "Should respect limit");

    // Test that episodes are sorted by start_time (newest first)
    let mut episodes_by_time = all_episodes_list.clone();
    episodes_by_time.sort_by_key(|b| std::cmp::Reverse(b.start_time));
    assert_eq!(
        all_episodes_list, episodes_by_time,
        "Episodes should be sorted by start_time (newest first)"
    );
}

/// Test `get_episode` with lazy loading.
#[tokio::test]
pub async fn test_get_episode_lazy_loading() {
    let memory = SelfLearningMemory::new();

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test lazy loading".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Get episode should work from in-memory
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test lazy loading");

    // Note: In test environment without storage backends,
    // lazy loading fallback won't work since episodes aren't persisted
    // This test mainly verifies the method doesn't panic
    // and works correctly when episode is in memory

    // Verify episode is in in-memory cache
    {
        let episodes = memory.episodes_fallback.read().await;
        assert!(
            episodes.contains_key(&episode_id),
            "Episode should be in memory cache"
        );
    }

    // The existing get_episode method with lazy loading should work
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test lazy loading");
}

/// Issue #831: patterns extracted on complete are visible via get_all_patterns.
#[tokio::test(flavor = "multi_thread")]
pub async fn test_get_all_patterns_after_complete() {
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        enable_summarization: false,
        enable_embeddings: false,
        batch_config: None,
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    let episode_id = memory
        .start_episode(
            "rust error handling".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: "rust".to_string(),
                tags: vec![],
            },
            TaskType::CodeGeneration,
        )
        .await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .expect("complete");

    let patterns = memory.get_all_patterns().await.expect("get_all_patterns");
    assert!(
        !patterns.is_empty(),
        "complete should extract at least a ContextPattern (issue #831)"
    );
}

/// queries::get_all_patterns with empty backends returns in-memory only.
#[tokio::test]
pub async fn test_get_all_patterns_memory_only() {
    let id = Uuid::new_v4();
    let sample = Pattern::ContextPattern {
        id,
        context_features: vec!["domain:test".to_string()],
        recommended_approach: "Test".to_string(),
        evidence: vec![],
        success_rate: 1.0,
        effectiveness: PatternEffectiveness::new(),
    };

    let fallback = RwLock::new(HashMap::from([(id, sample)]));
    // `queries` is a private sibling of `tests` under `memory`.
    let result = super::super::queries::get_all_patterns(&fallback, None, None)
        .await
        .expect("get_all_patterns");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id(), id);
}
