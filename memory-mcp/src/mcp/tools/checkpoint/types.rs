//! Checkpoint tool types and input/output structures.

use do_memory_core::HandoffPack;
use serde::{Deserialize, Serialize};

/// Input parameters for creating a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointEpisodeInput {
    /// Episode ID to create checkpoint for
    pub episode_id: String,
    /// Reason for creating the checkpoint (e.g., "Agent switch", "Long-running task pause")
    pub reason: String,
    /// Optional additional context about the checkpoint
    #[serde(default)]
    pub note: Option<String>,
}

/// Output from creating a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointEpisodeOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Checkpoint ID that was created
    pub checkpoint_id: String,
    /// Episode ID that was checkpointed
    pub episode_id: String,
    /// Step number at which checkpoint was taken
    pub step_number: usize,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for getting a handoff pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHandoffPackInput {
    /// Checkpoint ID to generate handoff pack from
    pub checkpoint_id: String,
}

/// Output from getting a handoff pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHandoffPackOutput {
    /// Whether operation was successful
    pub success: bool,
    /// The handoff pack (null if not found)
    pub handoff_pack: Option<HandoffPackResponse>,
    /// Message describing the result
    pub message: String,
}

/// Serializable handoff pack for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPackResponse {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Episode ID
    pub episode_id: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Current goal
    pub current_goal: String,
    /// Number of steps completed
    pub steps_completed_count: usize,
    /// What worked
    pub what_worked: Vec<String>,
    /// What failed
    pub what_failed: Vec<String>,
    /// Salient facts
    pub salient_facts: Vec<String>,
    /// Suggested next steps
    pub suggested_next_steps: Vec<String>,
    /// Pattern count
    pub pattern_count: usize,
    /// Heuristic count
    pub heuristic_count: usize,
}

impl From<HandoffPack> for HandoffPackResponse {
    fn from(pack: HandoffPack) -> Self {
        Self {
            checkpoint_id: pack.checkpoint_id.to_string(),
            episode_id: pack.episode_id.to_string(),
            timestamp: pack.timestamp.to_rfc3339(),
            current_goal: pack.current_goal,
            steps_completed_count: pack.steps_completed.len(),
            what_worked: pack.what_worked,
            what_failed: pack.what_failed,
            salient_facts: pack.salient_facts,
            suggested_next_steps: pack.suggested_next_steps,
            pattern_count: pack.relevant_patterns.len(),
            heuristic_count: pack.relevant_heuristics.len(),
        }
    }
}

/// Input parameters for resuming from a handoff pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeFromHandoffInput {
    /// The handoff pack to resume from
    pub handoff_pack: HandoffPack,
}

/// Output from resuming from a handoff pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeFromHandoffOutput {
    /// Whether operation was successful
    pub success: bool,
    /// New episode ID created for resumption
    pub new_episode_id: Option<String>,
    /// Original checkpoint ID
    pub checkpoint_id: String,
    /// Original episode ID
    pub original_episode_id: String,
    /// Message describing the result
    pub message: String,
}
