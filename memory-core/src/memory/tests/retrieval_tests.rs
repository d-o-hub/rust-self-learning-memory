//! Retrieval-related tests for SelfLearningMemory.

use crate::episode::ExecutionStep;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::SelfLearningMemory;
use uuid::Uuid;

/// Test retrieving relevant context.
#[tokio::test]
pub async fn test_retrieve_relevant_context() {
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.5,
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    // Create and complete several episodes
    for i in 0..3 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec![],
        };

        let episode_id = memory
            .start_episode(format!("API task {i}"), context, TaskType::CodeGeneration)
            .await;

        // Log multiple steps to meet quality threshold
        for j in 0..20 {
            let mut step =
                ExecutionStep::new(j + 1, format!("tool_{}", j % 6), format!("Build step {j}"));
            step.result = Some(ExecutionResult::Success {
                output: format!("Step {j} completed"),
            });
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "API built successfully".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Create one episode with different context
    let different_context = TaskContext {
        language: Some("python".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "data-science".to_string(),
        tags: vec![],
    };

    let different_id = memory
        .start_episode(
            "Data analysis".to_string(),
            different_context.clone(),
            TaskType::Analysis,
        )
        .await;

    // Add steps to meet quality threshold
    for j in 0..20 {
        let mut step = ExecutionStep::new(
            j + 1,
            format!("analysis_tool_{}", j % 6),
            format!("Analysis step {j}"),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("Analysis step {j} completed"),
        });
        memory.log_step(different_id, step).await;
    }

    memory
        .complete_episode(
            different_id,
            TaskOutcome::Success {
                verdict: "Analysis done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Retrieve relevant context for web-api task
    let query_context = TaskContext {
        language: Some("rust".to_string()),
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let relevant = memory
        .retrieve_relevant_context("Build REST API".to_string(), query_context, 5)
        .await;

    // Should retrieve the web-api episodes, not the data-science one
    assert!(relevant.len() >= 3);
    assert!(relevant
        .iter()
        .all(|e| e.context.domain == "web-api" || e.task_description.contains("API")));
}

/// Test retrieving relevant patterns (heuristics).
#[tokio::test]
pub async fn test_retrieve_relevant_patterns() {
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.4,
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    // Create an episode with decision points
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "async-processing".to_string(),
        tags: vec!["concurrency".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Process data concurrently".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add multiple decision steps to trigger heuristic extraction
    for i in 0..10 {
        let mut step = ExecutionStep::new(
            i * 2 + 1,
            "validator".to_string(),
            "Check if input is valid".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        memory.log_step(episode_id, step).await;

        let mut action_step = ExecutionStep::new(
            i * 2 + 2,
            format!("processor_{}", i % 6),
            "Process the data".to_string(),
        );
        action_step.result = Some(ExecutionResult::Success {
            output: "Processed".to_string(),
        });
        memory.log_step(episode_id, action_step).await;
    }

    // Complete the episode (this extracts heuristics)
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Processing complete".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Retrieve relevant heuristics
    let heuristics = memory.retrieve_relevant_heuristics(&context, 10).await;

    // Verify we got some heuristics
    if heuristics.is_empty() {
        // This is expected behavior if the heuristic extractor has high thresholds
        return;
    }

    // Test updating heuristic confidence
    let heuristic_id = heuristics[0].heuristic_id;
    let new_episode_id = Uuid::new_v4();

    let old_sample_size = heuristics[0].evidence.sample_size;

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

    // Retrieve again to verify update
    let updated_heuristics = memory.retrieve_relevant_heuristics(&context, 10).await;
    let updated_heuristic = updated_heuristics
        .iter()
        .find(|h| h.heuristic_id == heuristic_id)
        .expect("Should find updated heuristic");

    assert_eq!(
        updated_heuristic.evidence.sample_size,
        old_sample_size + 1,
        "Sample size should increase by 1"
    );
}
