# Test Naming Conventions for Rust

## Standard Format

Use: `test_<function>_<scenario>_<expected_behavior>`

### Benefits
- Instant debugging context during failures
- Self-documenting test purpose
- No need to read implementation
- Team communication clarity

## Examples by Context

### Business Logic
```rust
#[test]
fn test_calculate_discount_new_customer_returns_zero()

#[test]
fn test_calculate_discount_loyal_customer_returns_10_percent()

#[test]
fn test_calculate_discount_vip_customer_returns_25_percent()
```

### Async Operations
```rust
#[tokio::test]
async fn test_start_episode_valid_task_creates_episode()

#[tokio::test]
async fn test_start_episode_empty_task_returns_error()

#[tokio::test]
async fn test_complete_episode_valid_id_updates_status()
```

### Error Cases
```rust
#[test]
fn test_parse_date_invalid_format_returns_error()

#[test]
fn test_withdraw_insufficient_funds_returns_error()

#[test]
fn test_get_episode_nonexistent_id_returns_none()
```

### State Changes
```rust
#[test]
fn test_withdraw_valid_amount_decreases_balance()

#[test]
fn test_deposit_valid_amount_increases_balance()

#[test]
fn test_add_step_valid_step_increments_count()
```

### Edge Cases
```rust
#[test]
fn test_parse_date_leap_year_february_29_succeeds()

#[test]
fn test_divide_by_zero_panics()

#[test]
fn test_empty_collection_returns_none()
```

## Anti-Patterns

### Too Vague
❌ `test_payment()`
❌ `test_validation()`
❌ `test_episode_service()`

### Implementation-Focused
❌ `test_process_payment_calls_gateway_twice()` (tests implementation detail)
✅ `test_process_payment_transient_failure_retries()` (tests behavior)

### Multiple Behaviors
❌ `test_process_order_creates_order_and_sends_email_and_updates_inventory()`
✅ Split into three tests:
- `test_process_order_valid_order_creates_order()`
- `test_process_order_success_sends_confirmation_email()`
- `test_process_order_completion_updates_inventory()`

## Rust Conventions

### Use Snake Case
```rust
✅ test_function_name_scenario_expected()
❌ testFunctionNameScenarioExpected()
❌ TestFunctionNameScenarioExpected()
```

### Async Tests
```rust
#[tokio::test]
async fn test_fetch_user_valid_id_returns_user()

#[tokio::test]
async fn test_save_episode_database_error_propagates()
```

### Result Return Type
```rust
#[test]
fn test_operation_valid_input_succeeds() -> anyhow::Result<()> {
    let result = operation()?;
    assert_eq!(result, expected);
    Ok(())
}
```

### Panic Tests
```rust
#[test]
#[should_panic(expected = "divide by zero")]
fn test_divide_zero_panics() {
    divide(10, 0);
}
```

## Project-Specific Patterns

### Episode Tests
```rust
#[test]
fn test_create_episode_valid_task_initializes_fields()

#[tokio::test]
async fn test_start_episode_with_context_stores_metadata()

#[tokio::test]
async fn test_complete_episode_success_verdict_updates_patterns()
```

### Pattern Tests
```rust
#[test]
fn test_extract_tool_sequence_multiple_steps_identifies_pattern()

#[test]
fn test_calculate_success_rate_mixed_results_returns_average()

#[test]
fn test_merge_patterns_similar_tools_combines_statistics()
```

### Storage Tests
```rust
#[tokio::test]
async fn test_save_episode_turso_stores_persistently()

#[tokio::test]
async fn test_cache_episode_redb_enables_fast_retrieval()

#[tokio::test]
async fn test_sync_memories_turso_to_redb_reconciles_state()
```

## When Scenario is Complex

For complex scenarios, keep naming clear and split if needed:

✅ `test_calculate_shipping_international_order_over_threshold_gets_free_shipping`
✅ `test_process_refund_partial_refund_within_window_updates_balance`

If name becomes unwieldy (>80 chars), split into multiple tests.

## Doc Comments for Complex Tests

When test name alone isn't enough:

```rust
/// Tests that concurrent episode creation doesn't cause ID collisions
///
/// Creates 100 episodes in parallel and verifies all have unique IDs
/// and are properly persisted to both Turso and redb.
#[tokio::test]
async fn test_concurrent_episode_creation_unique_ids() {
    // ...
}
```
