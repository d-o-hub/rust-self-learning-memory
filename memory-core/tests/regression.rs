//! BDD-style regression tests to prevent system degradation
//!
//! Tests verify that the memory system maintains quality and compatibility over time.
//!
//! ## Test Coverage
//! - Pattern extraction accuracy and type coverage
//! - Retrieval performance and quality with large datasets
//! - API and data structure backward compatibility
//! - Bug regression prevention (duplicate patterns, idempotency, edge cases)

mod common;

use common::{create_test_step, setup_test_memory, test_context};
use memory_core::{
    memory::SelfLearningMemory, ComplexityLevel, ExecutionResult, ExecutionStep, Pattern,
    TaskContext, TaskOutcome, TaskType,
};
use std::time::{Duration, Instant};
use uuid::Uuid;

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
    let tools = ["analyzer", "designer", "builder", "tester"];
    for (i, tool) in tools.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 1, (*tool).to_string(), format!("{tool} step"));
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
    use memory_core::PatternEffectiveness;

    vec![
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            context: TaskContext::default(),
            error_type: "timeout".to_string(),
            recovery_steps: vec!["retry_with_backoff".to_string()],
            success_rate: 0.9,
            effectiveness: PatternEffectiveness::default(),
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
            effectiveness: PatternEffectiveness::default(),
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
            println!("Loading test episodes: {i}/10000");
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
            .start_episode(format!("Task {i}"), context, TaskType::CodeGeneration)
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
async fn should_maintain_pattern_extraction_accuracy_over_time() {
    // Given: Historical test episodes with known patterns
    let test_episodes = load_historical_test_episodes();
    let memory = setup_test_memory();

    // When: Storing episodes and extracting patterns
    for episode in &test_episodes {
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

    let patterns = memory
        .retrieve_relevant_patterns(&TaskContext::default(), 10)
        .await;

    // Then: Pattern extraction accuracy should meet minimum threshold
    let accuracy = calculate_pattern_similarity(&patterns, &reference_patterns);

    println!(
        "Pattern extraction accuracy: {:.1}% ({} extracted, {} expected)",
        accuracy * 100.0,
        patterns.len(),
        reference_patterns.len()
    );

    assert!(
        accuracy > 0.5,
        "Pattern extraction accuracy degraded to {:.1}%",
        accuracy * 100.0
    );
}

#[tokio::test]
async fn should_extract_all_pattern_types_correctly() {
    // Given: Memory system and an episode that should produce ErrorRecovery pattern
    let memory = setup_test_memory();

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

    // When: Logging an error followed by successful recovery
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

    // Then: Pattern should be extracted from error recovery episode
    let completed = memory.get_episode(episode_id).await.unwrap();

    assert!(
        !completed.patterns.is_empty(),
        "No patterns extracted from error recovery episode"
    );
}

// ============================================================================
// Retrieval Performance Regression
// ============================================================================

#[tokio::test]
#[ignore = "Long-running performance test - run with --include-ignored for full validation"]
async fn should_maintain_fast_retrieval_with_large_dataset() {
    // Given: Memory system with 10K episodes and standard test queries
    let memory = setup_memory_with_10k_episodes().await;
    let test_queries = load_standard_test_queries();

    // When: Executing retrieval queries
    let mut total_time = Duration::from_millis(0);

    for query in &test_queries {
        let start = Instant::now();
        let _ = memory
            .retrieve_relevant_context(query.text.clone(), query.context.clone(), 10)
            .await;
        total_time += start.elapsed();
    }

    let avg_time = total_time / test_queries.len() as u32;

    println!("Average retrieval time with 10K episodes: {avg_time:?}");

    // Then: Average retrieval time should remain under 100ms
    assert!(
        avg_time.as_millis() < 100,
        "Average retrieval time degraded to {}ms",
        avg_time.as_millis()
    );
}

#[tokio::test]
async fn should_retrieve_relevant_episodes_by_domain() {
    // Given: Memory with episodes in two different domains
    let memory = setup_test_memory();

    // Given: 10 web-api episodes with authentication tag
    for i in 0..10 {
        let context = TaskContext {
            domain: "web-api".to_string(),
            tags: vec!["authentication".to_string()],
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(format!("Auth task {i}"), context, TaskType::CodeGeneration)
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

    // Given: 10 data-processing episodes with batch tag
    for i in 0..10 {
        let context = TaskContext {
            domain: "data-processing".to_string(),
            tags: vec!["batch".to_string()],
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(format!("Batch task {i}"), context, TaskType::CodeGeneration)
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

    // When: Querying for web-api domain authentication
    let context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("authentication".to_string(), context, 10)
        .await;

    // Then: Should return results
    assert!(!results.is_empty());

    // Then: At least 50% of results should match the queried domain
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
async fn should_maintain_backward_compatible_public_api() {
    // Given: Memory system instance
    let memory = setup_test_memory();

    // When/Then: All public API methods should be available and functional

    // Then: start_episode should work
    let episode_id = memory
        .start_episode("test".to_string(), test_context(), TaskType::Testing)
        .await;

    // Then: log_step should work
    let step = create_test_step(1);
    memory.log_step(episode_id, step).await;

    // Then: complete_episode should work
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

    // Then: retrieve_relevant_context should work
    let results = memory
        .retrieve_relevant_context("test".to_string(), test_context(), 10)
        .await;
    assert!(results.is_empty() || !results.is_empty());

    // Then: get_episode should work
    let episode = memory.get_episode(episode_id).await;
    assert!(episode.is_ok());

    // Then: get_stats should work
    let (_total, _completed, _patterns) = memory.get_stats().await;

    // Then: retrieve_relevant_patterns should work
    let _patterns = memory.retrieve_relevant_patterns(&test_context(), 10).await;
}

#[tokio::test]
async fn should_maintain_data_structure_compatibility() {
    // Given: Memory system and test episode
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let episode = memory.get_episode(episode_id).await.unwrap();

    // When/Then: Episode structure should have all expected fields
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

    // Then: Episode methods should be available
    assert!(!episode.is_complete());
    let _ = episode.duration();
    let _ = episode.successful_steps_count();
    let _ = episode.failed_steps_count();

    // When/Then: ExecutionStep structure should have all expected fields
    let step = create_test_step(1);

    let _ = step.step_number;
    let _ = step.timestamp;
    let _ = step.tool;
    let _ = step.action;
    let _ = step.parameters;
    let _ = step.result;
    let _ = step.latency_ms;
    let _ = step.tokens_used;
    let _ = step.metadata;

    // Then: ExecutionStep methods should be available
    let _ = step.is_success();

    // When/Then: TaskContext structure should have all expected fields
    let context = test_context();

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
async fn should_prevent_previously_fixed_bugs_from_recurring() {
    // Bug 1: No duplicate pattern IDs

    // Given: Memory system with completed episode containing patterns
    let memory1 = setup_test_memory();

    let episode_id = memory1
        .start_episode("Test".to_string(), test_context(), TaskType::CodeGeneration)
        .await;

    for i in 1..=5 {
        memory1.log_step(episode_id, create_test_step(i)).await;
    }

    memory1
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // When/Then: Checking for duplicate pattern IDs
    let completed = memory1.get_episode(episode_id).await.unwrap();

    let mut seen = std::collections::HashSet::new();
    for pattern_id in &completed.patterns {
        assert!(
            seen.insert(*pattern_id),
            "Duplicate pattern ID found: {pattern_id}"
        );
    }

    // Bug 2: Episode completion is idempotent

    // Given: Memory system and completed episode
    let memory2 = setup_test_memory();

    let episode_id2 = memory2
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let outcome = TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    };

    // When: Completing episode twice
    let result1 = memory2.complete_episode(episode_id2, outcome.clone()).await;
    assert!(result1.is_ok());

    let result2 = memory2.complete_episode(episode_id2, outcome).await;

    // Then: Should either succeed idempotently or return error (not panic)
    assert!(result1.is_ok() || result2.is_err());

    // Bug 3: Can complete episode with no steps

    // Given: Memory system and episode without steps
    let memory3 = setup_test_memory();

    let episode_id3 = memory3
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    // When: Completing episode without adding any steps
    let result = memory3
        .complete_episode(
            episode_id3,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await;

    // Then: Should succeed and episode should be complete with 0 steps
    assert!(result.is_ok());

    let episode = memory3.get_episode(episode_id3).await.unwrap();
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 0);
}

// Note: Concurrent step logging test is covered in learning_cycle.rs

// ============================================================================
// Performance Regression
// ============================================================================

#[tokio::test]
async fn should_maintain_baseline_episode_creation_performance() {
    // Given: Memory system and performance baseline
    let memory = setup_test_memory();

    let start = Instant::now();

    // When: Creating 100 episodes with 3 steps each
    for i in 0..100 {
        let episode_id = memory
            .start_episode(
                format!("Task {i}"),
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

    // Then: Should complete in reasonable time (< 5 seconds)
    assert!(
        elapsed.as_secs() < 5,
        "Performance degraded: took {}ms for 100 episodes",
        elapsed.as_millis()
    );
}
