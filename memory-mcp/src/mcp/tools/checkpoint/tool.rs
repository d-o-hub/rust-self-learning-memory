//! Checkpoint tool implementations.

use super::types::{
    CheckpointEpisodeInput, CheckpointEpisodeOutput, GetHandoffPackInput, GetHandoffPackOutput,
    HandoffPackResponse, ResumeFromCompactInput, ResumeFromHandoffInput, ResumeFromHandoffOutput,
};
use crate::constants;
use crate::types::Tool;
use anyhow::{Result, anyhow};
use do_memory_core::SelfLearningMemory;
use do_memory_core::memory::checkpoint::{
    checkpoint_episode, checkpoint_episode_with_note, get_compact_handoff_pack, get_handoff_pack,
    resume_from_compact, resume_from_handoff,
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
                        "maxLength": 1000,
                        "description": "Why the checkpoint is being created (e.g., 'Agent switch', 'Long-running task pause') (max 1000 chars)"
                    },
                    "note": {
                        "type": "string",
                        "maxLength": 5000,
                        "description": "Optional additional context about the checkpoint (max 5000 chars)"
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
            "Generate a handoff pack from a checkpoint. Compact mode (default) returns a byte-budgeted profile with findings, decisions, pending actions, and omission receipts; full mode returns the unbounded pack for audit/debug.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "checkpoint_id": {
                        "type": "string",
                        "description": "Checkpoint ID to generate handoff pack from (UUID format)"
                    },
                    "mode": {
                        "type": "string",
                        "description": "Pack mode: 'compact' (default, byte-budgeted) or 'full' (unbounded, for audit/debug)"
                    },
                    "max_bytes": {
                        "type": "integer",
                        "description": "Compact payload ceiling in bytes, minimum 1024 (default 8192, compact mode only)"
                    }
                },
                "required": ["checkpoint_id"]
            }),
        )
    }

    /// Get the tool definition for resume_from_compact
    pub fn resume_from_compact_tool() -> Tool {
        Tool::new(
            "resume_from_compact".to_string(),
            "Resume work from a compact handoff pack. Creates a new episode initialized with the compact goal, findings, decisions, and pending actions.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "compact_handoff": {
                        "type": "object",
                        "description": "The compact handoff pack to resume from (obtained from get_handoff_pack in compact mode)"
                    }
                },
                "required": ["compact_handoff"]
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
        mut input: CheckpointEpisodeInput,
    ) -> Result<CheckpointEpisodeOutput> {
        // Clamp reason and note lengths with UTF-8 safe truncation (CWE-770).
        // Manual char-boundary walk: floor_char_boundary() requires Rust 1.73+.
        let reason_max = constants::MAX_CHECKPOINT_REASON_LEN;
        let reason_truncate_at = if input.reason.len() <= reason_max {
            input.reason.len()
        } else {
            let mut end = reason_max;
            while !input.reason.is_char_boundary(end) {
                end -= 1;
            }
            end
        };
        input.reason.truncate(reason_truncate_at);
        if let Some(note) = &mut input.note {
            let note_max = constants::MAX_CHECKPOINT_NOTE_LEN;
            let note_truncate_at = if note.len() <= note_max {
                note.len()
            } else {
                let mut end = note_max;
                while !note.is_char_boundary(end) {
                    end -= 1;
                }
                end
            };
            note.truncate(note_truncate_at);
        }

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
    /// Compact mode (default) returns a byte-budgeted profile;
    /// `"full"` returns the unbounded pack for audit/debug.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing checkpoint ID, mode, and budget
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

        if input.effective_mode() != "compact" && input.effective_mode() != "full" {
            return Ok(GetHandoffPackOutput {
                success: false,
                handoff_pack: None,
                compact_handoff: None,
                message: format!(
                    "Invalid mode '{}': expected 'compact' or 'full'",
                    input.effective_mode()
                ),
            });
        }

        if input.effective_mode() == "full" {
            return self.get_full_handoff_pack(checkpoint_id).await;
        }

        let budget = input.effective_budget();
        if budget.max_bytes < GetHandoffPackInput::MIN_HANDOFF_BYTES {
            return Ok(GetHandoffPackOutput {
                success: false,
                handoff_pack: None,
                compact_handoff: None,
                message: format!(
                    "max_bytes {} below minimum {}",
                    budget.max_bytes,
                    GetHandoffPackInput::MIN_HANDOFF_BYTES
                ),
            });
        }

        // Accounting for MCP response wrapper overhead so total output fits budget.max_bytes
        let message = format!(
            "Successfully generated compact handoff pack for checkpoint {checkpoint_id}. For full fidelity, call get_handoff_pack with mode: 'full'."
        );
        let wrapper_template = GetHandoffPackOutput {
            success: true,
            handoff_pack: None,
            compact_handoff: None,
            message: message.clone(),
        };
        let wrapper_overhead = serde_json::to_string(&wrapper_template)
            .map(|s| s.len())
            .unwrap_or(200)
            + 20; // safety margin for key overhead
        let mut inner_budget = budget.clone();
        inner_budget.max_bytes = budget
            .max_bytes
            .saturating_sub(wrapper_overhead)
            .max(GetHandoffPackInput::MIN_HANDOFF_BYTES);

        // Get compact handoff pack
        match get_compact_handoff_pack(&self.memory, checkpoint_id, inner_budget).await {
            Ok(compact) => {
                info!(
                    "Generated compact handoff pack ({} bytes, ~{} tokens, {} omitted steps)",
                    compact.payload_bytes(),
                    compact.approx_tokens,
                    compact.omitted.omitted_steps
                );

                Ok(GetHandoffPackOutput {
                    success: true,
                    handoff_pack: None,
                    compact_handoff: Some(compact),
                    message,
                })
            }
            Err(e) => {
                info!("Failed to get compact handoff pack: {}", e);
                Ok(GetHandoffPackOutput {
                    success: false,
                    handoff_pack: None,
                    compact_handoff: None,
                    message: format!("Failed to get handoff pack: {}", e),
                })
            }
        }
    }

    /// Get the full (unbounded) handoff pack for audit/debug use.
    async fn get_full_handoff_pack(&self, checkpoint_id: Uuid) -> Result<GetHandoffPackOutput> {
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
                    compact_handoff: None,
                    message: "Successfully generated handoff pack".to_string(),
                })
            }
            Err(e) => {
                info!("Failed to get handoff pack: {}", e);
                Ok(GetHandoffPackOutput {
                    success: false,
                    handoff_pack: None,
                    compact_handoff: None,
                    message: format!("Failed to get handoff pack: {}", e),
                })
            }
        }
    }

    /// Resume work from a compact handoff pack
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing the compact handoff pack
    ///
    /// # Returns
    ///
    /// Returns the new episode ID for resumption.
    #[instrument(skip(self, input))]
    pub async fn resume_from_compact(
        &self,
        input: ResumeFromCompactInput,
    ) -> Result<ResumeFromHandoffOutput> {
        info!(
            "Resuming from compact handoff: checkpoint_id={}",
            input.compact_handoff.checkpoint_id
        );

        let checkpoint_id = input.compact_handoff.checkpoint_id;
        let episode_id = input.compact_handoff.episode_id;

        // Resume from compact handoff
        match resume_from_compact(&self.memory, input.compact_handoff).await {
            Ok(new_episode_id) => {
                info!("Created new episode {} for resumption", new_episode_id);

                Ok(ResumeFromHandoffOutput {
                    success: true,
                    new_episode_id: Some(new_episode_id.to_string()),
                    checkpoint_id: checkpoint_id.to_string(),
                    original_episode_id: episode_id.to_string(),
                    message: format!("Successfully resumed work in new episode {new_episode_id}"),
                })
            }
            Err(e) => {
                info!("Failed to resume from compact handoff: {}", e);
                Ok(ResumeFromHandoffOutput {
                    success: false,
                    new_episode_id: None,
                    checkpoint_id: checkpoint_id.to_string(),
                    original_episode_id: episode_id.to_string(),
                    message: format!("Failed to resume from handoff: {e}"),
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
mod tests;
