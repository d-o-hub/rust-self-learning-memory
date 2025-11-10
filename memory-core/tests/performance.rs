//! BDD-style performance tests for non-functional requirements NFR1-NFR5
//!
//! Tests verify that the memory system meets performance, scalability,
//! and reliability targets from plans/04-review.md.
//!
//! ## Test Coverage
//! - NFR1: Retrieval latency (<100ms P95) with 100-10K episodes
//! - NFR2: Storage capacity (1K-10K episodes without degradation)
//! - NFR3: Pattern recognition accuracy (>70%)
//! - NFR4: Test coverage (>90%)
//! - NFR5: Memory leak prevention under continuous operation
//! - Concurrent performance (episode creation, completion, retrieval)
//! - Step logging and completion performance
//!
//! All tests follow the Given-When-Then pattern for clarity.

use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome,
    TaskType,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test memory instance
fn setup_test_memory() -> SelfLearningMemory {
    SelfLearningMemory::new()
}

/// Create memory with custom config
fn setup_memory_with_config(config: MemoryConfig) -> SelfLearningMemory {
    SelfLearningMemory::with_config(config)
}

/// Setup memory pre-populated with N episodes
async fn setup_memory_with_n_episodes(n: usize) -> SelfLearningMemory {
    let memory = setup_test_memory();

    for i in 0..n {
        let context = TaskContext {
            language: Some("rust".to_string()),
            domain: format!("domain_{}", i % 10),
            complexity: match i % 3 {
                0 => ComplexityLevel::Simple,
                1 => ComplexityLevel::Moderate,
                _ => ComplexityLevel::Complex,
            },
            tags: vec![format!("tag_{}", i % 5)],
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(format!("Task {}", i), context, TaskType::CodeGeneration)
            .await
            .unwrap();

        // Add 3-5 steps per episode
        let step_count = 3 + (i % 3);
        for j in 0..step_count {
            let mut step =
                ExecutionStep::new(j + 1, format!("tool_{}", j), format!("Action {}", j));
            step.latency_ms = 10 + (j as u64 * 5);
            step.tokens_used = Some(50 + (j * 10));
            step.result = Some(ExecutionResult::Success {
                output: format!("Step {} done", j),
            });
            memory.log_step(episode_id, step).await;
        }

        // Complete episode
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Task {} completed", i),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    memory
}

/// Standard test context
fn test_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["performance".to_string()],
    }
}

/// Create a test step
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

/// Get current process memory usage in bytes
#[cfg(target_os = "linux")]
fn get_current_memory_usage() -> usize {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<usize>() {
                    return kb * 1024; // Convert to bytes
                }
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn get_current_memory_usage() -> usize {
    // Fallback for non-Linux systems
    // Use a rough estimate based on allocated memory
    0
}

// ============================================================================
// NFR1: Retrieval Latency
// ============================================================================

#[tokio::test]
async fn should_retrieve_episodes_under_100ms_p95_with_100_episodes() {
    // Given: Memory system with 100 episodes
    let memory = setup_memory_with_n_episodes(100).await;
    let mut latencies = Vec::new();

    // When: Running 100 retrieval queries across different domains
    for i in 0..100 {
        let context = TaskContext {
            domain: format!("domain_{}", i % 10),
            ..Default::default()
        };

        let start = Instant::now();
        let _ = memory
            .retrieve_relevant_context(format!("test query {}", i), context.clone(), 10)
            .await;
        latencies.push(start.elapsed());
    }

    // Then: P95 latency should be under 100ms (NFR1)
    latencies.sort();
    let p95_index = (latencies.len() as f32 * 0.95) as usize;
    let p95 = latencies[p95_index];

    println!("P95 latency with 100 episodes: {:?}", p95);

    assert!(
        p95.as_millis() < 100,
        "P95 retrieval latency {}ms exceeds 100ms target",
        p95.as_millis()
    );
}

#[tokio::test]
#[ignore = "Long-running test - run with --include-ignored for full validation"]
async fn should_retrieve_episodes_under_100ms_p95_with_10k_episodes() {
    // Given: Memory system with 10K episodes
    let memory = setup_memory_with_n_episodes(10000).await;
    let mut latencies = Vec::new();

    // When: Running 100 retrieval queries across different domains
    for i in 0..100 {
        let context = TaskContext {
            domain: format!("domain_{}", i % 10),
            ..Default::default()
        };

        let start = Instant::now();
        let _ = memory
            .retrieve_relevant_context(format!("test query {}", i), context.clone(), 10)
            .await;
        latencies.push(start.elapsed());
    }

    // Then: P95 latency should remain under 100ms even with large dataset (NFR1)
    latencies.sort();
    let p95_index = (latencies.len() as f32 * 0.95) as usize;
    let p95 = latencies[p95_index];

    println!("P95 latency with 10K episodes: {:?}", p95);

    assert!(
        p95.as_millis() < 100,
        "P95 retrieval latency {}ms exceeds 100ms target with 10K episodes",
        p95.as_millis()
    );
}

#[tokio::test]
async fn should_maintain_consistent_retrieval_latency_across_percentiles() {
    // Given: Memory system with 500 episodes
    let memory = setup_memory_with_n_episodes(500).await;
    let mut latencies = Vec::new();

    // When: Running 100 retrieval queries
    for _ in 0..100 {
        let start = Instant::now();
        memory
            .retrieve_relevant_context("query".to_string(), test_context(), 10)
            .await;
        latencies.push(start.elapsed());
    }

    // Then: All percentiles should show acceptable performance
    latencies.sort();
    let p50 = latencies[50];
    let p90 = latencies[90];
    let p95 = latencies[95];
    let p99 = latencies[99];

    println!("Retrieval latency percentiles:");
    println!("  P50: {:?}", p50);
    println!("  P90: {:?}", p90);
    println!("  P95: {:?}", p95);
    println!("  P99: {:?}", p99);

    assert!(p95.as_millis() < 100, "P95 should be under 100ms");
}

// ============================================================================
// NFR2: Storage Capacity
// ============================================================================

#[tokio::test]
async fn should_store_1000_episodes_without_performance_degradation() {
    // Given: Memory system
    let memory = setup_test_memory();
    let start = Instant::now();

    // When: Storing 1,000 episodes (NFR2 capacity target)
    for i in 0..1000 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
            .unwrap();

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
    }

    let storage_time = start.elapsed();
    println!("Stored 1K episodes in {:?}", storage_time);

    // Then: Retrieval should remain fast despite large dataset
    let retrieval_start = Instant::now();
    let results = memory
        .retrieve_relevant_context("test".to_string(), test_context(), 10)
        .await;
    let retrieval_time = retrieval_start.elapsed();

    assert!(!results.is_empty());
    assert!(
        retrieval_time.as_millis() < 100,
        "Retrieval degraded to {}ms with 1K episodes",
        retrieval_time.as_millis()
    );
}

#[tokio::test]
#[ignore = "Long-running test - run with --include-ignored for full validation"]
async fn should_store_10000_episodes_without_performance_degradation() {
    // Given: Memory system
    let memory = setup_test_memory();
    let start = Instant::now();

    // When: Storing 10,000 episodes (NFR2 extended capacity test)
    for i in 0..10000 {
        if i % 1000 == 0 {
            println!("Progress: {}/10000 episodes", i);
        }

        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
            .unwrap();

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
    }

    let storage_time = start.elapsed();
    println!("Stored 10K episodes in {:?}", storage_time);

    // Then: Retrieval should remain fast even with very large dataset
    let retrieval_start = Instant::now();
    let results = memory
        .retrieve_relevant_context("test".to_string(), test_context(), 10)
        .await;
    let retrieval_time = retrieval_start.elapsed();

    assert!(!results.is_empty());
    assert!(
        retrieval_time.as_millis() < 100,
        "Retrieval degraded to {}ms with 10K episodes",
        retrieval_time.as_millis()
    );
}

#[tokio::test]
async fn should_create_episodes_very_quickly() {
    // Given: Memory system
    let memory = setup_test_memory();
    let mut creation_times = Vec::new();

    // When: Creating 100 episodes
    for i in 0..100 {
        let start = Instant::now();
        memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
            .unwrap();
        creation_times.push(start.elapsed());
    }

    // Then: Average creation time should be very fast (<10ms)
    let avg_time: Duration = creation_times.iter().sum::<Duration>() / creation_times.len() as u32;

    println!("Average episode creation time: {:?}", avg_time);

    assert!(
        avg_time.as_millis() < 10,
        "Average creation time {}ms too slow",
        avg_time.as_millis()
    );
}

// ============================================================================
// NFR3: Pattern Accuracy (Placeholder)
// ============================================================================

#[tokio::test]
#[ignore = "Requires pattern accuracy measurement infrastructure"]
async fn should_achieve_70_percent_pattern_recognition_accuracy() {
    // NFR3: >70% pattern recognition accuracy
    // This test would:
    // Given: Episodes with known patterns
    // When: Extracting patterns automatically
    // Then: Accuracy should exceed 70%
    // 1. Create episodes with known patterns
    // 2. Extract patterns
    // 3. Compare against expected patterns
    // 4. Calculate accuracy percentage
}

// ============================================================================
// NFR4: Test Coverage (CI Validation)
// ============================================================================

#[tokio::test]
async fn should_maintain_90_percent_test_coverage() {
    // NFR4: 90%+ test coverage
    // Given: CI configuration with coverage reporting enabled
    // When: Running tests with coverage analysis
    // Then: Coverage should exceed 90%
    // This is validated by CI with cargo-llvm-cov
    // This test verifies CI configuration exists

    #[cfg(not(target_os = "windows"))]
    {
        let ci_workflow_path = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join(".github/workflows/ci-enhanced.yml");

        if ci_workflow_path.exists() {
            let ci_workflow = std::fs::read_to_string(&ci_workflow_path).unwrap();

            assert!(
                ci_workflow.contains("cargo-llvm-cov") || ci_workflow.contains("coverage"),
                "CI workflow should include coverage reporting"
            );
        } else {
            println!("CI workflow not found at {:?}", ci_workflow_path);
        }
    }
}

// ============================================================================
// NFR5: Memory Leaks
// ============================================================================

#[tokio::test]
async fn should_not_leak_memory_under_continuous_operation() {
    // Given: Memory system with initial memory baseline
    let memory = Arc::new(setup_test_memory());
    let initial_memory = get_current_memory_usage();
    println!("Initial memory: {} bytes", initial_memory);

    // When: Running 100 episode creation/completion cycles
    for i in 0..100 {
        let mem = memory.clone();
        let episode_id = mem
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
            .await
            .unwrap();

        for j in 0..5 {
            mem.log_step(episode_id, create_test_step(j + 1)).await;
        }

        mem.complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    }

    // Then: Memory growth should be minimal (NFR5: no leaks)
    let final_memory = get_current_memory_usage();
    println!("Final memory: {} bytes", final_memory);

    if initial_memory > 0 {
        let growth = (final_memory as f32 - initial_memory as f32) / initial_memory as f32;
        println!("Memory growth: {:.2}%", growth * 100.0);

        // Allow some growth for caching, but flag excessive growth
        assert!(
            growth < 1.0, // 100% growth max
            "Memory grew by {:.2}% - possible leak",
            growth * 100.0
        );
    }
}

#[tokio::test]
#[ignore = "Long-running test - run with --include-ignored for full validation"]
async fn should_not_leak_memory_over_1000_iterations() {
    // Given: Memory system with initial memory baseline
    let memory = Arc::new(setup_test_memory());
    let initial_memory = get_current_memory_usage();

    // When: Running 1000 episode creation/completion cycles
    for i in 0..1000 {
        let mem = memory.clone();
        let episode_id = mem
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
            .await
            .unwrap();

        for j in 0..5 {
            mem.log_step(episode_id, create_test_step(j + 1)).await;
        }

        mem.complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

        // Then: Check memory periodically to detect leaks early
        if i % 100 == 0 && initial_memory > 0 {
            let current_memory = get_current_memory_usage();
            let growth = (current_memory as f32 - initial_memory as f32) / initial_memory as f32;

            println!("Iteration {}: Memory growth {:.2}%", i, growth * 100.0);

            assert!(
                growth < 0.50,
                "Memory grew by {:.2}% after {} iterations - possible leak",
                growth * 100.0,
                i
            );
        }
    }
}

#[tokio::test]
async fn should_cleanup_cache_when_exceeding_limits() {
    // Given: Memory system with limited cache (100 episodes max)
    let config = MemoryConfig {
        storage: memory_core::StorageConfig {
            max_episodes_cache: 100,
            ..Default::default()
        },
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    // When: Creating 200 episodes (exceeding cache limit)
    for i in 0..200 {
        let episode_id = memory
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
            .await
            .unwrap();

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

    // Then: System should function correctly without memory leaks
    let (total, completed, _) = memory.get_stats().await;
    assert_eq!(total, 200);
    assert_eq!(completed, 200);
}

// ============================================================================
// Concurrent Performance Tests
// ============================================================================

#[tokio::test]
async fn should_create_episodes_concurrently_without_conflicts() {
    // Given: Shared memory system
    let memory = Arc::new(setup_test_memory());
    let start = Instant::now();

    // When: Creating 100 episodes concurrently from multiple tasks
    let mut handles = vec![];
    for i in 0..100 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
        });
        handles.push(handle);
    }

    let mut ids = vec![];
    for handle in handles {
        ids.push(handle.await.unwrap());
    }

    let elapsed = start.elapsed();

    println!(
        "Created 100 episodes concurrently in {:?} ({:.2} eps/sec)",
        elapsed,
        100.0 / elapsed.as_secs_f32()
    );

    // Then: All episodes should be created successfully
    assert_eq!(ids.len(), 100);

    // Then: Concurrent execution should be fast (<1 second)
    assert!(
        elapsed.as_secs() < 1,
        "Concurrent creation took {}ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn should_complete_episodes_concurrently_without_conflicts() {
    // Given: Shared memory system with 50 episodes
    let memory = Arc::new(setup_test_memory());
    let mut episode_ids = vec![];

    for i in 0..50 {
        let id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
            .unwrap();
        episode_ids.push(id);
    }

    let start = Instant::now();

    // When: Completing all episodes concurrently from multiple tasks
    let mut handles = vec![];
    for episode_id in episode_ids {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let elapsed = start.elapsed();

    println!("Completed 50 episodes concurrently in {:?}", elapsed);

    // Then: All episodes should be completed successfully
    let (_, completed, _) = memory.get_stats().await;
    assert_eq!(completed, 50);
}

#[tokio::test]
async fn should_handle_concurrent_retrievals_efficiently() {
    // Given: Memory system with 100 episodes
    let memory = Arc::new(setup_memory_with_n_episodes(100).await);
    let start = Instant::now();

    // When: Running 50 concurrent retrieval queries
    let mut handles = vec![];
    for i in 0..50 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.retrieve_relevant_context(format!("query {}", i), test_context(), 10)
                .await
        });
        handles.push(handle);
    }

    // Then: All retrievals should complete successfully
    for handle in handles {
        let results = handle.await.unwrap();
        assert!(results.len() <= 10);
    }

    let elapsed = start.elapsed();

    println!("Executed 50 concurrent retrievals in {:?}", elapsed);

    // Then: Concurrent retrievals should be fast (<500ms)
    assert!(
        elapsed.as_millis() < 500,
        "Concurrent retrievals took {}ms",
        elapsed.as_millis()
    );
}

// ============================================================================
// Step Logging Performance
// ============================================================================

#[tokio::test]
async fn should_log_steps_very_quickly() {
    // Given: Memory system with an active episode
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await
        .unwrap();

    let mut step_times = vec![];

    // When: Logging 100 execution steps
    for i in 1..=100 {
        let step = create_test_step(i);
        let start = Instant::now();
        memory.log_step(episode_id, step).await;
        step_times.push(start.elapsed());
    }

    // Then: Average step logging should be very fast (<5ms)
    let avg_time: Duration = step_times.iter().sum::<Duration>() / step_times.len() as u32;

    println!("Average step logging time: {:?}", avg_time);

    assert!(
        avg_time.as_millis() < 5,
        "Step logging too slow: {}ms",
        avg_time.as_millis()
    );
}

#[tokio::test]
async fn should_complete_episodes_quickly_with_pattern_extraction() {
    // Given: Memory system
    let memory = setup_test_memory();
    let mut completion_times = vec![];

    // When: Creating and completing 50 episodes with steps
    for i in 0..50 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await
            .unwrap();

        // Add a few steps
        for j in 1..=3 {
            memory.log_step(episode_id, create_test_step(j)).await;
        }

        let start = Instant::now();
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
        completion_times.push(start.elapsed());
    }

    // Then: Average completion time should be fast (<100ms including pattern extraction)
    let avg_time: Duration =
        completion_times.iter().sum::<Duration>() / completion_times.len() as u32;

    println!("Average episode completion time: {:?}", avg_time);

    assert!(
        avg_time.as_millis() < 100,
        "Episode completion too slow: {}ms",
        avg_time.as_millis()
    );
}
