---
name: test-patterns
description: Unified testing patterns for Rust unit testing quality, episodic memory operations, and async/tokio code. Use when writing tests, reviewing test code, or diagnosing test failures.
---

# Test Patterns for Rust Self-Learning Memory

Unified testing patterns covering unit testing quality, episodic memory domain tests, and async/tokio patterns.

## Unit Testing Quality

### Naming: `test_<function>_<scenario>_<expected_behavior>`

```rust
#[test]
fn test_withdraw_valid_amount_decreases_balance()
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode()
```

### AAA Pattern

```rust
#[tokio::test]
async fn test_save_episode() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await;
    // Act
    memory.save_episode(&episode).await?;
    // Assert
    assert_eq!(retrieved.task_description, "Test");
    Ok(())
}
```

### Core Principles
- Single behavior per test, one reason to fail
- Mock external deps (APIs, DBs, filesystem)
- Don't mock: value types, pure functions
- Use `do-memory-test-utils` for setup

### Anti-Patterns

| Bad | Fix |
|-----|-----|
| Testing std library | Test your code |
| No assertions | Verify outcomes |
| Multiple behaviors | Split tests |
| Real external deps | Mock/test instances |
| `.unwrap()` chains | Use `Result<()>` with `?` |

## Episodic Memory Testing

### Lifecycle Pattern

```rust
#[tokio::test]
async fn test_episode_lifecycle() {
    let id = memory.start_episode("Test", ctx, TaskType::CodeGen).await;
    memory.log_execution_step(id.clone(), step).await;
    memory.complete_episode(id.clone(), TaskOutcome::Success, None).await?;
    let episode = memory.get_episode(&id).await?;
    assert_eq!(episode.outcome, TaskOutcome::Success);
}
```

### State: Created -> InProgress -> Completed/Failed

### Domain Patterns

```rust
// Pattern extraction
let patterns = memory.extract_patterns(episode_id).await?;
assert!(patterns.iter().all(|p| p.confidence > 0.0));

// Reward scoring
assert!(calculate_reward_score(1.0, 1.0) >= 0.9);

// Retrieval
let results = memory.retrieve_context("rust", Some(10)).await?;
assert!(results.iter().all(|m| m.context.language == "rust"));
```

## Async/Tokio Testing

### Basic Async

```rust
#[tokio::test]
async fn test_episode_creation() {
    let memory = SelfLearningMemory::new(Default::default()).await?;
    let id = memory.start_episode("Test", ctx, TaskType::CodeGen).await;
    assert!(!id.is_empty());
}
```

### Time Control

```rust
#[tokio::test(start_paused = true)]
async fn test_timeout() {
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(start.elapsed().as_millis() < 100); // No real wait
}
```

### Concurrent Ops

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent() {
    let handles: Vec<_> = (0..10).map(|i| tokio::spawn(async { i })).collect();
    assert_eq!(futures::future::join_all(handles).await.len(), 10);
}
```

### Mock Storage

```rust
pub struct MockTursoStorage { episodes: Arc<Mutex<Vec<Episode>>>, }
#[async_trait]
impl StorageBackend for MockTursoStorage {
    async fn create_episode(&self, episode: &Episode) -> Result<()> {
        self.episodes.lock().unwrap().push(episode.clone());
        Ok(())
    }
}
```

### Async Pitfalls

| Bad | Good |
|-----|------|
| `std::thread::sleep()` | `tokio::time::sleep().await` |
| Missing `#[tokio::test]` | Use async test attribute |
| Blocking runtime | `spawn_blocking` for sync |

## Pre-Commit Checklist

- [ ] Names follow `test_<function>_<scenario>_<expected>` pattern
- [ ] Clear AAA separation
- [ ] Single behavior per test
- [ ] Dependencies mocked
- [ ] Tests run in milliseconds
- [ ] Uses `#[tokio::test]` for async
- [ ] Uses `Result<()>` with `?`

## Best Practices

1. Quality over coverage - catch real bugs
2. AAA pattern for debuggability
3. Single responsibility - one failure reason
4. Isolate dependencies
5. `#[tokio::test]` for async, `spawn_blocking` for sync
6. Use `?` instead of `.unwrap()` chains

## Success Metrics

- Deploy without manual testing
- Failures pinpoint exact problems
- Refactoring doesn't break unrelated tests
- Tests run in milliseconds