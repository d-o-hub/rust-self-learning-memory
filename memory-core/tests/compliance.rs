//! BDD-style compliance tests for functional requirements FR1-FR7
//!
//! This test suite validates that the memory system meets all documented
//! functional requirements from plans/04-review.md.
//!
//! All tests follow the Given-When-Then pattern for clarity and focus on
//! observable behaviors rather than implementation details.

mod common;

use chrono::Utc;
use common::{
    ContextBuilder, PatternType, StepBuilder, create_completed_episode_with_pattern,
    create_test_episode_with_domain, setup_simple_test_memory, setup_test_memory, test_context,
};
use memory_core::{Pattern, TaskOutcome, TaskType};
use serde_json::json;
use uuid::Uuid;

// ============================================================================
// FR1: Episode Creation
// ============================================================================

#[tokio::test]
async fn should_create_episodes_with_unique_ids_and_timestamps() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create a new episode
    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // Then: The episode should have a unique ID and valid timestamp
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_ne!(episode.episode_id, Uuid::nil());
    assert_eq!(episode.episode_id, episode_id);
    assert!(episode.start_time <= Utc::now());
    assert!(episode.end_time.is_none());
    assert!(!episode.is_complete());
    assert_eq!(episode.steps.len(), 0);
    assert!(episode.outcome.is_none());

    // Given: Multiple episodes are created
    let mut episode_ids = Vec::new();
    for i in 0..10 {
        let id = memory
            .start_episode(
                format!("Task {i}"),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;
        episode_ids.push(id);
    }

    // Then: All episode IDs should be unique
    for i in 0..episode_ids.len() {
        for j in i + 1..episode_ids.len() {
            assert_ne!(episode_ids[i], episode_ids[j], "Episode IDs must be unique");
        }
    }
}

// ============================================================================
// FR2: Step Logging
// ============================================================================

#[tokio::test]
async fn should_log_execution_steps_with_ordering_and_metadata() {
    // Given: An episode in the memory system
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // When: We log a step with tool usage and metadata
    let step = StepBuilder::new(1, "test_tool", "Test action")
        .parameters(json!({"key": "value"}))
        .latency_ms(10)
        .tokens_used(50)
        .success("Success")
        .build();
    memory.log_step(episode_id, step).await;

    // Flush buffered steps before checking
    memory.flush_steps(episode_id).await.unwrap();

    // Then: The step should be recorded with all metadata
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 1);
    assert_eq!(episode.steps[0].tool, "test_tool");
    assert_eq!(episode.steps[0].latency_ms, 10);
    assert_eq!(episode.steps[0].tokens_used, Some(50));
    assert!(episode.steps[0].is_success());

    // When: We log multiple steps in sequence
    for i in 2..=5 {
        let step = StepBuilder::new(i, format!("tool_{i}"), format!("Action {i}"))
            .success("OK")
            .build();
        memory.log_step(episode_id, step).await;
    }

    // Flush buffered steps before checking
    memory.flush_steps(episode_id).await.unwrap();

    // Then: Steps should be ordered correctly
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 5);
    for (i, step) in episode.steps.iter().enumerate() {
        assert_eq!(step.step_number, i + 1);
    }

    // When: We log a step with complex metadata
    let metadata_step = StepBuilder::new(6, "metadata_tool", "Metadata action")
        .latency_ms(150)
        .tokens_used(1200)
        .parameters(json!({"model": "claude-3", "temperature": 0.7}))
        .metadata("custom_key", "custom_value")
        .success("OK")
        .build();
    memory.log_step(episode_id, metadata_step).await;

    // Flush buffered steps before checking
    memory.flush_steps(episode_id).await.unwrap();

    // Then: All metadata should be preserved
    let episode = memory.get_episode(episode_id).await.unwrap();
    let recorded_step = &episode.steps[5];
    assert_eq!(recorded_step.latency_ms, 150);
    assert_eq!(recorded_step.tokens_used, Some(1200));
    assert_eq!(recorded_step.parameters["model"], "claude-3");
    assert_eq!(
        recorded_step.metadata.get("custom_key"),
        Some(&"custom_value".to_string())
    );
}

// ============================================================================
// FR3: Episode Completion
// ============================================================================

#[tokio::test]
async fn should_complete_episodes_with_reward_scoring_and_reflection() {
    // Given: An episode in the memory system
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // When: We complete the episode with a successful outcome
    let outcome = TaskOutcome::Success {
        verdict: "Test passed".to_string(),
        artifacts: vec!["test.rs".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Then: The episode should have end time, reward, and reflection
    let completed = memory.get_episode(episode_id).await.unwrap();
    assert!(completed.end_time.is_some());
    assert!(completed.reward.is_some());
    assert!(completed.reflection.is_some());

    // And: The reward should reflect success
    let reward = completed.reward.unwrap();
    assert!(reward.total >= 0.0);
    assert!((reward.base - 1.0).abs() < f32::EPSILON); // Success

    // And: The reflection should contain insights or successes
    let reflection = completed.reflection.unwrap();
    assert!(!reflection.successes.is_empty() || !reflection.insights.is_empty());
}

#[tokio::test]
async fn should_handle_failed_episodes_with_improvements() {
    // Given: An episode in the memory system
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // When: We complete the episode with a failure outcome
    let outcome = TaskOutcome::Failure {
        reason: "Test failed".to_string(),
        error_details: Some("Assertion error".to_string()),
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Then: The reward should reflect failure
    let completed = memory.get_episode(episode_id).await.unwrap();
    let reward = completed.reward.unwrap();
    assert!((reward.base - 0.0).abs() < f32::EPSILON); // Failure

    // And: The reflection should contain improvement suggestions
    let reflection = completed.reflection.unwrap();
    assert!(!reflection.improvements.is_empty());
}

#[tokio::test]
async fn should_score_partial_success_between_failure_and_success() {
    // Given: An episode in the memory system
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // When: We complete the episode with a partial success outcome
    let outcome = TaskOutcome::PartialSuccess {
        verdict: "Some tests passed".to_string(),
        completed: vec!["test1".to_string(), "test2".to_string()],
        failed: vec!["test3".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Then: The reward should be between failure (0.0) and full success (1.0)
    let completed = memory.get_episode(episode_id).await.unwrap();
    let reward = completed.reward.unwrap();
    assert!(reward.base >= 0.5 && reward.base < 1.0);
    assert!(reward.total > 0.0);
}

// ============================================================================
// FR4: Pattern Extraction
// ============================================================================

#[tokio::test]
async fn should_extract_patterns_from_completed_episodes() {
    // Given: A memory system with a completed episode containing clear patterns
    let memory = setup_test_memory();
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;

    // When: We retrieve the completed episode
    let episode = memory.get_episode(episode_id).await.unwrap();

    // Then: Patterns should have been extracted during completion
    assert!(
        !episode.patterns.is_empty(),
        "Expected patterns to be extracted from episode with clear retry pattern"
    );

    // When: We retrieve patterns for the same context
    let context = ContextBuilder::new("error-handling").build();
    let patterns = memory.retrieve_relevant_patterns(&context, 10).await;

    // Then: The system should return extracted patterns
    assert!(!patterns.is_empty(), "Expected at least one pattern");

    // And: We should find the error recovery pattern
    let has_error_recovery = patterns
        .iter()
        .any(|p| matches!(p, Pattern::ErrorRecovery { .. }));
    assert!(
        has_error_recovery,
        "Expected ErrorRecovery pattern to be extracted"
    );
}

#[tokio::test]
async fn should_extract_different_pattern_types_based_on_episode_structure() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create an episode with a tool sequence
    let episode_id = memory
        .start_episode(
            "Multi-step task".to_string(),
            test_context(),
            TaskType::CodeGeneration,
        )
        .await;

    // And: Add sequential steps
    for i in 1..=3 {
        let step = StepBuilder::new(i, format!("tool_{i}"), format!("Action {i}"))
            .success("Done")
            .build();
        memory.log_step(episode_id, step).await;
    }

    // And: Complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Then: Patterns should be extracted from the episode
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(
        !episode.patterns.is_empty(),
        "Expected patterns from multi-step episode"
    );
}

// ============================================================================
// FR5: Episode Retrieval
// ============================================================================

#[tokio::test]
async fn should_retrieve_relevant_episodes_with_context_filtering_and_limits() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create 20 episodes across different domains
    for i in 0..20 {
        let domain = if i % 2 == 0 { "web-api" } else { "cli-tool" };
        create_test_episode_with_domain(&memory, domain).await;
    }

    // And: We query with a web-api context
    let web_context = ContextBuilder::new("web-api").build();
    let results = memory
        .retrieve_relevant_context("test query".to_string(), web_context, 10)
        .await;

    // Then: Results should prioritize matching domain and respect limits
    assert!(!results.is_empty());
    assert!(results.len() <= 10);
    let web_count = results
        .iter()
        .filter(|e| e.context.domain == "web-api")
        .count();
    {
        #[allow(clippy::cast_precision_loss)]
        let ratio = web_count as f32 / results.len() as f32;
        assert!(
            ratio > 0.5,
            "Expected majority of results to match domain filter"
        );
    }

    // Given: A memory system with 50 episodes (using simple setup for predictable limits)
    let memory2 = setup_simple_test_memory();
    for _i in 0..50 {
        create_test_episode_with_domain(&memory2, "test-domain").await;
    }

    // When: We query with different retrieval limits
    let context = ContextBuilder::new("test-domain").build();
    let results_5 = memory2
        .retrieve_relevant_context("query".to_string(), context.clone(), 5)
        .await;
    let results_20 = memory2
        .retrieve_relevant_context("query".to_string(), context, 20)
        .await;

    // Then: The system should respect the specified limits
    assert_eq!(results_5.len(), 5, "Should respect limit of 5");
    assert_eq!(results_20.len(), 20, "Should respect limit of 20");

    // Given: A memory system with episodes in different languages
    // Use simple setup to ensure language filtering works predictably
    let memory3 = setup_simple_test_memory();
    for lang in ["rust", "python", "typescript"] {
        let context = ContextBuilder::new("code-gen").language(lang).build();

        for _ in 0..5 {
            let episode_id = memory3
                .start_episode(
                    "Task".to_string(),
                    context.clone(),
                    TaskType::CodeGeneration,
                )
                .await;

            memory3
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
    }

    // Verify episodes were stored
    let (total, completed, _) = memory3.get_stats().await;
    assert_eq!(total, 15, "Should have 15 episodes total");
    assert_eq!(completed, 15, "All 15 episodes should be completed");

    // When: We query for rust-specific episodes
    let rust_context = ContextBuilder::new("code-gen").language("rust").build();
    let results = memory3
        .retrieve_relevant_context("task".to_string(), rust_context, 10)
        .await;

    // Then: The system should return rust episodes
    let rust_count = results
        .iter()
        .filter(|e| e.context.language.as_deref() == Some("rust"))
        .count();
    assert!(
        !results.is_empty(),
        "Should return some results (got {} results)",
        results.len()
    );
    assert!(
        rust_count > 0,
        "Should return rust episodes (got {} results, {} rust)",
        results.len(),
        rust_count
    );
}

// ============================================================================
// FR6 & FR7: MCP Integration (Placeholder)
// ============================================================================
// Note: These tests would require the MCP server implementation.
// Included as placeholders for future implementation.

#[tokio::test]
#[ignore = "Requires MCP server implementation"]
async fn should_execute_typescript_code_in_secure_sandbox() {
    // Given: A TypeScript code snippet and execution environment
    // When: The code is executed in the sandbox
    // Then: The system should:
    //   - Execute code in isolated environment
    //   - Enforce security boundaries
    //   - Handle timeouts appropriately
    //   - Capture and return results
}

#[tokio::test]
#[ignore = "Requires MCP server implementation"]
async fn should_generate_mcp_tools_from_memory_patterns() {
    // Given: Patterns stored in the memory system
    // When: We request tool generation from patterns
    // Then: The system should:
    //   - Generate MCP tools from patterns
    //   - Provide correct tool metadata
    //   - Support tool invocation
}

// ============================================================================
// Additional Compliance Tests
// ============================================================================

#[tokio::test]
async fn should_maintain_episode_integrity_after_completion() {
    // Given: A completed episode
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // When: We attempt to log a step to the completed episode
    let step = StepBuilder::new(1, "test_tool", "Test action")
        .success("OK")
        .build();
    memory.log_step(episode_id, step).await;

    // Then: The episode should still show as complete (not panic or corrupt)
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

#[tokio::test]
async fn should_report_accurate_statistics() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create 5 episodes and complete 3 of them
    for i in 0..5 {
        let episode_id = memory
            .start_episode(
                format!("Task {i}"),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;

        if i < 3 {
            memory
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
    }

    // Then: The statistics should accurately reflect the counts
    let (total, completed, _patterns) = memory.get_stats().await;
    assert_eq!(total, 5);
    assert_eq!(completed, 3);
}

// Note: Concurrent episode creation test is covered in learning_cycle.rs
