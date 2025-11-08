//! Compliance tests for functional requirements FR1-FR7
//!
//! This test suite validates that the memory system meets all documented
//! functional requirements from plans/04-review.md.

use chrono::Utc;
use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, Pattern, TaskContext, TaskOutcome, TaskType,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test memory instance with default configuration
fn setup_test_memory() -> SelfLearningMemory {
    SelfLearningMemory::new()
}

/// Create a standard test context
fn test_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["test".to_string(), "compliance".to_string()],
    }
}

/// Create a test execution step
fn create_test_step(step_number: usize) -> ExecutionStep {
    let mut step = ExecutionStep::new(
        step_number,
        format!("test_tool_{}", step_number),
        format!("Test action {}", step_number),
    );
    step.parameters = json!({"param": "value"});
    step.latency_ms = 10 + (step_number as u64 * 5);
    step.tokens_used = Some(50);
    step.result = Some(ExecutionResult::Success {
        output: format!("Step {} completed", step_number),
    });
    step
}

/// Create a completed episode with clear patterns
async fn create_completed_episode_with_clear_pattern(memory: &SelfLearningMemory) -> Uuid {
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "error-handling".to_string(),
        tags: vec!["retry".to_string(), "recovery".to_string()],
        ..Default::default()
    };

    let episode_id = memory
        .start_episode(
            "Implement retry logic".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Create error recovery pattern
    let mut error_step = ExecutionStep::new(
        1,
        "initial_attempt".to_string(),
        "Try operation".to_string(),
    );
    error_step.result = Some(ExecutionResult::Error {
        message: "Connection timeout".to_string(),
    });
    memory.log_step(episode_id, error_step).await;

    let mut retry_step = ExecutionStep::new(
        2,
        "retry_handler".to_string(),
        "Retry with backoff".to_string(),
    );
    retry_step.result = Some(ExecutionResult::Success {
        output: "Operation succeeded".to_string(),
    });
    memory.log_step(episode_id, retry_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Retry logic working".to_string(),
                artifacts: vec!["retry.rs".to_string()],
            },
        )
        .await
        .unwrap();

    episode_id
}

/// Create test episode with specific domain
async fn create_test_episode_with_domain(memory: &SelfLearningMemory, domain: &str) -> Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        ..Default::default()
    };

    let episode_id = memory
        .start_episode(
            format!("Task in {}", domain),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    let step = create_test_step(1);
    memory.log_step(episode_id, step).await;

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

    episode_id
}

// ============================================================================
// FR1: Episode Creation
// ============================================================================

#[tokio::test]
async fn verify_fr1_episode_creation() {
    // FR1: Create episodes with unique IDs and timestamps
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let episode = memory.get_episode(episode_id).await.unwrap();

    // Validate unique ID
    assert_ne!(episode.episode_id, Uuid::nil());
    assert_eq!(episode.episode_id, episode_id);

    // Validate timestamp
    assert!(episode.start_time <= Utc::now());
    assert!(episode.end_time.is_none());

    // Validate initial state
    assert!(!episode.is_complete());
    assert_eq!(episode.steps.len(), 0);
    assert!(episode.outcome.is_none());
}

#[tokio::test]
async fn verify_fr1_unique_episode_ids() {
    let memory = setup_test_memory();

    // Create multiple episodes
    let mut episode_ids = Vec::new();
    for i in 0..10 {
        let id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;
        episode_ids.push(id);
    }

    // Verify all IDs are unique
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
async fn verify_fr2_step_logging() {
    // FR2: Log execution steps with tool usage and outcomes
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let step = ExecutionStep {
        step_number: 1,
        timestamp: Utc::now(),
        tool: "test_tool".to_string(),
        action: "Test action".to_string(),
        parameters: json!({"key": "value"}),
        result: Some(ExecutionResult::Success {
            output: "Success".to_string(),
        }),
        latency_ms: 10,
        tokens_used: Some(50),
        metadata: HashMap::new(),
    };

    memory.log_step(episode_id, step.clone()).await;

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 1);
    assert_eq!(episode.steps[0].tool, "test_tool");
    assert_eq!(episode.steps[0].latency_ms, 10);
    assert_eq!(episode.steps[0].tokens_used, Some(50));
    assert!(episode.steps[0].is_success());
}

#[tokio::test]
async fn verify_fr2_step_ordering() {
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    // Log multiple steps
    for i in 1..=5 {
        let step = create_test_step(i);
        memory.log_step(episode_id, step).await;
    }

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 5);

    // Verify ordering
    for (i, step) in episode.steps.iter().enumerate() {
        assert_eq!(step.step_number, i + 1);
    }
}

#[tokio::test]
async fn verify_fr2_step_metadata() {
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let mut step = create_test_step(1);
    step.latency_ms = 150;
    step.tokens_used = Some(1200);
    step.parameters = json!({"model": "claude-3", "temperature": 0.7});

    memory.log_step(episode_id, step).await;

    let episode = memory.get_episode(episode_id).await.unwrap();
    let recorded_step = &episode.steps[0];

    assert_eq!(recorded_step.latency_ms, 150);
    assert_eq!(recorded_step.tokens_used, Some(1200));
    assert_eq!(recorded_step.parameters["model"], "claude-3");
}

// ============================================================================
// FR3: Episode Completion
// ============================================================================

#[tokio::test]
async fn verify_fr3_episode_completion() {
    // FR3: Complete episodes with reward scoring and reflection
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let outcome = TaskOutcome::Success {
        verdict: "Test passed".to_string(),
        artifacts: vec!["test.rs".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    let completed = memory.get_episode(episode_id).await.unwrap();

    assert!(completed.end_time.is_some());
    assert!(completed.reward.is_some());
    assert!(completed.reflection.is_some());

    let reward = completed.reward.unwrap();
    assert!(reward.total >= 0.0);
    assert_eq!(reward.base, 1.0); // Success

    let reflection = completed.reflection.unwrap();
    assert!(!reflection.successes.is_empty() || !reflection.insights.is_empty());
}

#[tokio::test]
async fn verify_fr3_failure_handling() {
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let outcome = TaskOutcome::Failure {
        reason: "Test failed".to_string(),
        error_details: Some("Assertion error".to_string()),
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    let completed = memory.get_episode(episode_id).await.unwrap();

    let reward = completed.reward.unwrap();
    assert_eq!(reward.base, 0.0); // Failure

    let reflection = completed.reflection.unwrap();
    assert!(!reflection.improvements.is_empty());
}

#[tokio::test]
async fn verify_fr3_partial_success() {
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test task".to_string(), test_context(), TaskType::Testing)
        .await;

    let outcome = TaskOutcome::PartialSuccess {
        verdict: "Some tests passed".to_string(),
        completed: vec!["test1".to_string(), "test2".to_string()],
        failed: vec!["test3".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    let completed = memory.get_episode(episode_id).await.unwrap();

    let reward = completed.reward.unwrap();
    // Partial success reward is calculated based on completion ratio
    assert!(reward.base >= 0.5 && reward.base < 1.0); // Between failure and full success
    assert!(reward.total > 0.0);
}

// ============================================================================
// FR4: Pattern Extraction
// ============================================================================

#[tokio::test]
async fn verify_fr4_pattern_extraction() {
    // FR4: Extract patterns from completed episodes
    let memory = setup_test_memory();

    let episode_id = create_completed_episode_with_clear_pattern(&memory).await;

    let episode = memory.get_episode(episode_id).await.unwrap();

    // Patterns should have been extracted during completion
    assert!(
        !episode.patterns.is_empty(),
        "Expected patterns to be extracted from episode with clear retry pattern"
    );

    // Retrieve patterns from storage
    let context = TaskContext {
        domain: "error-handling".to_string(),
        ..Default::default()
    };
    let patterns = memory.retrieve_relevant_patterns(&context, 10).await;

    assert!(!patterns.is_empty(), "Expected at least one pattern");

    // Check for error recovery pattern
    let has_error_recovery = patterns
        .iter()
        .any(|p| matches!(p, Pattern::ErrorRecovery { .. }));
    assert!(
        has_error_recovery,
        "Expected ErrorRecovery pattern to be extracted"
    );
}

#[tokio::test]
async fn verify_fr4_pattern_types() {
    let memory = setup_test_memory();

    // Create episode with tool sequence
    let episode_id = memory
        .start_episode(
            "Multi-step task".to_string(),
            test_context(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add sequential steps
    for i in 1..=3 {
        let step = create_test_step(i);
        memory.log_step(episode_id, step).await;
    }

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

    let episode = memory.get_episode(episode_id).await.unwrap();

    // Should extract at least one pattern
    assert!(
        !episode.patterns.is_empty(),
        "Expected patterns from multi-step episode"
    );
}

// ============================================================================
// FR5: Episode Retrieval
// ============================================================================

#[tokio::test]
async fn verify_fr5_episode_retrieval() {
    // FR5: Retrieve relevant episodes based on context
    let memory = setup_test_memory();

    // Store multiple episodes with different contexts
    for i in 0..20 {
        let domain = if i % 2 == 0 { "web-api" } else { "cli-tool" };
        create_test_episode_with_domain(&memory, domain).await;
    }

    let web_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("test query".to_string(), web_context, 10)
        .await;

    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    // Should prefer web-api domain episodes
    let web_count = results
        .iter()
        .filter(|e| e.context.domain == "web-api")
        .count();
    assert!(
        web_count as f32 / results.len() as f32 > 0.5,
        "Expected majority of results to match domain filter"
    );
}

#[tokio::test]
async fn verify_fr5_retrieval_limits() {
    let memory = setup_test_memory();

    // Create 50 episodes
    for i in 0..50 {
        create_test_episode_with_domain(&memory, "test-domain").await;
    }

    let context = TaskContext {
        domain: "test-domain".to_string(),
        ..Default::default()
    };

    // Request limit of 5
    let results = memory
        .retrieve_relevant_context("query".to_string(), context.clone(), 5)
        .await;

    assert_eq!(results.len(), 5, "Should respect retrieval limit");

    // Request limit of 20
    let results = memory
        .retrieve_relevant_context("query".to_string(), context, 20)
        .await;

    assert_eq!(results.len(), 20, "Should respect higher limit");
}

#[tokio::test]
async fn verify_fr5_context_based_filtering() {
    let memory = setup_test_memory();

    // Create episodes with different languages
    for lang in ["rust", "python", "typescript"] {
        let context = TaskContext {
            language: Some(lang.to_string()),
            domain: "code-gen".to_string(),
            ..Default::default()
        };

        for _ in 0..5 {
            let episode_id = memory
                .start_episode(
                    "Task".to_string(),
                    context.clone(),
                    TaskType::CodeGeneration,
                )
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
        }
    }

    // Query for rust episodes
    let rust_context = TaskContext {
        language: Some("rust".to_string()),
        domain: "code-gen".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("task".to_string(), rust_context, 10)
        .await;

    // Should prioritize rust episodes
    let rust_count = results
        .iter()
        .filter(|e| e.context.language.as_deref() == Some("rust"))
        .count();

    assert!(rust_count > 0, "Should return rust episodes");
}

// ============================================================================
// FR6 & FR7: MCP Integration (Placeholder)
// ============================================================================
// Note: These tests would require the MCP server implementation.
// Included as placeholders for future implementation.

#[tokio::test]
#[ignore] // Requires MCP server implementation
async fn verify_fr6_code_execution() {
    // FR6: Execute TypeScript code in secure sandbox
    // This test would validate:
    // - Code execution in isolated environment
    // - Security boundaries
    // - Timeout handling
    // - Result capture
}

#[tokio::test]
#[ignore] // Requires MCP server implementation
async fn verify_fr7_tool_generation() {
    // FR7: Generate MCP tools from memory patterns
    // This test would validate:
    // - Tool generation from patterns
    // - Tool metadata correctness
    // - Tool invocation
}

// ============================================================================
// Additional Compliance Tests
// ============================================================================

#[tokio::test]
async fn verify_episode_immutability_after_completion() {
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

    // Attempting to log step to completed episode should not panic
    // (behavior is to ignore or warn, not crash)
    let step = create_test_step(1);
    memory.log_step(episode_id, step).await;

    // Episode should still show as complete
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

#[tokio::test]
async fn verify_statistics_accuracy() {
    let memory = setup_test_memory();

    // Create 5 episodes: 3 complete, 2 incomplete
    for i in 0..5 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
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

    let (total, completed, _patterns) = memory.get_stats().await;
    assert_eq!(total, 5);
    assert_eq!(completed, 3);
}

#[tokio::test]
async fn verify_concurrent_episode_creation() {
    let memory = setup_test_memory();

    // Create 100 episodes concurrently
    let mut handles = vec![];
    for i in 0..100 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.start_episode(
                format!("Concurrent task {}", i),
                test_context(),
                TaskType::Testing,
            )
            .await
        });
        handles.push(handle);
    }

    let mut ids = vec![];
    for handle in handles {
        ids.push(handle.await.unwrap());
    }

    // All IDs should be unique
    assert_eq!(ids.len(), 100);
    for i in 0..ids.len() {
        for j in i + 1..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }

    let (total, _, _) = memory.get_stats().await;
    assert_eq!(total, 100);
}
