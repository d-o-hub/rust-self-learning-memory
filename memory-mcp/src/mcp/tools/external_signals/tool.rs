//! External signal provider tool definitions.

use crate::types::Tool;
use serde_json::json;

/// Get the tool definition for configure_agentfs
pub fn configure_agentfs_tool() -> Tool {
    Tool::new(
        "configure_agentfs".to_string(),
        "Configure AgentFS external signal provider for reward system integration.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "db_path": {
                    "type": "string",
                    "description": "Path to the AgentFS database file"
                },
                "enabled": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether the provider is enabled"
                },
                "weight": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.3,
                    "description": "Weight for merging external signals with internal rewards (0.0-1.0)"
                },
                "min_samples": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 5,
                    "description": "Minimum samples required before using external signals"
                },
                "sanitize": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether to sanitize/summarize toolcall parameters for privacy"
                }
            },
            "required": ["db_path"]
        }),
    )
}

/// Get the tool definition for external_signal_status
pub fn external_signal_status_tool() -> Tool {
    Tool::new(
        "external_signal_status".to_string(),
        "Get status of configured external signal providers for the reward system.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string",
                    "description": "Optional provider name to filter by (e.g., 'agentfs')"
                }
            },
            "additionalProperties": false
        }),
    )
}

/// Get the tool definition for test_agentfs_connection
pub fn test_agentfs_connection_tool() -> Tool {
    Tool::new(
        "test_agentfs_connection".to_string(),
        "Test connectivity to the AgentFS external signal provider.".to_string(),
        json!({
            "type": "object",
            "properties": {
                "db_path": {
                    "type": "string",
                    "description": "Optional database path override (uses configured path if not provided)"
                }
            },
            "additionalProperties": false
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_agentfs_tool_definition() {
        let tool = configure_agentfs_tool();
        assert_eq!(tool.name, "configure_agentfs");
    }

    #[test]
    fn test_external_signal_status_tool_definition() {
        let tool = external_signal_status_tool();
        assert_eq!(tool.name, "external_signal_status");
    }

    #[test]
    fn test_test_agentfs_connection_tool_definition() {
        let tool = test_agentfs_connection_tool();
        assert_eq!(tool.name, "test_agentfs_connection");
    }
}
