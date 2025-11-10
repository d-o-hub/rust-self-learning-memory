# AAA Pattern: Arrange-Act-Assert (Rust)

The AAA pattern structures tests for maximum clarity and debuggability.

## Structure

### Arrange
Set up the test context - create objects, prepare data, configure mocks.

### Act
Execute the specific behavior being tested.

### Assert
Verify the expected outcome.

## Basic Example

```rust
#[test]
fn test_withdraw_valid_amount_decreases_balance() {
    // Arrange
    let mut account = Account::new(100);
    let withdraw_amount = 30;

    // Act
    let result = account.withdraw(withdraw_amount);

    // Assert
    assert!(result.is_ok());
    assert_eq!(account.balance(), 70);
}
```

## Async Example

```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode() {
    // Arrange
    let memory = create_test_memory().await;
    let context = TaskContext::default();
    let task = "Test task";

    // Act
    let episode_id = memory.start_episode(task, context).await.unwrap();

    // Assert
    assert!(!episode_id.is_empty());
    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.task_description, task);
}
```

## With Result Return Type

```rust
#[tokio::test]
async fn test_save_episode_valid_data_persists() -> anyhow::Result<()> {
    // Arrange
    let memory = create_test_memory().await;
    let episode = create_test_episode("Test");

    // Act
    memory.save_episode(&episode).await?;

    // Assert
    let retrieved = memory.get_episode(&episode.id).await?;
    assert_eq!(retrieved.task_description, "Test");
    Ok(())
}
```

## Benefits

### Debugging Clarity
When a test fails, you immediately know which section broke:
- **Arrange failed**: Test setup issue (bad test data, missing mock)
- **Act failed**: Code under test broke
- **Assert failed**: Behavior doesn't match expectations

### Maintainability
Clear sections make tests easier to:
- Read and understand
- Modify when requirements change
- Debug when failures occur

### Review Quality
Code reviewers can quickly verify:
- Setup is appropriate
- Action is isolated
- Assertions verify the right things

## Common Mistakes

### No Clear Separation

❌ Bad:
```rust
#[test]
fn test_process_order() {
    let order = Order::new();
    let result = processor.process(order);
    assert!(result.success);
    let inventory = get_inventory();
    assert_eq!(inventory.count, 90);
}
```

✅ Good:
```rust
#[test]
fn test_process_order_valid_order_updates_inventory() {
    // Arrange
    let inventory = Inventory::new(100);
    let order = Order::new("Widget", 10);
    let processor = OrderProcessor::new(inventory);

    // Act
    let result = processor.process(&order);

    // Assert
    assert!(result.is_ok());
    assert_eq!(processor.inventory().count(), 90);
}
```

### Multiple Actions

❌ Bad:
```rust
#[tokio::test]
async fn test_user_workflow() {
    // Arrange
    let user = create_user();

    // Act
    let login_result = user.login().await;  // First action
    let order_result = user.place_order().await;  // Second action

    // Assert
    assert!(login_result.is_ok());
    assert!(order_result.is_ok());
}
```

✅ Good - Split into separate tests:
```rust
#[tokio::test]
async fn test_login_valid_credentials_succeeds() {
    // Arrange
    let user = create_user();

    // Act
    let result = user.login().await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_place_order_authenticated_user_creates_order() {
    // Arrange
    let user = create_authenticated_user().await;

    // Act
    let result = user.place_order().await;

    // Assert
    assert!(result.is_ok());
}
```

### Assertions in Arrange

❌ Bad:
```rust
#[test]
fn test_process_payment() {
    // Arrange
    let payment = create_payment();
    assert!(payment.is_some()); // ❌ Assertion in Arrange

    // Act
    let result = processor.process(payment.unwrap());

    // Assert
    assert!(result.success);
}
```

✅ Good:
```rust
#[test]
fn test_process_payment_valid_payment_succeeds() {
    // Arrange
    let payment = create_payment();

    // Act
    let result = processor.process(&payment);

    // Assert
    assert!(result.is_ok());
}
```

## Advanced Patterns

### Setup with Test Fixtures

```rust
struct TestContext {
    memory: SelfLearningMemory,
    _temp_dir: TempDir,
}

impl TestContext {
    async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.db");
        let memory = SelfLearningMemory::new("test_url", "token", path)
            .await
            .unwrap();

        Self {
            memory,
            _temp_dir: temp_dir,
        }
    }
}

#[tokio::test]
async fn test_episode_lifecycle() {
    // Arrange
    let ctx = TestContext::new().await;
    let task = "Test task";

    // Act
    let id = ctx.memory.start_episode(task, TaskContext::default())
        .await
        .unwrap();

    // Assert
    assert!(!id.is_empty());
}
```

### Exception/Error Testing

```rust
#[tokio::test]
async fn test_get_episode_invalid_id_returns_error() {
    // Arrange
    let memory = create_test_memory().await;

    // Act
    let result = memory.get_episode("invalid_id").await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::EpisodeNotFound(_)));
}
```

### Panic Testing

```rust
#[test]
#[should_panic(expected = "divide by zero")]
fn test_divide_zero_panics() {
    // Arrange
    let divisor = 0;

    // Act (will panic)
    divide(10, divisor);

    // No assert needed - we expect panic
}
```

### Complex Async Flows

```rust
#[tokio::test]
async fn test_concurrent_episode_updates_maintain_consistency() {
    // Arrange
    let memory = create_test_memory().await;
    let episode_id = memory.start_episode("Test", TaskContext::default())
        .await
        .unwrap();

    // Act - Multiple concurrent updates
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

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Assert
    let episode = memory.get_episode(&episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 10);
}
```

## Visual Separation

Use blank lines and comments to make AAA sections obvious:

```rust
#[tokio::test]
async fn test_retrieve_context_multiple_episodes_returns_relevant() {
    // Arrange
    let memory = create_test_memory().await;
    let episodes = create_test_episodes(5, "testing");
    for ep in &episodes {
        memory.save_episode(ep).await.unwrap();
    }

    // Act
    let results = memory
        .retrieve_relevant_context("testing", TaskContext::default(), 3)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.domain == "testing"));
}
```

## When AAA Isn't Strictly Needed

For very simple tests, strict AAA might be overkill:

```rust
#[test]
fn test_new_episode_has_empty_steps() {
    let episode = Episode::new("Test", TaskContext::default(), TaskType::Testing);
    assert!(episode.steps.is_empty());
}
```

But as tests grow, AAA becomes essential:

```rust
#[test]
fn test_episode_completion_calculates_reward() {
    // Arrange
    let mut episode = Episode::new("Test", TaskContext::default(), TaskType::Testing);
    episode.add_step(create_successful_step(1));
    episode.add_step(create_successful_step(2));

    // Act
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    // Assert
    assert!(episode.is_complete());
    assert!(episode.reward_score.is_some());
    assert!(episode.reward_score.unwrap().total > 0.0);
}
```
