//! # Pattern Extractors
//!
//! Trait-based system for extracting patterns from episodes.
//! Each extractor specializes in a different pattern type.

use crate::episode::Episode;
use crate::pattern::Pattern;
use anyhow::Result;
use async_trait::async_trait;

mod clustering;
mod context_pattern;
mod decision_point;
mod error_recovery;
mod hybrid;
mod tool_sequence;

pub use clustering::{cluster_similar_patterns, deduplicate_patterns};
pub use context_pattern::ContextPatternExtractor;
pub use decision_point::DecisionPointExtractor;
pub use error_recovery::ErrorRecoveryExtractor;
pub use hybrid::HybridPatternExtractor;
pub use tool_sequence::ToolSequenceExtractor;

/// Base trait for all pattern extractors
#[async_trait]
pub trait PatternExtractor: Send + Sync {
    /// Extract patterns from an episode
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>>;

    /// Get the name of this extractor
    fn name(&self) -> &str;

    /// Get the confidence threshold for this extractor
    fn confidence_threshold(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

    pub(crate) fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec!["async".to_string()],
        };

        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    pub(crate) fn add_successful_steps(episode: &mut Episode, count: usize) {
        for i in 0..count {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i), format!("Action {}", i));
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            step.latency_ms = 100;
            episode.add_step(step);
        }
    }

    pub(crate) fn complete_episode_successfully(episode: &mut Episode) {
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
    }
}
