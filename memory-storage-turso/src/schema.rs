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
