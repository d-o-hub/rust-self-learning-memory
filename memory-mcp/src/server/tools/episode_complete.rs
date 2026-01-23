//! Episode completion tool for MCP server
//!
//! This module provides the tool for finalizing episodes by recording
//! the outcome and triggering the learning cycle.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::TaskOutcome;
use serde_json::{json, Value};
use tracing::debug;
use tracing::info;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Complete an episode with an outcome
    ///
    /// This tool finalizes an episode by recording the outcome and
    /// triggering the learning cycle (reward calculation, reflection,
    /// pattern extraction).
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to complete
    /// * `outcome_type` - Type of outcome ("success", "partial_success", "failure")
    /// * `verdict` - Description of the outcome (required for success/partial)
    /// * `artifacts` - Array of artifact names (optional, for success)
    /// * `completed` - Array of completed items (required for partial_success)
    /// * `failed` - Array of failed items (required for partial_success)
    /// * `reason` - Failure reason (required for failure)
    /// * `error_details` - Detailed error information (optional, for failure)
    pub async fn complete_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Completing episode with args: {}", args);

        // Extract required fields
        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        let outcome_type = args
            .get("outcome_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: outcome_type"))?;

        // Parse outcome based on type
        let outcome = match outcome_type {
            "success" => {
                let verdict = args
                    .get("verdict")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing required field for success: verdict"))?
                    .to_string();

                let artifacts = args
                    .get("artifacts")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                TaskOutcome::Success { verdict, artifacts }
            }
            "partial_success" => {
                let verdict = args
                    .get("verdict")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing required field for partial_success: verdict"))?
                    .to_string();

                let completed = args
                    .get("completed")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        anyhow!("Missing required field for partial_success: completed")
                    })?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let failed = args
                    .get("failed")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing required field for partial_success: failed"))?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                TaskOutcome::PartialSuccess {
                    verdict,
                    completed,
                    failed,
                }
            }
            "failure" => {
                let reason = args
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing required field for failure: reason"))?
                    .to_string();

                let error_details = args
                    .get("error_details")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                TaskOutcome::Failure {
                    reason,
                    error_details,
                }
            }
            _ => {
                return Err(anyhow!(
                    "Invalid outcome_type: {}. Must be one of: success, partial_success, failure",
                    outcome_type
                ))
            }
        };

        // Complete the episode
        self.memory
            .complete_episode(episode_id, outcome.clone())
            .await
            .map_err(|e| anyhow!("Failed to complete episode: {}", e))?;

        info!(
            episode_id = %episode_id,
            outcome_type = %outcome_type,
            "Completed episode via MCP"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "outcome_type": outcome_type,
            "message": "Episode completed successfully. Learning cycle triggered (reward, reflection, patterns)."
        }))
    }
}
