//! Extended tool definitions for the MCP server
//!
//! This module contains additional tool definitions for episodes and batch operations.

use crate::types::Tool;
use serde_json::json;

/// Create the extended tool definitions (episodes and batch operations).
///
/// This function defines tools for episode lifecycle management and batch operations.
pub fn create_extended_tools() -> Vec<Tool> {
    let mut tools = Vec::new();

    // Bulk episode retrieval tool
    tools.push(Tool::new(
        "bulk_episodes".to_string(),
        "Retrieve multiple episodes by their IDs in a single efficient operation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_ids": {
                    "type": "array",
                    "items": {"type": "string", "description": "Episode UUIDs"},
                    "description": "Array of episode IDs to retrieve"
                }
            },
            "required": ["episode_ids"]
        }),
    ));

    // Episode lifecycle management tools - create
    tools.push(Tool::new(
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
                "language": {
                    "type": "string",
                    "description": "Programming language (optional)"
                },
                "framework": {
                    "type": "string",
                    "description": "Framework name (optional)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional context tags"
                },
                "complexity": {
                    "type": "string",
                    "enum": ["simple", "moderate", "complex"],
                    "description": "Task complexity level (optional, default: moderate)"
                }
            },
            "required": ["task_description", "domain", "task_type"]
        }),
    ));

    // Episode lifecycle management tools - add step
    tools.push(Tool::new(
        "add_episode_step".to_string(),
        "Add an execution step to an ongoing episode to track progress".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "UUID of the episode"
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
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["success", "error", "timeout"],
                            "description": "Result type"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output message (for success)"
                        },
                        "message": {
                            "type": "string",
                            "description": "Error message (for error)"
                        }
                    },
                    "description": "Optional result of the step"
                },
                "latency_ms": {
                    "type": "integer",
                    "description": "Optional execution time in milliseconds"
                }
            },
            "required": ["episode_id", "step_number", "tool", "action"]
        }),
    ));

    // Episode lifecycle management tools - complete
    tools.push(Tool::new(
        "complete_episode".to_string(),
        "Complete an episode with an outcome and trigger the learning cycle (reward, reflection, patterns)".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "UUID of the episode to complete"
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
                "artifacts": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Array of artifact names (optional, for success)"
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
                "error_details": {
                    "type": "string",
                    "description": "Detailed error information (optional, for failure)"
                }
            },
            "required": ["episode_id", "outcome_type"]
        }),
    ));

    // Episode lifecycle management tools - get
    tools.push(Tool::new(
        "get_episode".to_string(),
        "Get complete details of an episode including steps, outcome, reflection, and patterns"
            .to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "UUID of the episode to retrieve"
                }
            },
            "required": ["episode_id"]
        }),
    ));

    // Episode lifecycle management tools - delete
    tools.push(Tool::new(
        "delete_episode".to_string(),
        "Delete an episode permanently from all storage backends (requires confirmation)"
            .to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "UUID of the episode to delete"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Must be set to true to confirm deletion"
                }
            },
            "required": ["episode_id", "confirm"]
        }),
    ));

    // Episode lifecycle management tools - timeline
    tools.push(Tool::new(
        "get_episode_timeline".to_string(),
        "Get a chronological timeline view of all steps in an episode".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "UUID of the episode"
                }
            },
            "required": ["episode_id"]
        }),
    ));

    // Advanced batch operation tools - query
    tools.push(Tool::new(
        "batch_query_episodes".to_string(),
        "Efficiently query multiple episodes with filtering, aggregation, and optional inclusion of steps/patterns/reflections".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional specific episode UUIDs to retrieve"
                },
                "filter": {
                    "type": "object",
                    "properties": {
                        "domain": {"type": "string"},
                        "task_type": {"type": "string"},
                        "tags": {"type": "array", "items": {"type": "string"}},
                        "success_only": {"type": "boolean"}
                    },
                    "description": "Optional filter criteria"
                },
                "include_steps": {
                    "type": "boolean",
                    "description": "Include execution steps (default: true)"
                },
                "include_patterns": {
                    "type": "boolean",
                    "description": "Include extracted patterns (default: false)"
                },
                "include_reflections": {
                    "type": "boolean",
                    "description": "Include reflections (default: false)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum episodes to return (default: 100, max: 1000)"
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset (default: 0)"
                },
                "aggregate_stats": {
                    "type": "boolean",
                    "description": "Compute aggregate statistics (default: true)"
                }
            }
        }),
    ));

    // Advanced batch operation tools - pattern analysis
    tools.push(Tool::new(
        "batch_pattern_analysis".to_string(),
        "Analyze patterns across multiple episodes to identify successful approaches and anti-patterns".to_string(),
        json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "Task domain to analyze (required)"
                },
                "task_type": {
                    "type": "string",
                    "description": "Optional specific task type"
                },
                "time_range": {
                    "type": "object",
                    "properties": {
                        "start": {"type": "string", "description": "ISO8601 start date"},
                        "end": {"type": "string", "description": "ISO8601 end date"}
                    },
                    "description": "Optional time range filter"
                },
                "min_episodes": {
                    "type": "integer",
                    "description": "Minimum episodes a pattern must appear in (default: 3)"
                },
                "min_success_rate": {
                    "type": "number",
                    "description": "Minimum success rate 0.0-1.0 (default: 0.6)"
                },
                "include_anti_patterns": {
                    "type": "boolean",
                    "description": "Include patterns with low success (default: true)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum patterns to return (default: 50)"
                }
            },
            "required": ["domain"]
        }),
    ));

    // Advanced batch operation tools - compare
    tools.push(Tool::new(
        "batch_compare_episodes".to_string(),
        "Compare multiple episodes to identify differences in approach, performance, and outcomes".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Array of episode UUIDs to compare (2-10 episodes)"
                },
                "compare_metrics": {
                    "type": "array",
                    "items": {"type": "string", "enum": ["duration", "reward_score", "step_count", "efficiency"]},
                    "description": "Metrics to compare (default: duration, reward_score, step_count)"
                },
                "compare_approaches": {
                    "type": "boolean",
                    "description": "Compare execution approaches (default: true)"
                },
                "generate_insights": {
                    "type": "boolean",
                    "description": "Generate AI insights (default: true)"
                }
            },
            "required": ["episode_ids"]
        }),
    ));

    // Episode tagging tools - add tags
    tools.push(Tool::new(
        "add_episode_tags".to_string(),
        "Add tags to an episode. Tags are validated and normalized (lowercase, trimmed). Duplicate tags are ignored.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "Episode ID to add tags to"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags to add"
                }
            },
            "required": ["episode_id", "tags"]
        }),
    ));

    // Episode tagging tools - remove tags
    tools.push(Tool::new(
        "remove_episode_tags".to_string(),
        "Remove tags from an episode. Non-existent tags are silently ignored. Matching is case-insensitive.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "Episode ID to remove tags from"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags to remove"
                }
            },
            "required": ["episode_id", "tags"]
        }),
    ));

    // Episode tagging tools - set tags
    tools.push(Tool::new(
        "set_episode_tags".to_string(),
        "Set/replace all tags on an episode. Useful for complete tag reorganization. Empty tag list will clear all tags.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "Episode ID to set tags on"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "New tags to set (replaces all existing)"
                }
            },
            "required": ["episode_id", "tags"]
        }),
    ));

    // Episode tagging tools - get tags
    tools.push(Tool::new(
        "get_episode_tags".to_string(),
        "Get tags for an episode".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {
                    "type": "string",
                    "description": "Episode ID to get tags for"
                }
            },
            "required": ["episode_id"]
        }),
    ));

    // Episode tagging tools - search by tags
    tools.push(Tool::new(
        "search_episodes_by_tags".to_string(),
        "Search episodes by tags using AND or OR logic. Matching is case-insensitive.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags to search for"
                },
                "require_all": {
                    "type": "boolean",
                    "description": "Whether to require all tags (AND) or any tag (OR). Default: false (OR)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results. Default: 100"
                }
            },
            "required": ["tags"]
        }),
    ));

    tools
}
