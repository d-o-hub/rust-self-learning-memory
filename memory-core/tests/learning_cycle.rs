//! BDD-style integration tests for the complete learning cycle
//!
//! These tests verify end-to-end learning workflows from episode creation
//! through completion, pattern extraction, and retrieval.
//! All tests follow the Given-When-Then pattern for clarity.

mod common;

use common::{
    api_context, assert_episode_completed, assert_has_patterns, assert_min_patterns,
    assert_reward_in_range, create_success_step, create_test_context, rust_context,
    setup_memory_with_n_episodes, setup_test_memory, ContextBuilder, StepBuilder,
};
use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};

#[tokio::test]
async fn should_execute_complete_learning_cycle_end_to_end() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We start an episode with context
    let context = ContextBuilder::new("async-web-api")
        .language("rust")
        .framework("tokio")
        .complexity(ComplexityLevel::Moderate)
        .tag("concurrency")
        .tag("rest")
        .build();

    let episode_id = memory
        .start_episode(
            "Implement async REST API endpoint".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Then: The episode should be created and incomplete
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(!episode.is_complete());
    assert_eq!(episode.task_type, TaskType::CodeGeneration);

    // When: We log execution steps
    let steps = [
        ("analyzer", "Analyze requirements"),
        ("designer", "Design API structure"),
        ("builder", "Build endpoint handler"),
        ("validator", "Validate request/response"),
        ("tester", "Run integration tests"),
    ];

    for (i, (tool, action)) in steps.iter().enumerate() {
        let step = StepBuilder::new(i + 1, *tool, *action)
            .latency_ms(100 + (i as u64 * 50))
            .success(format!("{} completed", action))
            .build();
        memory.log_step(episode_id, step).await;
    }

    // When: We complete the episode
    let outcome = TaskOutcome::Success {
        verdict: "REST API endpoint implemented successfully".to_string(),
        artifacts: vec!["api/handlers.rs".to_string(), "api/routes.rs".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Then: The episode should be complete with learning artifacts
    let completed_episode = memory.get_episode(episode_id).await.unwrap();
    assert_episode_completed(&completed_episode);
    assert_has_patterns(&completed_episode);

    // Check reward
    let reward = completed_episode.reward.unwrap();
    assert_eq!(reward.base, 1.0); // Success
    assert_reward_in_range(&reward, 1.0, 3.0); // Reasonable range for rewards
    assert_eq!(reward.complexity_bonus, 1.1); // Moderate complexity

    // Check reflection
    let reflection = completed_episode.reflection.unwrap();
    assert!(!reflection.successes.is_empty());
    assert!(!reflection.insights.is_empty());

    // When: We retrieve context for a similar task
    let similar_context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "async-web-api".to_string(),
        tags: vec!["rest".to_string()],
    };

    let relevant = memory
        .retrieve_relevant_context(
            "Add authentication to API".to_string(),
            similar_context.clone(),
            5,
        )
        .await;

    // Then: The system should return relevant past episodes and patterns
    assert!(!relevant.is_empty());
    assert_eq!(relevant[0].episode_id, episode_id);

    let patterns = memory
        .retrieve_relevant_patterns(&similar_context, 10)
        .await;
    assert!(!patterns.is_empty());
}

#[tokio::test]
async fn should_learn_from_multiple_episodes_in_same_domain() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create multiple episodes in the same domain
    for i in 0..3 {
        let context = ContextBuilder::new("data-processing")
            .language("rust")
            .complexity(ComplexityLevel::Simple)
            .tag("batch")
            .build();

        let episode_id = memory
            .start_episode(
                format!("Process batch {}", i),
                context,
                TaskType::CodeGeneration,
            )
            .await;

        // Add steps - using create_success_step helper
        for j in 0..3 {
            let step = create_success_step(j + 1, &format!("processor_{}", j), "Process data");
            memory.log_step(episode_id, step).await;
        }

        // Complete
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Batch {} processed", i),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Then: Statistics should reflect all episodes
    let (total, completed, patterns) = memory.get_stats().await;
    assert_eq!(total, 3);
    assert_eq!(completed, 3);
    assert!(patterns > 0);

    // And: Retrieval should return all relevant episodes
    let context = create_test_context("data-processing", Some("rust"));
    let relevant = memory
        .retrieve_relevant_context("Process new batch".to_string(), context, 10)
        .await;
    assert_eq!(relevant.len(), 3);
}

#[tokio::test]
async fn should_learn_from_failed_episodes_with_improvement_insights() {
    // Given: A memory system and a complex task context
    let memory = SelfLearningMemory::new();
    let context = TaskContext {
        complexity: ComplexityLevel::Complex,
        domain: "distributed-systems".to_string(),
        ..Default::default()
    };

    // When: We create an episode with failing steps
    let episode_id = memory
        .start_episode(
            "Implement distributed consensus".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;
    let mut step1 = ExecutionStep::new(1, "raft_impl".to_string(), "Implement Raft".to_string());
    step1.result = Some(ExecutionResult::Error {
        message: "Network partition".to_string(),
    });
    memory.log_step(episode_id, step1).await;

    let mut step2 = ExecutionStep::new(2, "retry".to_string(), "Retry with timeout".to_string());
    step2.result = Some(ExecutionResult::Error {
        message: "Still partitioned".to_string(),
    });
    memory.log_step(episode_id, step2).await;

    // When: We complete with failure outcome
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Failure {
                reason: "Network issues prevented consensus".to_string(),
                error_details: Some("Multiple partition errors".to_string()),
            },
        )
        .await
        .unwrap();

    // Then: The failure should be recorded with improvement suggestions
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());

    let reward = episode.reward.unwrap();
    assert_eq!(reward.base, 0.0); // Failure

    let reflection = episode.reflection.unwrap();
    assert!(!reflection.improvements.is_empty());
    assert!(reflection
        .improvements
        .iter()
        .any(|i| i.contains("Task failed")));
}

#[tokio::test]
async fn should_handle_concurrent_episode_operations_safely() {
    // Given: A shared memory system
    let memory = SelfLearningMemory::new();

    // When: We create and complete multiple episodes concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            let context = TaskContext {
                domain: format!("domain_{}", i % 2),
                ..Default::default()
            };

            let episode_id = mem
                .start_episode(format!("Task {}", i), context, TaskType::CodeGeneration)
                .await;

            // Add step
            let mut step = ExecutionStep::new(1, "worker".to_string(), "Work".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Done".to_string(),
            });
            mem.log_step(episode_id, step).await;

            // Complete
            mem.complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

            episode_id
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut episode_ids = vec![];
    for handle in handles {
        let id = handle.await.unwrap();
        episode_ids.push(id);
    }

    // Then: All episodes should be created and completed safely
    assert_eq!(episode_ids.len(), 5);
    let (total, completed, _) = memory.get_stats().await;
    assert_eq!(total, 5);
    assert_eq!(completed, 5);
}

#[tokio::test]
async fn should_extract_patterns_accurately_from_error_recovery_episodes() {
    // Given: A memory system
    let memory = SelfLearningMemory::new();

    // When: We create an error recovery episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "error-handling".to_string(),
        tags: vec!["retry".to_string()],
        ..Default::default()
    };

    let episode_id = memory
        .start_episode(
            "Implement retry logic".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Simulate error recovery pattern
    let mut error_step = ExecutionStep::new(
        1,
        "initial_attempt".to_string(),
        "Try operation".to_string(),
    );
    error_step.result = Some(ExecutionResult::Error {
        message: "Connection timeout".to_string(),
    });
    memory.log_step(episode_id, error_step).await;

    let mut recovery_step = ExecutionStep::new(
        2,
        "retry_with_backoff".to_string(),
        "Retry with exponential backoff".to_string(),
    );
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Operation succeeded".to_string(),
    });
    memory.log_step(episode_id, recovery_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Retry logic implemented".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // When: We retrieve patterns for the same context
    let patterns = memory.retrieve_relevant_patterns(&context, 10).await;

    // Then: An error recovery pattern should have been extracted
    assert!(patterns
        .iter()
        .any(|p| matches!(p, memory_core::Pattern::ErrorRecovery { .. })));
}
