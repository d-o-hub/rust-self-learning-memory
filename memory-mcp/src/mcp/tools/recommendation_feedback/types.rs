//! Recommendation Feedback Types
//!
//! Types for the MCP tool that records feedback about recommendation effectiveness.

use serde::{Deserialize, Serialize};

use memory_core::types::TaskOutcome;

/// Input for recording recommendation feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRecommendationFeedbackInput {
    /// The session ID this feedback relates to
    pub session_id: String,
    /// Pattern IDs that were actually applied by the agent
    #[serde(default)]
    pub applied_pattern_ids: Vec<String>,
    /// Episode IDs that were consulted from retrieval results
    #[serde(default)]
    pub consulted_episode_ids: Vec<String>,
    /// Final outcome of the task
    pub outcome: TaskOutcomeJson,
    /// Optional agent rating of recommendation quality (0.0-1.0)
    pub agent_rating: Option<f32>,
}

/// JSON-compatible representation of TaskOutcome for MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskOutcomeJson {
    /// Task completed successfully
    Success {
        /// Verdict message
        verdict: String,
        /// Artifacts produced
        #[serde(default)]
        artifacts: Vec<String>,
    },
    /// Task partially completed
    PartialSuccess {
        /// What was completed
        #[serde(default)]
        completed: Vec<String>,
        /// What failed
        #[serde(default)]
        failed: Vec<String>,
        /// Verdict message
        verdict: String,
    },
    /// Task failed
    Failure {
        /// Reason for failure
        reason: String,
        /// Detailed error information
        #[serde(default)]
        error_details: Option<String>,
    },
}

impl TaskOutcomeJson {
    /// Convert to the core TaskOutcome type.
    pub fn to_task_outcome(&self) -> TaskOutcome {
        match self {
            Self::Success { verdict, artifacts } => TaskOutcome::Success {
                verdict: verdict.clone(),
                artifacts: artifacts.clone(),
            },
            Self::PartialSuccess {
                completed,
                failed,
                verdict,
            } => TaskOutcome::PartialSuccess {
                completed: completed.clone(),
                failed: failed.clone(),
                verdict: verdict.clone(),
            },
            Self::Failure {
                reason,
                error_details,
            } => TaskOutcome::Failure {
                reason: reason.clone(),
                error_details: error_details.clone(),
            },
        }
    }
}

/// Output from recording recommendation feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRecommendationFeedbackOutput {
    /// Whether the operation succeeded
    pub success: bool,
    /// Session ID
    pub session_id: String,
    /// Number of patterns marked as applied
    pub patterns_applied: usize,
    /// Number of episodes consulted
    pub episodes_consulted: usize,
    /// Message describing the result
    pub message: String,
}

/// Input for recording a recommendation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRecommendationSessionInput {
    /// Episode ID for which recommendations are made
    pub episode_id: String,
    /// Pattern IDs that were recommended
    #[serde(default)]
    pub recommended_pattern_ids: Vec<String>,
    /// Playbook IDs that were recommended
    #[serde(default)]
    pub recommended_playbook_ids: Vec<String>,
}

/// Output from recording a recommendation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRecommendationSessionOutput {
    /// Whether the operation succeeded
    pub success: bool,
    /// Generated session ID
    pub session_id: String,
    /// Episode ID
    pub episode_id: String,
    /// Number of patterns recommended
    pub patterns_recommended: usize,
    /// Number of playbooks recommended
    pub playbooks_recommended: usize,
    /// Message describing the result
    pub message: String,
}

/// Output for recommendation statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationStatsOutput {
    /// Whether the operation succeeded
    pub success: bool,
    /// Total recommendation sessions recorded
    pub total_sessions: usize,
    /// Total feedback records received
    pub total_feedback: usize,
    /// Patterns recommended and applied
    pub patterns_applied: usize,
    /// Patterns recommended but not applied
    pub patterns_ignored: usize,
    /// Overall adoption rate (applied / recommended)
    pub adoption_rate: f32,
    /// Success rate after adoption
    pub success_after_adoption_rate: f32,
    /// Average agent rating
    pub avg_agent_rating: Option<f32>,
    /// Message describing the result
    pub message: String,
}
