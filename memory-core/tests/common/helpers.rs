//! Helper functions for common test operations

use memory_core::{
    memory::SelfLearningMemory, ComplexityLevel, ExecutionStep, MemoryConfig, TaskContext,
    TaskOutcome, TaskType,
};
use uuid::Uuid;

use super::fixtures::{ContextBuilder, StepBuilder};

/// Create a test memory instance with default configuration
///
/// Note: Uses a very low quality threshold (0.1) for testing to avoid
/// rejecting test episodes that are intentionally simple or minimal
/// (e.g., episodes with no steps to test edge cases).
///
/// # Examples
///
/// ```ignore
/// let memory = setup_test_memory();
/// ```
pub fn setup_test_memory() -> SelfLearningMemory {
    let config = MemoryConfig {
        quality_threshold: 0.1, // Very low threshold for test episodes (allows minimal episodes)
        ..Default::default()
    };
    SelfLearningMemory::with_config(config)
}

/// Create a simple test memory instance without advanced features
///
/// This disables spatiotemporal indexing and diversity maximization
/// for tests that need predictable, deterministic behavior (e.g., testing
/// exact retrieval limits with similar episodes).
///
/// # Examples
///
/// ```ignore
/// let memory = setup_simple_test_memory();
/// ```
pub fn setup_simple_test_memory() -> SelfLearningMemory {
    let config = MemoryConfig {
        quality_threshold: 0.1,                // Very low threshold for test episodes
        enable_diversity_maximization: false,  // Disable for predictable results
        enable_spatiotemporal_indexing: false, // Disable for exact limit control
        ..Default::default()
    };
    SelfLearningMemory::with_config(config)
}

/// Create a memory instance with custom configuration
///
/// # Examples
///
/// ```ignore
/// let config = MemoryConfig {
///     storage: StorageConfig {
///         max_episodes_cache: 500,
///         ..Default::default()
///     },
///     ..Default::default()
/// };
/// let memory = setup_memory_with_config(config);
/// ```
#[allow(dead_code)]
pub fn setup_memory_with_config(config: MemoryConfig) -> SelfLearningMemory {
    SelfLearningMemory::with_config(config)
}

/// Create a memory instance pre-populated with N episodes
///
/// # Arguments
///
/// * `n` - Number of episodes to create
///
/// # Examples
///
/// ```ignore
/// let memory = setup_memory_with_n_episodes(100).await;
/// ```
pub async fn setup_memory_with_n_episodes(n: usize) -> SelfLearningMemory {
    let memory = setup_test_memory();

    for i in 0..n {
        let context = ContextBuilder::new(format!("domain_{}", i % 10))
            .language("rust")
            .complexity(match i % 3 {
                0 => ComplexityLevel::Simple,
                1 => ComplexityLevel::Moderate,
                _ => ComplexityLevel::Complex,
            })
            .tag(format!("tag_{}", i % 5))
            .build();

        let episode_id = memory
            .start_episode(format!("Task {i}"), context, TaskType::CodeGeneration)
            .await;

        // Add 3-5 steps per episode
        let step_count = 3 + (i % 3);
        for j in 0..step_count {
            let step = StepBuilder::new(j + 1, format!("tool_{j}"), format!("Action {j}"))
                .latency_ms(10 + (j as u64 * 5))
                .tokens_used(50 + (j * 10))
                .success(format!("Step {j} done"))
                .build();
            memory.log_step(episode_id, step).await;
        }

        // Complete episode
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Task {i} completed"),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    memory
}

/// Create a `TaskContext` for testing
///
/// # Arguments
///
/// * `domain` - Domain string
/// * `language` - Optional language string
///
/// # Examples
///
/// ```ignore
/// let context = create_test_context("web-api", Some("rust"));
/// let context2 = create_test_context("cli-tool", None);
/// ```
pub fn create_test_context(domain: &str, language: Option<&str>) -> TaskContext {
    let mut builder = ContextBuilder::new(domain)
        .framework("tokio")
        .complexity(ComplexityLevel::Moderate)
        .tag("test");

    if let Some(lang) = language {
        builder = builder.language(lang);
    }

    builder.build()
}

/// Create a successful execution step
///
/// # Arguments
///
/// * `step_number` - Step number in sequence
/// * `tool` - Tool name
/// * `action` - Action description
///
/// # Examples
///
/// ```ignore
/// let step = create_success_step(1, "file_reader", "Read config");
/// ```
pub fn create_success_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    StepBuilder::new(step_number, tool, action)
        .latency_ms(100)
        .tokens_used(50)
        .success("OK")
        .build()
}

/// Create a failed execution step
///
/// # Arguments
///
/// * `step_number` - Step number in sequence
/// * `tool` - Tool name
/// * `action` - Action description
/// * `error_msg` - Error message
///
/// # Examples
///
/// ```ignore
/// let step = create_error_step(1, "connector", "Connect", "Timeout");
/// ```
pub fn create_error_step(
    step_number: usize,
    tool: &str,
    action: &str,
    error_msg: &str,
) -> ExecutionStep {
    StepBuilder::new(step_number, tool, action)
        .latency_ms(50)
        .error(error_msg)
        .build()
}

/// Create a generic test step (for backwards compatibility)
///
/// # Examples
///
/// ```ignore
/// let step = create_test_step(1);
/// ```
pub fn create_test_step(step_number: usize) -> ExecutionStep {
    StepBuilder::new(
        step_number,
        format!("test_tool_{step_number}"),
        format!("Test action {step_number}"),
    )
    .parameters(serde_json::json!({"param": "value"}))
    .latency_ms(10 + (step_number as u64 * 5))
    .tokens_used(50)
    .success(format!("Step {step_number} completed"))
    .build()
}

/// Create a completed episode with a clear error recovery pattern
///
/// # Examples
///
/// ```ignore
/// let episode_id = create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
/// ```
#[allow(dead_code)]
pub async fn create_completed_episode_with_pattern(
    memory: &SelfLearningMemory,
    pattern_type: PatternType,
) -> Uuid {
    match pattern_type {
        PatternType::ErrorRecovery => create_error_recovery_episode(memory).await,
        PatternType::ToolSequence => create_tool_sequence_episode(memory).await,
        PatternType::DecisionPoint => create_decision_point_episode(memory).await,
    }
}

/// Pattern type for test episode creation
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    ErrorRecovery,
    ToolSequence,
    DecisionPoint,
}

/// Create an episode with error recovery pattern
#[allow(dead_code)]
async fn create_error_recovery_episode(memory: &SelfLearningMemory) -> Uuid {
    let context = ContextBuilder::new("error-handling")
        .language("rust")
        .tag("retry")
        .tag("recovery")
        .build();

    let episode_id = memory
        .start_episode(
            "Implement retry logic".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    // Create error recovery pattern
    let error_step = StepBuilder::new(1, "initial_attempt", "Try operation")
        .error("Connection timeout")
        .build();
    memory.log_step(episode_id, error_step).await;

    let retry_step = StepBuilder::new(2, "retry_handler", "Retry with backoff")
        .success("Operation succeeded")
        .build();
    memory.log_step(episode_id, retry_step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Retry logic working".to_string(),
                artifacts: vec!["retry.rs".to_string()],
            },
        )
        .await
        .unwrap();

    episode_id
}

/// Create an episode with tool sequence pattern
#[allow(dead_code)]
async fn create_tool_sequence_episode(memory: &SelfLearningMemory) -> Uuid {
    let context = ContextBuilder::new("api-testing")
        .language("rust")
        .framework("tokio")
        .build();

    let episode_id = memory
        .start_episode(
            "Read and validate config".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    memory
        .log_step(
            episode_id,
            create_success_step(1, "file_reader", "Read config file"),
        )
        .await;
    memory
        .log_step(
            episode_id,
            create_success_step(2, "json_parser", "Parse JSON content"),
        )
        .await;
    memory
        .log_step(
            episode_id,
            create_success_step(3, "validator", "Validate config schema"),
        )
        .await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Config validated".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    episode_id
}

/// Create an episode with decision point pattern
#[allow(dead_code)]
async fn create_decision_point_episode(memory: &SelfLearningMemory) -> Uuid {
    let context = ContextBuilder::new("api-testing").language("rust").build();

    let episode_id = memory
        .start_episode("Check cache".to_string(), context, TaskType::CodeGeneration)
        .await;

    memory
        .log_step(
            episode_id,
            create_success_step(1, "cache_validator", "Check if cache is valid"),
        )
        .await;
    memory
        .log_step(
            episode_id,
            create_success_step(2, "cache_reader", "Read from cache"),
        )
        .await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Cache hit".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    episode_id
}

/// Create a test episode in a specific domain
///
/// # Examples
///
/// ```ignore
/// let episode_id = create_test_episode_with_domain(&memory, "web-api").await;
/// ```
pub async fn create_test_episode_with_domain(memory: &SelfLearningMemory, domain: &str) -> Uuid {
    let context = create_test_context(domain, Some("rust"));

    let episode_id = memory
        .start_episode(
            format!("Task in {domain}"),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    let step = create_test_step(1);
    memory.log_step(episode_id, step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    episode_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_memory() {
        let memory = setup_test_memory();
        let (total, _, _) = memory.get_stats().await;
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_setup_memory_with_n_episodes() {
        let memory = setup_memory_with_n_episodes(5).await;
        let (total, completed, _) = memory.get_stats().await;
        assert_eq!(total, 5);
        assert_eq!(completed, 5);
    }

    #[test]
    fn test_create_test_context() {
        let ctx = create_test_context("web-api", Some("rust"));
        assert_eq!(ctx.domain, "web-api");
        assert_eq!(ctx.language, Some("rust".to_string()));
    }

    #[test]
    fn test_create_success_step() {
        let step = create_success_step(1, "tool", "action");
        assert_eq!(step.step_number, 1);
        assert!(step.is_success());
    }

    #[test]
    fn test_create_error_step() {
        let step = create_error_step(1, "tool", "action", "error");
        assert_eq!(step.step_number, 1);
        assert!(!step.is_success());
    }

    #[tokio::test]
    async fn test_create_test_episode_with_domain() {
        let memory = setup_test_memory();
        let episode_id = create_test_episode_with_domain(&memory, "test-domain").await;
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.context.domain, "test-domain");
        assert!(episode.is_complete());
    }
}
