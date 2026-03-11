//! Tool parameter schema definitions
//!
//! This module contains JSON schema definitions for tool parameters.

use serde_json::{Value, json};

// ============================================================================
// Episode Lifecycle Parameter Schemas
// ============================================================================

/// Parameter schema for bulk_episodes tool
pub fn bulk_episodes_params() -> Value {
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
    })
}

/// Parameter schema for create_episode tool
pub fn create_episode_params() -> Value {
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
    })
}

/// Parameter schema for add_episode_step tool
pub fn add_episode_step_params() -> Value {
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
    })
}

/// Parameter schema for complete_episode tool
pub fn complete_episode_params() -> Value {
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
    })
}

/// Parameter schema for get_episode tool
pub fn get_episode_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "description": "UUID of the episode to retrieve"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for delete_episode tool
pub fn delete_episode_params() -> Value {
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
    })
}

/// Parameter schema for update_episode tool
pub fn update_episode_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "description": "UUID of the episode to update"
            },
            "description": {
                "type": "string",
                "description": "New task description (optional)"
            },
            "add_tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Tags to add to the episode (optional)"
            },
            "remove_tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Tags to remove from the episode (optional)"
            },
            "set_tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Replace all existing tags with these (optional)"
            },
            "metadata": {
                "type": "object",
                "additionalProperties": {"type": "string"},
                "description": "Metadata key-value pairs to merge (optional)"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for get_episode_timeline tool
pub fn get_episode_timeline_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "description": "UUID of the episode"
            }
        },
        "required": ["episode_id"]
    })
