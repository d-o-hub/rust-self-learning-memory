//! Episode timeline tool for MCP server
//!
//! This module provides the tool for retrieving a chronological view
//! of all steps in an episode.

use crate::server::MemoryMCPServer;
use anyhow::{Result, anyhow};
use memory_core::{ExecutionResult, TaskOutcome};
use serde_json::{Value, json};
use tracing::debug;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Get a timeline view of episode execution
    ///
    /// This tool provides a chronological view of all steps in an episode,
    /// useful for visualizing task progression.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode
    pub async fn get_episode_timeline_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting episode timeline with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        let episode = self
            .memory
            .get_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to get episode: {}", e))?;

        // Build timeline
        let timeline: Vec<Value> = episode
            .steps
            .iter()
            .map(|step| {
                json!({
                    "step_number": step.step_number,
                    "timestamp": step.timestamp.to_rfc3339(),
                    "tool": step.tool,
                    "action": step.action,
                    "result_type": match &step.result {
                        Some(ExecutionResult::Success { .. }) => "success",
                        Some(ExecutionResult::Error { .. }) => "error",
                        Some(ExecutionResult::Timeout) => "timeout",
                        None => "pending",
                    },
                    "latency_ms": step.latency_ms,
                })
            })
            .collect();

        let duration_seconds = episode
            .end_time
            .map(|end| (end.timestamp() - episode.start_time.timestamp()) as f64);

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "task_description": episode.task_description,
            "start_time": episode.start_time.to_rfc3339(),
            "end_time": episode.end_time.map(|t| t.to_rfc3339()),
            "duration_seconds": duration_seconds,
            "step_count": episode.steps.len(),
            "timeline": timeline,
            "outcome": episode.outcome.as_ref().map(|o| match o {
                TaskOutcome::Success { .. } => "success",
                TaskOutcome::PartialSuccess { .. } => "partial_success",
                TaskOutcome::Failure { .. } => "failure",
            }),
        }))
    }
}
