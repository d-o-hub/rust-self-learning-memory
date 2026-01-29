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
        assert!(episode.add_tag("bug-fix".to_string()).unwrap());
        assert!(episode.has_tag("bug-fix"));
        assert_eq!(episode.tags.len(), 1);

        // Add duplicate (normalized)
        assert!(!episode.add_tag("BUG-FIX".to_string()).unwrap());
        assert_eq!(episode.tags.len(), 1);

        // Add another tag
        assert!(episode.add_tag("feature".to_string()).unwrap());
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
        assert!(episode.add_tag(String::new()).is_err());
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

    #[test]
    fn test_tag_validation_error_messages() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Empty tag error message
        let result = episode.add_tag(String::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tag cannot be empty");

        // Whitespace-only tag error message
        let result = episode.add_tag("   ".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tag cannot be empty");

        // Invalid characters error message
        let result = episode.add_tag("bug fix".to_string());
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("invalid characters"));
        assert!(error_msg.contains("bug fix"));

        // Too long tag error message
        let long_tag = "a".repeat(101);
        let result = episode.add_tag(long_tag.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tag cannot exceed 100 characters");
    }

    #[test]
    fn test_tag_minimum_length() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Single character should be valid
        let result = episode.add_tag("a".to_string());
        assert!(result.is_ok());
        assert!(episode.has_tag("a"));
    }

    #[test]
    fn test_tag_boundary_lengths() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Exactly 2 characters
        assert!(episode.add_tag("ab".to_string()).is_ok());
        assert_eq!(episode.tags[0], "ab");

        // Exactly 100 characters
        let tag_100 = "a".repeat(100);
        assert!(episode.add_tag(tag_100.clone()).is_ok());
        assert_eq!(episode.tags[1].len(), 100);

        // 101 characters should fail
        let tag_101 = "b".repeat(101);
        assert!(episode.add_tag(tag_101).is_err());
    }

    #[test]
    fn test_add_tag_with_combined_normalization() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Test combined normalization: trim + lowercase
        let result = episode.add_tag("  FEATURE-123  ".to_string());
        assert!(result.is_ok());
        assert_eq!(episode.tags[0], "feature-123");

        // Verify case-insensitive duplicate detection
        let result = episode.add_tag("feature-123".to_string());
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Should return false for duplicate");
        assert_eq!(episode.tags.len(), 1);

        let result = episode.add_tag("  FEATURE-123  ".to_string());
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "Should return false for duplicate with whitespace"
        );
        assert_eq!(episode.tags.len(), 1);
    }

    #[test]
    fn test_tag_ordering() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add tags in non-alphabetical order
        episode.add_tag("zebra".to_string()).unwrap();
        episode.add_tag("alpha".to_string()).unwrap();
        episode.add_tag("beta".to_string()).unwrap();

        // Tags should maintain insertion order (not sorted)
        assert_eq!(episode.tags, vec!["zebra", "alpha", "beta"]);
    }

    #[test]
    fn test_remove_tag_with_invalid_input() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();
        episode.add_tag("feature".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 2);

        // Try to remove invalid tag (should return false, not panic)
        assert!(!episode.remove_tag("")); // Empty
        assert!(!episode.remove_tag("   ")); // Whitespace only
        assert!(!episode.remove_tag("invalid tag")); // Space in tag
        assert!(!episode.remove_tag("tag@invalid")); // Invalid characters
        assert_eq!(episode.tags.len(), 2); // Tags should remain unchanged
    }

    #[test]
    fn test_has_tag_with_invalid_input() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();

        // Invalid tags should return false, not panic
        assert!(!episode.has_tag(""));
        assert!(!episode.has_tag("   "));
        assert!(!episode.has_tag("invalid tag"));
        assert!(!episode.has_tag("tag@invalid"));
        assert!(!episode.has_tag("nonexistent"));
    }

    #[test]
    fn test_get_tags_on_empty_episode() {
        let context = TaskContext::default();
        let episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        let tags = episode.get_tags();
        assert!(tags.is_empty());
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_has_tag_on_empty_episode() {
        let context = TaskContext::default();
        let episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Should return false for any tag when episode has no tags
        assert!(!episode.has_tag("bug-fix"));
        assert!(!episode.has_tag("feature"));
        assert!(!episode.has_tag("anything"));
    }

    #[test]
    fn test_multiple_tags_with_special_characters_valid() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Test all valid character combinations
        assert!(episode.add_tag("bug-fix".to_string()).is_ok()); // Hyphen
        assert!(episode.add_tag("bug_fix".to_string()).is_ok()); // Underscore
        assert!(episode.add_tag("bug123".to_string()).is_ok()); // Numbers
        assert!(episode.add_tag("123bug".to_string()).is_ok()); // Starts with number
        assert!(episode.add_tag("priority_high".to_string()).is_ok()); // Underscore
        assert!(episode.add_tag("test-123".to_string()).is_ok()); // Hyphen with numbers
        assert!(episode.add_tag("A1B2_C3-D4".to_string()).is_ok()); // Mixed

        assert_eq!(episode.tags.len(), 7);
    }

    #[test]
    fn test_tags_with_only_numbers() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Tags with only numbers should be valid
        assert!(episode.add_tag("123".to_string()).is_ok());
        assert!(episode.add_tag("456".to_string()).is_ok());
        assert!(episode.has_tag("123"));
        assert!(episode.has_tag("456"));
        assert_eq!(episode.tags.len(), 2);
    }

    #[test]
    fn test_tag_serialization() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        episode.add_tag("bug-fix".to_string()).unwrap();
        episode.add_tag("feature".to_string()).unwrap();
        episode.add_tag("priority_high".to_string()).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(&episode).unwrap();
        assert!(json.contains("bug-fix"));
        assert!(json.contains("feature"));
        assert!(json.contains("priority_high"));

        // Deserialize from JSON
        let deserialized: Episode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tags.len(), 3);
        assert!(deserialized.has_tag("bug-fix"));
        assert!(deserialized.has_tag("feature"));
        assert!(deserialized.has_tag("priority_high"));
    }

    #[test]
    fn test_tag_operations_chain() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add multiple tags
        episode.add_tag("tag1".to_string()).unwrap();
        episode.add_tag("tag2".to_string()).unwrap();
        episode.add_tag("tag3".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 3);

        // Check presence
        assert!(episode.has_tag("tag1"));
        assert!(episode.has_tag("tag2"));
        assert!(episode.has_tag("tag3"));

        // Remove middle tag
        assert!(episode.remove_tag("tag2"));
        assert_eq!(episode.tags.len(), 2);
        assert!(!episode.has_tag("tag2"));

        // Add new tag
        episode.add_tag("tag4".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 3);

        // Clear all
        episode.clear_tags();
        assert_eq!(episode.tags.len(), 0);
        assert!(!episode.has_tag("tag1"));
        assert!(!episode.has_tag("tag3"));
        assert!(!episode.has_tag("tag4"));
    }

    #[test]
    fn test_tag_persistence_after_operations() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add tags with various operations
        let mut added = episode.add_tag("persistent".to_string()).unwrap();
        assert!(added);

        added = episode.add_tag("PERSISTENT".to_string()).unwrap();
        assert!(!added, "Duplicate should return false");

        // Verify tag is still there after failed add
        assert!(episode.has_tag("persistent"));

        // Try to remove with wrong case (should still work due to normalization)
        let removed = episode.remove_tag("PERSISTENT");
        assert!(removed);
        assert!(!episode.has_tag("persistent"));
    }

    #[test]
    fn test_tags_are_case_insensitive() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add tag in uppercase
        episode.add_tag("BUG-FIX".to_string()).unwrap();

        // Should find it with various cases
        assert!(episode.has_tag("bug-fix"));
        assert!(episode.has_tag("BUG-FIX"));
        assert!(episode.has_tag("Bug-Fix"));
        assert!(episode.has_tag("  bug-fix  "));

        // Should be able to remove with different case
        assert!(episode.remove_tag("BuG-FiX"));
        assert!(!episode.has_tag("bug-fix"));
    }

    #[test]
    fn test_tag_whitespace_variations() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        // Add tag with leading whitespace
        episode.add_tag("  tag1".to_string()).unwrap();
        assert_eq!(episode.tags[0], "tag1");

        // Add tag with trailing whitespace
        episode.add_tag("tag2  ".to_string()).unwrap();
        assert_eq!(episode.tags[1], "tag2");

        // Add tag with both
        episode.add_tag("  tag3  ".to_string()).unwrap();
        assert_eq!(episode.tags[2], "tag3");

        // All should be found without whitespace
        assert!(episode.has_tag("tag1"));
        assert!(episode.has_tag("tag2"));
        assert!(episode.has_tag("tag3"));

        // Verify no duplicates from whitespace variations
        episode.add_tag(" tag1 ".to_string()).unwrap();
        assert_eq!(episode.tags.len(), 3);
    }

    #[test]
    fn test_clear_tags_on_empty_episode() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        assert_eq!(episode.tags.len(), 0);
        episode.clear_tags(); // Should not panic
        assert_eq!(episode.tags.len(), 0);
    }
}
