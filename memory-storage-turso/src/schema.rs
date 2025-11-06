//! Database schema definitions for Turso storage

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
    metadata TEXT NOT NULL,
    domain TEXT NOT NULL,
    language TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create the patterns table
pub const CREATE_PATTERNS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS patterns (
    pattern_id TEXT PRIMARY KEY NOT NULL,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL,
    success_rate REAL NOT NULL,
    context_domain TEXT,
    context_language TEXT,
    context_tags TEXT,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create the heuristics table
pub const CREATE_HEURISTICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS heuristics (
    heuristic_id TEXT PRIMARY KEY NOT NULL,
    condition_text TEXT NOT NULL,
    action_text TEXT NOT NULL,
    confidence REAL NOT NULL,
    evidence TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// Index on episodes task_type for fast filtering
pub const CREATE_EPISODES_TASK_TYPE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_episodes_task_type
ON episodes(task_type)
"#;

/// Index on episodes timestamp for chronological queries
pub const CREATE_EPISODES_TIMESTAMP_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_episodes_start_time
ON episodes(start_time DESC)
"#;

/// Index on episodes domain for context-based retrieval
pub const CREATE_EPISODES_DOMAIN_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_episodes_domain
ON episodes(domain)
"#;

/// Index on patterns context for relevance matching
pub const CREATE_PATTERNS_CONTEXT_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_patterns_context
ON patterns(context_domain, context_language)
"#;

/// Index on heuristics confidence for quality filtering
pub const CREATE_HEURISTICS_CONFIDENCE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_heuristics_confidence
ON heuristics(confidence DESC)
"#;
