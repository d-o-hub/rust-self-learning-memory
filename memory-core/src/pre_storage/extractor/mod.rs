//! Salient feature extraction for episodic memory enhancement.
//!
//! Implements extraction of key information from episodes before storage,
//! identifying critical decisions, effective tool sequences, error recovery
//! patterns, and key insights.

mod decisions;
mod insights;
mod recovery;
mod tools;
mod types;

pub use types::SalientFeatures;

use crate::episode::Episode;

/// Extract salient features from episodes.
///
/// Analyzes episodes to identify and extract the most important information
/// before storage, enabling better retrieval and pattern learning.
///
/// # Examples
///
/// ```no_run
/// use memory_core::pre_storage::SalientExtractor;
/// use memory_core::{Episode, TaskContext, TaskType};
///
/// let extractor = SalientExtractor::new();
///
/// let episode = Episode::new(
///     "Implement authentication".to_string(),
///     TaskContext::default(),
///     TaskType::CodeGeneration,
/// );
///
/// let features = extractor.extract(&episode);
/// println!("Extracted {} salient features", features.count());
/// ```
#[derive(Debug, Clone)]
pub struct SalientExtractor {
    /// Minimum tool sequence length to be considered a combination
    min_tool_sequence_length: usize,
}

impl SalientExtractor {
    /// Create a new salient feature extractor with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientExtractor;
    ///
    /// let extractor = SalientExtractor::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_tool_sequence_length: 2,
        }
    }

    /// Extract salient features from an episode.
    ///
    /// Analyzes the episode to identify:
    /// - Critical decisions (branching points, important choices)
    /// - Effective tool sequences (2+ tools used in sequence)
    /// - Error recovery patterns (error -> recovery steps)
    /// - Key insights (from reflections, outcomes)
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to extract features from
    ///
    /// # Returns
    ///
    /// [`SalientFeatures`] containing all extracted information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::pre_storage::SalientExtractor;
    /// use memory_core::{Episode, TaskContext, TaskType, ExecutionStep, ExecutionResult};
    ///
    /// let extractor = SalientExtractor::new();
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// // Add some steps
    /// let mut step1 = ExecutionStep::new(1, "parser".to_string(), "Parse input".to_string());
    /// step1.result = Some(ExecutionResult::Success { output: "Parsed".to_string() });
    /// episode.add_step(step1);
    ///
    /// let features = extractor.extract(&episode);
    /// ```
    #[must_use]
    pub fn extract(&self, episode: &Episode) -> SalientFeatures {
        let mut features = SalientFeatures::new();

        // Extract critical decisions from steps and outcome
        features.critical_decisions = decisions::extract_critical_decisions(episode);

        // Extract effective tool combinations
        features.tool_combinations =
            tools::extract_tool_combinations(episode, self.min_tool_sequence_length);

        // Extract error recovery patterns
        features.error_recovery_patterns = recovery::extract_error_recovery_patterns(episode);

        // Extract key insights from reflections
        features.key_insights = insights::extract_key_insights(episode);

        features
    }
}

impl Default for SalientExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
