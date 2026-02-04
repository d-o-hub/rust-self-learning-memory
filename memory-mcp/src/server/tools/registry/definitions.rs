//! Tool definitions for the lazy-loading registry
//!
//! This module defines the core and extended tools for the MCP server.

use crate::server::tool_definitions_extended;
use crate::types::Tool;
use serde_json::json;
use std::collections::HashMap;

/// Create the default tool registry with core and extended tools
pub fn create_default_registry() -> super::ToolRegistry {
    // Core tools: essential for basic operations
    let core_tools = create_core_tools();

    // All extended tools from existing tool definitions
    let extended_defs = tool_definitions_extended::create_extended_tools();
    let mut extended_tools = HashMap::new();

    // Convert Vec<Tool> to HashMap
    for tool in extended_defs {
        // Skip core tools that are already defined above
        if !core_tools.iter().any(|t| t.name == tool.name) {
            extended_tools.insert(tool.name.clone(), tool);
        }
    }

    super::ToolRegistry::new(core_tools, extended_tools)
}

/// Create core tools that are always loaded
fn create_core_tools() -> Vec<Tool> {
    vec![
        // Memory query
        Tool::new(
            "query_memory".to_string(),
            "Query episodic memory for relevant past experiences and learned patterns".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query describing the task or context"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Task domain (e.g., 'web-api', 'data-processing')"
                    },
                    "task_type": {
                        "type": "string",
                        "enum": ["code_generation", "debugging", "refactoring", "testing", "analysis", "documentation"],
                        "description": "Type of task being performed"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of episodes to retrieve"
                    },
                    "sort": {
                        "type": "string",
                        "enum": ["relevance", "newest", "oldest", "duration", "success"],
                        "default": "relevance",
                        "description": "Sort order for results"
                    },
                    "fields": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Fields to return (e.g., ['episodes.id', 'episodes.task_description', 'patterns.success_rate'])",
                        "default": null
                    }
                },
                "required": ["query", "domain"]
            }),
        ),
        // Health and monitoring
        Tool::new(
            "health_check".to_string(),
            "Check the health status of the MCP server and its components".to_string(),
            json!({"type": "object", "properties": {}}),
        ),
        Tool::new(
            "get_metrics".to_string(),
            "Get comprehensive monitoring metrics and statistics".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "metric_type": {
                        "type": "string",
                        "enum": ["all", "performance", "episodes", "system"],
                        "default": "all",
                        "description": "Type of metrics to retrieve"
                    }
                }
            }),
        ),
        // Core pattern analysis
        Tool::new(
            "analyze_patterns".to_string(),
            "Analyze patterns from past episodes to identify successful strategies".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "task_type": {
                        "type": "string",
                        "description": "Type of task to analyze patterns for"
                    },
                    "min_success_rate": {
                        "type": "number",
                        "default": 0.7,
                        "description": "Minimum success rate for patterns (0.0-1.0)"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 20,
                        "description": "Maximum number of patterns to return"
                    },
                    "fields": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Fields to return (e.g., ['patterns.tool_sequence', 'statistics.most_common_tools'])",
                        "default": null
                    }
                },
                "required": ["task_type"]
            }),
        ),
        // Episode lifecycle
        Tool::new(
            "create_episode".to_string(),
            "Create a new episode to track task execution programmatically".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "task_description": {
                        "type": "string",
                        "description": "Clear description of the task to be performed"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Task domain (e.g., 'web-api', 'cli', 'data-processing')"
                    },
                    "task_type": {
                        "type": "string",
                        "enum": ["code_generation", "debugging", "refactoring", "testing", "analysis", "documentation"],
                        "description": "Type of task being performed"
                    },
                    "complexity": {
                        "type": "string",
                        "enum": ["simple", "moderate", "complex"],
                        "default": "moderate",
                        "description": "Task complexity level"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional context tags"
                    }
                },
                "required": ["task_description", "domain", "task_type"]
            }),
        ),
        Tool::new(
            "add_episode_step".to_string(),
            "Add an execution step to an ongoing episode to track progress".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode",
                        "format": "uuid"
                    },
                    "step_number": {
                        "type": "integer",
                        "description": "Sequential step number"
                    },
                    "tool": {
                        "type": "string",
                        "description": "Name of the tool/component performing the action"
                    },
                    "action": {
                        "type": "string",
                        "description": "Description of the action taken"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Optional parameters used in this step"
                    },
                    "result": {
                        "type": "object",
                        "description": "Optional result of the step"
                    }
                },
                "required": ["episode_id", "step_number", "tool", "action"]
            }),
        ),
        Tool::new(
            "complete_episode".to_string(),
            "Complete an episode with an outcome and trigger the learning cycle".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode",
                        "format": "uuid"
                    },
                    "outcome_type": {
                        "type": "string",
                        "enum": ["success", "partial_success", "failure"],
                        "description": "Type of outcome"
                    },
                    "verdict": {
                        "type": "string",
                        "description": "Description of the outcome (required for success/partial_success)"
                    },
                    "completed": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of completed items (required for partial_success)"
                    },
                    "failed": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of failed items (required for partial_success)"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Failure reason (required for failure)"
                    },
                    "artifacts": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of artifact names (optional, for success)"
                    }
                },
                "required": ["episode_id", "outcome_type"]
            }),
        ),
        // Get episode details
        Tool::new(
            "get_episode".to_string(),
            "Get complete details of an episode including steps, outcome, reflection, and patterns"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode to retrieve",
                        "format": "uuid"
                    },
                    "fields": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Fields to return (e.g., ['episode.id', 'episode.task_description', 'episode.outcome'])",
                        "default": null
                    }
                },
                "required": ["episode_id"]
            }),
        ),
    ]
}
