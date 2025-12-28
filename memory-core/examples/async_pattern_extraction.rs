//! Example demonstrating async pattern extraction queue
//!
//! Shows the difference between sync and async pattern extraction
//! and how to use the queue system.

use memory_core::{
    ExecutionResult, ExecutionStep, QueueConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::cast_precision_loss)]
async fn main() {
    println!("=== Async Pattern Extraction Demo ===\n");

    // ===== Synchronous Pattern Extraction =====
    println!("1. Synchronous Pattern Extraction (baseline):");
    let sync_memory = SelfLearningMemory::new();

    let start = Instant::now();
    let sync_episode_id = create_and_complete_episode(&sync_memory, "Sync task").await;
    let sync_duration = start.elapsed();

    println!("   Episode completed in: {sync_duration:?}");
    println!("   Patterns extracted: immediately\n");

    // ===== Asynchronous Pattern Extraction =====
    println!("2. Asynchronous Pattern Extraction:");

    // Create memory with async extraction enabled
    let async_memory = Arc::new(
        SelfLearningMemory::new().enable_async_extraction(QueueConfig {
            worker_count: 2,
            max_queue_size: 100,
            poll_interval_ms: 50,
        }),
    );

    // Start background workers
    async_memory.start_workers().await;
    println!("   Started 2 background workers");

    let start = Instant::now();
    let async_episode_id = create_and_complete_episode(&async_memory, "Async task").await;
    let async_duration = start.elapsed();

    println!("   Episode completed in: {async_duration:?}");
    println!("   Patterns extracting: in background\n");

    // Check queue stats
    if let Some(stats) = async_memory.get_queue_stats().await {
        println!("   Queue statistics:");
        println!("     - Enqueued: {}", stats.total_enqueued);
        println!("     - Queue size: {}", stats.current_queue_size);
        println!("     - Active workers: {}", stats.active_workers);
    }

    // ===== Performance Comparison =====
    println!("\n3. Performance Comparison:");
    println!("   Sync:  {:?}", sync_duration);
    println!("   Async: {:?}", async_duration);

    if async_duration < sync_duration {
        let speedup = sync_duration.as_micros() as f64 / async_duration.as_micros() as f64;
        println!("   Speedup: {:.2}x faster with async extraction", speedup);
    }

    println!("\n4. Verifying Episode Completion:");

    // Both episodes should be complete
    let sync_episode = sync_memory.get_episode(sync_episode_id).await.unwrap();
    let async_episode = async_memory.get_episode(async_episode_id).await.unwrap();

    println!("   Sync episode complete: {}", sync_episode.is_complete());
    println!("   Async episode complete: {}", async_episode.is_complete());

    // Wait a bit for async extraction to finish
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if let Some(final_stats) = async_memory.get_queue_stats().await {
        println!("\n5. Final Queue Statistics:");
        println!("   - Total processed: {}", final_stats.total_processed);
        println!("   - Total failed: {}", final_stats.total_failed);
        println!("   - Queue empty: {}", final_stats.current_queue_size == 0);
    }

    println!("\n=== Demo Complete ===");
}

/// Helper to create and complete an episode
async fn create_and_complete_episode(memory: &SelfLearningMemory, description: &str) -> uuid::Uuid {
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode(description.to_string(), context, TaskType::Testing)
        .await;

    // Add some execution steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        step.latency_ms = 50;
        memory.log_step(episode_id, step).await;
    }

    // Complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec!["result.txt".to_string()],
            },
        )
        .await
        .unwrap();

    episode_id
}
