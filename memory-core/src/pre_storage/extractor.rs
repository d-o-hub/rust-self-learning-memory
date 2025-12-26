//! Salient feature extraction for episodic memory enhancement.
//!
//! Implements extraction of key information from episodes before storage,
//! identifying critical decisions, effective tool sequences, error recovery
//! patterns, and key insights.

use crate::episode::{Episode, ExecutionStep};
use crate::types::ExecutionResult;
use serde::{Deserialize, Serialize};

/// Salient features extracted from an episode.
///
/// Contains the most important and reusable information from an episode,
/// enabling more effective retrieval and pattern learning.
///
/// # Fields
///
/// * `critical_decisions` - Key decision points and branching logic
/// * `tool_combinations` - Effective sequences of tools used together
/// * `error_recovery_patterns` - How errors were detected and resolved
/// * `key_insights` - Important discoveries and learnings
///
/// # Examples
///
/// ```
/// use memory_core::pre_storage::SalientFeatures;
///
/// let features = SalientFeatures {
///     critical_decisions: vec![
///         "Chose async implementation for better performance".to_string(),
///     ],
///     tool_combinations: vec![
///         vec!["parser".to_string(), "validator".to_string(), "generator".to_string()],
///     ],
///     error_recovery_patterns: vec![
///         "Connection timeout -> retry with exponential backoff".to_string(),
///     ],
///     key_insights: vec![
///         "Builder pattern simplifies configuration".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalientFeatures {
    /// Critical decision points identified in the episode
    pub critical_decisions: Vec<String>,
    /// Effective tool sequences (2+ tools used together)
    pub tool_combinations: Vec<Vec<String>>,
    /// Error recovery patterns (error -> recovery steps)
    pub error_recovery_patterns: Vec<String>,
    /// Key insights from reflections and outcomes
    pub key_insights: Vec<String>,
}

impl SalientFeatures {
    /// Create a new empty `SalientFeatures`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let features = SalientFeatures::new();
    /// assert!(features.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            critical_decisions: Vec::new(),
            tool_combinations: Vec::new(),
            error_recovery_patterns: Vec::new(),
            key_insights: Vec::new(),
        }
    }

    /// Check if the features are empty (no salient information extracted).
    ///
    /// # Returns
    ///
    /// `true` if all feature vectors are empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let empty = SalientFeatures::new();
    /// assert!(empty.is_empty());
    ///
    /// let mut features = SalientFeatures::new();
    /// features.key_insights.push("Important insight".to_string());
    /// assert!(!features.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.critical_decisions.is_empty()
            && self.tool_combinations.is_empty()
            && self.error_recovery_patterns.is_empty()
            && self.key_insights.is_empty()
    }

    /// Count total number of extracted features.
    ///
    /// # Returns
    ///
    /// Total count across all feature categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let mut features = SalientFeatures::new();
    /// features.critical_decisions.push("Decision 1".to_string());
    /// features.key_insights.push("Insight 1".to_string());
    /// features.key_insights.push("Insight 2".to_string());
    ///
    /// assert_eq!(features.count(), 3);
    /// ```
    #[must_use]
    pub fn count(&self) -> usize {
        self.critical_decisions.len()
            + self.tool_combinations.len()
            + self.error_recovery_patterns.len()
            + self.key_insights.len()
    }
}

impl Default for SalientFeatures {
    fn default() -> Self {
        Self::new()
    }
}

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
        features.critical_decisions = self.extract_critical_decisions(episode);

        // Extract effective tool combinations
        features.tool_combinations = self.extract_tool_combinations(episode);

        // Extract error recovery patterns
        features.error_recovery_patterns = self.extract_error_recovery_patterns(episode);

        // Extract key insights from reflections
        features.key_insights = self.extract_key_insights(episode);

        features
    }

    /// Extract parameter decisions from a JSON object.
    ///
    /// Helper method to extract strategic parameter choices.
    fn extract_parameter_decisions(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
        step_number: usize,
        decisions: &mut Vec<String>,
    ) {
        if obj.is_empty() {
            return;
        }

        // Look for strategy, approach, method parameters
        for key in obj.keys() {
            let key_lower = key.to_lowercase();
            if key_lower.contains("strategy")
                || key_lower.contains("approach")
                || key_lower.contains("method")
                || key_lower.contains("algorithm")
            {
                if let Some(value) = obj.get(key) {
                    decisions.push(format!(
                        "Chose {key} = {} (step {step_number})",
                        value.to_string().trim_matches('"'),
                    ));
                }
            }
        }
    }

    /// Extract critical decisions from episode steps and outcome.
    ///
    /// Identifies important decision points such as:
    /// - Choice of tools or approaches
    /// - Branching logic (if different paths were taken)
    /// - Significant parameter choices
    /// - Final outcome decisions
    fn extract_critical_decisions(&self, episode: &Episode) -> Vec<String> {
        let mut decisions = Vec::new();

        // Look for steps with significant parameter choices
        for step in &episode.steps {
            // Check for decision-indicating keywords in actions
            let action_lower = step.action.to_lowercase();
            if action_lower.contains("choose")
                || action_lower.contains("decide")
                || action_lower.contains("select")
                || action_lower.contains("opt for")
            {
                decisions.push(format!(
                    "Step {}: {} using {}",
                    step.step_number, step.action, step.tool
                ));
            }

            // Check for complex parameters that indicate choices
            if let Some(obj) = step.parameters.as_object() {
                self.extract_parameter_decisions(obj, step.step_number, &mut decisions);
            }
        }

        // Extract decision from outcome
        if let Some(ref outcome) = episode.outcome {
            match outcome {
                crate::types::TaskOutcome::Success { verdict, .. } => {
                    if verdict.len() > 10 {
                        // Meaningful verdict
                        decisions.push(format!("Outcome: {verdict}"));
                    }
                }
                crate::types::TaskOutcome::PartialSuccess { verdict, .. } => {
                    decisions.push(format!("Partial success: {verdict}"));
                }
                crate::types::TaskOutcome::Failure { reason, .. } => {
                    decisions.push(format!("Failure reason: {reason}"));
                }
            }
        }

        // Limit to most relevant decisions
        decisions.truncate(10);
        decisions
    }

    /// Extract effective tool combinations from sequential steps.
    ///
    /// Identifies sequences of 2+ tools used together successfully,
    /// which represent reusable patterns.
    fn extract_tool_combinations(&self, episode: &Episode) -> Vec<Vec<String>> {
        let mut combinations = Vec::new();

        if episode.steps.len() < self.min_tool_sequence_length {
            return combinations;
        }

        // Find sequences of successful steps
        let mut current_sequence = Vec::new();

        for step in &episode.steps {
            if step.is_success() {
                current_sequence.push(step.tool.clone());
            } else {
                // Sequence broken by failure
                if current_sequence.len() >= self.min_tool_sequence_length {
                    combinations.push(current_sequence.clone());
                }
                current_sequence.clear();
            }
        }

        // Add final sequence if long enough
        if current_sequence.len() >= self.min_tool_sequence_length {
            combinations.push(current_sequence);
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        combinations.retain(|combo| {
            let key = combo.join("->");
            seen.insert(key)
        });

        // Limit to most relevant combinations
        combinations.truncate(5);
        combinations
    }

    /// Extract multi-step error recovery patterns.
    ///
    /// Helper method to find longer recovery sequences.
    fn extract_multi_step_recovery(&self, episode: &Episode, patterns: &mut Vec<String>) {
        let mut i = 0;
        while i < episode.steps.len() {
            let step_failed = !episode.steps[i].is_success();
            if step_failed {
                let error_step = &episode.steps[i];
                let error_msg = self.get_error_message(error_step);

                // Count successful recovery steps
                let mut recovery_steps = Vec::new();
                let mut j = i + 1;
                while j < episode.steps.len()
                    && episode.steps[j].is_success()
                    && recovery_steps.len() < 3
                {
                    recovery_steps.push(&episode.steps[j]);
                    j += 1;
                }

                if recovery_steps.len() >= 2 {
                    let recovery_desc = recovery_steps
                        .iter()
                        .map(|s| s.action.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    patterns.push(format!("{error_msg} -> [{recovery_desc}]"));
                }

                i = j;
            } else {
                i += 1;
            }
        }
    }

    /// Extract error recovery patterns from failed steps followed by successes.
    ///
    /// Identifies how errors were detected and resolved, creating
    /// reusable recovery strategies.
    fn extract_error_recovery_patterns(&self, episode: &Episode) -> Vec<String> {
        let mut patterns = Vec::new();

        // Look for error -> success transitions
        for window in episode.steps.windows(2) {
            let error_step = &window[0];
            let recovery_step = &window[1];

            if !error_step.is_success() && recovery_step.is_success() {
                let error_msg = self.get_error_message(error_step);
                patterns.push(format!(
                    "{} -> {} ({})",
                    error_msg, recovery_step.action, recovery_step.tool
                ));
            }
        }

        // Look for longer recovery sequences (error -> multiple recovery steps)
        self.extract_multi_step_recovery(episode, &mut patterns);

        // Deduplicate patterns
        let mut seen = std::collections::HashSet::new();
        patterns.retain(|p| seen.insert(p.clone()));

        // Limit to most relevant patterns
        patterns.truncate(10);
        patterns
    }

    /// Extract key insights from reflection and outcome.
    ///
    /// Pulls out important learnings and discoveries that can be
    /// applied to future tasks.
    fn extract_key_insights(&self, episode: &Episode) -> Vec<String> {
        let mut insights = Vec::new();

        // Extract from reflection
        if let Some(ref reflection) = episode.reflection {
            // Add notable successes (limit to most important)
            for success in reflection.successes.iter().take(3) {
                if success.len() > 10 {
                    // Filter out trivial successes
                    insights.push(format!("Success: {success}"));
                }
            }

            // Add all insights from reflection (these are already curated)
            for insight in &reflection.insights {
                insights.push(format!("Insight: {insight}"));
            }

            // Add key improvements (limit to most important)
            for improvement in reflection.improvements.iter().take(2) {
                if improvement.len() > 10 {
                    insights.push(format!("Improvement: {improvement}"));
                }
            }
        }

        // Extract from outcome artifacts (if meaningful)
        if let Some(crate::types::TaskOutcome::Success { artifacts, .. }) = &episode.outcome {
            if !artifacts.is_empty() && artifacts.len() <= 5 {
                // Only include if reasonable number of artifacts
                insights.push(format!("Artifacts produced: {}", artifacts.join(", ")));
            }
        }

        // Deduplicate insights
        let mut seen = std::collections::HashSet::new();
        insights.retain(|i| seen.insert(i.clone()));

        // Limit to most relevant insights
        insights.truncate(15);
        insights
    }

    /// Helper to extract error message from a failed step.
    fn get_error_message(&self, step: &ExecutionStep) -> String {
        match &step.result {
            Some(ExecutionResult::Error { message }) => {
                // Truncate long error messages
                if message.len() > 50 {
                    format!("{}...", &message[..50])
                } else {
                    message.clone()
                }
            }
            Some(ExecutionResult::Timeout) => "Timeout".to_string(),
            _ => format!("Error in {}", step.action),
        }
    }
}

impl Default for SalientExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ExecutionResult, Reflection, TaskContext, TaskOutcome, TaskType};
    use chrono::Utc;

    fn create_test_episode() -> Episode {
        Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
    }

    #[test]
    fn test_salient_features_new() {
        let features = SalientFeatures::new();
        assert!(features.is_empty());
        assert_eq!(features.count(), 0);
    }

    #[test]
    fn test_salient_features_count() {
        let mut features = SalientFeatures::new();
        assert_eq!(features.count(), 0);

        features.critical_decisions.push("Decision 1".to_string());
        assert_eq!(features.count(), 1);

        features
            .tool_combinations
            .push(vec!["tool1".to_string(), "tool2".to_string()]);
        assert_eq!(features.count(), 2);

        features.key_insights.push("Insight 1".to_string());
        features.key_insights.push("Insight 2".to_string());
        assert_eq!(features.count(), 4);
    }

    #[test]
    fn test_extract_empty_episode() {
        let extractor = SalientExtractor::new();
        let episode = create_test_episode();
        let features = extractor.extract(&episode);

        assert!(features.is_empty());
    }

    #[test]
    fn test_extract_critical_decisions() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add step with decision keyword
        let mut step1 = ExecutionStep::new(
            1,
            "planner".to_string(),
            "Choose async implementation strategy".to_string(),
        );
        step1.result = Some(ExecutionResult::Success {
            output: "Strategy selected".to_string(),
        });
        episode.add_step(step1);

        // Add step with strategy parameter
        let mut step2 = ExecutionStep::new(
            2,
            "executor".to_string(),
            "Execute with chosen strategy".to_string(),
        );
        step2.parameters = serde_json::json!({
            "strategy": "async",
            "approach": "tokio"
        });
        step2.result = Some(ExecutionResult::Success {
            output: "Executed".to_string(),
        });
        episode.add_step(step2);

        episode.complete(TaskOutcome::Success {
            verdict: "Successfully implemented async solution".to_string(),
            artifacts: vec![],
        });

        let features = extractor.extract(&episode);
        assert!(!features.critical_decisions.is_empty());
        assert!(features
            .critical_decisions
            .iter()
            .any(|d| d.contains("Choose async")));
        assert!(features
            .critical_decisions
            .iter()
            .any(|d| d.contains("strategy")));
    }

    #[test]
    fn test_extract_tool_combinations() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add successful sequence
        for i in 0..4 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i), format!("action_{}", i));
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        let features = extractor.extract(&episode);
        assert!(!features.tool_combinations.is_empty());
        assert_eq!(features.tool_combinations[0].len(), 4);
        assert_eq!(features.tool_combinations[0][0], "tool_0");
        assert_eq!(features.tool_combinations[0][3], "tool_3");
    }

    #[test]
    fn test_extract_tool_combinations_with_failures() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add success sequence
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Add failure
        let mut fail_step = ExecutionStep::new(4, "tool_3".to_string(), "action".to_string());
        fail_step.result = Some(ExecutionResult::Error {
            message: "Error".to_string(),
        });
        episode.add_step(fail_step);

        // Add another success sequence
        for i in 4..6 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        let features = extractor.extract(&episode);
        assert_eq!(features.tool_combinations.len(), 2);
        assert_eq!(features.tool_combinations[0].len(), 3); // First sequence
        assert_eq!(features.tool_combinations[1].len(), 2); // Second sequence
    }

    #[test]
    fn test_extract_error_recovery_patterns() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add error step
        let mut error_step = ExecutionStep::new(
            1,
            "connector".to_string(),
            "Connect to database".to_string(),
        );
        error_step.result = Some(ExecutionResult::Error {
            message: "Connection timeout".to_string(),
        });
        episode.add_step(error_step);

        // Add recovery step
        let mut recovery_step =
            ExecutionStep::new(2, "connector".to_string(), "Retry with backoff".to_string());
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Connected".to_string(),
        });
        episode.add_step(recovery_step);

        let features = extractor.extract(&episode);
        assert!(!features.error_recovery_patterns.is_empty());
        assert!(features.error_recovery_patterns[0].contains("Connection timeout"));
        assert!(features.error_recovery_patterns[0].contains("Retry with backoff"));
    }

    #[test]
    fn test_extract_multi_step_error_recovery() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add error
        let mut error_step = ExecutionStep::new(1, "parser".to_string(), "Parse input".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Invalid format".to_string(),
        });
        episode.add_step(error_step);

        // Add multiple recovery steps
        let recovery_actions = vec!["Sanitize input", "Validate format", "Re-parse"];
        for (i, action) in recovery_actions.iter().enumerate() {
            let mut step = ExecutionStep::new(i + 2, "parser".to_string(), action.to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        let features = extractor.extract(&episode);
        assert!(!features.error_recovery_patterns.is_empty());
        // Should capture multi-step recovery
        assert!(features
            .error_recovery_patterns
            .iter()
            .any(|p| p.contains("[")));
    }

    #[test]
    fn test_extract_key_insights_from_reflection() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        episode.reflection = Some(Reflection {
            successes: vec![
                "Efficient error handling pattern".to_string(),
                "Good test coverage".to_string(),
            ],
            improvements: vec!["Could reduce duplication".to_string()],
            insights: vec![
                "Builder pattern works well".to_string(),
                "Async improves performance".to_string(),
            ],
            generated_at: Utc::now(),
        });

        let features = extractor.extract(&episode);
        assert!(!features.key_insights.is_empty());
        assert!(features
            .key_insights
            .iter()
            .any(|i| i.contains("Builder pattern")));
        assert!(features
            .key_insights
            .iter()
            .any(|i| i.contains("Async improves")));
    }

    #[test]
    fn test_extract_key_insights_from_outcome() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        episode.complete(TaskOutcome::Success {
            verdict: "Implementation complete".to_string(),
            artifacts: vec!["auth.rs".to_string(), "auth_test.rs".to_string()],
        });

        let features = extractor.extract(&episode);
        assert!(!features.key_insights.is_empty());
        assert!(features
            .key_insights
            .iter()
            .any(|i| i.contains("Artifacts produced")));
    }

    #[test]
    fn test_extract_comprehensive_features() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add diverse steps with decisions
        let mut step1 = ExecutionStep::new(
            1,
            "planner".to_string(),
            "Choose implementation strategy".to_string(),
        );
        step1.parameters = serde_json::json!({"strategy": "async"});
        step1.result = Some(ExecutionResult::Success {
            output: "Strategy chosen".to_string(),
        });
        episode.add_step(step1);

        // Add tool sequence
        for i in 1..4 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i), format!("action_{}", i));
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Add error and recovery
        let mut error_step =
            ExecutionStep::new(5, "validator".to_string(), "Validate result".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Validation failed".to_string(),
        });
        episode.add_step(error_step);

        let mut recovery_step = ExecutionStep::new(
            6,
            "validator".to_string(),
            "Re-validate with fix".to_string(),
        );
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(recovery_step);

        // Add reflection and outcome
        episode.reflection = Some(Reflection {
            successes: vec!["Async strategy worked well".to_string()],
            improvements: vec!["Better error messages needed".to_string()],
            insights: vec!["Validation should happen earlier".to_string()],
            generated_at: Utc::now(),
        });

        episode.complete(TaskOutcome::Success {
            verdict: "Successfully implemented".to_string(),
            artifacts: vec!["implementation.rs".to_string()],
        });

        let features = extractor.extract(&episode);

        // Should have extracted features in all categories
        assert!(!features.critical_decisions.is_empty());
        assert!(!features.tool_combinations.is_empty());
        assert!(!features.error_recovery_patterns.is_empty());
        assert!(!features.key_insights.is_empty());

        assert!(features.count() > 5);
    }

    #[test]
    fn test_extract_handles_partial_success() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Core functionality working".to_string(),
            completed: vec!["login".to_string()],
            failed: vec!["logout".to_string()],
        });

        let features = extractor.extract(&episode);
        assert!(!features.critical_decisions.is_empty());
        assert!(features
            .critical_decisions
            .iter()
            .any(|d| d.contains("Partial success")));
    }

    #[test]
    fn test_extract_handles_failure() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        episode.complete(TaskOutcome::Failure {
            reason: "Compilation errors".to_string(),
            error_details: Some("Type mismatch".to_string()),
        });

        let features = extractor.extract(&episode);
        assert!(!features.critical_decisions.is_empty());
        assert!(features
            .critical_decisions
            .iter()
            .any(|d| d.contains("Failure reason")));
    }

    #[test]
    fn test_no_tool_combinations_for_short_sequences() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add only one successful step
        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);

        let features = extractor.extract(&episode);
        assert!(features.tool_combinations.is_empty());
    }

    #[test]
    fn test_timeout_error_recovery() {
        let extractor = SalientExtractor::new();
        let mut episode = create_test_episode();

        // Add timeout error
        let mut timeout_step =
            ExecutionStep::new(1, "fetcher".to_string(), "Fetch data".to_string());
        timeout_step.result = Some(ExecutionResult::Timeout);
        episode.add_step(timeout_step);

        // Add recovery
        let mut recovery_step =
            ExecutionStep::new(2, "fetcher".to_string(), "Retry with timeout".to_string());
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Fetched".to_string(),
        });
        episode.add_step(recovery_step);

        let features = extractor.extract(&episode);
        assert!(!features.error_recovery_patterns.is_empty());
        assert!(features.error_recovery_patterns[0].contains("Timeout"));
    }
}
