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

/// Maximum allowed number of fields for memory queries.
pub const MAX_QUERY_FIELDS: usize = 20;

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

// ──── Bulk operations limits ────

/// Maximum allowed number of episode IDs in a single bulk request.
pub const MAX_BULK_EPISODE_IDS: usize = 100;

/// Maximum allowed number of tags in a single tags operation (add/remove/set).
pub const MAX_TAGS_PER_OPERATION: usize = 100;

// ──── Task description limits ────

/// Maximum allowed length (in bytes) for task descriptions.
pub const MAX_TASK_DESCRIPTION_LEN: usize = 10_000;

// ──── Dependency graph limits ────

/// Minimum allowed depth for dependency graph traversal.
pub const MIN_DEPTH: usize = 1;

/// Maximum allowed depth for dependency graph traversal.
pub const MAX_DEPTH: usize = 10;

/// Default depth for dependency graph traversal.
pub const DEFAULT_DEPTH: usize = 2;

/// Maximum allowed limit for find_related_episodes.
pub const MAX_FIND_RELATED_LIMIT: usize = 100;

/// Default limit for find_related_episodes.
pub const DEFAULT_FIND_RELATED_LIMIT: usize = 10;

// ──── Embedding limits ────

/// Maximum allowed length (in bytes) for text sent to generate_embedding.
pub const MAX_EMBEDDING_TEXT_LEN: usize = 50_000;

// ──── Checkpoint limits ────

/// Maximum allowed length for checkpoint reason.
pub const MAX_CHECKPOINT_REASON_LEN: usize = 1000;

/// Maximum allowed length for checkpoint note.
pub const MAX_CHECKPOINT_NOTE_LEN: usize = 5000;

// ──── Recommendation feedback limits ────

/// Maximum allowed number of recommended pattern/playbook IDs.
pub const MAX_RECOMMENDED_IDS: usize = 1000;

/// Minimum rating for agent recommendation feedback.
pub const MIN_AGENT_RATING: f32 = 0.0;

/// Maximum rating for agent recommendation feedback.
pub const MAX_AGENT_RATING: f32 = 1.0;

// ──── External signal provider limits ────

/// Minimum allowed weight for external signal merging.
pub const MIN_EXTERNAL_WEIGHT: f32 = 0.0;

/// Maximum allowed weight for external signal merging.
pub const MAX_EXTERNAL_WEIGHT: f32 = 1.0;

/// Minimum allowed min_samples for external signal configuration.
pub const MIN_EXTERNAL_SAMPLES: usize = 1;

/// Maximum allowed min_samples for external signal configuration.
pub const MAX_EXTERNAL_SAMPLES: usize = 10_000;

/// Maximum allowed length for AgentFS database path.
pub const MAX_DB_PATH_LEN: usize = 4096;

// ──── Advanced analysis limits ────

/// Maximum allowed number of variables in a time series analysis.
pub const MAX_SERIES_VARS: usize = 10;

/// Maximum allowed data points per variable in a time series.
pub const MAX_SERIES_POINTS: usize = 1000;

/// Maximum allowed length for metric type identifier.
pub const MAX_METRICS_TYPE_LEN: usize = 100;

/// Truncate a string to a maximum length in a UTF-8 safe manner.
pub fn truncate_safe(s: &mut String, max: usize) {
    if s.len() > max {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_truncate_safe() {
        let mut s = "a🚀b".to_string();
        truncate_safe(&mut s, 3);
        assert_eq!(s, "a");
        let mut s = "hi".to_string();
        truncate_safe(&mut s, 10);
        assert_eq!(s, "hi");
    }
}
