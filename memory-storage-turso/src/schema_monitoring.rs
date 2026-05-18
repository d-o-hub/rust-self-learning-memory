//! Monitoring-related database schema definitions.
//!
//! Extracted from schema.rs to keep file sizes ≤500 LOC.

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
