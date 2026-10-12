//! Episodes-table schema constants (table, columns, modification revisions).
//!
//! Split out of `schema/mod.rs` to keep every source file within the LOC budget.

/// SQL to create the episodes table
pub const CREATE_EPISODES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS episodes (
    episode_id TEXT PRIMARY KEY NOT NULL,
    task_type TEXT NOT NULL,
    task_description TEXT NOT NULL,
    context TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    steps TEXT NOT NULL,
    outcome TEXT,
    reward TEXT,
    reflection TEXT,
    patterns TEXT NOT NULL,
    heuristics TEXT NOT NULL DEFAULT '[]',
    checkpoints TEXT NOT NULL DEFAULT '[]',
    metadata TEXT NOT NULL,
    domain TEXT NOT NULL,
    language TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    archived_at INTEGER
)
"#;

/// Migration SQL to add checkpoints column to existing episodes table.
pub const ADD_EPISODES_CHECKPOINTS_COLUMN: &str =
    "ALTER TABLE episodes ADD COLUMN checkpoints TEXT NOT NULL DEFAULT '[]'";

/// SQL to create the episode modification revision table.
///
/// The postcard-serialized `Episode` cannot gain a `modified_at` field without
/// breaking decoding of existing rows (postcard is positional), so the
/// modification watermark lives in this side table instead. It is keyed by
/// episode id and rewritten by every episode write.
pub const CREATE_EPISODE_REVISIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS episode_revisions (
    episode_id TEXT PRIMARY KEY,
    modified_at_ms INTEGER NOT NULL
);
"#;

/// Index on episode_revisions for `(modified_at, episode_id)` keyset pagination.
pub const CREATE_EPISODE_REVISIONS_WATERMARK_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_episode_revisions_watermark
ON episode_revisions (modified_at_ms, episode_id);
"#;
