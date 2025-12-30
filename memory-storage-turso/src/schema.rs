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
    heuristics TEXT NOT NULL DEFAULT '[]',
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

/// SQL to create the embeddings table with native vector support
///
/// Uses Turso's F32_BLOB(384) for native vector storage with DiskANN indexing.
/// The embedding_data column is kept for JSON serialization compatibility.
pub const CREATE_EMBEDDINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(384),
    dimension INTEGER NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;
/// SQL to create embeddings table for 384-dimension vectors
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_384_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_384 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(384),
    dimension INTEGER NOT NULL DEFAULT 384,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create embeddings table for 1024-dimension vectors
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1024_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_1024 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(1024),
    dimension INTEGER NOT NULL DEFAULT 1024,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create embeddings table for 1536-dimension vectors
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1536_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_1536 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(1536),
    dimension INTEGER NOT NULL DEFAULT 1536,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create embeddings table for 3072-dimension vectors
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_3072_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_3072 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(3072),
    dimension INTEGER NOT NULL DEFAULT 3072,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create embeddings table for other dimension vectors (no native vector support)
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_OTHER_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_other (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector BLOB,
    dimension INTEGER NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create DiskANN vector index for fast similarity search
///
/// This creates a specialized vector index that enables 10-100x faster
/// similarity search compared to brute-force scanning.
pub const CREATE_EMBEDDINGS_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_vector
ON embeddings(libsql_vector_idx(embedding_vector))
"#;
/// SQL to create DiskANN vector index for 384-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_384_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_384_vector
ON embeddings_384(libsql_vector_idx(embedding_vector))
"#;

/// SQL to create DiskANN vector index for 1024-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1024_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_1024_vector
ON embeddings_1024(libsql_vector_idx(embedding_vector))
"#;

/// SQL to create DiskANN vector index for 1536-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1536_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_1536_vector
ON embeddings_1536(libsql_vector_idx(embedding_vector))
"#;

/// SQL to create DiskANN vector index for 3072-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_3072_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_3072_vector
ON embeddings_3072(libsql_vector_idx(embedding_vector))
"#;

/// SQL to create item index for 384-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_384_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_384_item
ON embeddings_384(item_id, item_type)
"#;

/// SQL to create item index for 1024-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1024_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_1024_item
ON embeddings_1024(item_id, item_type)
"#;

/// SQL to create item index for 1536-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_1536_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_1536_item
ON embeddings_1536(item_id, item_type)
"#;

/// SQL to create item index for 3072-dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_3072_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_3072_item
ON embeddings_3072(item_id, item_type)
"#;

/// SQL to create item index for other dimension embeddings
#[allow(dead_code)]
pub const CREATE_EMBEDDINGS_OTHER_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_other_item
ON embeddings_other(item_id, item_type)
"#;

/// Index on embeddings for fast item lookups
pub const CREATE_EMBEDDINGS_ITEM_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_item
ON embeddings(item_id, item_type)
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

/// SQL to create the execution_records table for monitoring
pub const CREATE_EXECUTION_RECORDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS execution_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_name TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    duration_ms INTEGER NOT NULL,
    started_at INTEGER NOT NULL,
    task_description TEXT,
    error_message TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create the agent_metrics table for monitoring
pub const CREATE_AGENT_METRICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS agent_metrics (
    agent_name TEXT PRIMARY KEY NOT NULL,
    agent_type TEXT NOT NULL,
    total_executions INTEGER NOT NULL DEFAULT 0,
    successful_executions INTEGER NOT NULL DEFAULT 0,
    total_duration_ms INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms INTEGER NOT NULL DEFAULT 0,
    min_duration_ms INTEGER NOT NULL DEFAULT 0,
    max_duration_ms INTEGER NOT NULL DEFAULT 0,
    last_execution INTEGER,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// SQL to create the task_metrics table for monitoring
pub const CREATE_TASK_METRICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS task_metrics (
    task_type TEXT PRIMARY KEY NOT NULL,
    total_tasks INTEGER NOT NULL DEFAULT 0,
    completed_tasks INTEGER NOT NULL DEFAULT 0,
    avg_completion_time_ms INTEGER NOT NULL DEFAULT 0,
    agent_success_rates TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// Index on execution_records for time-based queries
pub const CREATE_EXECUTION_RECORDS_TIME_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_execution_records_time
ON execution_records(started_at DESC)
"#;

/// Index on execution_records for agent-based queries
pub const CREATE_EXECUTION_RECORDS_AGENT_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_execution_records_agent
ON execution_records(agent_name, started_at DESC)
"#;

/// Index on agent_metrics for type-based queries
pub const CREATE_AGENT_METRICS_TYPE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_agent_metrics_type
ON agent_metrics(agent_type)
"#;

// ======= Phase 2 (GENESIS) Schema =======

/// SQL to create the episode_summaries table
///
/// Stores semantic summaries for episodes with optional embeddings.
/// Summaries are CASCADE deleted when episodes are removed.
pub const CREATE_EPISODE_SUMMARIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS episode_summaries (
    episode_id TEXT PRIMARY KEY NOT NULL,
    summary_text TEXT NOT NULL,
    key_concepts TEXT NOT NULL,
    key_steps TEXT NOT NULL,
    summary_embedding BLOB,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
)
"#;

/// Index on episode_summaries for time-based queries
pub const CREATE_SUMMARIES_CREATED_AT_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_summaries_created_at
ON episode_summaries(created_at)
"#;

/// SQL to create the metadata table for capacity management
///
/// Stores configuration and runtime metadata like episode counts.
pub const CREATE_METADATA_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;
