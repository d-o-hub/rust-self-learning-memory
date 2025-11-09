# Async Testing in Rust

Guide to testing async code with Tokio and async/await.

## Basic Async Test

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Common Patterns

### Testing Async Functions

```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode() {
    // Arrange
    let memory = create_test_memory().await;
    let context = TaskContext::default();

    // Act
    let episode_id = memory.start_episode("Test task", context).await;

    // Assert
    assert!(episode_id.is_ok());
}
```

### Testing Error Propagation

```rust
#[tokio::test]
async fn test_save_episode_database_error_propagates() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await;
    let episode = create_test_episode("Test");

    // Act
    let result = memory.save_episode(&episode).await?;

    // Assert
    assert!(result.is_ok());
    Ok(())
}
```

### Concurrent Operations

```rust
#[tokio::test]
async fn test_concurrent_episode_creation() {
    // Arrange
    let memory = create_test_memory().await;

    // Act - Create episodes concurrently
    let handles: Vec<_> = (0..10)
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

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();

    // Assert
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Tokio Test Attributes

### Basic Async Test
```rust
#[tokio::test]
async fn test_basic() {
    // ...
}
```

### Multi-threaded Runtime (Default)
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent() {
    // ...
}
```

### Single-threaded Runtime
```rust
#[tokio::test(flavor = "current_thread")]
async fn test_sequential() {
    // Useful for debugging race conditions
}
```

## Testing Timeouts

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
}
```

## Testing Channels

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
async fn test_spawned_task_completes() {
    // Arrange
    let (tx, mut rx) = mpsc::channel(1);

    // Act
    let handle = tokio::spawn(async move {
        tx.send("done").await.unwrap();
    });

    // Wait for task
    handle.await.unwrap();

    // Assert
    assert_eq!(rx.recv().await, Some("done"));
}
```

## Common Pitfalls

### Missing `.await`

❌ Bad:
```rust
#[tokio::test]
async fn test_operation() {
    let result = async_function(); // Missing .await
    assert!(result.is_ok()); // Won't compile
}
```

✅ Good:
```rust
#[tokio::test]
async fn test_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Blocking the Runtime

❌ Bad:
```rust
#[tokio::test]
async fn test_with_blocking_operation() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks runtime!
    let result = async_function().await;
    assert!(result.is_ok());
}
```

✅ Good:
```rust
#[tokio::test]
async fn test_with_async_sleep() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Not Handling Panics in Spawned Tasks

❌ Bad:
```rust
#[tokio::test]
async fn test_spawned_task() {
    tokio::spawn(async {
        panic!("This panic is swallowed!");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    // Test passes even though task panicked
}
```

✅ Good:
```rust
#[tokio::test]
async fn test_spawned_task() {
    let handle = tokio::spawn(async {
        panic!("This panic is caught!");
    });

    // Will propagate panic
    let result = handle.await;
    assert!(result.is_err());
}
```

## Testing with redb (Synchronous DB)

Use `spawn_blocking` for synchronous operations:

```rust
#[tokio::test]
async fn test_redb_operation() {
    // Arrange
    let db = create_test_redb();

    // Act - Wrap sync operation in spawn_blocking
    let result = tokio::task::spawn_blocking(move || {
        let write_txn = db.begin_write()?;
        // ... perform write operations
        write_txn.commit()?;
        Ok::<_, anyhow::Error>(())
    }).await;

    // Assert
    assert!(result.is_ok());
}
```

## Testing Turso (Async DB)

```rust
#[tokio::test]
async fn test_turso_query() {
    // Arrange
    let client = create_test_turso_client().await;

    // Act
    let result = client
        .execute("SELECT * FROM episodes WHERE id = ?", &[&"test_id"])
        .await;

    // Assert
    assert!(result.is_ok());
}
```

## Mocking Async Dependencies

```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
#[async_trait::async_trait]
trait AsyncStorage {
    async fn save(&self, data: &str) -> Result<(), Error>;
}

#[tokio::test]
async fn test_with_mock_storage() {
    // Arrange
    let mut mock = MockAsyncStorage::new();
    mock.expect_save()
        .with(eq("test_data"))
        .times(1)
        .returning(|_| Ok(()));

    // Act
    let result = mock.save("test_data").await;

    // Assert
    assert!(result.is_ok());
}
```

## Testing Retry Logic

```rust
#[tokio::test]
async fn test_retry_on_failure() {
    // Arrange
    let mut attempt = 0;

    // Act
    let result = retry_with_backoff(|| async {
        attempt += 1;
        if attempt < 3 {
            Err(anyhow!("Transient error"))
        } else {
            Ok("success")
        }
    }).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(attempt, 3);
}
```

## Deterministic Time Testing

```rust
use tokio::time::{self, Duration, Instant};

#[tokio::test(start_paused = true)]
async fn test_with_paused_time() {
    // Time starts paused
    let start = Instant::now();

    // Advance time without actually waiting
    time::advance(Duration::from_secs(60)).await;

    assert_eq!(start.elapsed(), Duration::from_secs(60));
}
```

## Best Practices

1. **Always use `#[tokio::test]`** for async tests
2. **Don't block the runtime** - use `spawn_blocking` for sync code
3. **Handle spawned task errors** - await handles to catch panics
4. **Use timeouts** for operations that might hang
5. **Test concurrent behavior** explicitly
6. **Clean up resources** in Drop implementations
7. **Use `start_paused = true`** for time-dependent tests

## Project-Specific Examples

### Testing Episode Lifecycle

```rust
#[tokio::test]
async fn test_episode_full_lifecycle() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await;
    let context = TaskContext::default();

    // Act - Start
    let id = memory.start_episode("Test task", context.clone()).await?;

    // Act - Log steps
    memory.log_step(&id, create_test_step(1)).await?;
    memory.log_step(&id, create_test_step(2)).await?;

    // Act - Complete
    memory.complete_episode(&id, TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    }).await?;

    // Assert
    let episode = memory.get_episode(&id).await?;
    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 2);
    Ok(())
}
```

### Testing Pattern Extraction

```rust
#[tokio::test]
async fn test_extract_patterns_multiple_episodes() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await;
    let episodes = create_test_episodes(10, "testing");

    for ep in &episodes {
        memory.save_episode(ep).await?;
    }

    // Act
    let patterns = memory.extract_patterns().await?;

    // Assert
    assert!(!patterns.is_empty());
    assert!(patterns.iter().any(|p| p.pattern_type() == "ToolSequence"));
    Ok(())
}
```
