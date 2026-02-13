# Advanced Tokio Patterns

## Time Manipulation

### Paused Time for Deterministic Testing

```rust
#[tokio::test(start_paused = true)]
async fn test_with_paused_time() {
    let start = tokio::time::Instant::now();

    // Advance time by 1 hour
    tokio::time::sleep(Duration::from_secs(3600)).await;

    // Time hasn't actually passed
    assert!(start.elapsed().as_secs() < 1);

    // Advance time manually
    tokio::time::advance(Duration::from_secs(3600)).await;

    // Now time has passed
    assert!(start.elapsed().as_secs() >= 3600);
}
```

### Timeout Scenarios

```rust
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        slow_operation()
    ).await;

    match result {
        Ok(value) => assert!(value.is_ok()),
        Err(_) => println!("Operation timed out as expected"),
    }
}
```

## Runtime Configuration

### Multi-Threaded Runtime

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_operations() {
    let barrier = Arc::new(Barrier::new(4));
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                // All tasks start at the same time
                format!("Task {}", i)
            })
        })
        .collect();

    let results: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(results.len(), 4);
}
```

### Current Runtime Inspection

```rust
#[tokio::test]
async fn test_runtime_info() {
    let handle = tokio::runtime::Handle::current();
    let runtime = handle.runtime();

    // Check if in current thread runtime
    assert!(runtime.is_none()); // We're in a test context

    // Spawn a task and verify it's running
    let handle = tokio::spawn(async { "running" });
    assert_eq!(handle.await.unwrap(), "running");
}
```
