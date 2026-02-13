# Concurrent Operations Patterns

## Parallel Execution

### Basic Parallel Spawn

```rust
#[tokio::test]
async fn test_parallel_spawn() {
    let handles: Vec<_> = (0..10)
        .map(|i| tokio::spawn(async move { i * 2 }))
        .collect();

    let results: Vec<i32> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(results, (0..10).map(|i| i * 2).collect::<Vec<_>>());
}
```

### Semaphore-Controlled Concurrency

```rust
#[tokio::test]
async fn test_semaphore_limited_concurrency() {
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for i in 0..10 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            drop(permit); // Release permit when done
            i
        }));
    }

    let results: Vec<i32> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(results.len(), 10);
}
```

### Channel-Based Coordination

```rust
#[tokio::test]
async fn test_channel_coordination() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);
    let mut handles = Vec::new();

    for i in 0..5 {
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 10)).await;
            tx.send(i).await.unwrap();
        }));
    }

    drop(tx); // Close sender

    let mut received = Vec::new();
    while let Some(val) = rx.recv().await {
        received.push(val);
    }

    assert_eq!(received.len(), 5);
}
```

## Error Handling in Concurrent Code

### Early Failure Detection

```rust
#[tokio::test]
async fn test_early_failure() {
    let results: Vec<Result<i32, &str>> = (0..10)
        .map(|i| {
            if i == 5 {
                Err("fail at 5")
            } else {
                Ok(i)
            }
        })
        .collect();

    let fail_index = results.iter()
        .position(|r| r.is_err())
        .expect("Should find error");

    assert_eq!(fail_index, 5);
}
```

### Retry with Concurrency

```rust
async fn retry_with_backoff<F, T, E>(
    mut f: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let delay = Duration::from_millis(2u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```
