//! Error recovery pattern extractor
//!
//! Extracts patterns where errors are successfully recovered from.

use super::PatternExtractor as PatternExtractorTrait;
use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::{ExecutionResult, TaskOutcome};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Extracts error recovery patterns from episodes
pub struct ErrorRecoveryExtractor {
    confidence_threshold: f32,
}

impl Default for ErrorRecoveryExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryExtractor {
    /// Create new error recovery extractor
    #[must_use]
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
        }
    }

    /// Create with custom confidence threshold
    #[must_use]
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            confidence_threshold: threshold,
        }
    }

    /// Calculate success rate based on episode outcome
    fn calculate_success_rate(outcome: &Option<TaskOutcome>) -> f32 {
        match outcome {
            Some(TaskOutcome::Success { .. }) => 1.0,
            Some(TaskOutcome::PartialSuccess { .. }) => 0.5,
            _ => 0.0,
        }
    }
}

#[async_trait]
impl PatternExtractorTrait for ErrorRecoveryExtractor {
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Only extract from complete episodes
        if !episode.is_complete() {
            return Ok(patterns);
        }

        let mut recovery_sequences = Vec::new();

        // Look for error -> success patterns
        for i in 0..episode.steps.len().saturating_sub(1) {
            let current = &episode.steps[i];
            let next = &episode.steps[i + 1];

            // Found an error followed by success
            if !current.is_success() && next.is_success() {
                let error_type = if let Some(ExecutionResult::Error { message }) = &current.result {
                    message.clone()
                } else {
                    "Unknown error".to_string()
                };

                let recovery_step = format!("{}: {}", next.tool, next.action);
                recovery_sequences.push((error_type, recovery_step));
            }
        }

        if recovery_sequences.is_empty() {
            return Ok(patterns);
        }

        // Group by error type
        let mut error_recoveries: HashMap<String, Vec<String>> = HashMap::new();
        for (error_type, recovery) in recovery_sequences {
            error_recoveries
                .entry(error_type)
                .or_default()
                .push(recovery);
        }

        // Create pattern for each error type
        for (error_type, recovery_steps) in error_recoveries {
            let success_rate = Self::calculate_success_rate(&episode.outcome);

            // Only include if above threshold
            if success_rate >= self.confidence_threshold {
                patterns.push(Pattern::ErrorRecovery {
                    id: Uuid::new_v4(),
                    error_type,
                    recovery_steps,
                    success_rate,
                    context: episode.context.clone(),
                    effectiveness: crate::pattern::PatternEffectiveness::new(),
                });
            }
        }

        Ok(patterns)
    }

    fn name(&self) -> &'static str {
        "ErrorRecoveryExtractor"
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::patterns::extractors::tests::create_test_episode;
    use crate::types::{ExecutionResult, TaskOutcome};

    #[tokio::test]
    async fn test_extract_error_recovery() {
        let extractor = ErrorRecoveryExtractor::new();
        let mut episode = create_test_episode();

        // Add error step
        let mut error_step =
            ExecutionStep::new(1, "failing_tool".to_string(), "Try action".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Connection timeout".to_string(),
        });
        episode.add_step(error_step);

        // Add recovery step
        let mut recovery_step = ExecutionStep::new(
            2,
            "retry_tool".to_string(),
            "Retry with backoff".to_string(),
        );
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        episode.add_step(recovery_step);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::ErrorRecovery {
            error_type,
            recovery_steps,
            success_rate,
            ..
        } = &patterns[0]
        {
            assert!(error_type.contains("Connection timeout"));
            assert_eq!(recovery_steps.len(), 1);
            assert_eq!(*success_rate, 1.0);
        } else {
            panic!("Expected ErrorRecovery pattern");
        }
    }

    #[tokio::test]
    async fn test_no_recovery_on_failure() {
        let extractor = ErrorRecoveryExtractor::new();
        let mut episode = create_test_episode();

        // Add error step without recovery
        let mut error_step =
            ExecutionStep::new(1, "failing_tool".to_string(), "Try action".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Fatal error".to_string(),
        });
        episode.add_step(error_step);

        episode.complete(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });

        let patterns = extractor.extract(&episode).await.unwrap();
        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_recovery_attempts() {
        let extractor = ErrorRecoveryExtractor::new();
        let mut episode = create_test_episode();

        // Error 1 -> Recovery 1
        let mut error1 = ExecutionStep::new(1, "tool1".to_string(), "Action 1".to_string());
        error1.result = Some(ExecutionResult::Error {
            message: "Error type A".to_string(),
        });
        episode.add_step(error1);

        let mut recovery1 = ExecutionStep::new(2, "recover1".to_string(), "Recovery 1".to_string());
        recovery1.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(recovery1);

        // Error 2 -> Recovery 2 (same error type)
        let mut error2 = ExecutionStep::new(3, "tool2".to_string(), "Action 2".to_string());
        error2.result = Some(ExecutionResult::Error {
            message: "Error type A".to_string(),
        });
        episode.add_step(error2);

        let mut recovery2 = ExecutionStep::new(4, "recover2".to_string(), "Recovery 2".to_string());
        recovery2.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(recovery2);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered multiple times".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::ErrorRecovery { recovery_steps, .. } = &patterns[0] {
            assert_eq!(recovery_steps.len(), 2);
        }
    }
}
