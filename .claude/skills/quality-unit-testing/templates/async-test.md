# Async Test Template

Use this template for asynchronous Rust tests with Tokio:

## Basic Async Test

```rust
#[tokio::test]
async fn test_async_function_scenario_expected_behavior() {
    // Arrange
    let input = create_test_input();

    // Act
    let result = async_function(input).await;

    // Assert
    assert!(result.is_ok());
}
```

## Async Test with Error Propagation

```rust
#[tokio::test]
async fn test_async_function_scenario_expected() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await?;
    let episode = create_test_episode("Test");

    // Act
    memory.save_episode(&episode).await?;

    // Assert
    let retrieved = memory.get_episode(&episode.id).await?;
    assert_eq!(retrieved.task_description, episode.task_description);
    Ok(())
}
```

## Testing Concurrent Operations

```rust
#[tokio::test]
async fn test_concurrent_operations_maintain_consistency() {
    // Arrange
    let memory = create_test_memory().await;
    let episode_id = memory
        .start_episode("Test", TaskContext::default())
        .await
        .unwrap();

    // Act - Spawn concurrent tasks
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let mem = memory.clone();
            let id = episode_id.clone();
            tokio::spawn(async move {
                let step = create_test_step(i);
                mem.log_step(&id, step).await
            })
        })
        .collect();

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Assert
    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 10);
}
```

## Testing with Timeout

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_operation_completes_within_timeout() {
    // Arrange
    let memory = create_test_memory().await;

    // Act
    let result = timeout(
        Duration::from_secs(5),
        memory.long_running_operation()
    ).await;

    // Assert
    assert!(result.is_ok(), "Operation timed out");
    assert!(result.unwrap().is_ok());
}
```

## Testing with Channels

```rust
use tokio::sync::mpsc;

#[tokio::test]
async fn test_channel_communication() {
    // Arrange
    let (tx, mut rx) = mpsc::channel(10);

    // Act
    tx.send("message").await.unwrap();

    // Assert
    let received = rx.recv().await;
    assert_eq!(received, Some("message"));
}
```

## Testing Spawned Tasks

```rust
#[tokio::test]
async fn test_background_task_completes() {
    // Arrange
    let (tx, mut rx) = mpsc::channel(1);

    // Act
    let handle = tokio::spawn(async move {
        // Simulate work
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx.send("done").await.unwrap();
    });

    // Wait for task
    handle.await.unwrap();

    // Assert
    assert_eq!(rx.recv().await, Some("done"));
}
```

## Testing with redb (Sync Code in Async Context)

```rust
#[tokio::test]
async fn test_redb_write_operation() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.db");
    let db = redb::Database::create(&path).unwrap();

    // Act - Wrap sync operation in spawn_blocking
    let result = tokio::task::spawn_blocking(move || {
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE_DEF)?;
            table.insert("key", "value")?;
        }
        write_txn.commit()?;
        Ok::<_, anyhow::Error>(())
    }).await;

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}
```

## Testing Full Async Lifecycle

```rust
#[tokio::test]
async fn test_episode_full_lifecycle() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await?;
    let context = TaskContext::default();

    // Act - Start
    let episode_id = memory.start_episode("Test task", context.clone()).await?;

    // Act - Add steps
    memory.log_step(&episode_id, create_test_step(1)).await?;
    memory.log_step(&episode_id, create_test_step(2)).await?;

    // Act - Complete
    memory.complete_episode(&episode_id, TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    }).await?;

    // Assert
    let episode = memory.get_episode(&episode_id).await?;
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 2);
    assert!(episode.reward_score.is_some());

    Ok(())
}
```

## Testing Error Cases

```rust
#[tokio::test]
async fn test_get_episode_invalid_id_returns_error() {
    // Arrange
    let memory = create_test_memory().await;

    // Act
    let result = memory.get_episode("nonexistent_id").await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::EpisodeNotFound(_)
    ));
}
```

## Testing with Multi-threaded Runtime

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_high_concurrency_scenario() {
    // Arrange
    let memory = create_test_memory().await;

    // Act - Create many episodes concurrently
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let mem = memory.clone();
            tokio::spawn(async move {
                mem.start_episode(
                    &format!("Task {}", i),
                    TaskContext::default()
                ).await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert
    assert_eq!(results.len(), 100);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Testing with Deterministic Time

```rust
#[tokio::test(start_paused = true)]
async fn test_with_paused_time() {
    use tokio::time::{self, Duration, Instant};

    // Time starts paused
    let start = Instant::now();

    // Advance time without actually waiting
    time::advance(Duration::from_secs(60)).await;

    // Assert
    assert_eq!(start.elapsed(), Duration::from_secs(60));
}
```
