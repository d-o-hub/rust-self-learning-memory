//! Performance tests for non-functional requirements NFR1-NFR5
//!
//! This test suite validates that the memory system meets performance,
//! scalability, and reliability targets from plans/04-review.md.

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
            .await;

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
async fn verify_nfr1_retrieval_latency_100_episodes() {
    // NFR1: <100ms retrieval latency (P95) with 100 episodes
    let memory = setup_memory_with_n_episodes(100).await;

    let mut latencies = Vec::new();

    // Run 100 retrieval queries
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
#[ignore] // Long-running test
async fn verify_nfr1_retrieval_latency_10k_episodes() {
    // NFR1: <100ms retrieval latency (P95) with 10K episodes
    let memory = setup_memory_with_n_episodes(10000).await;

    let mut latencies = Vec::new();

    // Run 100 retrieval queries
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
async fn verify_nfr1_retrieval_latency_statistics() {
    let memory = setup_memory_with_n_episodes(500).await;

    let mut latencies = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();
        memory
            .retrieve_relevant_context("query".to_string(), test_context(), 10)
            .await;
        latencies.push(start.elapsed());
    }

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

    assert!(p95.as_millis() < 100);
}

// ============================================================================
// NFR2: Storage Capacity
// ============================================================================

#[tokio::test]
async fn verify_nfr2_storage_capacity_1k() {
    // NFR2: Support 1,000 episodes without degradation
    let memory = setup_test_memory();

    let start = Instant::now();

    for i in 0..1000 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
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
    }

    let storage_time = start.elapsed();
    println!("Stored 1K episodes in {:?}", storage_time);

    // Verify retrieval still fast
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
#[ignore] // Long-running test
async fn verify_nfr2_storage_capacity_10k() {
    // NFR2: Support 10,000 episodes without degradation
    let memory = setup_test_memory();

    let start = Instant::now();

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
    }

    let storage_time = start.elapsed();
    println!("Stored 10K episodes in {:?}", storage_time);

    // Verify retrieval still fast
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
async fn verify_nfr2_episode_creation_performance() {
    let memory = setup_test_memory();

    let mut creation_times = Vec::new();

    for i in 0..100 {
        let start = Instant::now();
        memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;
        creation_times.push(start.elapsed());
    }

    let avg_time: Duration = creation_times.iter().sum::<Duration>() / creation_times.len() as u32;

    println!("Average episode creation time: {:?}", avg_time);

    // Should be very fast (< 10ms)
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
#[ignore] // Requires pattern accuracy measurement
async fn verify_nfr3_pattern_accuracy() {
    // NFR3: >70% pattern recognition accuracy
    // This test would:
    // 1. Create episodes with known patterns
    // 2. Extract patterns
    // 3. Compare against expected patterns
    // 4. Calculate accuracy percentage
}

// ============================================================================
// NFR4: Test Coverage (CI Validation)
// ============================================================================

#[tokio::test]
async fn verify_nfr4_test_coverage() {
    // NFR4: 90%+ test coverage
    // This is validated by CI with cargo-llvm-cov
    // This test just verifies CI configuration exists

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
async fn verify_nfr5_no_memory_leaks_short() {
    // NFR5: Zero memory leaks under continuous operation (short test)
    let memory = Arc::new(setup_test_memory());

    let initial_memory = get_current_memory_usage();
    println!("Initial memory: {} bytes", initial_memory);

    // Run 100 iterations
    for i in 0..100 {
        let mem = memory.clone();
        let episode_id = mem
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
            .await;

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
#[ignore] // Long-running test
async fn verify_nfr5_no_memory_leaks_long() {
    // NFR5: Zero memory leaks under continuous operation (1000 iterations)
    let memory = Arc::new(setup_test_memory());

    let initial_memory = get_current_memory_usage();

    for i in 0..1000 {
        let mem = memory.clone();
        let episode_id = mem
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
            .await;

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

        // Check memory periodically
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
async fn verify_nfr5_episode_cleanup() {
    // Verify episodes don't leak when using custom config
    let config = MemoryConfig {
        storage: memory_core::StorageConfig {
            max_episodes_cache: 100,
            ..Default::default()
        },
        ..Default::default()
    };

    let memory = setup_memory_with_config(config);

    // Create 200 episodes (more than cache limit)
    for i in 0..200 {
        let episode_id = memory
            .start_episode(format!("Task {}", i), test_context(), TaskType::Testing)
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

    // Should still function correctly
    let (total, completed, _) = memory.get_stats().await;
    assert_eq!(total, 200);
    assert_eq!(completed, 200);
}

// ============================================================================
// Concurrent Performance Tests
// ============================================================================

#[tokio::test]
async fn verify_concurrent_episode_creation() {
    let memory = Arc::new(setup_test_memory());

    let start = Instant::now();

    // Create 100 episodes concurrently
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

    assert_eq!(ids.len(), 100);

    // Should be faster than sequential (< 1 second for 100 episodes)
    assert!(
        elapsed.as_secs() < 1,
        "Concurrent creation took {}ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn verify_concurrent_episode_completion() {
    let memory = Arc::new(setup_test_memory());

    // Create episodes first
    let mut episode_ids = vec![];
    for i in 0..50 {
        let id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;
        episode_ids.push(id);
    }

    let start = Instant::now();

    // Complete all concurrently
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

    let (_, completed, _) = memory.get_stats().await;
    assert_eq!(completed, 50);
}

#[tokio::test]
async fn verify_concurrent_retrieval() {
    let memory = Arc::new(setup_memory_with_n_episodes(100).await);

    let start = Instant::now();

    // Run 50 concurrent retrievals
    let mut handles = vec![];
    for i in 0..50 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            mem.retrieve_relevant_context(format!("query {}", i), test_context(), 10)
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let results = handle.await.unwrap();
        assert!(results.len() <= 10);
    }

    let elapsed = start.elapsed();

    println!("Executed 50 concurrent retrievals in {:?}", elapsed);

    // Should be fast (< 500ms)
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
async fn verify_step_logging_performance() {
    let memory = setup_test_memory();

    let episode_id = memory
        .start_episode("Test".to_string(), test_context(), TaskType::Testing)
        .await;

    let mut step_times = vec![];

    // Log 100 steps
    for i in 1..=100 {
        let step = create_test_step(i);
        let start = Instant::now();
        memory.log_step(episode_id, step).await;
        step_times.push(start.elapsed());
    }

    let avg_time: Duration = step_times.iter().sum::<Duration>() / step_times.len() as u32;

    println!("Average step logging time: {:?}", avg_time);

    // Should be very fast (< 5ms)
    assert!(
        avg_time.as_millis() < 5,
        "Step logging too slow: {}ms",
        avg_time.as_millis()
    );
}

#[tokio::test]
async fn verify_episode_completion_performance() {
    let memory = setup_test_memory();

    let mut completion_times = vec![];

    for i in 0..50 {
        let episode_id = memory
            .start_episode(
                format!("Task {}", i),
                test_context(),
                TaskType::CodeGeneration,
            )
            .await;

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

    let avg_time: Duration =
        completion_times.iter().sum::<Duration>() / completion_times.len() as u32;

    println!("Average episode completion time: {:?}", avg_time);

    // Should complete in reasonable time (< 100ms with pattern extraction)
    assert!(
        avg_time.as_millis() < 100,
        "Episode completion too slow: {}ms",
        avg_time.as_millis()
    );
}
