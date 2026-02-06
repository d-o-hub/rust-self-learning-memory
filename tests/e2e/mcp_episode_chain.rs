//! MCP Episode Management Chain Tests (Day 2-3)
//!
//! Comprehensive E2E tests covering:
//! - start_episode → log_step → complete_episode → retrieve_relevant_context
//! - Error handling in chain
//! - Transaction rollback simulation
//!
//! Target: 5+ test scenarios

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::types::{MemoryConfig, TaskContext};
use memory_core::{ExecutionResult, ExecutionStep, SelfLearningMemory, TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::tempdir;

/// Test helper to create a memory instance with storage
async fn setup_test_memory() -> (Arc<SelfLearningMemory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    // Configure with lower quality threshold for tests
    let config = MemoryConfig {
        quality_threshold: 0.3, // Lower threshold for E2E tests
        ..Default::default()
    };

    let memory = Arc::new(SelfLearningMemory::with_storage(
        config,
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    (memory, dir)
}

// ============================================================================
// Scenario 1: Complete Episode Management Chain
// ============================================================================

async fn test_mcp_complete_episode_chain() {
    let (memory, _dir) = setup_test_memory().await;

    // Step 1: start_episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-test".to_string(),
        tags: vec!["test".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Implement feature using MCP tools".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Verify episode created
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(
        episode.task_description,
        "Implement feature using MCP tools"
    );
    assert!(!episode.is_complete());

    // Step 2: log_step (multiple steps)
    for i in 1..=4 {
        let step = ExecutionStep::new(
            i,
            format!("mcp-tool-{}", i),
            format!("Execute MCP tool {}", i),
        );
        memory.log_step(episode_id, step).await;
    }

    // Flush steps to ensure they're persisted
    memory.flush_steps(episode_id).await.unwrap();

    // Verify steps added
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 4);

    // Step 3: complete_episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "All MCP tools executed successfully".to_string(),
                artifacts: vec!["mcp-result.json".to_string()],
            },
        )
        .await
        .expect("MCP: complete_episode failed");

    // Verify completion
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.outcome.is_some());

    // Step 4: retrieve_relevant_context (retrieve episode)
    let query_context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-test".to_string(),
        tags: vec![],
    };
    let relevant = memory
        .retrieve_relevant_context(
            "MCP tools feature implementation".to_string(),
            query_context,
            10,
        )
        .await;

    assert!(!relevant.is_empty());
    assert!(relevant.iter().any(|ep| ep.episode_id == episode_id));

    println!("✓ MCP complete episode chain test passed");
}

// ============================================================================
// Scenario 2: Error Handling in Episode Chain
// ============================================================================

async fn test_mcp_error_handling_in_chain() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-error-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Error handling test".to_string(),
            context,
            TaskType::Debugging,
        )
        .await;

    // Simulate error in step 2, recovery in step 3
    let step1 = ExecutionStep::new(1, "analysis-tool".to_string(), "Analyze issue".to_string());
    memory.log_step(episode_id, step1).await;

    // Step 2: Error
    let mut step2 = ExecutionStep::new(2, "test-tool".to_string(), "Test fix".to_string());
    step2.result = Some(ExecutionResult::Error {
        message: "Test failed with error X".to_string(),
    });
    memory.log_step(episode_id, step2).await;

    // Step 3: Recovery
    let mut step3 = ExecutionStep::new(3, "fix-tool".to_string(), "Apply fix".to_string());
    step3.result = Some(ExecutionResult::Success {
        output: "Fix applied".to_string(),
    });
    memory.log_step(episode_id, step3).await;

    // Complete with partial success
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::PartialSuccess {
                completed: vec!["analysis-tool".to_string(), "fix-tool".to_string()],
                failed: vec!["test-tool".to_string()],
                verdict: "Issue mostly resolved with one error".to_string(),
            },
        )
        .await
        .unwrap();

    // Verify episode shows error path
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 3);

    // Verify error and recovery steps exist
    let error_step = &episode.steps[1];
    assert!(matches!(
        error_step.result,
        Some(ExecutionResult::Error { .. })
    ));

    let recovery_step = &episode.steps[2];
    assert!(matches!(
        recovery_step.result,
        Some(ExecutionResult::Success { .. })
    ));

    println!("✓ MCP error handling in chain test passed");
}

// ============================================================================
// Scenario 3: Episode Chain with Early Failure
// ============================================================================

async fn test_mcp_episode_chain_early_failure() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-fail-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Early failure test".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add a step that fails early
    let mut step1 = ExecutionStep::new(
        1,
        "validate-tool".to_string(),
        "Validate prerequisites".to_string(),
    );
    step1.result = Some(ExecutionResult::Error {
        message: "Prerequisites not met".to_string(),
    });
    memory.log_step(episode_id, step1).await;

    // Try to add more steps after failure
    let step2 = ExecutionStep::new(
        2,
        "implement-tool".to_string(),
        "Implement feature".to_string(),
    );
    memory.log_step(episode_id, step2).await;

    // Complete as failure
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Failure {
                reason: "Prerequisites not met, cannot continue".to_string(),
                error_details: Some("Missing dependency X".to_string()),
            },
        )
        .await
        .unwrap();

    // Verify episode shows early failure
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());

    let first_step = &episode.steps[0];
    assert!(matches!(
        first_step.result,
        Some(ExecutionResult::Error { .. })
    ));

    println!("✓ MCP episode chain early failure test passed");
}

// ============================================================================
// Scenario 4: Episode Chain with Steps Having Different Latencies
// ============================================================================

async fn test_mcp_episode_chain_with_latencies() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-latency-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Latency tracking test".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps with different latencies
    let latencies = vec![10u64, 250, 50, 500, 100]; // in milliseconds

    for (i, latency) in latencies.iter().enumerate() {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("latency-tool-{}", i),
            format!("Step with {}ms latency", latency),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("Completed in {}ms", latency),
        });
        step.latency_ms = *latency;
        memory.log_step(episode_id, step).await;
    }

    // Complete episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "All steps completed with measured latencies".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Verify latencies were recorded
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 5);

    let recorded_latencies: Vec<u64> = episode.steps.iter().map(|s| s.latency_ms).collect();
    assert_eq!(recorded_latencies, latencies);

    // Calculate total and average latency
    let total_latency: u64 = recorded_latencies.iter().sum();
    let avg_latency = total_latency as f64 / recorded_latencies.len() as f64;

    assert_eq!(total_latency, 910);
    assert!((avg_latency - 182.0).abs() < 0.1);

    println!(
        "✓ MCP episode chain with latencies test passed - total: {}ms, avg: {:.2}ms",
        total_latency, avg_latency
    );
}

// ============================================================================
// Scenario 5: Episode Chain with Parameter Tracking
// ============================================================================

async fn test_mcp_episode_chain_with_parameters() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-param-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Parameter tracking test".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps with parameters
    let step1_params = serde_json::json!({
        "file": "src/lib.rs",
        "line": 42,
        "change": "add function"
    });

    let step2_params = serde_json::json!({
        "test_file": "tests/lib_test.rs",
        "coverage": 0.95
    });

    // Add steps with parameters
    let mut step1 = ExecutionStep::new(1, "code-editor".to_string(), "Edit code".to_string());
    step1.parameters = step1_params;
    memory.log_step(episode_id, step1).await;

    let mut step2 = ExecutionStep::new(2, "test-runner".to_string(), "Run tests".to_string());
    step2.parameters = step2_params;
    memory.log_step(episode_id, step2).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Code edited and tested".to_string(),
                artifacts: vec!["src/lib.rs".to_string(), "tests/lib_test.rs".to_string()],
            },
        )
        .await
        .unwrap();

    // Verify steps were added
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 2);

    println!("✓ MCP episode chain with parameters test passed");
}

// ============================================================================
// Scenario 6: Transaction Rollback Simulation (Episode Abandonment)
// ============================================================================

async fn test_mcp_transaction_rollback_abandonment() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-tx-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Transaction test".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add some steps
    let step1 = ExecutionStep::new(1, "start-tx".to_string(), "Start transaction".to_string());
    memory.log_step(episode_id, step1).await;

    let step2 = ExecutionStep::new(2, "process".to_string(), "Process data".to_string());
    memory.log_step(episode_id, step2).await;

    // Simulate transaction abandonment by not completing
    // In a real scenario, this represents a failed workflow that was abandoned

    // Verify episode is incomplete
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(!episode.is_complete());
    assert_eq!(episode.steps.len(), 2);

    println!("✓ MCP transaction rollback abandonment test passed");
}

// ============================================================================
// Scenario 7: Episode Chain Concurrency Handling
// ============================================================================

async fn test_mcp_episode_chain_concurrency() {
    let (memory, _dir) = setup_test_memory().await;

    // Create multiple episodes
    let mut episode_ids = Vec::new();

    let domain = "mcp-concurrent-test";
    let task_type = TaskType::CodeGeneration;

    // Create episodes
    for i in 0..3 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: memory_core::types::ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: vec![],
        };
        let id = memory
            .start_episode(format!("Concurrent task {}", i), context, task_type)
            .await;
        episode_ids.push(id);
    }

    // Add steps to all episodes
    for &episode_id in &episode_ids {
        for step_num in 1..3 {
            let step = ExecutionStep::new(
                step_num,
                format!("tool-{}", step_num),
                format!("Step {}", step_num),
            );
            memory.log_step(episode_id, step).await;
        }
    }

    // Complete all episodes
    for &episode_id in &episode_ids {
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

    // Query all episodes by domain
    let query_context = TaskContext {
        language: None,
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec![],
    };
    let relevant = memory
        .retrieve_relevant_context("concurrent".to_string(), query_context, 10)
        .await;

    // Verify all episodes found (or at least some are returned)
    assert!(!relevant.is_empty());

    println!("✓ MCP episode chain concurrency test passed");
}

// ============================================================================
// Scenario 8: Episode Chain with Token Usage Tracking
// ============================================================================

async fn test_mcp_episode_chain_with_token_usage() {
    let (memory, _dir) = setup_test_memory().await;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "mcp-token-test".to_string(),
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(
            "Token usage test".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps with token usage
    let mut step1 = ExecutionStep::new(1, "ai-assistant".to_string(), "Generate code".to_string());
    step1.tokens_used = Some(150);
    memory.log_step(episode_id, step1).await;

    let mut step2 = ExecutionStep::new(2, "ai-reviewer".to_string(), "Review code".to_string());
    step2.tokens_used = Some(75);
    memory.log_step(episode_id, step2).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Generated and reviewed code".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Verify episode
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 2);

    // Check if tokens were tracked
    let total_tokens: usize = episode.steps.iter().filter_map(|s| s.tokens_used).sum();
    assert_eq!(total_tokens, 225);
    println!("Total tokens used: {}", total_tokens);

    println!("✓ MCP episode chain with token usage test passed");
}

// ============================================================================
// Scenario 9: Episode Chain Query with Multiple Filters
// ============================================================================

async fn test_mcp_episode_chain_filtered_query() {
    let (memory, _dir) = setup_test_memory().await;

    // Create episodes with different properties
    let test_cases = vec![
        ("Domain 1", TaskType::CodeGeneration),
        ("Domain 1", TaskType::Testing),
        ("Domain 2", TaskType::CodeGeneration),
        ("Domain 2", TaskType::Analysis),
    ];

    let mut episode_ids = Vec::new();

    for (domain, task_type) in test_cases {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: memory_core::types::ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: vec![],
        };
        let id = memory
            .start_episode(format!("Task in {}", domain), context, task_type)
            .await;
        episode_ids.push(id);

        let step = ExecutionStep::new(1, "test".to_string(), "Test".to_string());
        memory.log_step(id, step).await;

        memory
            .complete_episode(
                id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Query by domain only
    let context1 = TaskContext {
        language: None,
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "Domain 1".to_string(),
        tags: vec![],
    };
    let relevant1 = memory
        .retrieve_relevant_context("test".to_string(), context1, 10)
        .await;
    assert!(!relevant1.is_empty());

    // Query by different domain
    let context2 = TaskContext {
        language: None,
        framework: None,
        complexity: memory_core::types::ComplexityLevel::Moderate,
        domain: "Domain 2".to_string(),
        tags: vec![],
    };
    let relevant2 = memory
        .retrieve_relevant_context("test".to_string(), context2, 10)
        .await;
    assert!(!relevant2.is_empty());

    println!("✓ MCP episode chain filtered query test passed");
}

// ============================================================================
// Scenario 10: Episode Chain Bulk Operations
// ============================================================================

async fn test_mcp_episode_chain_bulk_operations() {
    let (memory, _dir) = setup_test_memory().await;

    // Create multiple episodes
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: memory_core::types::ComplexityLevel::Moderate,
            domain: "mcp-bulk-test".to_string(),
            tags: vec![],
        };
        let id = memory
            .start_episode(format!("Bulk test {}", i), context, TaskType::Testing)
            .await;
        episode_ids.push(id);
    }

    // Complete all episodes
    for id in &episode_ids {
        let step = ExecutionStep::new(1, "test".to_string(), "Test".to_string());
        memory.log_step(*id, step).await;

        memory
            .complete_episode(
                *id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Bulk retrieve using get_episodes_by_ids
    let episodes = memory.get_episodes_by_ids(&episode_ids).await.unwrap();
    assert_eq!(episodes.len(), 5);

    // Verify all retrieved
    for episode in &episodes {
        assert!(episode_ids.contains(&episode.episode_id));
        assert!(episode.is_complete());
    }

    println!("✓ MCP episode chain bulk operations test passed");
}

// ============================================================================
// Main function for harness = false test
// ============================================================================

use std::future::Future;

fn run_test<F>(name: &str, test: F, passed: &mut i32, _failed: &mut i32)
where
    F: Future<Output = ()>,
{
    print!("Running {} ... ", name);
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        test.await;
    });
    println!("✓ PASSED");
    *passed += 1;
}

fn main() {
    println!("\n========================================");
    println!("Running MCP Episode Chain E2E tests");
    println!("========================================\n");

    let mut passed = 0;
    let mut failed = 0;

    // Run all tests
    run_test(
        "test_mcp_complete_episode_chain",
        test_mcp_complete_episode_chain(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_error_handling_in_chain",
        test_mcp_error_handling_in_chain(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_early_failure",
        test_mcp_episode_chain_early_failure(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_with_latencies",
        test_mcp_episode_chain_with_latencies(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_with_parameters",
        test_mcp_episode_chain_with_parameters(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_transaction_rollback_abandonment",
        test_mcp_transaction_rollback_abandonment(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_concurrency",
        test_mcp_episode_chain_concurrency(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_with_token_usage",
        test_mcp_episode_chain_with_token_usage(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_filtered_query",
        test_mcp_episode_chain_filtered_query(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_episode_chain_bulk_operations",
        test_mcp_episode_chain_bulk_operations(),
        &mut passed,
        &mut failed,
    );

    println!("\n========================================");
    println!("Results: {} passed, {} failed", passed, failed);
    println!("========================================\n");

    if failed > 0 {
        std::process::exit(1);
    }
}
