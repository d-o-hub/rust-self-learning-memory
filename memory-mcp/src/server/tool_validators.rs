//! Tool validation and relationship parameter schemas
//!
//! This module contains JSON schema definitions for tagging and relationship tools.
//! Relationship tools include validation-related functionality like cycle detection.

use serde_json::{Value, json};

// ============================================================================
// Tagging Parameter Schemas
// ============================================================================

/// Parameter schema for add_episode_tags tool
pub fn add_episode_tags_params() -> Value {
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
    })
}

/// Parameter schema for remove_episode_tags tool
pub fn remove_episode_tags_params() -> Value {
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
    })
}

/// Parameter schema for set_episode_tags tool
pub fn set_episode_tags_params() -> Value {
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
    })
}

/// Parameter schema for get_episode_tags tool
pub fn get_episode_tags_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "description": "Episode ID to get tags for"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for search_episodes_by_tags tool
pub fn search_episodes_by_tags_params() -> Value {
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
    })
}

// ============================================================================
// Relationship Parameter Schemas (with validation support)
// ============================================================================

/// Parameter schema for add_episode_relationship tool
pub fn add_episode_relationship_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from_episode_id": {
                "type": "string",
                "description": "Source episode UUID",
                "format": "uuid"
            },
            "to_episode_id": {
                "type": "string",
                "description": "Target episode UUID",
                "format": "uuid"
            },
            "relationship_type": {
                "type": "string",
                "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"],
                "description": "Type of relationship"
            },
            "reason": {
                "type": "string",
                "description": "Optional explanation"
            },
            "priority": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10,
                "description": "Optional priority (1-10)"
            },
            "created_by": {
                "type": "string",
                "description": "Optional creator identifier"
            }
        },
        "required": ["from_episode_id", "to_episode_id", "relationship_type"]
    })
}

/// Parameter schema for remove_episode_relationship tool
pub fn remove_episode_relationship_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "relationship_id": {
                "type": "string",
                "format": "uuid",
                "description": "Relationship UUID to remove"
            }
        },
        "required": ["relationship_id"]
    })
}

/// Parameter schema for get_episode_relationships tool
pub fn get_episode_relationships_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Episode UUID to query"
            },
            "direction": {
                "type": "string",
                "enum": ["outgoing", "incoming", "both"],
                "default": "both",
                "description": "Direction filter"
            },
            "relationship_type": {
                "type": "string",
                "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"],
                "description": "Optional relationship type filter"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for find_related_episodes tool
pub fn find_related_episodes_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Episode UUID to find relationships for"
            },
            "relationship_type": {
                "type": "string",
                "description": "Optional relationship type filter"
            },
            "limit": {
                "type": "integer",
                "minimum": 1,
                "default": 10,
                "description": "Maximum number of results"
            },
            "include_metadata": {
                "type": "boolean",
                "default": false,
                "description": "Whether to include relationship metadata"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for check_relationship_exists tool
pub fn check_relationship_exists_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from_episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Source episode UUID"
            },
            "to_episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Target episode UUID"
            },
            "relationship_type": {
                "type": "string",
                "description": "Type of relationship to check"
            }
        },
        "required": ["from_episode_id", "to_episode_id", "relationship_type"]
    })
}

/// Parameter schema for get_dependency_graph tool
pub fn get_dependency_graph_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Root episode UUID"
            },
            "depth": {
                "type": "integer",
                "minimum": 1,
                "maximum": 5,
                "default": 2,
                "description": "Maximum traversal depth"
            },
            "format": {
                "type": "string",
                "enum": ["json", "dot"],
                "default": "json",
                "description": "Output format"
            }
        },
        "required": ["episode_id"]
    })
}

/// Parameter schema for validate_no_cycles tool
pub fn validate_no_cycles_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from_episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Source episode UUID"
            },
            "to_episode_id": {
                "type": "string",
                "format": "uuid",
                "description": "Target episode UUID"
            },
            "relationship_type": {
                "type": "string",
                "description": "Type of relationship being added"
            }
        },
        "required": ["from_episode_id", "to_episode_id", "relationship_type"]
    })
}

/// Parameter schema for get_topological_order tool
pub fn get_topological_order_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_ids": {
                "type": "array",
                "items": {
                    "type": "string",
                    "format": "uuid"
                },
                "minItems": 1,
                "description": "Array of episode UUIDs to sort"
            }
        },
        "required": ["episode_ids"]
    })
}
