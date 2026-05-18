//! Table definitions and constants for redb storage

use redb::TableDefinition;

// Table definitions are kept in lib.rs for this crate based on initial exploration
// But we can also define them here if preferred.
// Based on lib.rs content:

pub(crate) const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
pub(crate) const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
pub(crate) const HEURISTICS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("heuristics");
pub(crate) const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("embeddings");
pub(crate) const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
pub(crate) const SUMMARIES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("summaries");
pub(crate) const RELATIONSHIPS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("relationships");
pub(crate) const RECOMMENDATION_SESSIONS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_sessions");
pub(crate) const RECOMMENDATION_FEEDBACK_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_feedback");
pub(crate) const RECOMMENDATION_EPISODE_INDEX_TABLE: TableDefinition<&str, &str> =
    TableDefinition::new("recommendation_episode_index");

pub(crate) const PROCEDURAL_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("procedural");
