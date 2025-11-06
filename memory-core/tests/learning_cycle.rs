//! Integration tests for the complete learning cycle

use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};

#[tokio::test]
async fn test_complete_learning_cycle() {
    let memory = SelfLearningMemory::new();

    // Phase 1: Start Episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "async-web-api".to_string(),
        tags: vec!["concurrency".to_string(), "rest".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Implement async REST API endpoint".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Verify episode was created
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(!episode.is_complete());
    assert_eq!(episode.task_type, TaskType::CodeGeneration);

    // Phase 2: Log Execution Steps
    let steps = [
        ("analyzer", "Analyze requirements", true),
        ("designer", "Design API structure", true),
        ("builder", "Build endpoint handler", true),
        ("validator", "Validate request/response", true),
        ("tester", "Run integration tests", true),
    ];

    for (i, (tool, action, success)) in steps.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 1, tool.to_string(), action.to_string());
        step.latency_ms = 100 + (i as u64 * 50);
        step.result = Some(if *success {
            ExecutionResult::Success {
                output: format!("{} completed", action),
            }
        } else {
            ExecutionResult::Error {
                message: format!("{} failed", action),
            }
        });
        memory.log_step(episode_id, step).await;
    }

    // Phase 3: Complete Episode
    let outcome = TaskOutcome::Success {
        verdict: "REST API endpoint implemented successfully".to_string(),
        artifacts: vec!["api/handlers.rs".to_string(), "api/routes.rs".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Verify episode completion and learning
    let completed_episode = memory.get_episode(episode_id).await.unwrap();
    assert!(completed_episode.is_complete());
    assert!(completed_episode.reward.is_some());
    assert!(completed_episode.reflection.is_some());
    assert!(!completed_episode.patterns.is_empty());

    // Check reward
    let reward = completed_episode.reward.unwrap();
    assert_eq!(reward.base, 1.0); // Success
    assert!(reward.total > 0.0);
    assert_eq!(reward.complexity_bonus, 1.1); // Moderate complexity

    // Check reflection
    let reflection = completed_episode.reflection.unwrap();
    assert!(!reflection.successes.is_empty());
    assert!(!reflection.insights.is_empty());

    // Phase 4: Retrieve Relevant Context
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

    assert!(!relevant.is_empty());
    assert_eq!(relevant[0].episode_id, episode_id);

    // Retrieve patterns
    let patterns = memory
        .retrieve_relevant_patterns(&similar_context, 10)
        .await;
    assert!(!patterns.is_empty());
}

#[tokio::test]
async fn test_multiple_episodes_learning() {
    let memory = SelfLearningMemory::new();

    // Create multiple episodes in the same domain
    for i in 0..3 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            domain: "data-processing".to_string(),
            tags: vec!["batch".to_string()],
        };

        let episode_id = memory
            .start_episode(
                format!("Process batch {}", i),
                context,
                TaskType::CodeGeneration,
            )
            .await;

        // Add steps
        for j in 0..3 {
            let mut step = ExecutionStep::new(
                j + 1,
                format!("processor_{}", j),
                "Process data".to_string(),
            );
            step.result = Some(ExecutionResult::Success {
                output: "Processed".to_string(),
            });
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

    // Check statistics
    let (total, completed, patterns) = memory.get_stats().await;
    assert_eq!(total, 3);
    assert_eq!(completed, 3);
    assert!(patterns > 0);

    // Retrieve context should return all relevant episodes
    let context = TaskContext {
        domain: "data-processing".to_string(),
        ..Default::default()
    };

    let relevant = memory
        .retrieve_relevant_context("Process new batch".to_string(), context, 10)
        .await;

    assert_eq!(relevant.len(), 3);
}

#[tokio::test]
async fn test_failure_episode_learning() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        complexity: ComplexityLevel::Complex,
        domain: "distributed-systems".to_string(),
        ..Default::default()
    };

    let episode_id = memory
        .start_episode(
            "Implement distributed consensus".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps with failures
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

    // Complete with failure
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

    // Verify failure is recorded
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
async fn test_concurrent_episode_handling() {
    let memory = SelfLearningMemory::new();

    // Start multiple episodes concurrently
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

    // Verify all episodes were created and completed
    assert_eq!(episode_ids.len(), 5);

    let (total, completed, _) = memory.get_stats().await;
    assert_eq!(total, 5);
    assert_eq!(completed, 5);
}

#[tokio::test]
async fn test_pattern_extraction_accuracy() {
    let memory = SelfLearningMemory::new();

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

    // Retrieve patterns
    let patterns = memory.retrieve_relevant_patterns(&context, 10).await;

    // Should have extracted error recovery pattern
    assert!(patterns
        .iter()
        .any(|p| matches!(p, memory_core::Pattern::ErrorRecovery { .. })));
}
