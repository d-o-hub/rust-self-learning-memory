# Unit Testing Anti-Patterns (Rust)

Common mistakes to avoid when writing Rust tests.

## Testing Anti-Patterns

### 1. Testing Standard Library Code

❌ **Bad**: Testing Rust's std library
```rust
#[test]
fn test_vec_push_increases_length() {
    let mut vec = Vec::new();
    vec.push(1);
    assert_eq!(vec.len(), 1); // Testing std::Vec, not your code
}
```

✅ **Good**: Test your code's behavior
```rust
#[test]
fn test_add_step_valid_step_increments_count() {
    let mut episode = Episode::new("Test", TaskContext::default(), TaskType::Testing);
    episode.add_step(create_test_step(1));
    assert_eq!(episode.steps.len(), 1);
}
```

### 2. No Assertions (Execution-Only Tests)

❌ **Bad**: Just executing code without verification
```rust
#[tokio::test]
async fn test_start_episode() {
    let memory = create_test_memory().await;
    memory.start_episode("Test", TaskContext::default()).await.unwrap();
    // No assertions!
}
```

✅ **Good**: Verify behavior
```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode() {
    let memory = create_test_memory().await;
    let id = memory.start_episode("Test", TaskContext::default()).await.unwrap();

    assert!(!id.is_empty());
    let episode = memory.get_episode(&id).await.unwrap();
    assert_eq!(episode.task_description, "Test");
}
```

### 3. Multiple Unrelated Assertions

❌ **Bad**: Testing multiple behaviors
```rust
#[tokio::test]
async fn test_memory_operations() {
    let memory = create_test_memory().await;

    // Testing episode creation
    let id = memory.start_episode("Test", TaskContext::default()).await.unwrap();
    assert!(!id.is_empty());

    // Testing pattern extraction (unrelated!)
    let patterns = memory.extract_patterns().await.unwrap();
    assert!(!patterns.is_empty());

    // Testing retrieval (also unrelated!)
    let results = memory.retrieve_relevant_context("test", TaskContext::default(), 5).await.unwrap();
    assert!(results.len() > 0);
}
```

✅ **Good**: One behavior per test
```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode() {
    let memory = create_test_memory().await;
    let id = memory.start_episode("Test", TaskContext::default()).await.unwrap();
    assert!(!id.is_empty());
}

#[tokio::test]
async fn test_extract_patterns_completed_episodes_finds_patterns() {
    let memory = create_test_memory().await;
    // ... setup episodes
    let patterns = memory.extract_patterns().await.unwrap();
    assert!(!patterns.is_empty());
}
```

### 4. Testing Implementation Details

❌ **Bad**: Coupling tests to internal structure
```rust
#[test]
fn test_process_calls_private_helper() {
    let processor = Processor::new();
    // Testing that internal method is called
    // Breaks when refactoring
}
```

✅ **Good**: Test public behavior
```rust
#[test]
fn test_process_valid_input_returns_expected_output() {
    let processor = Processor::new();
    let result = processor.process("input");
    assert_eq!(result, "expected_output");
}
```

### 5. Using Real External Resources

❌ **Bad**: Tests depend on real database/network
```rust
#[tokio::test]
async fn test_save_episode() {
    let client = TursoClient::new(
        "https://production-db.turso.io", // ❌ Real DB!
        "real_token"
    ).await;

    client.save_episode(&episode).await.unwrap();
}
```

✅ **Good**: Use test environment
```rust
#[tokio::test]
async fn test_save_episode_valid_episode_persists() {
    let client = create_test_turso_client().await; // Test instance
    let episode = create_test_episode("Test");

    let result = client.save_episode(&episode).await;

    assert!(result.is_ok());
}
```

### 6. Flaky Tests with Timing Dependencies

❌ **Bad**: Tests with sleep/timing assumptions
```rust
#[tokio::test]
async fn test_async_process_completes() {
    processor.process_async(data).await;
    tokio::time::sleep(Duration::from_secs(1)).await; // ❌ Maybe not enough time
    assert!(processor.is_complete());
}
```

✅ **Good**: Deterministic async handling
```rust
#[tokio::test]
async fn test_async_process_completes_successfully() {
    let result = processor.process_async(data).await;
    assert!(result.is_ok());
    assert!(processor.is_complete());
}
```

### 7. Test-Order Dependencies

❌ **Bad**: Tests that must run in specific order
```rust
static mut GLOBAL_COUNTER: i32 = 0;

#[test]
fn test_1_increment() {
    unsafe {
        GLOBAL_COUNTER += 1;
        assert_eq!(GLOBAL_COUNTER, 1);
    }
}

#[test]
fn test_2_increment_again() {
    unsafe {
        GLOBAL_COUNTER += 1;
        assert_eq!(GLOBAL_COUNTER, 2); // ❌ Depends on test_1
    }
}
```

✅ **Good**: Independent tests
```rust
#[test]
fn test_counter_increment_from_zero_returns_one() {
    let mut counter = Counter::new();
    counter.increment();
    assert_eq!(counter.value(), 1);
}

#[test]
fn test_counter_increment_twice_returns_two() {
    let mut counter = Counter::new();
    counter.increment();
    counter.increment();
    assert_eq!(counter.value(), 2);
}
```

### 8. Shared Mutable State

❌ **Bad**: Tests share state
```rust
lazy_static! {
    static ref SHARED_DB: Mutex<Database> = Mutex::new(Database::new());
}

#[test]
fn test_1() {
    let mut db = SHARED_DB.lock().unwrap();
    db.insert("key", "value1");
}

#[test]
fn test_2() {
    let db = SHARED_DB.lock().unwrap();
    // ❌ May see data from test_1
    assert_eq!(db.get("key"), None);
}
```

✅ **Good**: Isolated state
```rust
#[test]
fn test_insert_new_key_stores_value() {
    let mut db = Database::new(); // Fresh state
    db.insert("key", "value");
    assert_eq!(db.get("key"), Some("value"));
}
```

### 9. Magic Numbers and Unclear Data

❌ **Bad**: Unclear test data
```rust
#[test]
fn test_discount() {
    let result = calculator.calculate(42, true, "ABC123");
    assert_eq!(result, 37.8); // What do these numbers mean?
}
```

✅ **Good**: Clear, meaningful data
```rust
#[test]
fn test_calculate_discount_vip_customer_returns_10_percent_off() {
    let order_amount = 100.0;
    let is_vip = true;
    let expected_discount = 10.0;

    let discount = calculator.calculate_discount(order_amount, is_vip);

    assert_eq!(discount, expected_discount);
}
```

### 10. Ignoring or Skipping Failing Tests

❌ **Bad**: Hiding problems
```rust
#[test]
#[ignore] // ❌ No reason given
fn test_broken_feature() {
    // ...
}
```

✅ **Good**: Document reason with issue number
```rust
#[test]
#[ignore = "Requires external Turso instance - Issue #123"]
fn test_production_sync() {
    // ...
}
```

### 11. Testing Trivial Code

❌ **Bad**: Testing getters/setters
```rust
#[test]
fn test_set_task_description() {
    let mut episode = Episode::new("Test", TaskContext::default(), TaskType::Testing);
    episode.task_description = "New task".to_string();
    assert_eq!(episode.task_description, "New task"); // Trivial
}
```

✅ **Good**: Test meaningful behavior
```rust
#[test]
fn test_complete_episode_success_verdict_sets_completion_fields() {
    let mut episode = Episode::new("Test", TaskContext::default(), TaskType::Testing);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    assert!(episode.is_complete());
    assert!(episode.completed_at.is_some());
    assert!(episode.reward_score.is_some());
}
```

### 12. Excessive `.unwrap()` Without Context

❌ **Bad**: Unclear failure points
```rust
#[tokio::test]
async fn test_episode_lifecycle() {
    let memory = create_test_memory().await.unwrap();
    let id = memory.start_episode("Test", TaskContext::default()).await.unwrap();
    memory.log_step(&id, create_test_step(1)).await.unwrap();
    memory.complete_episode(&id, outcome).await.unwrap();
    let episode = memory.get_episode(&id).await.unwrap();
    assert!(episode.is_complete());
}
```

✅ **Good**: Use `?` with Result return type
```rust
#[tokio::test]
async fn test_episode_lifecycle() -> anyhow::Result<()> {
    let memory = create_test_memory().await?;
    let id = memory.start_episode("Test", TaskContext::default()).await?;
    memory.log_step(&id, create_test_step(1)).await?;
    memory.complete_episode(&id, outcome).await?;

    let episode = memory.get_episode(&id).await?;
    assert!(episode.is_complete());
    Ok(())
}
```

## Rust-Specific Anti-Patterns

### 13. Not Using `#[tokio::test]` for Async Tests

❌ **Bad**: Won't compile
```rust
#[test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

✅ **Good**: Use proper async test attribute
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 14. Blocking the Tokio Runtime

❌ **Bad**: Blocks async runtime
```rust
#[tokio::test]
async fn test_with_blocking_operation() {
    std::thread::sleep(Duration::from_secs(1)); // ❌ Blocks!
    let result = async_function().await;
}
```

✅ **Good**: Use async sleep
```rust
#[tokio::test]
async fn test_with_async_delay() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = async_function().await;
}
```

### 15. Not Cleaning Up Resources

❌ **Bad**: Manual cleanup that might be skipped
```rust
#[tokio::test]
async fn test_with_temp_file() {
    let path = create_temp_file();
    // ... test code ...
    std::fs::remove_file(&path).unwrap(); // Might not run if test panics
}
```

✅ **Good**: Use RAII (Drop)
```rust
#[tokio::test]
async fn test_with_temp_file() {
    let _temp_dir = TempDir::new().unwrap();
    let path = _temp_dir.path().join("test.db");
    // ... test code ...
    // Cleanup happens automatically via Drop
}
```

## Quick Reference: Anti-Pattern Checklist

Before committing tests, check for these red flags:

❌ **Structure Issues**
- [ ] No clear AAA separation
- [ ] Multiple behaviors in one test
- [ ] Test names don't describe behavior

❌ **Isolation Issues**
- [ ] Uses real database/network/filesystem
- [ ] Tests depend on execution order
- [ ] Shared mutable state between tests

❌ **Reliability Issues**
- [ ] Contains `sleep()` or timing assumptions
- [ ] Flaky or intermittent failures
- [ ] Environment-dependent behavior

❌ **Value Issues**
- [ ] No assertions (execution-only)
- [ ] Tests std library code
- [ ] Tests trivial getters/setters
- [ ] High coverage, low confidence

❌ **Rust-Specific Issues**
- [ ] Missing `#[tokio::test]` for async tests
- [ ] Blocking the async runtime
- [ ] Excessive `.unwrap()` without context
- [ ] Manual cleanup instead of RAII
- [ ] Not using `?` operator with `Result<()>`

If you find any of these, refactor before proceeding!
