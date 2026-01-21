//! Episode lifecycle management tools for MCP server
//!
//! This module provides MCP tools for creating, updating, completing,
//! and managing episodes programmatically through the MCP interface.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{ExecutionStep, TaskContext, TaskOutcome, TaskType};
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use uuid::Uuid;

impl MemoryMCPServer {
    /// Create a new episode
    ///
    /// This tool allows AI agents to programmatically create new episodes
    /// for tracking task execution through the MCP interface.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `task_description` - Clear description of the task
    /// * `domain` - Task domain (e.g., "web-api", "cli")
    /// * `task_type` - Type of task (e.g., "code_generation", "debugging")
    /// * `language` - Optional programming language
    /// * `framework` - Optional framework name
    /// * `tags` - Optional array of tags
    /// * `complexity` - Optional complexity level ("simple", "moderate", "complex")
    pub async fn create_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Creating episode with args: {}", args);

        // Extract required fields
        let task_description = args
            .get("task_description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: task_description"))?
            .to_string();

        let domain = args
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: domain"))?
            .to_string();

        let task_type_str = args
            .get("task_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: task_type"))?;

        // Parse task type
        let task_type = match task_type_str {
            "code_generation" => TaskType::CodeGeneration,
            "debugging" => TaskType::Debugging,
            "refactoring" => TaskType::Refactoring,
            "testing" => TaskType::Testing,
            "analysis" => TaskType::Analysis,
            "documentation" => TaskType::Documentation,
            _ => {
                return Err(anyhow!(
                    "Invalid task_type: {}. Must be one of: code_generation, debugging, refactoring, testing, analysis, documentation",
                    task_type_str
                ))
            }
        };

        // Extract optional fields
        let language = args
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let framework = args
            .get("framework")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tags = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let complexity = args
            .get("complexity")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "simple" => memory_core::ComplexityLevel::Simple,
                "moderate" => memory_core::ComplexityLevel::Moderate,
                "complex" => memory_core::ComplexityLevel::Complex,
                _ => memory_core::ComplexityLevel::Moderate,
            })
            .unwrap_or(memory_core::ComplexityLevel::Moderate);

        // Create task context
        let context = TaskContext {
            language,
            framework,
            complexity,
            domain: domain.clone(),
            tags,
        };

        // Start the episode
        let episode_id = self
            .memory
            .start_episode(task_description.clone(), context, task_type)
            .await;

        info!(
            episode_id = %episode_id,
            task_description = %task_description,
            domain = %domain,
            "Created new episode via MCP"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "task_description": task_description,
            "domain": domain,
            "task_type": task_type_str,
            "message": "Episode created successfully"
        }))
    }

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
                    ))
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

    /// Get episode details by ID
    ///
    /// This tool retrieves complete details of an episode including
    /// all steps, outcome, reflection, and extracted patterns.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to retrieve
    pub async fn get_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting episode with args: {}", args);

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

        // Convert episode to JSON
        let episode_json = serde_json::to_value(&episode)
            .map_err(|e| anyhow!("Failed to serialize episode: {}", e))?;

        Ok(json!({
            "success": true,
            "episode": episode_json
        }))
    }

    /// Delete an episode permanently
    ///
    /// This tool removes an episode from all storage backends.
    /// **Warning**: This operation cannot be undone.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to delete
    /// * `confirm` - Must be set to true to confirm deletion
    pub async fn delete_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Deleting episode with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        // Require explicit confirmation
        let confirm = args
            .get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(anyhow!(
                "Deletion requires explicit confirmation. Set 'confirm' to true."
            ));
        }

        self.memory
            .delete_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to delete episode: {}", e))?;

        warn!(
            episode_id = %episode_id,
            "Deleted episode via MCP (permanent)"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "message": "Episode deleted permanently"
        }))
    }

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
                        Some(memory_core::ExecutionResult::Success { .. }) => "success",
                        Some(memory_core::ExecutionResult::Error { .. }) => "error",
                        Some(memory_core::ExecutionResult::Timeout) => "timeout",
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
