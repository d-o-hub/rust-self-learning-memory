//! Episode-related tests for `SelfLearningMemory`.

use crate::SelfLearningMemory;
use crate::episode::ExecutionStep;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

/// Test starting an episode.
#[tokio::test]
pub async fn test_start_episode() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["async".to_string()],
    };

    let episode_id = memory
        .start_episode("Test task".to_string(), context.clone(), TaskType::Testing)
        .await;

    // Verify episode was created
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test task");
    assert_eq!(episode.context.domain, "testing");
    assert!(!episode.is_complete());
}

/// Test logging execution steps.
#[tokio::test]
pub async fn test_log_steps() {
    let memory = SelfLearningMemory::new();

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Log some steps
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        memory.log_step(episode_id, step).await;
    }

    // Flush buffered steps (if batching enabled)
    memory.flush_steps(episode_id).await.unwrap();

    // Verify steps were logged
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 3);
}

/// Test completing an episode.
#[tokio::test(flavor = "multi_thread")]
pub async fn test_complete_episode() {
    // Optimized config for fast test execution
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.5,
        pattern_extraction_threshold: 1.0, // Skip pattern extraction (quality < 1.0)
        enable_summarization: false,       // Skip semantic summarization
        enable_embeddings: false,          // Skip embedding generation
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Log multiple steps to meet quality threshold
    for i in 0..20 {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Test action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {i} passed"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Complete the episode
    let outcome = TaskOutcome::Success {
        verdict: "Tests passed".to_string(),
        artifacts: vec!["test_results.json".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Verify episode was completed and analyzed
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());
    assert!(episode.reflection.is_some());

    // Note: Pattern extraction is skipped in tests for performance
    // (pattern_extraction_threshold: 1.0 means no patterns will be extracted)
}

// ============================================================================
// ACT-026: Additional episode lifecycle tests
// ============================================================================

/// Helper to create test memory with zero quality threshold
fn create_test_memory() -> SelfLearningMemory {
    let config = crate::MemoryConfig {
        quality_threshold: 0.0, // Accept any episode
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        ..Default::default()
    };
    SelfLearningMemory::with_config(config)
}

/// Test complete episode lifecycle with TaskOutcome::Success with artifacts
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_lifecycle_success_with_artifacts() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Implement feature".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Complex,
                domain: "web-api".to_string(),
                tags: vec!["feature".to_string()],
            },
            TaskType::CodeGeneration,
        )
        .await;

    // Log steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("Action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {i} done"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Complete with success and multiple artifacts
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Feature implemented with tests".to_string(),
                artifacts: vec![
                    "src/feature.rs".to_string(),
                    "tests/feature_test.rs".to_string(),
                    "docs/feature.md".to_string(),
                ],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());

    let reward = episode.reward.unwrap();
    assert_eq!(reward.base, 1.0); // Success
    assert!(reward.quality_multiplier > 1.0); // Multiple artifacts bonus
}

/// Test complete episode lifecycle with TaskOutcome::PartialSuccess
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_lifecycle_partial_success() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Deploy services".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: "deployment".to_string(),
                tags: vec!["deploy".to_string()],
            },
            TaskType::Other,
        )
        .await;

    // Log steps with some failures
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("deploy_step_{i}"), format!("Deploy {i}"));
        if i < 3 {
            step.result = Some(ExecutionResult::Success {
                output: "Deployed".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Error {
                message: "Failed".to_string(),
            });
        }
        memory.log_step(episode_id, step).await;
    }

    // Complete with partial success
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::PartialSuccess {
                verdict: "Some services deployed".to_string(),
                completed: vec!["service-a".to_string(), "service-b".to_string()],
                failed: vec!["service-c".to_string()],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());

    let reward = episode.reward.unwrap();
    // 2 out of 3 completed = 0.667
    assert!((reward.base - 0.667).abs() < 0.01);
}

/// Test complete episode lifecycle with TaskOutcome::Failure
#[tokio::test]
pub async fn test_episode_lifecycle_failure() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Fix critical bug".to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Simple,
                domain: "debugging".to_string(),
                tags: vec!["bug".to_string()],
            },
            TaskType::Debugging,
        )
        .await;

    // Log steps with errors
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("debug_step_{i}"), format!("Debug {i}"));
        step.result = Some(ExecutionResult::Error {
            message: format!("Error in step {i}"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Complete with failure
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Failure {
                reason: "Could not reproduce bug".to_string(),
                error_details: Some("Stack trace unavailable".to_string()),
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());

    let reward = episode.reward.unwrap();
    assert_eq!(reward.base, 0.0); // Failure
}

/// Test episode with error recovery pattern
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_error_recovery_pattern() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Retry operation".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Error followed by recovery
    let mut error_step =
        ExecutionStep::new(1, "operation".to_string(), "Attempt operation".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Connection failed".to_string(),
    });
    memory.log_step(episode_id, error_step).await;

    let mut retry_step = ExecutionStep::new(
        2,
        "retry_handler".to_string(),
        "Retry with backoff".to_string(),
    );
    retry_step.result = Some(ExecutionResult::Success {
        output: "Succeeded after retry".to_string(),
    });
    memory.log_step(episode_id, retry_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Operation completed after retry".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    let reward = episode.reward.unwrap();

    // Should have learning bonus for error recovery
    assert!(reward.learning_bonus > 0.0);
}

/// Test episode with Timeout result
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_with_timeout_result() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Long running task".to_string(),
            TaskContext::default(),
            TaskType::Analysis,
        )
        .await;

    // Add a timeout step
    let mut timeout_step = ExecutionStep::new(
        1,
        "slow_operation".to_string(),
        "Analyze large dataset".to_string(),
    );
    timeout_step.result = Some(ExecutionResult::Timeout);
    memory.log_step(episode_id, timeout_step).await;

    // Add successful step after timeout
    let mut success_step =
        ExecutionStep::new(2, "fallback".to_string(), "Use cached result".to_string());
    success_step.result = Some(ExecutionResult::Success {
        output: "Used fallback".to_string(),
    });
    memory.log_step(episode_id, success_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Completed with fallback".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

/// Test multiple episodes can be created and tracked
#[tokio::test]
pub async fn test_multiple_episodes() {
    let memory = create_test_memory();

    let mut episode_ids = Vec::new();

    // Create multiple episodes
    for i in 0..5 {
        let episode_id = memory
            .start_episode(
                format!("Task {i}"),
                TaskContext {
                    domain: format!("domain_{i}"),
                    ..TaskContext::default()
                },
                TaskType::Testing,
            )
            .await;
        episode_ids.push(episode_id);
    }

    // Verify all episodes exist
    for (i, id) in episode_ids.iter().enumerate() {
        let episode = memory.get_episode(*id).await.unwrap();
        assert_eq!(episode.task_description, format!("Task {i}"));
    }
}

/// Test get_episode returns error for non-existent episode
#[tokio::test]
pub async fn test_get_nonexistent_episode() {
    let memory = SelfLearningMemory::new();

    let fake_id = uuid::Uuid::new_v4();
    let result = memory.get_episode(fake_id).await;

    assert!(result.is_err());
}

/// Test episode with diverse tool usage
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_diverse_tool_usage() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Complex task".to_string(),
            TaskContext {
                complexity: ComplexityLevel::Complex,
                ..TaskContext::default()
            },
            TaskType::CodeGeneration,
        )
        .await;

    // Use many different tools (5+)
    let tools = vec![
        "file_reader",
        "parser",
        "analyzer",
        "generator",
        "formatter",
        "linter",
    ];

    for (i, tool) in tools.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 1, tool.to_string(), format!("Use {tool}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("{tool} done"),
        });
        memory.log_step(episode_id, step).await;
    }

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Complex task completed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    let reward = episode.reward.unwrap();

    // Should have learning bonus for diverse tools (5+)
    assert!(reward.learning_bonus >= 0.15);
}

/// Test episode with all task types
#[tokio::test]
pub async fn test_episode_all_task_types() {
    let memory = create_test_memory();

    let task_types = vec![
        TaskType::CodeGeneration,
        TaskType::Debugging,
        TaskType::Refactoring,
        TaskType::Testing,
        TaskType::Analysis,
        TaskType::Documentation,
        TaskType::Other,
    ];

    for (i, task_type) in task_types.into_iter().enumerate() {
        let episode_id = memory
            .start_episode(
                format!("Task type test {i}"),
                TaskContext::default(),
                task_type,
            )
            .await;

        // Log a step
        let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "Done".to_string(),
        });
        memory.log_step(episode_id, step).await;

        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_type, task_type);
    }
}

/// Test episode with all complexity levels
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_all_complexity_levels() {
    let memory = create_test_memory();

    let complexities = vec![
        ComplexityLevel::Simple,
        ComplexityLevel::Moderate,
        ComplexityLevel::Complex,
    ];

    for complexity in complexities {
        let episode_id = memory
            .start_episode(
                format!("{complexity:?} task"),
                TaskContext {
                    complexity,
                    ..TaskContext::default()
                },
                TaskType::Testing,
            )
            .await;

        // Log steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("Action {i}"));
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("{complexity:?} completed"),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        let episode = memory.get_episode(episode_id).await.unwrap();
        let reward = episode.reward.unwrap();

        match complexity {
            ComplexityLevel::Simple => assert_eq!(reward.complexity_bonus, 1.0),
            ComplexityLevel::Moderate => assert_eq!(reward.complexity_bonus, 1.1),
            ComplexityLevel::Complex => assert_eq!(reward.complexity_bonus, 1.2),
        }
    }
}

/// Test episode flush_steps is idempotent
#[tokio::test]
pub async fn test_flush_steps_idempotent() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Log steps
    for i in 0..3 {
        let step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        memory.log_step(episode_id, step).await;
    }

    // Flush multiple times
    memory.flush_steps(episode_id).await.unwrap();
    memory.flush_steps(episode_id).await.unwrap();
    memory.flush_steps(episode_id).await.unwrap();

    // Steps should still be there
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 3);
}

/// Test episode with very long task description
#[tokio::test]
pub async fn test_episode_long_task_description() {
    let memory = create_test_memory();

    let long_description = "a".repeat(1000);

    let episode_id = memory
        .start_episode(
            long_description.clone(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, long_description);
}

/// Test episode with special characters in context
#[tokio::test]
pub async fn test_episode_special_characters_in_context() {
    let memory = create_test_memory();

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio-1.0".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api/v2".to_string(),
        tags: vec!["feature-123".to_string(), "release/1.0".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Task with special chars: @#$%".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.context.domain, "web-api/v2");
    assert!(episode.context.tags.contains(&"feature-123".to_string()));
}

/// Test episode complete without any steps
#[tokio::test(flavor = "multi_thread")]
pub async fn test_episode_complete_without_steps() {
    let memory = create_test_memory();

    let episode_id = memory
        .start_episode(
            "Empty task".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Complete without any steps
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "No steps needed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 0);
}
