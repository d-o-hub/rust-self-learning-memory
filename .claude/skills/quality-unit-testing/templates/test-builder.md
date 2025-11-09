# Test Builder Pattern Template

Use this template to create test builders for complex domain objects:

## Basic Builder Structure

```rust
pub struct EpisodeBuilder {
    task_description: String,
    context: TaskContext,
    task_type: TaskType,
    steps: Vec<ExecutionStep>,
    completed: bool,
}

impl EpisodeBuilder {
    pub fn new(task_description: &str) -> Self {
        Self {
            task_description: task_description.to_string(),
            context: TaskContext::default(),
            task_type: TaskType::Testing,
            steps: Vec::new(),
            completed: false,
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

    pub fn completed(mut self, success: bool) -> Self {
        self.completed = true;
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
            let outcome = if success {
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                }
            } else {
                TaskOutcome::Failure {
                    reason: "Failed".to_string(),
                    error_details: None,
                }
            };
            episode.complete(outcome);
        }

        episode
    }
}
```

## Named Constructor Pattern

```rust
impl EpisodeBuilder {
    /// Creates a successful testing episode with default steps
    pub fn successful_test() -> Self {
        Self::new("Test task")
            .with_task_type(TaskType::Testing)
            .with_step(create_successful_step(1))
            .with_step(create_successful_step(2))
            .completed(true)
    }

    /// Creates a failed coding episode
    pub fn failed_coding() -> Self {
        Self::new("Code task")
            .with_task_type(TaskType::Coding)
            .with_step(create_successful_step(1))
            .with_step(create_failed_step(2))
            .completed(false)
    }
}
```

## Usage Examples

```rust
#[test]
fn test_simple_episode() {
    let episode = EpisodeBuilder::new("Test").build();
    assert_eq!(episode.task_description, "Test");
}

#[test]
fn test_episode_with_context() {
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "testing".to_string(),
        ..Default::default()
    };

    let episode = EpisodeBuilder::new("Test")
        .with_context(context)
        .build();

    assert_eq!(episode.context.language, Some("rust".to_string()));
}

#[test]
fn test_completed_episode() {
    let episode = EpisodeBuilder::new("Test")
        .with_step(create_test_step(1))
        .completed(true)
        .build();

    assert!(episode.is_complete());
}

#[test]
fn test_using_named_constructor() {
    let episode = EpisodeBuilder::successful_test().build();

    assert!(episode.is_complete());
    assert_eq!(episode.task_type, TaskType::Testing);
}
```

## Create Builder in test_utils

```rust
// In test-utils/src/lib.rs or test-utils/src/builders.rs

pub struct EpisodeBuilder {
    // ... fields ...
}

impl EpisodeBuilder {
    // ... methods ...
}

// Export for use in tests
pub use builders::EpisodeBuilder;
```

## Combining Multiple Builders

```rust
#[tokio::test]
async fn test_complex_scenario() {
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
                .successful()
                .build()
        )
        .with_step(
            StepBuilder::new(2)
                .with_tool("edit")
                .successful()
                .build()
        )
        .completed(true)
        .build();

    // Act
    memory.save_episode(&episode).await.unwrap();

    // Assert
    let retrieved = memory.get_episode(&episode.id).await.unwrap();
    assert_eq!(retrieved.steps.len(), 2);
}
```
