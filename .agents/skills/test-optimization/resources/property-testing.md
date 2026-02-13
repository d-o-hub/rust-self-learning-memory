# Property Testing Strategies

## Basic Strategies

```rust
use proptest::prelude::*;

// String strategies
prop_compose! {
    fn non_empty_string()(len in 1..100) -> String {
        (0..len).map(|_| 'a').collect()
    }
}

// Numeric strategies
prop_compose! {
    fn positive_float()(value in 0.01f64..1000.0) -> f64 {
        value
    }
}

// Collection strategies
prop_compose! {
    fn episode_steps()(count in 0..20) -> Vec<Step> {
        (0..count).map(|i| Step {
            id: format!("step_{}", i),
            tool: format!("tool_{}", i),
            duration_ms: (i + 1) * 10,
        }).collect()
    }
}
```

## Custom Strategies

```rust
// Episode ID format validation
fn valid_episode_id() -> impl Strategy<Value = String> {
    "[a-z0-9-]{36}".prop_filter_map("Invalid UUID", |s| {
        uuid::Uuid::parse_str(&s).ok().map(|_| s)
    })
}

// Task type with valid context
fn task_with_context() -> impl Strategy<Value = (TaskType, TaskContext)> {
    (any::<TaskType>(), task_context_strategy())
}

fn task_context_strategy() -> impl Strategy<Value = TaskContext> {
    (any::<String>(), any::<String>()).prop_map(|(lang, project)| {
        TaskContext {
            language: lang,
            project: Some(project),
            ..Default::default()
        }
    })
}
```

## Test Configuration

```rust
// proptest.toml
[proptest]
max_shrink_iters = 1000
max_global_rejects = 10000
cases = 1000
verbose = 1

[proptest.shelf]
min_local_samples = 5
max_local_samples = 10
```

## Common Patterns

| Pattern | Strategy | Use Case |
|---------|----------|----------|
| `any::<T>()` | Built-in | Basic random values |
| `1..N` | Range | Bounded integers |
| `vec(any::<T>(), N)` | Collection | Lists of values |
| `prop_sample::subsequence()` | Sampling | Random subsets |
| `prop_filter()` | Filtering | Valid values only |
| `prop_map()` | Transformation | Value conversion |
