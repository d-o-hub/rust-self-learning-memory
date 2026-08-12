//! Shared recommendation tool definitions (ADR-080 attribution surface).
//!
//! `recommend_patterns` and `recommend_playbook` are constructed from these
//! shared constructors by every registry that advertises them
//! (`create_default_tools` and the lazy registry's
//! `create_additional_extended_tools`), so the overlapping tool schemas are
//! textually identical wherever they are exposed. Both schemas carry an
//! optional `episode_id` (never required) that opts the call into ADR-080
//! attribution tracking.

use crate::types::Tool;
use serde_json::json;

/// Attribution description shared verbatim by both recommendation tools.
///
/// Documents the attributed `attribution` envelope, its `session_id`, and the
/// four `PersistenceReceipt` discriminants so clients can reason about the
/// durability of a recorded recommendation session before calling.
const ATTRIBUTION_DESCRIPTION: &str = "Optional episode UUID (format: uuid) for ADR-080 attribution tracking. \
When supplied, the response includes an attribution envelope with a session_id and a persistence receipt \
whose state is one of: persisted, partially_persisted, memory_only, persistence_failed. \
When omitted, the legacy unattributed response shape is returned.";

/// Construct the `recommend_patterns` tool definition.
pub fn recommend_patterns_tool() -> Tool {
    Tool::new(
        "recommend_patterns".to_string(),
        "Get pattern recommendations for a specific task with high-quality filtering".to_string(),
        json!({
            "type": "object",
            "properties": {
                "task_description": {
                    "type": "string",
                    "description": "Description of the task you're working on"
                },
                "domain": {
                    "type": "string",
                    "description": "Domain of the task (e.g., 'web-api', 'cli')"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "maxItems": 100,
                    "description": "Optional context tags (max 100)",
                    "default": []
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of recommendations (default: 3)",
                    "default": 3,
                    "minimum": 1,
                    "maximum": 50
                },
                "episode_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": ATTRIBUTION_DESCRIPTION
                }
            },
            "required": ["task_description", "domain"]
        }),
    )
}

/// Construct the `recommend_playbook` tool definition.
pub fn recommend_playbook_tool() -> Tool {
    Tool::new(
        "recommend_playbook".to_string(),
        "Get an actionable playbook with step-by-step guidance for a task (ADR-044 Feature 1)"
            .to_string(),
        json!({
            "type": "object",
            "properties": {
                "task_description": {
                    "type": "string",
                    "description": "Description of the task to perform"
                },
                "domain": {
                    "type": "string",
                    "description": "Domain of the task (e.g., 'web-api', 'testing', 'data-processing')"
                },
                "task_type": {
                    "type": "string",
                    "enum": ["code_generation", "debugging", "refactoring", "testing", "analysis", "documentation"],
                    "description": "Type of task being performed",
                    "default": "code_generation"
                },
                "max_steps": {
                    "type": "integer",
                    "description": "Maximum number of steps to include (default: 5)",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 100
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (optional)",
                    "default": null
                },
                "framework": {
                    "type": "string",
                    "description": "Framework being used (optional)",
                    "default": null
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "maxItems": 100,
                    "description": "Additional context tags (max 100)",
                    "default": []
                },
                "episode_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": ATTRIBUTION_DESCRIPTION
                }
            },
            "required": ["task_description", "domain"]
        }),
    )
}
