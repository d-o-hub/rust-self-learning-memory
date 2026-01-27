//! Episode and `ExecutionStep` structs and implementations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::pre_storage::SalientFeatures;
use crate::types::{ExecutionResult, Reflection, RewardScore, TaskContext, TaskOutcome, TaskType};

/// Records when a pattern was applied during episode execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternApplication {
    /// ID of the pattern that was applied
    pub pattern_id: PatternId,
    /// Step number when pattern was applied
    pub applied_at_step: usize,
    /// Outcome of applying this pattern
    pub outcome: ApplicationOutcome,
    /// Optional notes about the application
    pub notes: Option<String>,
}

/// Outcome of applying a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationOutcome {
    /// Pattern helped achieve the desired outcome
    Helped,
    /// Pattern was applied but had no noticeable effect
    NoEffect,
    /// Pattern hindered progress or caused issues
    Hindered,
    /// Outcome not yet determined
    Pending,
}

impl ApplicationOutcome {
    /// Check if this outcome counts as a success
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, ApplicationOutcome::Helped)
    }
}

/// Unique identifier for patterns extracted from episodes.
pub type PatternId = Uuid;

/// A single execution step within an episode.
///
/// Represents one discrete action or operation performed during task execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number in sequence (1-indexed)
    pub step_number: usize,
    /// When this step was executed
    pub timestamp: DateTime<Utc>,
    /// Tool or function used
    pub tool: String,
    /// Description of action taken
    pub action: String,
    /// Input parameters (as JSON)
    pub parameters: serde_json::Value,
    /// Result of execution
    pub result: Option<ExecutionResult>,
    /// Execution time in milliseconds
    pub latency_ms: u64,
    /// Number of tokens used (if applicable)
    pub tokens_used: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExecutionStep {
    /// Create a new execution step with default values.
    #[must_use]
    pub fn new(step_number: usize, tool: String, action: String) -> Self {
        Self {
            step_number,
            timestamp: Utc::now(),
            tool,
            action,
            parameters: serde_json::json!({}),
            result: None,
            latency_ms: 0,
            tokens_used: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if this step was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.result.as_ref().is_some_and(|r| r.is_success())
    }
}

/// Complete record of a task execution from start to finish.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    /// Unique episode identifier
    pub episode_id: Uuid,
    /// Type of task
    pub task_type: TaskType,
    /// Description of the task
    pub task_description: String,
    /// Task context and metadata
    pub context: TaskContext,
    /// When episode started
    pub start_time: DateTime<Utc>,
    /// When episode completed (None if in progress)
    pub end_time: Option<DateTime<Utc>>,
    /// Execution steps
    pub steps: Vec<ExecutionStep>,
    /// Final outcome
    pub outcome: Option<TaskOutcome>,
    /// Reward score
    pub reward: Option<RewardScore>,
    /// Reflection on execution
    pub reflection: Option<Reflection>,
    /// Extracted pattern IDs
    pub patterns: Vec<PatternId>,
    /// Extracted heuristic IDs
    pub heuristics: Vec<Uuid>,
    /// Record of patterns applied during execution
    #[serde(default)]
    pub applied_patterns: Vec<PatternApplication>,
    /// Salient features extracted during pre-storage reasoning (`PREMem`)
    #[serde(default)]
    pub salient_features: Option<SalientFeatures>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Tags for episode categorization (e.g., "bug-fix", "feature", "refactor")
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Episode {
    /// Create a new episode for a task.
    #[must_use]
    pub fn new(task_description: String, context: TaskContext, task_type: TaskType) -> Self {
        Self {
            episode_id: Uuid::new_v4(),
            task_type,
            task_description,
            context,
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            heuristics: Vec::new(),
            applied_patterns: Vec::new(),
            salient_features: None,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Record that a pattern was applied during this episode
    pub fn record_pattern_application(
        &mut self,
        pattern_id: PatternId,
        applied_at_step: usize,
        outcome: ApplicationOutcome,
        notes: Option<String>,
    ) {
        self.applied_patterns.push(PatternApplication {
            pattern_id,
            applied_at_step,
            outcome,
            notes,
        });
    }

    /// Check if the episode has been completed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some() && self.outcome.is_some()
    }

    /// Get the total duration of the episode.
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Add a new execution step to this episode.
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    /// Mark the episode as complete with a final outcome.
    pub fn complete(&mut self, outcome: TaskOutcome) {
        self.end_time = Some(Utc::now());
        self.outcome = Some(outcome);
    }

    /// Count the number of successful execution steps.
    #[must_use]
    pub fn successful_steps_count(&self) -> usize {
        self.steps.iter().filter(|s| s.is_success()).count()
    }

    /// Count the number of failed execution steps.
    #[must_use]
    pub fn failed_steps_count(&self) -> usize {
        self.steps.iter().filter(|s| !s.is_success()).count()
    }

    /// Normalize a tag: lowercase, trim whitespace, validate characters
    fn normalize_tag(tag: &str) -> Result<String, String> {
        let normalized = tag.trim().to_lowercase();

        if normalized.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }

        if normalized.len() > 100 {
            return Err("Tag cannot exceed 100 characters".to_string());
        }

        // Allow alphanumeric, hyphens, underscores
        if !normalized
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!(
                "Tag '{tag}' contains invalid characters. Only alphanumeric, hyphens, and underscores allowed"
            ));
        }

        Ok(normalized)
    }

    /// Add a tag to this episode (normalized, no duplicates)
    /// Returns `Ok(true)` if tag was added, `Ok(false)` if already exists, `Err` if invalid
    pub fn add_tag(&mut self, tag: String) -> Result<bool, String> {
        let normalized = Self::normalize_tag(&tag)?;

        if self.tags.contains(&normalized) {
            return Ok(false);
        }

        self.tags.push(normalized);
        Ok(true)
    }

    /// Remove a tag from this episode
    /// Returns `true` if tag was removed, `false` if not found
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Ok(normalized) = Self::normalize_tag(tag) {
            if let Some(pos) = self.tags.iter().position(|t| t == &normalized) {
                self.tags.remove(pos);
                return true;
            }
        }
        false
    }

    /// Check if episode has a specific tag
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        if let Ok(normalized) = Self::normalize_tag(tag) {
            self.tags.contains(&normalized)
        } else {
            false
        }
    }

    /// Clear all tags from this episode
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }

    /// Get all tags for this episode
    #[must_use]
    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;

    #[test]
    fn test_episode_creation() {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        let episode = Episode::new(
            "Test task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        );

        assert!(!episode.is_complete());
        assert_eq!(episode.task_description, "Test task");
        assert_eq!(episode.context.domain, "web-api");
        assert_eq!(episode.steps.len(), 0);
    }

    #[test]
    fn test_episode_completion() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        assert!(!episode.is_complete());

        let outcome = TaskOutcome::Success {
            verdict: "All tests passed".to_string(),
            artifacts: vec![],
        };

        episode.complete(outcome);

        assert!(episode.is_complete());
        assert!(episode.end_time.is_some());
        assert!(episode.duration().is_some());
    }

    #[test]
    fn test_execution_step() {
        let mut step =
            ExecutionStep::new(1, "read_file".to_string(), "Read source file".to_string());

        assert!(!step.is_success());

        step.result = Some(ExecutionResult::Success {
            output: "File contents".to_string(),
        });

        assert!(step.is_success());
    }

    #[test]
    fn test_add_steps() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        assert_eq!(episode.steps.len(), 3);
        assert_eq!(episode.successful_steps_count(), 3);
        assert_eq!(episode.failed_steps_count(), 0);
    }

    #[test]
    fn test_add_tag() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add valid tag
        assert_eq!(episode.add_tag("bug-fix".to_string()).unwrap(), true);
        assert!(episode.has_tag("bug-fix"));
        assert_eq!(episode.tags.len(), 1);

        // Add duplicate (normalized)
        assert_eq!(episode.add_tag("BUG-FIX".to_string()).unwrap(), false);
        assert_eq!(episode.tags.len(), 1);

        // Add another tag
        assert_eq!(episode.add_tag("feature".to_string()).unwrap(), true);
        assert_eq!(episode.tags.len(), 2);
    }

    #[test]
    fn test_tag_normalization() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Test case normalization
        episode.add_tag("Feature-123".to_string()).unwrap();
        assert!(episode.has_tag("feature-123"));
        assert!(episode.has_tag("FEATURE-123"));
        assert_eq!(episode.tags[0], "feature-123");

        // Test whitespace trimming
        episode.add_tag("  refactor  ".to_string()).unwrap();
        assert!(episode.has_tag("refactor"));
        assert_eq!(episode.tags[1], "refactor");
    }

    #[test]
    fn test_tag_validation() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Empty tag
        assert!(episode.add_tag("".to_string()).is_err());
        assert!(episode.add_tag("   ".to_string()).is_err());

        // Invalid characters
        assert!(episode.add_tag("bug fix".to_string()).is_err());
        assert!(episode.add_tag("bug@fix".to_string()).is_err());
        assert!(episode.add_tag("bug/fix".to_string()).is_err());

        // Valid characters
        assert!(episode.add_tag("bug-fix".to_string()).is_ok());
        assert!(episode.add_tag("bug_fix".to_string()).is_ok());
        assert!(episode.add_tag("bugfix123".to_string()).is_ok());
        assert_eq!(episode.tags.len(), 3);

        // Too long (>100 chars)
        let long_tag = "a".repeat(101);
        assert!(episode.add_tag(long_tag).is_err());
    }

    #[test]
    fn test_remove_tag() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();
        episode.add_tag("feature".to_string()).unwrap();
        episode.add_tag("refactor".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 3);

        // Remove existing tag
        assert!(episode.remove_tag("feature"));
        assert_eq!(episode.tags.len(), 2);
        assert!(!episode.has_tag("feature"));

        // Remove with different case
        assert!(episode.remove_tag("BUG-FIX"));
        assert_eq!(episode.tags.len(), 1);
        assert!(!episode.has_tag("bug-fix"));

        // Remove non-existent tag
        assert!(!episode.remove_tag("nonexistent"));
        assert_eq!(episode.tags.len(), 1);
    }

    #[test]
    fn test_clear_tags() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();
        episode.add_tag("feature".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 2);

        episode.clear_tags();
        assert_eq!(episode.tags.len(), 0);
        assert!(!episode.has_tag("bug-fix"));
    }

    #[test]
    fn test_get_tags() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();
        episode.add_tag("critical".to_string()).unwrap();

        let tags = episode.get_tags();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"bug-fix".to_string()));
        assert!(tags.contains(&"critical".to_string()));
    }
}
