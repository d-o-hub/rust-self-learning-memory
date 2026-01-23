//! Episode creation tool for MCP server
//!
//! This module provides the tool for creating new episodes to track
//! task execution programmatically.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{TaskContext, TaskType};
use serde_json::{json, Value};
use tracing::debug;
use tracing::info;

/// Parse complexity string to enum
fn parse_complexity(s: &str) -> memory_core::ComplexityLevel {
    match s {
        "simple" => memory_core::ComplexityLevel::Simple,
        "moderate" => memory_core::ComplexityLevel::Moderate,
        "complex" => memory_core::ComplexityLevel::Complex,
        _ => memory_core::ComplexityLevel::Moderate,
    }
}

/// Parse task type string to enum
fn parse_task_type(task_type: &str) -> Result<TaskType> {
    match task_type {
        "code_generation" => Ok(TaskType::CodeGeneration),
        "debugging" => Ok(TaskType::Debugging),
        "refactoring" => Ok(TaskType::Refactoring),
        "testing" => Ok(TaskType::Testing),
        "analysis" => Ok(TaskType::Analysis),
        "documentation" => Ok(TaskType::Documentation),
        _ => Err(anyhow!(
            "Invalid task_type: {}. Must be one of: code_generation, debugging, refactoring, testing, analysis, documentation",
            task_type
        )),
    }
}

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
        let task_type = parse_task_type(task_type_str)?;

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
            .map(parse_complexity)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_type_valid() {
        assert!(parse_task_type("code_generation").is_ok());
        assert!(parse_task_type("debugging").is_ok());
        assert!(parse_task_type("refactoring").is_ok());
        assert!(parse_task_type("testing").is_ok());
        assert!(parse_task_type("analysis").is_ok());
        assert!(parse_task_type("documentation").is_ok());
    }

    #[test]
    fn test_parse_task_type_invalid() {
        assert!(parse_task_type("invalid").is_err());
        assert!(parse_task_type("").is_err());
    }

    #[test]
    fn test_parse_complexity() {
        assert_eq!(
            parse_complexity("simple"),
            memory_core::ComplexityLevel::Simple
        );
        assert_eq!(
            parse_complexity("moderate"),
            memory_core::ComplexityLevel::Moderate
        );
        assert_eq!(
            parse_complexity("complex"),
            memory_core::ComplexityLevel::Complex
        );
        assert_eq!(
            parse_complexity("unknown"),
            memory_core::ComplexityLevel::Moderate
        );
    }
}
