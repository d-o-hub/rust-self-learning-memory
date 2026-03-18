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
use memory_mcp::MemoryMCPServer;
use serde_json::Value;

mod episode_handlers;
mod feature_handlers;
mod memory_handlers;
mod relationship_handlers;
mod tag_handlers;

pub use episode_handlers::*;
pub use feature_handlers::*;
pub use memory_handlers::*;
pub use relationship_handlers::*;
pub use tag_handlers::*;

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
