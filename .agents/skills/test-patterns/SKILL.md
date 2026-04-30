---
name: test-patterns
description: Unified testing patterns for Rust: unit testing quality, episodic memory operations, and async/tokio code. Use when writing tests, reviewing test code, or diagnosing test failures.
---

# Test Patterns for Rust Self-Learning Memory

Unified testing patterns covering unit testing quality, episodic memory domain tests, and async/tokio patterns.

## Naming Convention

`test_<function>_<scenario>_<expected_behavior>` (e.g., `test_withdraw_valid_amount_decreases_balance`)

## AAA Pattern

```rust
#[tokio::test]
async fn test_save_episode() -> anyhow::Result<()> {
    let memory = create_test_memory().await;    // Arrange
    memory.save_episode(&episode).await?;       // Act
    assert_eq!(retrieved.task_description, "Test");  // Assert
    Ok(())
}
```

## Core Principles

- Single behavior per test, one reason to fail
- Mock external deps (APIs, DBs, filesystem); don't mock value types
- Use `do-memory-test-utils` for setup

## Anti-Patterns

| Bad | Fix |
|-----|-----|
| Testing std library | Test your code |
| No assertions | Verify outcomes |
| Multiple behaviors | Split tests |
| `.unwrap()` chains | Use `Result<()>` with `?` |

## Episodic Memory Testing

State: Created -> InProgress -> Completed/Failed

```rust
#[tokio::test]
async fn test_episode_lifecycle() {
    let id = memory.start_episode("Test", ctx, TaskType::CodeGen).await;
    memory.complete_episode(id.clone(), TaskOutcome::Success, None).await?;
    assert_eq!(memory.get_episode(&id).await?.outcome, TaskOutcome::Success);
}
```

## Async/Tokio Patterns

```rust
#[tokio::test]                                       // Basic async
#[tokio::test(start_paused = true)]                  // Time control (no real wait)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]  // Concurrent
```

### Mock Storage

```rust
pub struct MockTursoStorage { episodes: Arc<Mutex<Vec<Episode>>> }
#[async_trait]
impl StorageBackend for MockTursoStorage {
    async fn create_episode(&self, episode: &Episode) -> Result<()> { /* ... */ }
}
```

### Async Pitfalls

| Bad | Good |
|-----|------|
| `std::thread::sleep()` | `tokio::time::sleep().await` |
| Blocking runtime | `spawn_blocking` for sync |

## Pre-Commit Checklist

- [ ] Names: `test_<function>_<scenario>_<expected>`, AAA separation, single behavior
- [ ] Dependencies mocked, `#[tokio::test]` for async, `Result<()>` with `?`

## Success Metrics

Deploy without manual testing | Failures pinpoint exact problems | Tests run in milliseconds
