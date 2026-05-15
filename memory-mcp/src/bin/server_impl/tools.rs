//! Memory tool handlers with audit logging
//!
//! This module contains individual tool handler functions for all MCP tools.
//! All security-relevant operations are logged to the audit logger.
//!
//! ## Audit Logging
//!
//! The following operations are logged:
//! - Episode creation/modification/deletion
//! - Relationship changes
//! - Configuration changes
//! - Authentication events
//! - Rate limit violations

use super::types::Content;
use do_memory_mcp::MemoryMCPServer;
use serde_json::Value;

mod episode_handlers;
mod external_signal_handlers;
mod feature_handlers;
mod memory_handlers;
mod relationship_handlers;
mod tag_handlers;

pub use episode_handlers::{
    handle_add_episode_step, handle_bulk_episodes, handle_complete_episode, handle_create_episode,
    handle_delete_episode, handle_get_episode, handle_get_episode_timeline, handle_update_episode,
};
pub use external_signal_handlers::{
    handle_configure_agentfs, handle_external_signal_status, handle_test_agentfs_connection,
};
pub use feature_handlers::{
    handle_checkpoint_episode, handle_explain_pattern, handle_get_handoff_pack,
    handle_get_recommendation_stats, handle_recommend_patterns, handle_recommend_playbook,
    handle_record_recommendation_feedback, handle_record_recommendation_session,
    handle_resume_from_handoff, handle_search_patterns,
};
pub use memory_handlers::{
    handle_advanced_pattern_analysis, handle_analyze_patterns, handle_configure_embeddings,
    handle_embedding_provider_status, handle_execute_code, handle_generate_embedding,
    handle_get_metrics, handle_health_check, handle_quality_metrics, handle_query_memory,
    handle_query_semantic_memory, handle_search_by_embedding, handle_test_embeddings,
};
pub use relationship_handlers::{
    handle_add_episode_relationship, handle_check_relationship_exists,
    handle_find_related_episodes, handle_get_dependency_graph, handle_get_episode_relationships,
    handle_get_topological_order, handle_remove_episode_relationship, handle_validate_no_cycles,
};
pub use tag_handlers::{
    handle_add_episode_tags, handle_get_episode_tags, handle_remove_episode_tags,
    handle_search_episodes_by_tags, handle_set_episode_tags,
};

/// Extract client ID from arguments or use default
pub(super) fn get_client_id(args: &Value) -> String {
    args.get("client_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "anonymous".to_string())
}

/// Get length from a JSON Value (array length or 0)
pub(super) fn json_value_len(value: &Value) -> usize {
    value.as_array().map(|a| a.len()).unwrap_or(0)
}

// All handler functions are now in submodules:
// - memory_handlers.rs: query_memory, execute_code, analyze_patterns, etc.
// - episode_handlers.rs: episode-related handlers
// - relationship_handlers.rs: relationship handlers
// - tag_handlers.rs: tag handlers
