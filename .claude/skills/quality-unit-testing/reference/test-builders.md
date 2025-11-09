# Test Builder Pattern for Rust

The Builder pattern creates complex test objects with readable, fluent APIs.

## Why Use Test Builders?

- **Readable**: Intent is clear from method names
- **Maintainable**: Adding fields doesn't break existing tests
- **Reusable**: Common scenarios become one-liners
- **Flexible**: Mix and match configurations easily

## Basic Builder

```rust
pub struct EpisodeBuilder {
    task_description: String,
    context: TaskContext,
    task_type: TaskType,
    steps: Vec<ExecutionStep>,
    completed: bool,
    verdict: Option<String>,
}

impl EpisodeBuilder {
    pub fn new(task_description: &str) -> Self {
        Self {
            task_description: task_description.to_string(),
            context: TaskContext::default(),
            task_type: TaskType::Testing,
            steps: Vec::new(),
            completed: false,
            verdict: None,
        }
    }

    pub fn with_context(mut self, context: TaskContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = task_type;
        self
    }

    pub fn with_step(mut self, step: ExecutionStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_steps(mut self, steps: Vec<ExecutionStep>) -> Self {
        self.steps = steps;
        self
    }

    pub fn completed(mut self, success: bool) -> Self {
        self.completed = true;
        self.verdict = Some(if success {
            "Success".to_string()
        } else {
            "Failed".to_string()
        });
        self
    }

    pub fn build(self) -> Episode {
        let mut episode = Episode::new(
            self.task_description,
            self.context,
            self.task_type,
        );

        for step in self.steps {
            episode.add_step(step);
        }

        if self.completed {
            let outcome = if self.verdict.as_deref() == Some("Success") {
                TaskOutcome::Success {
                    verdict: self.verdict.unwrap(),
                    artifacts: vec![],
                }
            } else {
                TaskOutcome::Failure {
                    reason: self.verdict.unwrap(),
                    error_details: None,
                }
            };
            episode.complete(outcome);
        }

        episode
    }
}
```

## Usage Examples

### Simple Episode
```rust
#[test]
fn test_simple_episode() {
    let episode = EpisodeBuilder::new("Test task").build();
    assert_eq!(episode.task_description, "Test task");
}
```

### Episode with Context
```rust
#[test]
fn test_episode_with_context() {
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "testing".to_string(),
        ..Default::default()
    };

    let episode = EpisodeBuilder::new("Test task")
        .with_context(context)
        .build();

    assert_eq!(episode.context.language, Some("rust".to_string()));
}
```

### Completed Episode
```rust
#[test]
fn test_completed_episode() {
    let episode = EpisodeBuilder::new("Test task")
        .with_step(create_test_step(1))
        .with_step(create_test_step(2))
        .completed(true)
        .build();

    assert!(episode.is_complete());
    assert_eq!(episode.steps.len(), 2);
}
```

### Complex Scenario
```rust
#[tokio::test]
async fn test_retrieve_context_filters_by_domain() {
    // Arrange
    let memory = create_test_memory().await;

    // Create episodes with different domains
    let testing_episode = EpisodeBuilder::new("Test task")
        .with_context(create_test_context("testing", Some("rust")))
        .with_step(create_test_step(1))
        .completed(true)
        .build();

    let coding_episode = EpisodeBuilder::new("Code task")
        .with_context(create_test_context("coding", Some("rust")))
        .with_step(create_test_step(1))
        .completed(true)
        .build();

    memory.save_episode(&testing_episode).await.unwrap();
    memory.save_episode(&coding_episode).await.unwrap();

    // Act
    let results = memory
        .retrieve_relevant_context(
            "testing",
            create_test_context("testing", Some("rust")),
            5
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].domain, "testing");
}
```

## Named Constructors Pattern

Create common scenarios as named methods:

```rust
impl EpisodeBuilder {
    pub fn successful_testing_episode() -> Self {
        Self::new("Test task")
            .with_task_type(TaskType::Testing)
            .with_step(create_successful_step(1))
            .with_step(create_successful_step(2))
            .completed(true)
    }

    pub fn failed_coding_episode() -> Self {
        Self::new("Code task")
            .with_task_type(TaskType::Coding)
            .with_step(create_successful_step(1))
            .with_step(create_failed_step(2))
            .completed(false)
    }

    pub fn async_episode_with_retries() -> Self {
        Self::new("Async task")
            .with_task_type(TaskType::AsyncOperation)
            .with_step(create_failed_step(1))
            .with_step(create_retry_step(2))
            .with_step(create_successful_step(3))
            .completed(true)
    }
}

// Usage
#[test]
fn test_successful_episode() {
    let episode = EpisodeBuilder::successful_testing_episode().build();
    assert!(episode.is_complete());
    assert_eq!(episode.verdict, Some(Verdict::Success));
}
```

## Builder for Test Context

```rust
pub struct ContextBuilder {
    language: Option<String>,
    framework: Option<String>,
    complexity: ComplexityLevel,
    domain: String,
    tags: Vec<String>,
}

impl ContextBuilder {
    pub fn new(domain: &str) -> Self {
        Self {
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: Vec::new(),
        }
    }

    pub fn rust(mut self) -> Self {
        self.language = Some("rust".to_string());
        self
    }

    pub fn with_tokio(mut self) -> Self {
        self.framework = Some("tokio".to_string());
        self
    }

    pub fn simple(mut self) -> Self {
        self.complexity = ComplexityLevel::Simple;
        self
    }

    pub fn complex(mut self) -> Self {
        self.complexity = ComplexityLevel::Complex;
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn build(self) -> TaskContext {
        TaskContext {
            language: self.language,
            framework: self.framework,
            complexity: self.complexity,
            domain: self.domain,
            tags: self.tags,
        }
    }
}

// Usage
#[test]
fn test_context_builder() {
    let context = ContextBuilder::new("testing")
        .rust()
        .with_tokio()
        .complex()
        .with_tag("async")
        .build();

    assert_eq!(context.language, Some("rust".to_string()));
    assert_eq!(context.framework, Some("tokio".to_string()));
}
```

## Builder for Execution Steps

```rust
pub struct StepBuilder {
    step_number: usize,
    tool: String,
    action: String,
    result: Option<ExecutionResult>,
    latency_ms: i64,
    tokens_used: Option<i32>,
}

impl StepBuilder {
    pub fn new(step_number: usize) -> Self {
        Self {
            step_number,
            tool: "test_tool".to_string(),
            action: "Test action".to_string(),
            result: None,
            latency_ms: 50,
            tokens_used: Some(100),
        }
    }

    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tool = tool.to_string();
        self
    }

    pub fn with_action(mut self, action: &str) -> Self {
        self.action = action.to_string();
        self
    }

    pub fn successful(mut self) -> Self {
        self.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        self
    }

    pub fn failed(mut self, error: &str) -> Self {
        self.result = Some(ExecutionResult::Error {
            message: error.to_string(),
        });
        self
    }

    pub fn with_latency(mut self, ms: i64) -> Self {
        self.latency_ms = ms;
        self
    }

    pub fn build(self) -> ExecutionStep {
        let mut step = ExecutionStep::new(
            self.step_number,
            self.tool,
            self.action,
        );
        step.result = self.result;
        step.latency_ms = self.latency_ms;
        step.tokens_used = self.tokens_used;
        step
    }
}

// Usage
#[test]
fn test_step_builder() {
    let step = StepBuilder::new(1)
        .with_tool("bash")
        .with_action("cargo test")
        .successful()
        .with_latency(150)
        .build();

    assert_eq!(step.tool, "bash");
    assert!(step.result.is_some());
}
```

## Combining Builders

```rust
#[tokio::test]
async fn test_episode_with_multiple_steps() {
    // Arrange
    let memory = create_test_memory().await;

    let episode = EpisodeBuilder::new("Complex task")
        .with_context(
            ContextBuilder::new("coding")
                .rust()
                .with_tokio()
                .build()
        )
        .with_step(
            StepBuilder::new(1)
                .with_tool("read")
                .with_action("Read file")
                .successful()
                .build()
        )
        .with_step(
            StepBuilder::new(2)
                .with_tool("edit")
                .with_action("Edit code")
                .successful()
                .build()
        )
        .with_step(
            StepBuilder::new(3)
                .with_tool("bash")
                .with_action("cargo test")
                .successful()
                .build()
        )
        .completed(true)
        .build();

    // Act
    memory.save_episode(&episode).await.unwrap();

    // Assert
    let retrieved = memory.get_episode(&episode.id).await.unwrap();
    assert_eq!(retrieved.steps.len(), 3);
}
```

## Best Practices

1. **Start simple** - Don't build every field initially
2. **Use sensible defaults** - Most tests need typical values
3. **Named constructors** - Create common scenarios
4. **Fluent API** - Return `self` from builder methods
5. **Don't overuse** - Simple objects don't need builders
6. **Document examples** - Show common usage patterns

## When to Use Builders

✅ Use builders when:
- Objects have many fields
- Most fields have defaults
- Creating multiple similar objects
- Need readable test setup

❌ Don't use builders for:
- Simple structs with 1-2 fields
- Objects created once
- When `Default` trait is sufficient
