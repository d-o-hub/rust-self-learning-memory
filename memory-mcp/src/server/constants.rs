//! MCP server constants

/// Minimum limit for query operations
pub const MIN_QUERY_LIMIT: usize = 1;
/// Maximum limit for query operations (prevents resource exhaustion)
pub const MAX_QUERY_LIMIT: usize = do_memory_core::MAX_QUERY_LIMIT;

/// Minimum success rate for patterns
pub const MIN_SUCCESS_RATE: f32 = 0.0;
/// Maximum success rate for patterns
pub const MAX_SUCCESS_RATE: f32 = 1.0;
