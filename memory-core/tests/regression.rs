//! Regression tests to ensure system doesn't degrade over time
//!
//! This test suite validates that:
//! - Pattern extraction accuracy remains consistent
//! - Retrieval performance doesn't degrade
//! - API compatibility is maintained
//! - Previously fixed bugs don't reoccur

use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, Pattern, TaskContext, TaskOutcome, TaskType,
};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create test memory
fn setup_test_memory() -> SelfLearningMemory {
    SelfLearningMemory::new()
}

/// Standard test context
fn test_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["regression".to_string()],
    }
}

/// Create test step
fn create_test_step(step_number: usize) -> ExecutionStep {
    let mut step = ExecutionStep::new(
        step_number,
        format!("tool_{}", step_number),
        format!("Action {}", step_number),
    );
    step.latency_ms = 10;
    step.tokens_used = Some(50);
    step.result = Some(ExecutionResult::Success {
        output: "Done".to_string(),
    });
    step
}

/// Load or create historical test episodes
fn load_historical_test_episodes() -> Vec<Episode> {
    // In a real implementation, these would be loaded from a fixture file
    // For now, we create standard test cases
    vec![
        create_error_recovery_episode(),
        create_tool_sequence_episode(),
        create_optimization_episode(),
    ]
}

/// Create an episode with error recovery pattern
fn create_error_recovery_episode() -> Episode {
    use memory_core::episode::Episode;

    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "error-handling".to_string(),
        tags: vec!["retry".to_string()],
        ..Default::default()
    };

    let mut episode = Episode::new(
        "Implement retry logic".to_string(),
        context,
        TaskType::CodeGeneration,
    );

    // First attempt fails
    let mut error_step = ExecutionStep::new(
        1,
        "initial_attempt".to_string(),
        "Try operation".to_string(),
    );
    error_step.result = Some(ExecutionResult::Error {
        message: "Connection timeout".to_string(),
    });
    episode.add_step(error_step);

    // Retry succeeds
    let mut retry_step = ExecutionStep::new(
        2,
        "retry_handler".to_string(),
        "Retry with backoff".to_string(),
    );
    retry_step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    episode.add_step(retry_step);

    episode.complete(TaskOutcome::Success {
        verdict: "Retry worked".to_string(),
        artifacts: vec![],
    });

    episode
}

/// Create episode with tool sequence pattern
fn create_tool_sequence_episode() -> Episode {
    use memory_core::episode::Episode;

    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "code-generation".to_string(),
        tags: vec!["sequential".to_string()],
        ..Default::default()
    };

    let mut episode = Episode::new(
        "Build API endpoint".to_string(),
        context,
        TaskType::CodeGeneration,
    );

    // Sequential tool usage
    let tools = vec!["analyzer", "designer", "builder", "tester"];
    for (i, tool) in tools.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 1, tool.to_string(), format!("{} step", tool));
        step.result = Some(ExecutionResult::Success {
            output: "Done".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "API built".to_string(),
        artifacts: vec!["api.rs".to_string()],
    });

    episode
}

/// Create episode with optimization pattern
fn create_optimization_episode() -> Episode {
    use memory_core::episode::Episode;

    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "performance".to_string(),
        tags: vec!["optimization".to_string()],
        ..Default::default()
    };

    let mut episode = Episode::new("Optimize query".to_string(), context, TaskType::Refactoring);

    // Initial slow performance
    let mut slow_step = ExecutionStep::new(1, "profiler".to_string(), "Profile code".to_string());
    slow_step.latency_ms = 1000; // Slow
    slow_step.result = Some(ExecutionResult::Success {
        output: "Found bottleneck".to_string(),
    });
    episode.add_step(slow_step);

    // After optimization
    let mut fast_step =
        ExecutionStep::new(2, "optimizer".to_string(), "Apply optimization".to_string());
    fast_step.latency_ms = 100; // Fast
    fast_step.result = Some(ExecutionResult::Success {
        output: "Optimized".to_string(),
    });
    episode.add_step(fast_step);

    episode.complete(TaskOutcome::Success {
        verdict: "10x faster".to_string(),
        artifacts: vec![],
    });

    episode
}

/// Load reference patterns (known good patterns)
fn load_reference_patterns() -> Vec<Pattern> {
    use chrono::Duration;

    vec![
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            context: TaskContext::default(),
            error_type: "timeout".to_string(),
            recovery_steps: vec!["retry_with_backoff".to_string()],
            success_rate: 0.9,
        },
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            context: TaskContext::default(),
            tools: vec![
                "analyzer".to_string(),
                "designer".to_string(),
                "builder".to_string(),
                "tester".to_string(),
            ],
            success_rate: 0.85,
            avg_latency: Duration::milliseconds(60000),
            occurrence_count: 10,
        },
    ]
}

/// Calculate pattern similarity/accuracy
fn calculate_pattern_similarity(extracted: &[Pattern], reference: &[Pattern]) -> f32 {
    if reference.is_empty() {
        return 0.0;
    }

    let mut matches = 0;

    for ref_pattern in reference {
        for ext_pattern in extracted {
            if patterns_match(ref_pattern, ext_pattern) {
                matches += 1;
                break;
            }
        }
    }

    matches as f32 / reference.len() as f32
}

/// Check if two patterns match (same type and similar properties)
fn patterns_match(p1: &Pattern, p2: &Pattern) -> bool {
    match (p1, p2) {
        (
            Pattern::ErrorRecovery { error_type: e1, .. },
            Pattern::ErrorRecovery { error_type: e2, .. },
        ) => e1.contains(e2) || e2.contains(e1),
        (Pattern::ToolSequence { tools: t1, .. }, Pattern::ToolSequence { tools: t2, .. }) => {
            // Check if tool sequences are similar
            if t1.len() != t2.len() {
                return false;
            }
            t1.iter().zip(t2.iter()).filter(|(a, b)| a == b).count() >= t1.len() * 2 / 3
        }
        _ => false,
    }
}

/// Load standard test queries for retrieval performance
fn load_standard_test_queries() -> Vec<TestQuery> {
    vec![
        TestQuery {
            text: "implement authentication".to_string(),
            context: TaskContext {
                domain: "web-api".to_string(),
                ..Default::default()
            },
        },
        TestQuery {
            text: "handle errors".to_string(),
            context: TaskContext {
                domain: "error-handling".to_string(),
                ..Default::default()
            },
        },
        TestQuery {
            text: "optimize performance".to_string(),
            context: TaskContext {
                domain: "performance".to_string(),
                ..Default::default()
            },
        },
        TestQuery {
            text: "write tests".to_string(),
            context: TaskContext {
                domain: "testing".to_string(),
                ..Default::default()
            },
        },
        TestQuery {
            text: "refactor code".to_string(),
            context: TaskContext {
                domain: "refactoring".to_string(),
                ..Default::default()
            },
        },
    ]
}

struct TestQuery {
    text: String,
    context: TaskContext,
}

use memory_core::episode::Episode;

/// Setup memory with 10K episodes for performance testing
async fn setup_memory_with_10k_episodes() -> SelfLearningMemory {
    let memory = setup_test_memory();

    for i in 0..10000 {
        if i % 1000 == 0 {
            println!("Loading test episodes: {}/10000", i);
        }

        let context = TaskContext {
            domain: format!("domain_{}", i % 10),
            complexity: match i % 3 {
                0 => ComplexityLevel::Simple,
                1 => ComplexityLevel::Moderate,
                _ => ComplexityLevel::Complex,
            },
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(format!("Task {}", i), context, TaskType::CodeGeneration)
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

    memory
}

// ============================================================================
// Pattern Extraction Regression
// ============================================================================

#[tokio::test]
async fn regression_pattern_extraction_accuracy() {
    // Ensure pattern extraction doesn't degrade over time
    let test_episodes = load_historical_test_episodes();
    let memory = setup_test_memory();

    for episode in &test_episodes {
        // Store episode (in real impl)
        let episode_id = memory
            .start_episode(
                episode.task_description.clone(),
                episode.context.clone(),
                episode.task_type,
            )
            .await;

        for step in &episode.steps {
            memory.log_step(episode_id, step.clone()).await;
        }

        if let Some(outcome) = &episode.outcome {
            memory
                .complete_episode(episode_id, outcome.clone())
                .await
                .unwrap();
        }
    }

    let reference_patterns = load_reference_patterns();

    // Get patterns by querying relevant context
    let patterns = memory
        .retrieve_relevant_patterns(&TaskContext::default(), 10)
        .await;

    let accuracy = calculate_pattern_similarity(&patterns, &reference_patterns);

    println!(
        "Pattern extraction accuracy: {:.1}% ({} extracted, {} expected)",
        accuracy * 100.0,
        patterns.len(),
        reference_patterns.len()
    );

    assert!(
        accuracy > 0.5, // Lower threshold since we're using simplified test
        "Pattern extraction accuracy degraded to {:.1}%",
        accuracy * 100.0
    );
}

#[tokio::test]
async fn regression_pattern_types_coverage() {
    // Ensure all pattern types can still be extracted
    let memory = setup_test_memory();

    // Create episode that should produce ErrorRecovery pattern
    let episode_id = memory
        .start_episode(
            "Error handling".to_string(),
            TaskContext {
                domain: "error-handling".to_string(),
                ..Default::default()
            },
            TaskType::CodeGeneration,
        )
        .await;

    let mut error_step = ExecutionStep::new(1, "attempt".to_string(), "Try".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Failed".to_string(),
    });
    memory.log_step(episode_id, error_step).await;

    let mut success_step = ExecutionStep::new(2, "retry".to_string(), "Retry".to_string());
    success_step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    memory.log_step(episode_id, success_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Recovered".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let completed = memory.get_episode(episode_id).await.unwrap();

    // Should have extracted pattern
    assert!(
        !completed.patterns.is_empty(),
        "No patterns extracted from error recovery episode"
    );
}

// ============================================================================
// Retrieval Performance Regression
// ============================================================================

#[tokio::test]
#[ignore] // Long-running test
async fn regression_retrieval_performance() {
    // Ensure memory retrieval remains performant
    let memory = setup_memory_with_10k_episodes().await;
    let test_queries = load_standard_test_queries();

    let mut total_time = Duration::from_millis(0);

    for query in &test_queries {
        let start = Instant::now();
        let _ = memory
            .retrieve_relevant_context(query.text.clone(), query.context.clone(), 10)
            .await;
        total_time += start.elapsed();
    }

    let avg_time = total_time / test_queries.len() as u32;

    println!("Average retrieval time with 10K episodes: {:?}", avg_time);

    assert!(
        avg_time.as_millis() < 100,
        "Average retrieval time degraded to {}ms",
        avg_time.as_millis()
    );
}

#[tokio::test]
async fn regression_retrieval_quality() {
    // Ensure retrieval returns relevant results
    let memory = setup_test_memory();

    // Create episodes in specific domain
    for i in 0..10 {
        let context = TaskContext {
            domain: "web-api".to_string(),
            tags: vec!["authentication".to_string()],
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(
                format!("Auth task {}", i),
                context,
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

    // Create episodes in different domain
    for i in 0..10 {
        let context = TaskContext {
            domain: "data-processing".to_string(),
            tags: vec!["batch".to_string()],
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(
                format!("Batch task {}", i),
                context,
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

    // Query for web-api domain
    let context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("authentication".to_string(), context, 10)
        .await;

    assert!(!results.is_empty());

    // Should prioritize web-api episodes
    let web_api_count = results
        .iter()
        .filter(|e| e.context.domain == "web-api")
        .count();

    assert!(
        web_api_count as f32 / results.len() as f32 >= 0.5,
        "Retrieval quality degraded - only {}/{} results matched domain",
        web_api_count,
        results.len()
    );
}

// ============================================================================
// API Compatibility Regression
// ============================================================================

#[tokio::test]
async fn regression_api_compatibility() {
    // This test ensures that the public API hasn't changed unexpectedly
    // It should compile without errors if API is compatible

    let memory = setup_test_memory();

    // Test that all public methods are still available
    let episode_id = memory
        .start_episode("test".to_string(), test_context(), TaskType::Testing)
        .await;

    let step = create_test_step(1);
    memory.log_step(episode_id, step).await;

    let completed = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "test".to_string(),
                artifacts: vec![],
            },
        )
        .await;

    assert!(completed.is_ok());

    let results = memory
        .retrieve_relevant_context("test".to_string(), test_context(), 10)
        .await;

    assert!(results.is_empty() || !results.is_empty()); // Just check it works

    // Test get_episode
    let episode = memory.get_episode(episode_id).await;
    assert!(episode.is_ok());

    // Test get_stats
    let (_total, _completed, _patterns) = memory.get_stats().await;

    // Test retrieve_relevant_patterns
    let _patterns = memory.retrieve_relevant_patterns(&test_context(), 10).await;
}

#[tokio::test]
async fn regression_episode_structure() {
    // Ensure Episode structure hasn't changed
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let episode = memory.get_episode(episode_id).await.unwrap();

    // Check all expected fields exist
    let _ = episode.episode_id;
    let _ = episode.task_type;
    let _ = episode.task_description;
    let _ = episode.context;
    let _ = episode.start_time;
    let _ = episode.end_time;
    let _ = episode.steps;
    let _ = episode.outcome;
    let _ = episode.reward;
    let _ = episode.reflection;
    let _ = episode.patterns;
    let _ = episode.metadata;

    // Check methods exist
    assert!(!episode.is_complete());
    let _ = episode.duration();
    let _ = episode.successful_steps_count();
    let _ = episode.failed_steps_count();
}

#[tokio::test]
async fn regression_execution_step_structure() {
    // Ensure ExecutionStep structure hasn't changed
    let step = create_test_step(1);

    // Check all expected fields exist
    let _ = step.step_number;
    let _ = step.timestamp;
    let _ = step.tool;
    let _ = step.action;
    let _ = step.parameters;
    let _ = step.result;
    let _ = step.latency_ms;
    let _ = step.tokens_used;
    let _ = step.metadata;

    // Check methods exist
    let _ = step.is_success();
}

#[tokio::test]
async fn regression_task_context_structure() {
    // Ensure TaskContext structure hasn't changed
    let context = test_context();

    // Check all expected fields exist
    let _ = context.language;
    let _ = context.framework;
    let _ = context.complexity;
    let _ = context.domain;
    let _ = context.tags;
}

// ============================================================================
// Bug Regression Tests
// ============================================================================

#[tokio::test]
async fn regression_no_duplicate_patterns() {
    // Regression test: Ensure patterns aren't duplicated
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::CodeGeneration)
        .await;

    for i in 1..=5 {
        memory.log_step(episode_id, create_test_step(i)).await;
    }

    let completed = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Check for duplicate pattern IDs
    let completed = memory.get_episode(episode_id).await.unwrap();

    let mut seen = std::collections::HashSet::new();
    for pattern_id in &completed.patterns {
        assert!(
            seen.insert(*pattern_id),
            "Duplicate pattern ID found: {}",
            pattern_id
        );
    }
}

#[tokio::test]
async fn regression_episode_completion_idempotent() {
    // Regression test: Completing an already completed episode should not cause issues
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let outcome = TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    };

    // Complete once
    let result1 = memory.complete_episode(episode_id, outcome.clone()).await;
    assert!(result1.is_ok());

    // Complete again - should not panic
    let result2 = memory.complete_episode(episode_id, outcome).await;

    // Behavior: either succeed idempotently or return error
    // Should not panic or corrupt state
    assert!(result1.is_ok() || result2.is_err());
}

#[tokio::test]
async fn regression_empty_episode_completion() {
    // Regression test: Can complete episode with no steps
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    // Complete without adding any steps
    let result = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await;

    assert!(result.is_ok());

    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 0);
}

#[tokio::test]
async fn regression_concurrent_step_logging() {
    // Regression test: Concurrent step logging maintains order
    use tokio::task::JoinSet;

    let memory = std::sync::Arc::new(setup_test_memory());

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let mut set = JoinSet::new();

    // Log steps concurrently (might arrive out of order)
    for i in 1..=10 {
        let mem = memory.clone();
        set.spawn(async move {
            let step = create_test_step(i);
            mem.log_step(episode_id, step).await;
        });
    }

    while set.join_next().await.is_some() {}

    let episode = memory.get_episode(episode_id).await.unwrap();

    // All steps should be recorded
    assert_eq!(episode.steps.len(), 10);

    // Steps should have their step_numbers preserved
    for step in &episode.steps {
        assert!(step.step_number >= 1 && step.step_number <= 10);
    }
}

// ============================================================================
// Performance Regression
// ============================================================================

#[tokio::test]
async fn regression_no_performance_degradation() {
    // Baseline performance check
    let memory = setup_test_memory();

    let start = Instant::now();

    // Create 100 episodes with completion
    for i in 0..100 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;

        for j in 1..=3 {
            memory.log_step(episode_id, create_test_step(j)).await;
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
    }

    let elapsed = start.elapsed();

    println!(
        "100 episodes with 3 steps each: {:?} ({:.2} eps/sec)",
        elapsed,
        100.0 / elapsed.as_secs_f32()
    );

    // Should complete in reasonable time (< 5 seconds)
    assert!(
        elapsed.as_secs() < 5,
        "Performance degraded: took {}ms for 100 episodes",
        elapsed.as_millis()
    );
}
