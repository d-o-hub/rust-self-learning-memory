//! Integration tests for step batching functionality
//!
//! Tests verify the complete batching workflow including:
//! - Auto-flush on size threshold
//! - Auto-flush on time threshold
//! - Manual flush
//! - Episode completion flushes steps
//! - Batching disabled (immediate persistence)
//! - Concurrent episodes with separate buffers
//! - Performance improvements from batching
//! - Data integrity and ordering

mod common;

use common::{create_success_step, setup_memory_with_config, ContextBuilder};
use memory_core::{BatchConfig, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome, TaskType};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test auto-flush when buffer reaches `max_batch_size`
#[tokio::test]
async fn test_step_buffering_with_auto_flush_on_size() -> anyhow::Result<()> {
    // Arrange: Configure batch size of 10 steps
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 10,
            flush_interval_ms: 60000, // Very long interval so only size triggers
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("step-batching-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test size-based auto-flush".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act: Log 9 steps - should not trigger flush
    for i in 1..=9 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Assert: Steps not yet persisted (still in buffer)
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        0,
        "Steps should still be buffered, not persisted"
    );

    // Act: Log 10th step - should trigger auto-flush
    let step = create_success_step(10, "test_tool", "Action 10");
    memory.log_step(episode_id, step).await;

    // Assert: All 10 steps should now be persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        10,
        "All 10 steps should be flushed and persisted"
    );

    // Verify ordering preserved
    for i in 0..10 {
        assert_eq!(
            episode.steps[i].step_number,
            i + 1,
            "Step ordering should be preserved"
        );
    }

    Ok(())
}

/// Test auto-flush when time threshold is exceeded
#[tokio::test]
async fn test_step_buffering_with_auto_flush_on_time() -> anyhow::Result<()> {
    // Arrange: Configure very short flush interval (100ms)
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 1000,   // Very large so only time triggers
            flush_interval_ms: 100, // Short interval for test
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("time-flush-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test time-based auto-flush".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act: Log 3 steps quickly
    for i in 1..=3 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Assert: Steps not yet persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        0,
        "Steps should still be buffered initially"
    );

    // Act: Wait for time threshold (100ms + buffer)
    sleep(Duration::from_millis(150)).await;

    // Log one more step - this should trigger time-based flush of all buffered steps
    let step = create_success_step(4, "test_tool", "Action 4");
    memory.log_step(episode_id, step).await;

    // Assert: All 4 steps should now be persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        4,
        "All steps should be flushed due to time threshold"
    );

    Ok(())
}

/// Test manual flush via `flush_steps()`
#[tokio::test]
async fn test_manual_flush() -> anyhow::Result<()> {
    // Arrange: Configure large batch size and long interval (no auto-flush)
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 100,
            flush_interval_ms: 60000,
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("manual-flush-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode("Test manual flush".to_string(), context, TaskType::Testing)
        .await;

    // Act: Log several steps
    for i in 1..=5 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Assert: Steps not yet persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(episode.steps.len(), 0, "Steps should still be buffered");

    // Act: Manually flush
    memory.flush_steps(episode_id).await?;

    // Assert: All steps now persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(episode.steps.len(), 5, "All steps should be flushed");

    // Verify buffer is cleared - log another step
    let step = create_success_step(6, "test_tool", "Action 6");
    memory.log_step(episode_id, step).await;

    // This step should still be buffered
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(episode.steps.len(), 5, "New step should be in fresh buffer");

    // Flush again to verify
    memory.flush_steps(episode_id).await?;
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        6,
        "Second flush should persist new step"
    );

    Ok(())
}

/// Test that `complete_episode` flushes buffered steps
#[tokio::test]
async fn test_complete_episode_flushes_steps() -> anyhow::Result<()> {
    // Arrange: Configure large batch to prevent auto-flush
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 100,
            flush_interval_ms: 60000,
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("complete-flush-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test completion flushes steps".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act: Log buffered steps
    for i in 1..=8 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Verify steps are buffered (not persisted yet)
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(episode.steps.len(), 0, "Steps should be buffered");

    // Act: Complete episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Task completed".to_string(),
                artifacts: vec![],
            },
        )
        .await?;

    // Assert: All steps flushed before completion
    let completed_episode = memory.get_episode(episode_id).await?;
    assert!(
        completed_episode.is_complete(),
        "Episode should be complete"
    );
    assert_eq!(
        completed_episode.steps.len(),
        8,
        "All buffered steps should be flushed on completion"
    );

    // Verify reward and reflection are present (episode fully analyzed)
    assert!(
        completed_episode.reward.is_some(),
        "Completed episode should have reward"
    );
    assert!(
        completed_episode.reflection.is_some(),
        "Completed episode should have reflection"
    );

    Ok(())
}

/// Test batching disabled (`batch_config`: None) - immediate persistence
#[tokio::test]
async fn test_batching_disabled() -> anyhow::Result<()> {
    // Arrange: Create memory with batching disabled
    let config = MemoryConfig {
        batch_config: None, // No batching
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("no-batch-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test immediate persistence".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act & Assert: Each step should be persisted immediately
    for i in 1..=5 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;

        // Verify immediate persistence
        let episode = memory.get_episode(episode_id).await?;
        assert_eq!(
            episode.steps.len(),
            i,
            "Step {i} should be immediately persisted"
        );
    }

    Ok(())
}

/// Test concurrent episodes maintain separate buffers
#[tokio::test]
async fn test_multiple_episodes_concurrent_buffering() -> anyhow::Result<()> {
    // Arrange: Configure batching
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 20,
            flush_interval_ms: 5000,
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    // Create multiple episodes concurrently
    let mut episode_ids = Vec::new();
    for i in 0..3 {
        let context = ContextBuilder::new(format!("concurrent-test-{i}"))
            .language("rust")
            .build();

        let episode_id = memory
            .start_episode(format!("Concurrent task {i}"), context, TaskType::Testing)
            .await;
        episode_ids.push(episode_id);
    }

    // Act: Log different number of steps to each episode
    for (idx, episode_id) in episode_ids.iter().enumerate() {
        let step_count = (idx + 1) * 3; // 3, 6, 9 steps
        for step_num in 1..=step_count {
            let step = create_success_step(
                step_num,
                "test_tool",
                &format!("Episode {idx} Step {step_num}"),
            );
            memory.log_step(*episode_id, step).await;
        }
    }

    // Assert: Steps are buffered (not persisted yet)
    for episode_id in &episode_ids {
        let episode = memory.get_episode(*episode_id).await?;
        assert_eq!(
            episode.steps.len(),
            0,
            "Steps should be buffered for episode {episode_id}"
        );
    }

    // Act: Flush all episodes
    for episode_id in &episode_ids {
        memory.flush_steps(*episode_id).await?;
    }

    // Assert: Each episode has correct number of steps (isolation verified)
    for (idx, episode_id) in episode_ids.iter().enumerate() {
        let episode = memory.get_episode(*episode_id).await?;
        let expected_steps = (idx + 1) * 3;
        assert_eq!(
            episode.steps.len(),
            expected_steps,
            "Episode {} should have {} steps (got {})",
            idx,
            expected_steps,
            episode.steps.len()
        );

        // Verify step ordering for this episode
        for (step_idx, step) in episode.steps.iter().enumerate() {
            assert_eq!(
                step.step_number,
                step_idx + 1,
                "Step ordering should be preserved"
            );
        }
    }

    Ok(())
}

/// Benchmark-style test to verify batching reduces operations
#[tokio::test]
async fn test_batching_performance_improvement() -> anyhow::Result<()> {
    let step_count = 100;

    // Test 1: Without batching (immediate persistence)
    let config_no_batch = MemoryConfig {
        batch_config: None,
        ..Default::default()
    };
    let memory_no_batch = setup_memory_with_config(config_no_batch);

    let context1 = ContextBuilder::new("perf-no-batch")
        .language("rust")
        .build();

    let episode_id1 = memory_no_batch
        .start_episode(
            "Performance test - no batching".to_string(),
            context1,
            TaskType::Testing,
        )
        .await;

    let start_no_batch = Instant::now();
    for i in 1..=step_count {
        let step = ExecutionStep::new(i, "tool".to_string(), format!("Action {i}"));
        memory_no_batch.log_step(episode_id1, step).await;
    }
    let duration_no_batch = start_no_batch.elapsed();

    // Test 2: With batching
    let config_batch = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 50,
            flush_interval_ms: 60000,
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory_batch = setup_memory_with_config(config_batch);

    let context2 = ContextBuilder::new("perf-batch").language("rust").build();

    let episode_id2 = memory_batch
        .start_episode(
            "Performance test - with batching".to_string(),
            context2,
            TaskType::Testing,
        )
        .await;

    let start_batch = Instant::now();
    for i in 1..=step_count {
        let step = ExecutionStep::new(i, "tool".to_string(), format!("Action {i}"));
        memory_batch.log_step(episode_id2, step).await;
    }
    // Flush buffered steps
    memory_batch.flush_steps(episode_id2).await?;
    let duration_batch = start_batch.elapsed();

    // Assert: Verify both completed successfully with same data
    let episode1 = memory_no_batch.get_episode(episode_id1).await?;
    let episode2 = memory_batch.get_episode(episode_id2).await?;

    assert_eq!(
        episode1.steps.len(),
        step_count,
        "No-batch should have all steps"
    );
    assert_eq!(
        episode2.steps.len(),
        step_count,
        "Batch should have all steps"
    );

    // Print performance comparison (informational)
    println!("\n=== Performance Comparison ===");
    println!("Without batching: {duration_no_batch:?} ({step_count} steps)");
    println!("With batching:    {duration_batch:?} ({step_count} steps)");
    println!(
        "Speedup:          {:.2}x",
        // Clippy: Precision loss acceptable for speedup calculation in test
        #[allow(clippy::cast_precision_loss)]
        duration_no_batch.as_micros() as f64
            / duration_batch.as_micros() as f64
    );

    // Note: We don't assert performance improvement as CI environments vary
    // But in local testing, batching should show improvement

    Ok(())
}

/// Test data integrity - no data loss during multiple flushes
#[tokio::test]
async fn test_no_data_loss_on_flush() -> anyhow::Result<()> {
    // Arrange: Configure small batch size for frequent flushes
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 5,
            flush_interval_ms: 60000,
            auto_flush: true,
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("data-integrity-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test data integrity".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act: Log many steps (will trigger multiple auto-flushes)
    let total_steps = 27; // Will flush at 5, 10, 15, 20, 25, then 2 remain
    for i in 1..=total_steps {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Manual flush to get remaining steps
    memory.flush_steps(episode_id).await?;

    // Assert: All steps preserved
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        total_steps,
        "All steps should be preserved across multiple flushes"
    );

    // Verify ordering maintained
    for (idx, step) in episode.steps.iter().enumerate() {
        assert_eq!(
            step.step_number,
            idx + 1,
            "Step numbering should be sequential"
        );
        assert_eq!(
            step.action,
            format!("Action {}", idx + 1),
            "Step content should match"
        );
    }

    Ok(())
}

/// Test edge case: flush with empty buffer
#[tokio::test]
async fn test_flush_empty_buffer() -> anyhow::Result<()> {
    // Arrange
    let config = MemoryConfig {
        batch_config: Some(BatchConfig::default()),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = TaskContext::default();
    let episode_id = memory
        .start_episode("Test empty flush".to_string(), context, TaskType::Testing)
        .await;

    // Act: Flush with no steps logged
    memory.flush_steps(episode_id).await?;

    // Assert: Should succeed with no errors
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(episode.steps.len(), 0, "Should have no steps");

    Ok(())
}

/// Test edge case: flush non-existent episode
#[tokio::test]
async fn test_flush_nonexistent_episode() -> anyhow::Result<()> {
    // Arrange
    let config = MemoryConfig {
        batch_config: Some(BatchConfig::default()),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let fake_id = uuid::Uuid::new_v4();

    // Act: Flush non-existent episode
    let result = memory.flush_steps(fake_id).await;

    // Assert: Should succeed (no-op for non-existent buffer)
    assert!(
        result.is_ok(),
        "Flushing non-existent episode should not error"
    );

    Ok(())
}

/// Test buffer behavior with `auto_flush` disabled
#[tokio::test]
async fn test_manual_flush_only_mode() -> anyhow::Result<()> {
    // Arrange: Configure manual-only flushing
    let config = MemoryConfig {
        batch_config: Some(BatchConfig {
            max_batch_size: 10,
            flush_interval_ms: 1000,
            auto_flush: false, // Manual only
        }),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = ContextBuilder::new("manual-only-test")
        .language("rust")
        .build();

    let episode_id = memory
        .start_episode(
            "Test manual-only mode".to_string(),
            context,
            TaskType::Testing,
        )
        .await;

    // Act: Log many steps (exceeding batch size and wait time)
    for i in 1..=15 {
        let step = create_success_step(i, "test_tool", &format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Wait for potential time-based flush
    sleep(Duration::from_millis(1200)).await;

    // Log one more to check if time triggered anything
    let step = create_success_step(16, "test_tool", "Action 16");
    memory.log_step(episode_id, step).await;

    // Assert: No auto-flush should have occurred
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        0,
        "With auto_flush=false, no steps should be auto-flushed"
    );

    // Act: Manual flush
    memory.flush_steps(episode_id).await?;

    // Assert: Now all steps should be persisted
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        16,
        "Manual flush should persist all buffered steps"
    );

    Ok(())
}

/// Test preset configurations work correctly
#[tokio::test]
async fn test_batch_config_presets() -> anyhow::Result<()> {
    // Test high_frequency preset
    let high_freq = BatchConfig::high_frequency();
    assert_eq!(high_freq.max_batch_size, 20);
    assert_eq!(high_freq.flush_interval_ms, 2000);
    assert!(high_freq.auto_flush);

    // Test low_frequency preset
    let low_freq = BatchConfig::low_frequency();
    assert_eq!(low_freq.max_batch_size, 100);
    assert_eq!(low_freq.flush_interval_ms, 10000);
    assert!(low_freq.auto_flush);

    // Test manual_only preset
    let manual = BatchConfig::manual_only();
    assert_eq!(manual.max_batch_size, usize::MAX);
    assert!(!manual.auto_flush);

    // Verify high_frequency actually flushes more often
    let config = MemoryConfig {
        batch_config: Some(BatchConfig::high_frequency()),
        ..Default::default()
    };
    let memory = setup_memory_with_config(config);

    let context = TaskContext::default();
    let episode_id = memory
        .start_episode("Test preset".to_string(), context, TaskType::Testing)
        .await;

    // Log exactly 20 steps (should trigger flush)
    for i in 1..=20 {
        let step = ExecutionStep::new(i, "tool".to_string(), format!("Action {i}"));
        memory.log_step(episode_id, step).await;
    }

    // Should auto-flush at 20
    let episode = memory.get_episode(episode_id).await?;
    assert_eq!(
        episode.steps.len(),
        20,
        "High frequency preset should auto-flush at 20 steps"
    );

    Ok(())
}
