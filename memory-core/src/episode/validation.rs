//! Episode validation logic.
//!
//! Contains validation methods for Episode and `ExecutionStep`.

use super::structs::{Episode, ExecutionStep};
use crate::types::ExecutionResult;

impl ExecutionStep {
    /// Validate that the step has required fields set.
    pub fn validate(&self) -> Result<(), String> {
        if self.step_number == 0 {
            return Err("Step number must be greater than 0".to_string());
        }
        if self.tool.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        if self.action.is_empty() {
            return Err("Action description cannot be empty".to_string());
        }
        Ok(())
    }

    /// Validate that the step has a valid result if present.
    pub fn validate_result(&self) -> Result<(), String> {
        if let Some(ref result) = self.result {
            match result {
                ExecutionResult::Success { output } if output.is_empty() => {
                    Err("Success result must have non-empty output".to_string())
                }
                ExecutionResult::Error { message } if message.is_empty() => {
                    Err("Error result must have non-empty message".to_string())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl Episode {
    /// Validate that the episode has required fields.
    pub fn validate(&self) -> Result<(), String> {
        if self.task_description.is_empty() {
            return Err("Task description cannot be empty".to_string());
        }
        if self.context.domain.is_empty() {
            return Err("Task domain cannot be empty".to_string());
        }
        Ok(())
    }

    /// Validate that the episode is ready for completion.
    pub fn validate_for_completion(&self) -> Result<(), String> {
        self.validate()?;

        if self.steps.is_empty() {
            return Err("Episode must have at least one step before completion".to_string());
        }

        for step in &self.steps {
            step.validate()?;
        }

        Ok(())
    }

    /// Get a summary of the episode for logging purposes.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Episode {}: {} ({} steps, {})",
            self.episode_id,
            self.task_description,
            self.steps.len(),
            match &self.outcome {
                Some(o) => o.to_string(),
                None => "in progress".to_string(),
            }
        )
    }
}
