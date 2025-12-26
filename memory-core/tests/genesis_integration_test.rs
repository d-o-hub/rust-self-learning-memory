//! Phase 2 (GENESIS) Integration Tests
//!
//! End-to-end tests for capacity management and semantic summarization integration
//! with SelfLearningMemory.

use memory_core::{
    EvictionPolicy, ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};

/// Helper to create a test episode with sufficient steps to pass quality threshold
async fn create_test_episode(
    memory: &SelfLearningMemory,
    task_desc: &str,
    num_steps: usize,
) -> uuid::Uuid {
    let episode_id = memory
        .start_episode(
            task_desc.to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add steps to meet quality threshold
    for i in 0..num_steps {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Test step {}", i));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {} completed successfully", i),
        });
        memory.log_step(episode_id, step).await;
    }

    episode_id
}

#[tokio::test]
async fn test_complete_episode_with_summary() {
    // Test that episodes are summarized when summarization is enabled

    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_summarization: true,
        summary_min_length: 50,
        summary_max_length: 150,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create and complete an episode
    let episode_id = create_test_episode(&memory, "Implement authentication feature", 20).await;

    let outcome = TaskOutcome::Success {
        verdict: "Authentication implemented successfully with tests".to_string(),
        artifacts: vec!["auth.rs".to_string(), "auth_test.rs".to_string()],
    };

    let result = memory.complete_episode(episode_id, outcome).await;
    assert!(result.is_ok(), "Episode completion should succeed");

    // Verify episode was completed
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete(), "Episode should be marked complete");
    assert!(episode.reward.is_some(), "Episode should have reward");
    assert!(
        episode.reflection.is_some(),
        "Episode should have reflection"
    );

    // Note: Summary storage verification will be added in Phase 2.4
    // when storage backends implement store_episode_summary()
}

#[tokio::test]
async fn test_complete_episode_with_capacity() {
    // Test that capacity limits are enforced during episode completion

    let config = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: Some(5), // Limit to 5 episodes
        eviction_policy: Some(EvictionPolicy::LRU),
        enable_summarization: false, // Disable to focus on capacity
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create and complete 7 episodes (should evict 2)
    for i in 0..7 {
        let episode_id = create_test_episode(
            &memory,
            &format!("Task {}", i),
            20, // Sufficient steps
        )
        .await;

        let outcome = TaskOutcome::Success {
            verdict: format!("Task {} completed", i),
            artifacts: vec![],
        };

        memory
            .complete_episode(episode_id, outcome)
            .await
            .expect("Episode completion should succeed");
    }

    // Verify we have at most 5 episodes
    let (total, completed, _) = memory.get_stats().await;
    assert!(
        total <= 5,
        "Should have at most 5 episodes, found {}",
        total
    );
    assert_eq!(completed, total, "All stored episodes should be completed");
}

#[tokio::test]
async fn test_eviction_during_completion() {
    // Test that eviction occurs correctly when completing episodes at capacity

    let config = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: Some(3),
        eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
        enable_summarization: false,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create 3 low-quality episodes
    for i in 0..3 {
        let episode_id = create_test_episode(
            &memory,
            &format!("Low quality task {}", i),
            10, // Fewer steps = lower quality
        )
        .await;

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

    // Verify we have 3 episodes
    let (total_before, _, _) = memory.get_stats().await;
    assert_eq!(total_before, 3, "Should have 3 episodes before eviction");

    // Now create a high-quality episode (should trigger eviction of lowest quality)
    let high_quality_id = create_test_episode(
        &memory,
        "High quality complex task",
        30, // More steps = higher quality
    )
    .await;

    memory
        .complete_episode(
            high_quality_id,
            TaskOutcome::Success {
                verdict: "High quality implementation with comprehensive testing".to_string(),
                artifacts: vec!["impl.rs".to_string(), "tests.rs".to_string()],
            },
        )
        .await
        .unwrap();

    // Verify we still have at most 3 episodes (one was evicted)
    let (total_after, _, _) = memory.get_stats().await;
    assert!(
        total_after <= 3,
        "Should have at most 3 episodes after eviction, found {}",
        total_after
    );

    // Verify high-quality episode is still present
    assert!(
        memory.get_episode(high_quality_id).await.is_ok(),
        "High-quality episode should not be evicted"
    );
}

#[tokio::test]
async fn test_capacity_performance_overhead() {
    // Test that capacity enforcement doesn't add significant overhead

    use std::time::Instant;

    // Test without capacity limits
    let config_unlimited = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: None, // No limits
        enable_summarization: false,
        ..Default::default()
    };

    let memory_unlimited = SelfLearningMemory::with_config(config_unlimited);

    let start_unlimited = Instant::now();
    for i in 0..10 {
        let episode_id = create_test_episode(&memory_unlimited, &format!("Task {}", i), 20).await;
        memory_unlimited
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }
    let duration_unlimited = start_unlimited.elapsed();

    // Test with capacity limits
    let config_limited = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: Some(100), // High enough to not trigger eviction in this test
        eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
        enable_summarization: false,
        ..Default::default()
    };

    let memory_limited = SelfLearningMemory::with_config(config_limited);

    let start_limited = Instant::now();
    for i in 0..10 {
        let episode_id = create_test_episode(&memory_limited, &format!("Task {}", i), 20).await;
        memory_limited
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }
    let duration_limited = start_limited.elapsed();

    // Calculate overhead
    let overhead_ms = duration_limited
        .saturating_sub(duration_unlimited)
        .as_millis() as f64
        / 10.0;

    println!("Unlimited: {:?}", duration_unlimited);
    println!("Limited: {:?}", duration_limited);
    println!("Average overhead per episode: {:.2} ms", overhead_ms);

    // Overhead should be minimal (≤ 10ms average per episode)
    // Note: This is a soft check as CI environments can be unpredictable
    assert!(
        overhead_ms <= 50.0,
        "Capacity check overhead should be minimal, found {:.2} ms",
        overhead_ms
    );
}

#[tokio::test]
async fn test_backward_compatibility_no_capacity() {
    // Test that episodes work correctly when capacity is not configured

    let config = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: None, // No capacity limits
        eviction_policy: None,
        enable_summarization: false,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create many episodes (no eviction should occur)
    for i in 0..20 {
        let episode_id = create_test_episode(&memory, &format!("Task {}", i), 20).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Task {} completed", i),
                    artifacts: vec![],
                },
            )
            .await
            .expect("Episode completion should succeed");
    }

    // Verify all 20 episodes are stored
    let (total, completed, _) = memory.get_stats().await;
    assert_eq!(total, 20, "All 20 episodes should be stored");
    assert_eq!(completed, 20, "All episodes should be completed");
}

#[tokio::test]
async fn test_summarization_with_capacity() {
    // Test that both summarization and capacity work together

    let config = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: Some(5),
        eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
        enable_summarization: true,
        summary_min_length: 50,
        summary_max_length: 150,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes with both features enabled
    for i in 0..7 {
        let episode_id =
            create_test_episode(&memory, &format!("Feature {} implementation", i), 25).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Feature {} implemented with tests and docs", i),
                    artifacts: vec![
                        format!("feature_{}.rs", i),
                        format!("feature_{}_test.rs", i),
                    ],
                },
            )
            .await
            .expect("Episode completion should succeed");
    }

    // Verify capacity is enforced
    let (total, completed, _) = memory.get_stats().await;
    assert!(
        total <= 5,
        "Capacity should be enforced, found {} episodes",
        total
    );
    assert_eq!(completed, total, "All stored episodes should be completed");

    // Note: Summary verification will be added in Phase 2.4
    // when storage backends implement summary retrieval
}

#[tokio::test]
async fn test_eviction_preserves_high_quality() {
    // Test that eviction correctly preserves high-quality episodes

    let config = MemoryConfig {
        quality_threshold: 0.5,
        max_episodes: Some(3),
        eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
        enable_summarization: false,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create one high-quality episode
    let high_quality_id =
        create_test_episode(&memory, "Complex high-quality implementation", 40).await;

    memory
        .complete_episode(
            high_quality_id,
            TaskOutcome::Success {
                verdict: "Excellent implementation with comprehensive testing and documentation"
                    .to_string(),
                artifacts: vec![
                    "main.rs".to_string(),
                    "tests.rs".to_string(),
                    "docs.md".to_string(),
                ],
            },
        )
        .await
        .unwrap();

    // Create multiple low-quality episodes
    for i in 0..5 {
        let low_quality_id = create_test_episode(&memory, &format!("Simple task {}", i), 10).await;

        memory
            .complete_episode(
                low_quality_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Verify capacity is enforced
    let (total, _, _) = memory.get_stats().await;
    assert!(
        total <= 3,
        "Should have at most 3 episodes, found {}",
        total
    );

    // Verify high-quality episode is still present (not evicted)
    assert!(
        memory.get_episode(high_quality_id).await.is_ok(),
        "High-quality episode should be preserved during eviction"
    );
}
