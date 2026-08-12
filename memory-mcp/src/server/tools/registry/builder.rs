//! Additional extended tool definitions for the registry.
//!
//! Core tools are defined in `builder_core.rs`. Both files extracted
//! from `definitions.rs` to maintain the ≤500 LOC invariant.

use crate::types::Tool;
use serde_json::json;

/// Create additional extended tools from tool_definitions.rs
/// These tools are loaded on-demand and not in the core set
pub(super) fn create_additional_extended_tools() -> Vec<Tool> {
    vec![
        // Advanced pattern analysis tool
        crate::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisTool::tool_definition(
        ),
        // Quality metrics tool
        crate::mcp::tools::quality_metrics::QualityMetricsTool::tool_definition(),
        // Embedding configuration and query tools
        crate::mcp::tools::embeddings::configure_embeddings_tool(),
        crate::mcp::tools::embeddings::query_semantic_memory_tool(),
        crate::mcp::tools::embeddings::test_embeddings_tool(),
        // New embedding tools
        crate::mcp::tools::embeddings::generate_embedding_tool(),
        crate::mcp::tools::embeddings::search_by_embedding_tool(),
        crate::mcp::tools::embeddings::embedding_provider_status_tool(),
        // External signal provider tools
        crate::mcp::tools::external_signals::configure_agentfs_tool(),
        crate::mcp::tools::external_signals::external_signal_status_tool(),
        crate::mcp::tools::external_signals::test_agentfs_connection_tool(),
        // Pattern search tool
        Tool::new(
            "search_patterns".to_string(),
            "Search for patterns semantically similar to a query using multi-signal ranking"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query describing what pattern to search for"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Domain to search in (e.g., 'web-api', 'cli', 'data-processing')"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "maxItems": 100,
                        "description": "Optional tags for filtering (max 100)",
                        "default": []
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 5)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 100
                    },
                    "min_relevance": {
                        "type": "number",
                        "description": "Minimum relevance score 0.0-1.0 (default: 0.3)",
                        "default": 0.3
                    },
                    "filter_by_domain": {
                        "type": "boolean",
                        "description": "Whether to filter by domain (default: false)",
                        "default": false
                    }
                },
                "required": ["query", "domain"]
            }),
        ),
        // Pattern recommendation tools (shared constructors so both registries
        // expose textually identical schemas, ADR-080 attribution surface)
        crate::server::recommendation_tool_definitions::recommend_patterns_tool(),
        // ADR-044 Feature 1: Playbook recommendation tool
        crate::server::recommendation_tool_definitions::recommend_playbook_tool(),
        // ADR-044 Feature 1: Pattern explanation tool
        Tool::new(
            "explain_pattern".to_string(),
            "Get a human-readable explanation of a pattern including when to use it and expected outcomes"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "pattern_id": {
                        "type": "string",
                        "description": "UUID of the pattern to explain",
                        "format": "uuid"
                    }
                },
                "required": ["pattern_id"]
            }),
        ),
        // ADR-044 Feature 2: Recommendation feedback tools
        Tool::new(
            "record_recommendation_session".to_string(),
            "Record a recommendation session when patterns/playbooks are suggested to an agent"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "Episode ID for which recommendations are made"
                    },
                    "recommended_pattern_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "maxItems": 1000,
                        "description": "Pattern IDs that were recommended (max 1000)",
                        "default": []
                    },
                    "recommended_playbook_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "maxItems": 1000,
                        "description": "Playbook IDs that were recommended (max 1000)",
                        "default": []
                    }
                },
                "required": ["episode_id"]
            }),
        ),
        Tool::new(
            "record_recommendation_feedback".to_string(),
            "Record feedback about which recommendations were used and the outcome".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Session ID from the recommendation session"
                    },
                    "applied_pattern_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "maxItems": 1000,
                        "description": "Pattern IDs that were actually applied (max 1000)",
                        "default": []
                    },
                    "consulted_episode_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "maxItems": 1000,
                        "description": "Episode IDs that were consulted (max 1000)",
                        "default": []
                    },
                    "outcome": {
                        "type": "object",
                        "description": "Final outcome of the task",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["success", "partial_success", "failure"]
                            },
                            "verdict": {"type": "string"},
                            "reason": {"type": "string"},
                            "artifacts": {
                                "type": "array",
                                "items": {"type": "string"},
                                "default": []
                            }
                        },
                        "required": ["type"]
                    },
                    "agent_rating": {
                        "type": "number",
                        "description": "Optional rating of recommendation quality (0.0-1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    }
                },
                "required": ["session_id", "outcome"]
            }),
        ),
        Tool::new(
            "get_recommendation_stats".to_string(),
            "Get statistics about recommendation effectiveness and adoption rates".to_string(),
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        // ADR-044 Feature 3: Checkpoint tools
        crate::mcp::tools::checkpoint::CheckpointTools::checkpoint_episode_tool(),
        crate::mcp::tools::checkpoint::CheckpointTools::get_handoff_pack_tool(),
        crate::mcp::tools::checkpoint::CheckpointTools::resume_from_handoff_tool(),
    ]
}
