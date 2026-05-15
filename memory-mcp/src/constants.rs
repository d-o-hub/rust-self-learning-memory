//! Centralized constants for tool input bounds (CWE-770 prevention).
//!
//! These constants define safe minimum/maximum values for public MCP tool
//! parameters to prevent resource exhaustion. All handlers should use these
//! constants via `.clamp(MIN, MAX)` instead of hardcoded values.

// ──── Query / Memory limits ────

/// Minimum allowed limit for query_memory, analyze_patterns, etc.
pub const MIN_QUERY_LIMIT: usize = 1;

/// Maximum allowed limit for query_memory, analyze_patterns, etc.
pub const MAX_QUERY_LIMIT: usize = 1000;

/// Default limit for query_memory.
pub const DEFAULT_QUERY_LIMIT: usize = 10;

/// Default limit for analyze_patterns.
pub const DEFAULT_ANALYZE_LIMIT: usize = 20;

// ──── Search / Recommend limits ────

/// Maximum allowed limit for search_patterns.
pub const MAX_SEARCH_LIMIT: usize = 100;

/// Default limit for search_patterns.
pub const DEFAULT_SEARCH_LIMIT: usize = 5;

/// Maximum allowed limit for recommend_patterns.
pub const MAX_RECOMMEND_LIMIT: usize = 50;

/// Default limit for recommend_patterns.
pub const DEFAULT_RECOMMEND_LIMIT: usize = 3;

// ──── Playbook / Steps limits ────

/// Minimum allowed max_steps for recommend_playbook.
pub const MIN_PLAYBOOK_STEPS: usize = 1;

/// Maximum allowed max_steps for recommend_playbook.
pub const MAX_PLAYBOOK_STEPS: usize = 100;

/// Default max_steps for recommend_playbook.
pub const DEFAULT_PLAYBOOK_STEPS: usize = 5;

// ──── Episode tags limits ────

/// Minimum allowed limit for search_episodes_by_tags.
pub const MIN_TAG_SEARCH_LIMIT: usize = 1;

/// Maximum allowed limit for search_episodes_by_tags.
pub const MAX_TAG_SEARCH_LIMIT: usize = 1000;

/// Default limit for search_episodes_by_tags.
pub const DEFAULT_TAG_SEARCH_LIMIT: usize = 100;
