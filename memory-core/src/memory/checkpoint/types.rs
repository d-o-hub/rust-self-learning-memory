//! Checkpoint and HandoffPack type definitions.
//!
//! Core data structures for the checkpoint system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::ExecutionStep;
use crate::memory::pattern_search::PatternSearchResult;
use crate::patterns::Heuristic;

use crate::pre_storage::SalientFeatures;

/// Metadata for an episode checkpoint.
///
/// Represents a saved snapshot of progress within an episode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CheckpointMeta {
    /// Unique checkpoint identifier
    pub checkpoint_id: Uuid,
    /// Episode this checkpoint belongs to
    #[serde(default)]
    pub episode_id: Uuid,
    /// Step number at which checkpoint was taken
    pub step_number: usize,
    /// When the checkpoint was created
    #[serde(alias = "created_at")]
    pub timestamp: DateTime<Utc>,
    /// Reason for creating the checkpoint (e.g., "Agent switch", "Long-running task pause")
    #[serde(alias = "reason")]
    pub label: String,
    /// Salient features snapshot at the time of checkpoint
    pub salient_features_snapshot: Option<SalientFeatures>,
    /// If true, this checkpoint was created automatically because the
    /// agent set TaskOutcome::Abstained. The `label` will contain the abstention reason.
    #[serde(default)]
    pub is_abstention_checkpoint: bool,
    /// Optional note about the checkpoint
    pub note: Option<String>,
}

impl CheckpointMeta {
    /// Create a new checkpoint metadata.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode
    /// * `label` - Why the checkpoint was created
    /// * `step_number` - Current step number in the episode
    /// * `note` - Optional additional context
    /// * `salient_features_snapshot` - Optional snapshot of salient features
    ///
    /// # Returns
    ///
    /// New checkpoint metadata with a fresh UUID and current timestamp.
    #[must_use]
    pub fn new(
        episode_id: Uuid,
        label: String,
        step_number: usize,
        note: Option<String>,
        salient_features_snapshot: Option<SalientFeatures>,
    ) -> Self {
        Self {
            checkpoint_id: Uuid::new_v4(),
            episode_id,
            step_number,
            timestamp: Utc::now(),
            label,
            salient_features_snapshot,
            is_abstention_checkpoint: false,
            note,
        }
    }
}

/// A comprehensive context package for transferring work between agents.
///
/// Contains everything needed to resume work or transfer context:
/// - Current progress (steps completed)
/// - Lessons learned (what worked, what failed)
/// - Relevant patterns and heuristics for guidance
/// - Suggested next steps
///
/// This is the primary data structure for multi-agent handoffs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPack {
    /// Unique identifier for this handoff pack
    pub checkpoint_id: Uuid,
    /// Episode this handoff is derived from
    pub episode_id: Uuid,
    /// When the handoff was created
    pub timestamp: DateTime<Utc>,
    /// Current goal or task description
    pub current_goal: String,
    /// Steps completed so far (up to checkpoint)
    pub steps_completed: Vec<ExecutionStep>,
    /// What approaches/tools worked well
    pub what_worked: Vec<String>,
    /// What approaches failed or caused issues
    pub what_failed: Vec<String>,
    /// Salient facts discovered during execution
    pub salient_facts: Vec<String>,
    /// Suggested next steps for continuation
    pub suggested_next_steps: Vec<String>,
    /// Relevant patterns that could help continuation
    pub relevant_patterns: Vec<PatternSearchResult>,
    /// Relevant heuristics for decision-making
    pub relevant_heuristics: Vec<Heuristic>,
}

impl HandoffPack {
    /// Get the number of completed steps.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps_completed.len()
    }

    /// Check if the handoff pack has any lessons learned.
    #[must_use]
    pub fn has_lessons(&self) -> bool {
        !self.what_worked.is_empty() || !self.what_failed.is_empty()
    }

    /// Check if the handoff pack has guidance (patterns or heuristics).
    #[must_use]
    pub fn has_guidance(&self) -> bool {
        !self.relevant_patterns.is_empty() || !self.relevant_heuristics.is_empty()
    }

    /// Get a summary of the handoff pack for display.
    #[must_use]
    pub fn summary(&self) -> HandoffSummary {
        HandoffSummary {
            checkpoint_id: self.checkpoint_id,
            episode_id: self.episode_id,
            step_count: self.steps_completed.len(),
            what_worked_count: self.what_worked.len(),
            what_failed_count: self.what_failed.len(),
            pattern_count: self.relevant_patterns.len(),
            heuristic_count: self.relevant_heuristics.len(),
            suggested_steps_count: self.suggested_next_steps.len(),
        }
    }
}

/// Summary of a HandoffPack for quick reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandoffSummary {
    /// Checkpoint ID
    pub checkpoint_id: Uuid,
    /// Source episode ID
    pub episode_id: Uuid,
    /// Number of steps completed
    pub step_count: usize,
    /// Number of "what worked" items
    pub what_worked_count: usize,
    /// Number of "what failed" items
    pub what_failed_count: usize,
    /// Number of relevant patterns
    pub pattern_count: usize,
    /// Number of relevant heuristics
    pub heuristic_count: usize,
    /// Number of suggested next steps
    pub suggested_steps_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_meta_creation() {
        let episode_id = Uuid::new_v4();
        let checkpoint = CheckpointMeta::new(
            episode_id,
            "Agent switch".to_string(),
            5,
            Some("Note".to_string()),
            None,
        );

        assert!(!checkpoint.checkpoint_id.is_nil());
        assert_eq!(checkpoint.episode_id, episode_id);
        assert_eq!(checkpoint.label, "Agent switch");
        assert_eq!(checkpoint.step_number, 5);
        assert!(!checkpoint.is_abstention_checkpoint);
    }

    #[test]
    fn test_checkpoint_meta_minimal() {
        let checkpoint =
            CheckpointMeta::new(Uuid::new_v4(), "Quick pause".to_string(), 0, None, None);

        assert!(!checkpoint.checkpoint_id.is_nil());
        assert_eq!(checkpoint.label, "Quick pause");
        assert_eq!(checkpoint.step_number, 0);
    }

    #[test]
    fn test_handoff_pack_summary() {
        let pack = HandoffPack {
            checkpoint_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            current_goal: "Test goal".to_string(),
            steps_completed: vec![
                ExecutionStep::new(1, "tool1".to_string(), "action1".to_string()),
                ExecutionStep::new(2, "tool2".to_string(), "action2".to_string()),
            ],
            what_worked: vec!["Approach A".to_string()],
            what_failed: vec!["Approach B".to_string(), "Approach C".to_string()],
            salient_facts: vec!["Fact 1".to_string()],
            suggested_next_steps: vec!["Step 1".to_string(), "Step 2".to_string()],
            relevant_patterns: vec![],
            relevant_heuristics: vec![],
        };

        let summary = pack.summary();
        assert_eq!(summary.step_count, 2);
        assert_eq!(summary.what_worked_count, 1);
        assert_eq!(summary.what_failed_count, 2);
        assert!(!pack.has_guidance());
        assert!(pack.has_lessons());
    }
}
