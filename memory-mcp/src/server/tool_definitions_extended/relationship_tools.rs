use crate::types::Tool;
use serde_json::json;

pub(super) fn append_episode_relationship_tools(tools: &mut Vec<Tool>) {
    tools.push(Tool::new(
        "add_episode_relationship".to_string(),
        "Add a relationship between two episodes with validation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "from_episode_id": {"type": "string", "description": "Source episode UUID", "format": "uuid"},
                "to_episode_id": {"type": "string", "description": "Target episode UUID", "format": "uuid"},
                "relationship_type": {
                    "type": "string",
                    "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"],
                    "description": "Type of relationship"
                },
                "reason": {"type": "string", "description": "Optional explanation"},
                "priority": {"type": "integer", "minimum": 1, "maximum": 10, "description": "Optional priority (1-10)"},
                "created_by": {"type": "string", "description": "Optional creator identifier"}
            },
            "required": ["from_episode_id", "to_episode_id", "relationship_type"]
        }),
    ));

    tools.push(Tool::new(
        "remove_episode_relationship".to_string(),
        "Remove a relationship by ID".to_string(),
        json!({
            "type": "object",
            "properties": {
                "relationship_id": {"type": "string", "format": "uuid", "description": "Relationship UUID to remove"}
            },
            "required": ["relationship_id"]
        }),
    ));

    tools.push(Tool::new(
        "get_episode_relationships".to_string(),
        "Get relationships for an episode".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {"type": "string", "format": "uuid", "description": "Episode UUID to query"},
                "direction": {"type": "string", "enum": ["outgoing", "incoming", "both"], "default": "both", "description": "Direction filter"},
                "relationship_type": {
                    "type": "string",
                    "enum": ["parent_child", "depends_on", "follows", "related_to", "blocks", "duplicates", "references"],
                    "description": "Optional relationship type filter"
                }
            },
            "required": ["episode_id"]
        }),
    ));

    tools.push(Tool::new(
        "find_related_episodes".to_string(),
        "Find episodes related to a given episode".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {"type": "string", "format": "uuid", "description": "Episode UUID to find relationships for"},
                "relationship_type": {"type": "string", "description": "Optional relationship type filter"},
                "limit": {"type": "integer", "minimum": 1, "default": 10, "description": "Maximum number of results"},
                "include_metadata": {"type": "boolean", "default": false, "description": "Whether to include relationship metadata"}
            },
            "required": ["episode_id"]
        }),
    ));

    tools.push(Tool::new(
        "check_relationship_exists".to_string(),
        "Check if a specific relationship exists".to_string(),
        json!({
            "type": "object",
            "properties": {
                "from_episode_id": {"type": "string", "format": "uuid", "description": "Source episode UUID"},
                "to_episode_id": {"type": "string", "format": "uuid", "description": "Target episode UUID"},
                "relationship_type": {"type": "string", "description": "Type of relationship to check"}
            },
            "required": ["from_episode_id", "to_episode_id", "relationship_type"]
        }),
    ));

    tools.push(Tool::new(
        "get_dependency_graph".to_string(),
        "Get relationship graph for visualization".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_id": {"type": "string", "format": "uuid", "description": "Root episode UUID"},
                "depth": {"type": "integer", "minimum": 1, "maximum": 5, "default": 2, "description": "Maximum traversal depth"},
                "format": {"type": "string", "enum": ["json", "dot"], "default": "json", "description": "Output format"}
            },
            "required": ["episode_id"]
        }),
    ));

    tools.push(Tool::new(
        "validate_no_cycles".to_string(),
        "Check if adding a relationship would create a cycle".to_string(),
        json!({
            "type": "object",
            "properties": {
                "from_episode_id": {"type": "string", "format": "uuid", "description": "Source episode UUID"},
                "to_episode_id": {"type": "string", "format": "uuid", "description": "Target episode UUID"},
                "relationship_type": {"type": "string", "description": "Type of relationship being added"}
            },
            "required": ["from_episode_id", "to_episode_id", "relationship_type"]
        }),
    ));

    tools.push(Tool::new(
        "get_topological_order".to_string(),
        "Get topological ordering of episodes".to_string(),
        json!({
            "type": "object",
            "properties": {
                "episode_ids": {
                    "type": "array",
                    "items": {"type": "string", "format": "uuid"},
                    "minItems": 1,
                    "description": "Array of episode UUIDs to sort"
                }
            },
            "required": ["episode_ids"]
        }),
    ));
}
