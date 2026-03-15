//! Checkpoint tool implementations.

use super::types::{
    CheckpointEpisodeInput, CheckpointEpisodeOutput, GetHandoffPackInput, GetHandoffPackOutput,
    HandoffPackResponse, ResumeFromHandoffInput, ResumeFromHandoffOutput,
};
use crate::types::Tool;
use anyhow::{Result, anyhow};
use memory_core::SelfLearningMemory;
use memory_core::memory::checkpoint::{
    checkpoint_episode, checkpoint_episode_with_note, get_handoff_pack, resume_from_handoff,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

/// Checkpoint tools for episode handoffs
#[derive(Clone)]
pub struct CheckpointTools {
    memory: Arc<SelfLearningMemory>,
}

impl CheckpointTools {
    /// Create a new checkpoint tools instance
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Get the tool definition for checkpoint_episode
    pub fn checkpoint_episode_tool() -> Tool {
        Tool::new(
            "checkpoint_episode".to_string(),
            "Create a checkpoint for an in-progress episode. Use this when switching agents, pausing long-running tasks, or before risky operations.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "Episode ID to checkpoint (UUID format)"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Why the checkpoint is being created (e.g., 'Agent switch', 'Long-running task pause')"
                    },
                    "note": {
                        "type": "string",
                        "description": "Optional additional context about the checkpoint"
                    }
                },
                "required": ["episode_id", "reason"]
            }),
        )
    }

    /// Get the tool definition for get_handoff_pack
    pub fn get_handoff_pack_tool() -> Tool {
        Tool::new(
            "get_handoff_pack".to_string(),
            "Generate a handoff pack from a checkpoint. Contains lessons learned, relevant patterns, and suggested next steps for transferring work to another agent.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "checkpoint_id": {
                        "type": "string",
                        "description": "Checkpoint ID to generate handoff pack from (UUID format)"
                    }
                },
                "required": ["checkpoint_id"]
            }),
        )
    }

    /// Get the tool definition for resume_from_handoff
    pub fn resume_from_handoff_tool() -> Tool {
        Tool::new(
            "resume_from_handoff".to_string(),
            "Resume work from a handoff pack. Creates a new episode initialized with context from a previous checkpoint for seamless task continuation.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "handoff_pack": {
                        "type": "object",
                        "description": "The handoff pack to resume from (obtained from get_handoff_pack)"
                    }
                },
                "required": ["handoff_pack"]
            }),
        )
    }

    /// Create a checkpoint for an episode
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and reason
    ///
    /// # Returns
    ///
    /// Returns the checkpoint ID and step number.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode ID is invalid (not a UUID)
    /// - Episode does not exist
    /// - Episode is already completed
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn checkpoint_episode(
        &self,
        input: CheckpointEpisodeInput,
    ) -> Result<CheckpointEpisodeOutput> {
        info!(
            "Creating checkpoint for episode: {} (reason: {})",
            input.episode_id, input.reason
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Create checkpoint
        let checkpoint = if let Some(note) = &input.note {
            checkpoint_episode_with_note(
                &self.memory,
                episode_id,
                input.reason.clone(),
                Some(note.clone()),
            )
            .await
        } else {
            checkpoint_episode(&self.memory, episode_id, input.reason.clone()).await
        };

        match checkpoint {
            Ok(checkpoint) => {
                info!(
                    "Created checkpoint {} for episode {} at step {}",
                    checkpoint.checkpoint_id, episode_id, checkpoint.step_number
                );

                Ok(CheckpointEpisodeOutput {
                    success: true,
                    checkpoint_id: checkpoint.checkpoint_id.to_string(),
                    episode_id: input.episode_id,
                    step_number: checkpoint.step_number,
                    message: format!(
                        "Created checkpoint at step {} with reason: {}",
                        checkpoint.step_number, input.reason
                    ),
                })
            }
            Err(e) => {
                info!("Failed to create checkpoint: {}", e);
                Ok(CheckpointEpisodeOutput {
                    success: false,
                    checkpoint_id: String::new(),
                    episode_id: input.episode_id,
                    step_number: 0,
                    message: format!("Failed to create checkpoint: {}", e),
                })
            }
        }
    }

    /// Get a handoff pack from a checkpoint
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing checkpoint ID
    ///
    /// # Returns
    ///
    /// Returns the handoff pack with lessons learned and guidance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Checkpoint ID is invalid (not a UUID)
    /// - Checkpoint does not exist
    #[instrument(skip(self, input), fields(checkpoint_id = %input.checkpoint_id))]
    pub async fn get_handoff_pack(
        &self,
        input: GetHandoffPackInput,
    ) -> Result<GetHandoffPackOutput> {
        info!(
            "Getting handoff pack for checkpoint: {}",
            input.checkpoint_id
        );

        // Parse checkpoint ID
        let checkpoint_id = Uuid::parse_str(&input.checkpoint_id)
            .map_err(|e| anyhow!("Invalid checkpoint ID: {}", e))?;

        // Get handoff pack
        match get_handoff_pack(&self.memory, checkpoint_id).await {
            Ok(handoff) => {
                info!(
                    "Generated handoff pack with {} steps, {} patterns, {} heuristics",
                    handoff.step_count(),
                    handoff.relevant_patterns.len(),
                    handoff.relevant_heuristics.len()
                );

                Ok(GetHandoffPackOutput {
                    success: true,
                    handoff_pack: Some(HandoffPackResponse::from(handoff)),
                    message: "Successfully generated handoff pack".to_string(),
                })
            }
            Err(e) => {
                info!("Failed to get handoff pack: {}", e);
                Ok(GetHandoffPackOutput {
                    success: false,
                    handoff_pack: None,
                    message: format!("Failed to get handoff pack: {}", e),
                })
            }
        }
    }

    /// Resume work from a handoff pack
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing the handoff pack
    ///
    /// # Returns
    ///
    /// Returns the new episode ID for resumption.
    #[instrument(skip(self, input))]
    pub async fn resume_from_handoff(
        &self,
        input: ResumeFromHandoffInput,
    ) -> Result<ResumeFromHandoffOutput> {
        info!(
            "Resuming from handoff pack: checkpoint_id={}",
            input.handoff_pack.checkpoint_id
        );

        let checkpoint_id = input.handoff_pack.checkpoint_id;
        let episode_id = input.handoff_pack.episode_id;

        // Resume from handoff
        match resume_from_handoff(&self.memory, input.handoff_pack).await {
            Ok(new_episode_id) => {
                info!("Created new episode {} for resumption", new_episode_id);

                Ok(ResumeFromHandoffOutput {
                    success: true,
                    new_episode_id: Some(new_episode_id.to_string()),
                    checkpoint_id: checkpoint_id.to_string(),
                    original_episode_id: episode_id.to_string(),
                    message: format!(
                        "Successfully resumed work in new episode {}",
                        new_episode_id
                    ),
                })
            }
            Err(e) => {
                info!("Failed to resume from handoff: {}", e);
                Ok(ResumeFromHandoffOutput {
                    success: false,
                    new_episode_id: None,
                    checkpoint_id: checkpoint_id.to_string(),
                    original_episode_id: episode_id.to_string(),
                    message: format!("Failed to resume from handoff: {}", e),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checkpoint_episode_invalid_uuid() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = CheckpointTools::new(memory);

        let input = CheckpointEpisodeInput {
            episode_id: "not-a-uuid".to_string(),
            reason: "test".to_string(),
            note: None,
        };

        let result = tools.checkpoint_episode(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_handoff_pack_invalid_uuid() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = CheckpointTools::new(memory);

        let input = GetHandoffPackInput {
            checkpoint_id: "not-a-uuid".to_string(),
        };

        let result = tools.get_handoff_pack(input).await;
        assert!(result.is_err());
    }
}
