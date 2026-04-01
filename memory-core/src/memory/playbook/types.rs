//! Playbook type definitions
//!
//! Core types for actionable recommendation playbooks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::types::TaskContext;
use crate::types::TaskType;

/// A single step in a playbook.
///
/// Each step represents an actionable instruction with context about
/// what tool to use, what to expect, and in what order to execute.
///
/// # Example
///
/// ```
/// use do_memory_core::memory::playbook::PlaybookStep;
///
/// let step = PlaybookStep {
///     order: 1,
///     action: "Analyze existing authentication patterns".to_string(),
///     tool_hint: Some("pattern_search".to_string()),
///     expected_result: Some("List of relevant authentication patterns".to_string()),
/// };
///
/// assert_eq!(step.order, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybookStep {
    /// Step order in the playbook (1-indexed)
    pub order: usize,
    /// Action to take
    pub action: String,
    /// Optional hint about which tool/agent to use
    pub tool_hint: Option<String>,
    /// Expected result of this step
    pub expected_result: Option<String>,
}

impl PlaybookStep {
    /// Create a new playbook step.
    #[must_use]
    pub fn new(order: usize, action: String) -> Self {
        Self {
            order,
            action,
            tool_hint: None,
            expected_result: None,
        }
    }

    /// Add a tool hint to this step.
    #[must_use]
    pub fn with_tool_hint(mut self, hint: impl Into<String>) -> Self {
        self.tool_hint = Some(hint.into());
        self
    }

    /// Add an expected result to this step.
    #[must_use]
    pub fn with_expected_result(mut self, result: impl Into<String>) -> Self {
        self.expected_result = Some(result.into());
        self
    }
}

/// A pitfall or warning to avoid during playbook execution.
///
/// Pitfalls are synthesized from failed episodes and error recovery patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybookPitfall {
    /// Description of the pitfall
    pub warning: String,
    /// Why this is a problem
    pub reason: String,
    /// How to avoid it
    pub mitigation: Option<String>,
}

impl PlaybookPitfall {
    /// Create a new pitfall warning.
    #[must_use]
    pub fn new(warning: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            warning: warning.into(),
            reason: reason.into(),
            mitigation: None,
        }
    }

    /// Add mitigation advice.
    #[must_use]
    pub fn with_mitigation(mut self, mitigation: impl Into<String>) -> Self {
        self.mitigation = Some(mitigation.into());
        self
    }
}

/// Source data used for playbook synthesis.
///
/// Tracks which patterns, episodes, and summaries contributed to a playbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybookSynthesisSource {
    /// Pattern IDs that were synthesized
    pub pattern_ids: Vec<PatternId>,
    /// Episode IDs whose reflections/summaries were used
    pub episode_ids: Vec<Uuid>,
    /// Summary IDs (episode IDs with summaries)
    pub summary_episode_ids: Vec<Uuid>,
}

impl Default for PlaybookSynthesisSource {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybookSynthesisSource {
    /// Create an empty source tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pattern_ids: Vec::new(),
            episode_ids: Vec::new(),
            summary_episode_ids: Vec::new(),
        }
    }

    /// Add a pattern to the source.
    pub fn add_pattern(&mut self, pattern_id: PatternId) {
        if !self.pattern_ids.contains(&pattern_id) {
            self.pattern_ids.push(pattern_id);
        }
    }

    /// Add an episode to the source.
    pub fn add_episode(&mut self, episode_id: Uuid) {
        if !self.episode_ids.contains(&episode_id) {
            self.episode_ids.push(episode_id);
        }
    }

    /// Add a summary source.
    pub fn add_summary(&mut self, episode_id: Uuid) {
        if !self.summary_episode_ids.contains(&episode_id) {
            self.summary_episode_ids.push(episode_id);
        }
    }

    /// Get total source count.
    #[must_use]
    pub fn total_sources(&self) -> usize {
        self.pattern_ids.len() + self.episode_ids.len() + self.summary_episode_ids.len()
    }
}

/// A recommended playbook synthesizing patterns, reflections, and summaries.
///
/// This is the core output of the playbook generator, providing actionable
/// guidance for agents with clear steps, applicability rules, and expected outcomes.
///
/// # Template-Driven Synthesis
///
/// Playbooks are generated without LLM on the hot path, using:
/// - Pattern templates based on pattern type
/// - Reflection templates based on episode outcomes
/// - Summary templates based on key concepts
///
/// # Example
///
/// ```
/// use do_memory_core::memory::playbook::{RecommendedPlaybook, PlaybookStep, PlaybookPitfall};
/// use uuid::Uuid;
/// use chrono::Utc;
///
/// let playbook = RecommendedPlaybook {
///     playbook_id: Uuid::new_v4(),
///     task_match_score: 0.85,
///     why_relevant: "Based on 3 similar authentication tasks".to_string(),
///     when_to_apply: vec!["When implementing new authentication".to_string()],
///     when_not_to_apply: vec!["For public read-only APIs".to_string()],
///     ordered_steps: vec![
///         PlaybookStep::new(1, "Search existing auth patterns".to_string())
///             .with_tool_hint("pattern_search"),
///         PlaybookStep::new(2, "Implement auth flow".to_string()),
///     ],
///     pitfalls: vec![
///         PlaybookPitfall::new(
///             "Don't store passwords in plain text",
///             "Security vulnerability"
///         ).with_mitigation("Use bcrypt or argon2"),
///     ],
///     expected_outcome: "Secure authentication with token-based sessions".to_string(),
///     confidence: 0.85,
///     supporting_pattern_ids: vec![Uuid::new_v4()],
///     supporting_episode_ids: vec![Uuid::new_v4()],
///     created_at: Utc::now(),
/// };
///
/// assert!(playbook.confidence > 0.5);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendedPlaybook {
    /// Unique playbook identifier
    pub playbook_id: Uuid,
    /// How well this playbook matches the task (0.0-1.0)
    pub task_match_score: f32,
    /// Human-readable explanation of relevance
    pub why_relevant: String,
    /// Conditions when this playbook should be applied
    pub when_to_apply: Vec<String>,
    /// Conditions when this playbook should NOT be applied
    pub when_not_to_apply: Vec<String>,
    /// Ordered steps to execute
    pub ordered_steps: Vec<PlaybookStep>,
    /// Pitfalls and warnings to avoid
    pub pitfalls: Vec<PlaybookPitfall>,
    /// Expected outcome description
    pub expected_outcome: String,
    /// Overall confidence score (0.0-1.0)
    pub confidence: f32,
    /// Pattern IDs that support this playbook
    pub supporting_pattern_ids: Vec<PatternId>,
    /// Episode IDs that support this playbook
    pub supporting_episode_ids: Vec<Uuid>,
    /// When this playbook was generated
    pub created_at: DateTime<Utc>,
}

impl RecommendedPlaybook {
    /// Create a new playbook with the given ID and match score.
    #[must_use]
    pub fn new(playbook_id: Uuid, task_match_score: f32) -> Self {
        Self {
            playbook_id,
            task_match_score,
            why_relevant: String::new(),
            when_to_apply: Vec::new(),
            when_not_to_apply: Vec::new(),
            ordered_steps: Vec::new(),
            pitfalls: Vec::new(),
            expected_outcome: String::new(),
            confidence: 0.0,
            supporting_pattern_ids: Vec::new(),
            supporting_episode_ids: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Check if this playbook has high confidence (>= 0.7).
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.7
    }

    /// Get the number of steps in this playbook.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.ordered_steps.len()
    }

    /// Get the number of supporting sources.
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.supporting_pattern_ids.len() + self.supporting_episode_ids.len()
    }

    /// Calculate an overall quality score combining match, confidence, and sources.
    #[must_use]
    pub fn quality_score(&self) -> f32 {
        let source_weight = (self.source_count() as f32).ln().max(0.0) / 3.0;
        (self.task_match_score * 0.4 + self.confidence * 0.4 + source_weight * 0.2).min(1.0)
    }
}

/// Request to generate a playbook for a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybookRequest {
    /// Description of the task
    pub task_description: String,
    /// Domain of the task
    pub domain: String,
    /// Type of task
    pub task_type: TaskType,
    /// Additional context
    pub context: TaskContext,
    /// Maximum number of steps to include
    pub max_steps: usize,
}

impl PlaybookRequest {
    /// Create a new playbook request.
    #[must_use]
    pub fn new(task_description: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            task_description: task_description.into(),
            domain: domain.into(),
            task_type: TaskType::CodeGeneration,
            context: TaskContext::default(),
            max_steps: 5,
        }
    }

    /// Set the task type.
    #[must_use]
    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = task_type;
        self
    }

    /// Set the context.
    #[must_use]
    pub fn with_context(mut self, context: TaskContext) -> Self {
        self.context = context;
        self
    }

    /// Set max steps.
    #[must_use]
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_step_creation() {
        let step = PlaybookStep::new(1, "Test action".to_string())
            .with_tool_hint("test_tool")
            .with_expected_result("Test result");

        assert_eq!(step.order, 1);
        assert_eq!(step.action, "Test action");
        assert_eq!(step.tool_hint, Some("test_tool".to_string()));
        assert_eq!(step.expected_result, Some("Test result".to_string()));
    }

    #[test]
    fn test_playbook_pitfall() {
        let pitfall = PlaybookPitfall::new("Warning", "Reason").with_mitigation("Do this instead");

        assert_eq!(pitfall.warning, "Warning");
        assert_eq!(pitfall.reason, "Reason");
        assert_eq!(pitfall.mitigation, Some("Do this instead".to_string()));
    }

    #[test]
    fn test_synthesis_source() {
        let mut source = PlaybookSynthesisSource::new();
        let pattern_id = Uuid::new_v4();
        let episode_id = Uuid::new_v4();

        source.add_pattern(pattern_id);
        source.add_pattern(pattern_id); // Duplicate, should not be added
        source.add_episode(episode_id);
        source.add_summary(episode_id);

        assert_eq!(source.pattern_ids.len(), 1);
        assert_eq!(source.episode_ids.len(), 1);
        assert_eq!(source.summary_episode_ids.len(), 1);
        assert_eq!(source.total_sources(), 3);
    }

    #[test]
    fn test_recommended_playbook() {
        let playbook = RecommendedPlaybook::new(Uuid::new_v4(), 0.85);

        assert!(!playbook.is_high_confidence()); // confidence is 0.0 initially

        let mut playbook = playbook;
        playbook.confidence = 0.8;
        assert!(playbook.is_high_confidence());

        playbook
            .ordered_steps
            .push(PlaybookStep::new(1, "Step 1".to_string()));
        assert_eq!(playbook.step_count(), 1);
    }

    #[test]
    fn test_playbook_quality_score() {
        let mut playbook = RecommendedPlaybook::new(Uuid::new_v4(), 0.9);
        playbook.confidence = 0.8;
        playbook.supporting_pattern_ids.push(Uuid::new_v4());
        playbook.supporting_pattern_ids.push(Uuid::new_v4());
        playbook.supporting_episode_ids.push(Uuid::new_v4());

        let score = playbook.quality_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_playbook_request() {
        let request = PlaybookRequest::new("Test task", "web-api")
            .with_task_type(TaskType::Debugging)
            .with_max_steps(10);

        assert_eq!(request.task_description, "Test task");
        assert_eq!(request.domain, "web-api");
        assert_eq!(request.task_type, TaskType::Debugging);
        assert_eq!(request.max_steps, 10);
    }
}
