//! Comprehensive integration tests for heuristic learning cycle
//!
//! Tests the complete workflow of heuristic extraction, storage, retrieval, and updates.
//! Verifies end-to-end functionality from episode completion through heuristic application.

mod common;

use common::{setup_test_memory, ContextBuilder};
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use uuid::Uuid;

/// Helper: Create an episode with decision points
async fn create_episode_with_decision_points(
    memory: &SelfLearningMemory,
    domain: &str,
    num_decisions: usize,
) -> Uuid {
    let context = ContextBuilder::new(domain)
        .language("rust")
        .framework("tokio")
        .complexity(ComplexityLevel::Moderate)
        .tag("decisions")
        .build();

    let episode_id = memory
        .start_episode(
            format!("Task with {num_decisions} decision points"),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add decision points with the same decision pattern
    for i in 0..num_decisions {
        // Decision step (contains decision keyword)
        let mut decision_step = ExecutionStep::new(
            (i * 2) + 1,
            "validator".to_string(),
            "Check if input is valid".to_string(), // Same decision text
        );
        decision_step.result = Some(ExecutionResult::Success {
            output: "Input is valid".to_string(),
        });
        memory.log_step(episode_id, decision_step).await;

        // Action step (what happens after the decision) - MUST be identical for grouping
        let mut action_step = ExecutionStep::new(
            (i * 2) + 2,
            "processor".to_string(),
            "Process the data".to_string(), // Same action for all
        );
        action_step.result = Some(ExecutionResult::Success {
            output: "Processed".to_string(),
        });
        memory.log_step(episode_id, action_step).await;
    }

    episode_id
}

#[tokio::test]
async fn test_heuristic_extraction_from_episode() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create an episode with decision points
    let context = ContextBuilder::new("data-validation")
        .language("rust")
        .complexity(ComplexityLevel::Moderate)
        .build();

    let episode_id = memory
        .start_episode(
            "Validate and process data".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add multiple similar decision points (needed for extraction threshold)
    for i in 0..3 {
        let mut decision_step = ExecutionStep::new(
            (i * 2) + 1,
            "validator".to_string(),
            "Check data integrity".to_string(), // Same condition
        );
        decision_step.result = Some(ExecutionResult::Success {
            output: "Data valid".to_string(),
        });
        memory.log_step(episode_id, decision_step).await;

        let mut action_step = ExecutionStep::new(
            (i * 2) + 2,
            "sanitizer".to_string(),
            "Sanitize input".to_string(), // Same action
        );
        action_step.result = Some(ExecutionResult::Success {
            output: "Sanitized".to_string(),
        });
        memory.log_step(episode_id, action_step).await;
    }

    // When: We complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Data validated and processed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Then: Heuristics should be extracted
    let episode = memory.get_episode(episode_id).await.unwrap();

    // Verify episode has heuristics linked
    assert!(
        !episode.heuristics.is_empty(),
        "Episode should have extracted heuristics. Got {} heuristics",
        episode.heuristics.len()
    );

    // Retrieve and verify heuristic fields
    let heuristics = memory.retrieve_relevant_heuristics(&context, 10).await;

    assert!(
        !heuristics.is_empty(),
        "Should retrieve extracted heuristics"
    );

    for heuristic in &heuristics {
        // Verify condition is not empty
        assert!(
            !heuristic.condition.is_empty(),
            "Heuristic condition should not be empty"
        );

        // Verify action is not empty
        assert!(
            !heuristic.action.is_empty(),
            "Heuristic action should not be empty"
        );

        // Verify confidence is in valid range (can exceed 1.0 due to sqrt formula)
        assert!(
            heuristic.confidence >= 0.0,
            "Confidence should be non-negative, got {}",
            heuristic.confidence
        );

        // Verify evidence
        assert!(
            heuristic.evidence.sample_size >= 2,
            "Sample size should meet minimum threshold of 2, got {}",
            heuristic.evidence.sample_size
        );
        assert!(
            heuristic.evidence.success_rate > 0.0,
            "Success rate should be positive for successful episode, got {}",
            heuristic.evidence.success_rate
        );
        assert!(
            !heuristic.evidence.episode_ids.is_empty(),
            "Evidence should contain episode IDs"
        );
    }
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn test_heuristic_storage_in_learning_cycle() {
    // Given: A memory system with storage backends
    let memory = setup_test_memory();

    // When: We complete an episode with decision points
    let episode_id = create_episode_with_decision_points(&memory, "api-validation", 3).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "API validation successful".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Then: Heuristics should be stored in the episode
    let episode = memory.get_episode(episode_id).await.unwrap();

    assert!(
        !episode.heuristics.is_empty(),
        "Episode should have heuristics stored"
    );

    // Verify data integrity: retrieve and check fields
    for heuristic_id in &episode.heuristics {
        // Try to retrieve the heuristic by context
        let context = TaskContext {
            domain: "api-validation".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            tags: vec!["decisions".to_string()],
        };

        let heuristics = memory.retrieve_relevant_heuristics(&context, 10).await;

        // Find our heuristic
        let found = heuristics.iter().any(|h| h.heuristic_id == *heuristic_id);

        assert!(
            found,
            "Should be able to retrieve stored heuristic {heuristic_id}"
        );
    }
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn test_heuristic_retrieval_by_context() {
    // Given: A memory system with heuristics in different contexts
    let memory = setup_test_memory();

    // Create episodes in different domains
    let domains = vec!["web-api", "database", "web-api"];

    for domain in &domains {
        let episode_id = create_episode_with_decision_points(&memory, domain, 3).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Completed in {domain}"),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // When: We retrieve heuristics for "web-api" context
    let web_api_context = ContextBuilder::new("web-api")
        .language("rust")
        .framework("tokio")
        .tag("decisions")
        .build();

    let web_api_heuristics = memory
        .retrieve_relevant_heuristics(&web_api_context, 10)
        .await;

    // Then: Should return heuristics from "web-api" domain
    assert!(
        !web_api_heuristics.is_empty(),
        "Should retrieve heuristics for web-api context"
    );

    // When: We retrieve heuristics for "database" context
    let database_context = ContextBuilder::new("database")
        .language("rust")
        .framework("tokio")
        .tag("decisions")
        .build();

    let database_heuristics = memory
        .retrieve_relevant_heuristics(&database_context, 10)
        .await;

    // Then: Should return heuristics from "database" domain
    assert!(
        !database_heuristics.is_empty(),
        "Should retrieve heuristics for database context"
    );

    // Verify ranking by confidence × relevance
    // Higher confidence heuristics should come first
    for i in 1..web_api_heuristics.len() {
        let prev = &web_api_heuristics[i - 1];
        let curr = &web_api_heuristics[i];

        // Note: ranking is by confidence × relevance, which is internal
        // We can at least verify all have valid confidence
        assert!(prev.confidence >= 0.0);
        assert!(curr.confidence >= 0.0);
    }
}

#[tokio::test]
#[allow(clippy::float_cmp)]
async fn test_heuristic_confidence_updates() {
    // Given: A memory system with an initial heuristic
    let memory = setup_test_memory();

    // Create initial episode with decision points
    let episode_id = create_episode_with_decision_points(&memory, "error-handling", 3).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Error handling implemented".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Get the extracted heuristic
    let context = ContextBuilder::new("error-handling")
        .language("rust")
        .framework("tokio")
        .tag("decisions")
        .build();

    let initial_heuristics = memory.retrieve_relevant_heuristics(&context, 1).await;

    assert!(
        !initial_heuristics.is_empty(),
        "Should have extracted initial heuristic"
    );

    let heuristic_id = initial_heuristics[0].heuristic_id;
    let initial_confidence = initial_heuristics[0].confidence;
    let initial_sample_size = initial_heuristics[0].evidence.sample_size;
    let initial_success_rate = initial_heuristics[0].evidence.success_rate;

    // When: We update with a successful outcome
    let new_episode_id = Uuid::new_v4();
    memory
        .update_heuristic_confidence(
            heuristic_id,
            new_episode_id,
            TaskOutcome::Success {
                verdict: "Applied heuristic successfully".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Then: Confidence should be updated
    let updated_heuristics = memory.retrieve_relevant_heuristics(&context, 1).await;

    let updated = updated_heuristics
        .iter()
        .find(|h| h.heuristic_id == heuristic_id)
        .expect("Should find updated heuristic");

    // Verify sample size increased
    assert_eq!(
        updated.evidence.sample_size,
        initial_sample_size + 1,
        "Sample size should increase by 1"
    );

    // Verify success rate changed (since we added a success)
    assert!(
        updated.evidence.success_rate >= initial_success_rate,
        "Success rate should increase or stay same after successful outcome"
    );

    // Verify confidence recalculated (success_rate × √sample_size)
    #[allow(clippy::cast_precision_loss)]
    let expected_confidence =
        updated.evidence.success_rate * (updated.evidence.sample_size as f32).sqrt();
    assert!(
        (updated.confidence - expected_confidence).abs() < 0.01,
        "Confidence should be recalculated correctly. Expected ~{}, got {}",
        expected_confidence,
        updated.confidence
    );

    // When: We update with a failure outcome
    let failure_episode_id = Uuid::new_v4();
    memory
        .update_heuristic_confidence(
            heuristic_id,
            failure_episode_id,
            TaskOutcome::Failure {
                reason: "Heuristic didn't work".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    // Then: Confidence should decrease
    let final_heuristics = memory.retrieve_relevant_heuristics(&context, 1).await;

    let final_heuristic = final_heuristics
        .iter()
        .find(|h| h.heuristic_id == heuristic_id)
        .expect("Should find heuristic after failure update");

    // Verify sample size increased again
    assert_eq!(
        final_heuristic.evidence.sample_size,
        initial_sample_size + 2,
        "Sample size should increase by 2 after two updates"
    );

    // Verify success rate decreased (failure added)
    assert!(
        final_heuristic.evidence.success_rate < updated.evidence.success_rate,
        "Success rate should decrease after failure"
    );

    // Verify changes persisted
    assert_ne!(
        final_heuristic.confidence, initial_confidence,
        "Confidence should have changed from initial value"
    );
}

#[tokio::test]
async fn test_heuristic_filtering_by_confidence() {
    // Given: A memory system configured with default thresholds
    let memory = setup_test_memory();

    // When: We create an episode with high success (should extract)
    let high_success_id = create_episode_with_decision_points(&memory, "high-confidence", 4).await;

    memory
        .complete_episode(
            high_success_id,
            TaskOutcome::Success {
                verdict: "Highly successful".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // When: We create an episode with partial success (may not extract due to threshold)
    let context = ContextBuilder::new("low-confidence")
        .language("rust")
        .complexity(ComplexityLevel::Simple)
        .build();

    let partial_id = memory
        .start_episode(
            "Partial success task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add only 1 decision point (below min_sample_size of 2)
    let mut decision_step =
        ExecutionStep::new(1, "validator".to_string(), "Check if valid".to_string());
    decision_step.result = Some(ExecutionResult::Success {
        output: "Valid".to_string(),
    });
    memory.log_step(partial_id, decision_step).await;

    memory
        .complete_episode(
            partial_id,
            TaskOutcome::PartialSuccess {
                verdict: "Partially successful".to_string(),
                completed: vec!["part1".to_string()],
                failed: vec!["part2".to_string()],
            },
        )
        .await
        .unwrap();

    // Then: High confidence heuristics should be extracted
    let high_context = ContextBuilder::new("high-confidence")
        .language("rust")
        .framework("tokio")
        .tag("decisions")
        .build();

    let high_heuristics = memory.retrieve_relevant_heuristics(&high_context, 10).await;

    // Should have extracted heuristics from successful episode with sufficient samples
    assert!(
        !high_heuristics.is_empty(),
        "Should extract heuristics from high-confidence episode"
    );

    // All extracted heuristics should meet confidence threshold (default 0.7)
    for heuristic in &high_heuristics {
        assert!(
            heuristic.confidence >= 0.7,
            "Extracted heuristic should meet minimum confidence threshold of 0.7, got {}",
            heuristic.confidence
        );
    }

    // Low confidence heuristics should be filtered out
    let low_episode = memory.get_episode(partial_id).await.unwrap();
    // Should have no heuristics due to low sample size and/or confidence
    // (1 decision point < min_sample_size of 2)
    assert!(
        low_episode.heuristics.is_empty(),
        "Low confidence episode should not have extracted heuristics"
    );
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn test_end_to_end_heuristic_learning() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create multiple episodes with similar decision points
    for i in 0..3 {
        let episode_id = create_episode_with_decision_points(&memory, "authentication", 3).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Authentication flow {i} completed"),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Then: Heuristics should be extracted and grouped
    let auth_context = ContextBuilder::new("authentication")
        .language("rust")
        .framework("tokio")
        .tag("decisions")
        .build();

    let learned_heuristics = memory.retrieve_relevant_heuristics(&auth_context, 10).await;

    assert!(
        !learned_heuristics.is_empty(),
        "Should have learned heuristics from multiple episodes"
    );

    // Verify heuristics have multiple episodes as evidence
    for heuristic in &learned_heuristics {
        assert!(
            heuristic.evidence.sample_size >= 2,
            "Heuristics should be based on multiple samples, got {}",
            heuristic.evidence.sample_size
        );

        // High success rate since all episodes succeeded
        assert!(
            heuristic.evidence.success_rate >= 0.9,
            "Success rate should be high for successful episodes, got {}",
            heuristic.evidence.success_rate
        );
    }

    // When: We retrieve heuristics for a new similar task
    let new_task_context = ContextBuilder::new("authentication")
        .language("rust")
        .framework("tokio")
        .tag("security")
        .build();

    let relevant = memory
        .retrieve_relevant_heuristics(&new_task_context, 5)
        .await;

    // Then: Learned heuristics should guide the new task
    assert!(
        !relevant.is_empty(),
        "Should retrieve relevant learned heuristics for similar task"
    );

    // Verify the heuristics are actually relevant
    for heuristic in &relevant {
        // Should have meaningful condition and action
        assert!(heuristic.condition.len() > 10);
        assert!(heuristic.action.len() > 10);

        // Should have reasonable confidence
        assert!(heuristic.confidence > 0.0);
    }

    // Verify we can access the heuristics' conditions and actions
    let first_heuristic = &relevant[0];
    assert!(
        first_heuristic.condition.contains("authentication")
            || first_heuristic.condition.contains("rust")
            || first_heuristic.condition.contains("Check")
            || first_heuristic.condition.contains("Validate"),
        "Heuristic condition should be contextually relevant: {}",
        first_heuristic.condition
    );
}

#[tokio::test]
async fn test_no_heuristic_extraction_from_incomplete_episode() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create an episode with decision points but don't complete it
    let context = ContextBuilder::new("incomplete-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Incomplete task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add decision steps
    for i in 0..3 {
        let mut step =
            ExecutionStep::new(i + 1, "validator".to_string(), "Check validity".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        memory.log_step(episode_id, step).await;
    }

    // Don't complete the episode!

    // Then: Episode should have no heuristics
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(
        episode.heuristics.is_empty(),
        "Incomplete episode should have no heuristics"
    );
    assert!(!episode.is_complete());
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn test_no_heuristic_extraction_from_failed_episode() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We create an episode that fails
    let episode_id = create_episode_with_decision_points(&memory, "failed-task", 3).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Failure {
                reason: "Task failed completely".to_string(),
                error_details: Some("Critical error".to_string()),
            },
        )
        .await
        .unwrap();

    // Then: Should not extract heuristics from failed episodes
    let episode = memory.get_episode(episode_id).await.unwrap();

    assert!(
        episode.heuristics.is_empty(),
        "Failed episode should not have extracted heuristics"
    );

    // Verify retrieval doesn't return heuristics from failed episode
    let context = ContextBuilder::new("failed-task")
        .language("rust")
        .framework("tokio")
        .build();

    let heuristics = memory.retrieve_relevant_heuristics(&context, 10).await;

    // Should be empty since the only episode failed
    assert!(
        heuristics.is_empty(),
        "Should not retrieve heuristics from failed episodes"
    );
}

#[tokio::test]
async fn test_heuristic_edge_cases() {
    // Given: A memory system
    let memory = setup_test_memory();

    // Edge case 1: Episode with no decision points
    let context = ContextBuilder::new("no-decisions").language("rust").build();

    let no_decision_id = memory
        .start_episode(
            "Task without decisions".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add regular steps (no decision keywords)
    let mut step = ExecutionStep::new(1, "reader".to_string(), "Read file".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Read".to_string(),
    });
    memory.log_step(no_decision_id, step).await;

    memory
        .complete_episode(
            no_decision_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Then: No heuristics should be extracted
    let episode = memory.get_episode(no_decision_id).await.unwrap();
    assert!(
        episode.heuristics.is_empty(),
        "Episode without decision points should have no heuristics"
    );

    // Edge case 2: Episode with all failed decision steps
    let all_failed_id = memory
        .start_episode(
            "All failures".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    for i in 0..3 {
        let mut step =
            ExecutionStep::new(i + 1, "validator".to_string(), "Check if valid".to_string());
        step.result = Some(ExecutionResult::Error {
            message: "Validation failed".to_string(),
        });
        memory.log_step(all_failed_id, step).await;
    }

    memory
        .complete_episode(
            all_failed_id,
            TaskOutcome::Failure {
                reason: "All validations failed".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    // Then: No heuristics from failed episode
    let failed_episode = memory.get_episode(all_failed_id).await.unwrap();
    assert!(
        failed_episode.heuristics.is_empty(),
        "Episode with all failed steps should have no heuristics"
    );
}
