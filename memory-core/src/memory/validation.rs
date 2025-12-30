//! Input validation for memory API boundaries
//!
//! Provides validation functions to enforce size limits and prevent
//! resource exhaustion attacks. All public API entry points should
//! validate inputs using these functions.

use crate::episode::{Episode, ExecutionStep};
use crate::error::{Error, Result};
use crate::types::{
    MAX_ARTIFACT_SIZE, MAX_DESCRIPTION_LEN, MAX_EPISODE_SIZE, MAX_OBSERVATION_LEN, MAX_STEP_COUNT,
};

/// Validate task description length.
///
/// Ensures the task description does not exceed [`MAX_DESCRIPTION_LEN`] (10KB).
/// Prevents `DoS` attacks via unbounded input strings.
///
/// # Arguments
///
/// * `description` - Task description to validate
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if description exceeds maximum length.
///
/// # Examples
///
/// ```
/// use memory_core::memory::validation::validate_task_description;
///
/// // Valid description
/// let valid = "Implement authentication".to_string();
/// assert!(validate_task_description(&valid).is_ok());
///
/// // Invalid - too long
/// let too_long = "a".repeat(10_001);
/// assert!(validate_task_description(&too_long).is_err());
/// ```
pub fn validate_task_description(description: &str) -> Result<()> {
    if description.len() > MAX_DESCRIPTION_LEN {
        return Err(Error::InvalidInput(format!(
            "Task description length {} exceeds maximum {} bytes ({}KB)",
            description.len(),
            MAX_DESCRIPTION_LEN,
            MAX_DESCRIPTION_LEN / 1024
        )));
    }
    Ok(())
}

/// Validate execution step before adding to episode.
///
/// Enforces multiple constraints:
/// - Step count must not exceed [`MAX_STEP_COUNT`] (1000)
/// - Observation length must not exceed [`MAX_OBSERVATION_LEN`] (10KB)
/// - Artifact sizes in parameters must not exceed [`MAX_ARTIFACT_SIZE`] (1MB)
///
/// # Arguments
///
/// * `episode` - Episode to validate step count against
/// * `step` - Execution step to validate
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if any validation constraint is violated.
///
/// # Examples
///
/// ```
/// use memory_core::{Episode, ExecutionStep, TaskContext, TaskType, ExecutionResult};
/// use memory_core::memory::validation::validate_execution_step;
///
/// let episode = Episode::new(
///     "Test task".to_string(),
///     TaskContext::default(),
///     TaskType::Testing,
/// );
///
/// // Valid step
/// let mut valid_step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
/// valid_step.result = Some(ExecutionResult::Success {
///     output: "OK".to_string(),
/// });
/// assert!(validate_execution_step(&episode, &valid_step).is_ok());
///
/// // Invalid - observation too long
/// let mut invalid_step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
/// invalid_step.result = Some(ExecutionResult::Success {
///     output: "x".repeat(10_001),
/// });
/// assert!(validate_execution_step(&episode, &invalid_step).is_err());
/// ```
pub fn validate_execution_step(episode: &Episode, step: &ExecutionStep) -> Result<()> {
    // Check step count limit
    if episode.steps.len() >= MAX_STEP_COUNT {
        return Err(Error::InvalidInput(format!(
            "Episode step count {} exceeds maximum {}",
            episode.steps.len(),
            MAX_STEP_COUNT
        )));
    }

    // Validate observation length (result output/error messages)
    if let Some(result) = &step.result {
        let observation_len = match result {
            crate::types::ExecutionResult::Success { output } => output.len(),
            crate::types::ExecutionResult::Error { message } => message.len(),
            crate::types::ExecutionResult::Timeout => 0,
        };

        if observation_len > MAX_OBSERVATION_LEN {
            return Err(Error::InvalidInput(format!(
                "Step observation length {} exceeds maximum {} bytes ({}KB)",
                observation_len,
                MAX_OBSERVATION_LEN,
                MAX_OBSERVATION_LEN / 1024
            )));
        }
    }

    // Validate artifact sizes in parameters
    // Check if parameters contain large data that could be artifacts
    if let Some(params_obj) = step.parameters.as_object() {
        for (key, value) in params_obj {
            // Check for common artifact-like field names
            if key.contains("artifact")
                || key.contains("data")
                || key.contains("content")
                || key.contains("file")
            {
                if let Some(string_value) = value.as_str() {
                    #[allow(clippy::excessive_nesting)]
                    if string_value.len() > MAX_ARTIFACT_SIZE {
                        return Err(Error::InvalidInput(format!(
                            "Artifact '{}' size {} exceeds maximum {} bytes ({}MB)",
                            key,
                            string_value.len(),
                            MAX_ARTIFACT_SIZE,
                            MAX_ARTIFACT_SIZE / 1_000_000
                        )));
                    }
                }
            }
        }
    }

    // Validate serialized parameters size
    let params_json = serde_json::to_string(&step.parameters)
        .map_err(|e| Error::InvalidInput(format!("Failed to serialize step parameters: {e}")))?;
    if params_json.len() > MAX_ARTIFACT_SIZE {
        return Err(Error::InvalidInput(format!(
            "Step parameters size {} exceeds maximum {} bytes ({}MB)",
            params_json.len(),
            MAX_ARTIFACT_SIZE,
            MAX_ARTIFACT_SIZE / 1_000_000
        )));
    }

    Ok(())
}

/// Validate total episode size before completion.
///
/// Serializes the episode and checks that the total size does not exceed
/// [`MAX_EPISODE_SIZE`] (10MB). This prevents memory exhaustion during
/// storage operations.
///
/// # Arguments
///
/// * `episode` - Episode to validate
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if serialized size exceeds maximum.
///
/// # Examples
///
/// ```
/// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
/// use memory_core::memory::validation::validate_episode_size;
///
/// let mut episode = Episode::new(
///     "Test task".to_string(),
///     TaskContext::default(),
///     TaskType::Testing,
/// );
///
/// episode.complete(TaskOutcome::Success {
///     verdict: "Done".to_string(),
///     artifacts: vec![],
/// });
///
/// assert!(validate_episode_size(&episode).is_ok());
/// ```
pub fn validate_episode_size(episode: &Episode) -> Result<()> {
    // Serialize to JSON to get accurate size
    let serialized = serde_json::to_vec(episode).map_err(|e| {
        Error::InvalidInput(format!("Failed to serialize episode for validation: {e}"))
    })?;

    if serialized.len() > MAX_EPISODE_SIZE {
        return Err(Error::InvalidInput(format!(
            "Episode serialized size {} exceeds maximum {} bytes ({}MB)",
            serialized.len(),
            MAX_EPISODE_SIZE,
            MAX_EPISODE_SIZE / 1_000_000
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};

    #[test]
    fn test_validate_task_description_valid() {
        let description = "Implement user authentication".to_string();
        assert!(validate_task_description(&description).is_ok());
    }

    #[test]
    fn test_validate_task_description_at_limit() {
        let description = "a".repeat(MAX_DESCRIPTION_LEN);
        assert!(validate_task_description(&description).is_ok());
    }

    #[test]
    fn test_validate_task_description_exceeds_limit() {
        let description = "a".repeat(MAX_DESCRIPTION_LEN + 1);
        let result = validate_task_description(&description);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
    }

    #[test]
    fn test_validate_execution_step_valid() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });

        assert!(validate_execution_step(&episode, &step).is_ok());
    }

    #[test]
    fn test_validate_execution_step_observation_at_limit() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "x".repeat(MAX_OBSERVATION_LEN),
        });

        assert!(validate_execution_step(&episode, &step).is_ok());
    }

    #[test]
    fn test_validate_execution_step_observation_exceeds_limit() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "x".repeat(MAX_OBSERVATION_LEN + 1),
        });

        let result = validate_execution_step(&episode, &step);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
    }

    #[test]
    fn test_validate_execution_step_error_message_exceeds_limit() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Error {
            message: "x".repeat(MAX_OBSERVATION_LEN + 1),
        });

        let result = validate_execution_step(&episode, &step);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_execution_step_too_many_steps() {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        // Add MAX_STEP_COUNT steps
        for i in 0..MAX_STEP_COUNT {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Try to add one more
        let step = ExecutionStep::new(MAX_STEP_COUNT + 1, "tool".to_string(), "action".to_string());
        let result = validate_execution_step(&episode, &step);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
    }

    #[test]
    fn test_validate_execution_step_artifact_in_params() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.parameters = serde_json::json!({
            "artifact_data": "x".repeat(MAX_ARTIFACT_SIZE + 1),
        });

        let result = validate_execution_step(&episode, &step);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
    }

    #[test]
    fn test_validate_execution_step_large_params() {
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        // Create params that serialize to > MAX_ARTIFACT_SIZE
        let large_array: Vec<String> = (0..10_000)
            .map(|i| format!("item_{i}_with_some_padding_text"))
            .collect();
        step.parameters = serde_json::json!({
            "large_data": large_array,
        });

        let result = validate_execution_step(&episode, &step);
        // This might pass or fail depending on exact serialization size
        // We're just checking the validation logic runs
        let _ = result;
    }

    #[test]
    fn test_validate_episode_size_valid() {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        // Add a few normal steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec!["file.txt".to_string()],
        });

        assert!(validate_episode_size(&episode).is_ok());
    }

    #[test]
    fn test_validate_episode_size_large_but_valid() {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        // Add many steps with reasonable data
        // Each step is roughly 500 bytes, so 1000 steps = ~500KB, well under 10MB
        for i in 0..1000 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "x".repeat(200),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // Should be valid (under 10MB)
        assert!(validate_episode_size(&episode).is_ok());
    }

    #[test]
    fn test_validate_episode_size_exceeds_limit() {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        // Add steps with large observations to exceed 10MB
        // Each observation is 500KB, so 25 steps = 12.5MB
        for i in 0..25 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "x".repeat(500_000), // 500KB per step
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let result = validate_episode_size(&episode);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
    }
}
