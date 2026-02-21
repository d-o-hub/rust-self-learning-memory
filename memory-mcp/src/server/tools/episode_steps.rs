//! Episode step logging tool for MCP server
//!
//! This module provides the tool for logging execution steps to track
//! progress within an episode.

use crate::server::MemoryMCPServer;
use anyhow::{Result, anyhow};
use memory_core::ExecutionStep;
use serde_json::{Value, json};
use tracing::debug;
use tracing::info;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Add a step to an existing episode
    ///
    /// This tool allows logging execution steps to track progress
    /// within an episode.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode
    /// * `step_number` - Sequential step number
    /// * `tool` - Name of the tool/component performing the action
    /// * `action` - Description of the action taken
    /// * `parameters` - Optional JSON object of parameters
    /// * `result` - Optional result object with type and details
    /// * `latency_ms` - Optional execution time in milliseconds
    pub async fn add_episode_step_tool(&self, args: Value) -> Result<Value> {
        debug!("Adding episode step with args: {}", args);

        // Extract required fields
        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        let step_number = args
            .get("step_number")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Missing required field: step_number"))?
            as usize;

        let tool = args
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: tool"))?
            .to_string();

        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: action"))?
            .to_string();

        // Create execution step
        let mut step = ExecutionStep::new(step_number, tool.clone(), action.clone());

        // Add optional parameters
        if let Some(params) = args.get("parameters") {
            step.parameters = params.clone();
        }

        // Add optional result
        if let Some(result) = args.get("result") {
            let result_type = result
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("success");

            step.result = Some(match result_type {
                "success" => memory_core::ExecutionResult::Success {
                    output: result
                        .get("output")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
                "error" => memory_core::ExecutionResult::Error {
                    message: result
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string(),
                },
                "timeout" => memory_core::ExecutionResult::Timeout,
                _ => {
                    return Err(anyhow!(
                        "Invalid result type: {}. Must be one of: success, error, timeout",
                        result_type
                    ));
                }
            });
        }

        // Add optional latency
        if let Some(latency) = args.get("latency_ms").and_then(|v| v.as_u64()) {
            step.latency_ms = latency;
        }

        // Log the step
        self.memory.log_step(episode_id, step).await;

        info!(
            episode_id = %episode_id,
            step_number = step_number,
            tool = %tool,
            "Added step to episode via MCP"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "step_number": step_number,
            "message": "Step added successfully"
        }))
    }
}
