# Basic Unit Test Template

Use this template for standard Rust unit tests:

## Simple Synchronous Test

```rust
#[test]
fn test_function_name_scenario_expected_behavior() {
    // Arrange
    let input = create_test_input();
    let expected = create_expected_output();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

## Test with Error Handling

```rust
#[test]
fn test_function_name_scenario_returns_error() -> anyhow::Result<()> {
    // Arrange
    let input = create_test_input();

    // Act
    let result = function_under_test(input)?;

    // Assert
    assert_eq!(result, expected);
    Ok(())
}
```

## Test with Panic Expectation

```rust
#[test]
#[should_panic(expected = "specific error message")]
fn test_function_name_invalid_input_panics() {
    // Arrange
    let invalid_input = create_invalid_input();

    // Act (will panic)
    function_under_test(invalid_input);
}
```

## Test with Multiple Assertions (Same Behavior)

```rust
#[test]
fn test_create_episode_initializes_all_fields() {
    // Arrange
    let task = "Test task";
    let context = TaskContext::default();
    let task_type = TaskType::Testing;

    // Act
    let episode = Episode::new(task.to_string(), context, task_type);

    // Assert - All verify initialization behavior
    assert_eq!(episode.task_description, task);
    assert!(episode.steps.is_empty());
    assert!(!episode.is_complete());
    assert!(episode.id.len() > 0);
}
```

## Test with Builder Pattern

```rust
#[test]
fn test_episode_with_steps_counts_correctly() {
    // Arrange
    let episode = EpisodeBuilder::new("Test")
        .with_step(create_test_step(1))
        .with_step(create_test_step(2))
        .build();

    // Act
    let count = episode.steps.len();

    // Assert
    assert_eq!(count, 2);
}
```

## Test with Test Utilities

```rust
use test_utils::*;

#[test]
fn test_completed_episode_has_verdict() {
    // Arrange
    let episode = create_completed_episode("Test task", true);

    // Act
    let verdict = episode.verdict;

    // Assert
    assert!(verdict.is_some());
    assert_eq!(verdict.unwrap(), Verdict::Success);
}
```

## Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_scenario_expected() {
        // ...
    }

    #[test]
    fn test_another_function_scenario_expected() {
        // ...
    }
}
```

## With Setup/Teardown

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestContext {
        TestContext {
            // ... common setup
        }
    }

    #[test]
    fn test_with_context() {
        // Arrange
        let ctx = setup();

        // Act
        let result = function_under_test(&ctx);

        // Assert
        assert!(result.is_ok());
    }
}
```
