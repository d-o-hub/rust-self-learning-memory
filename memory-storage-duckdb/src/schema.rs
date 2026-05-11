//! DuckDB schema definitions for episodic memory storage.

/// SQL to create the episodes table.
pub const CREATE_EPISODES_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS episodes (
    episode_id UUID PRIMARY KEY,
    task_type VARCHAR NOT NULL,
    task_description TEXT NOT NULL,
    context JSON NOT NULL,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    steps JSON NOT NULL,
    outcome JSON,
    reward JSON,
    reflection JSON,
    patterns JSON NOT NULL,
    heuristics JSON NOT NULL,
    checkpoints JSON NOT NULL,
    metadata JSON NOT NULL,
    domain VARCHAR NOT NULL,
    language VARCHAR,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP WITH TIME ZONE
)
";

/// SQL to create the patterns table.
pub const CREATE_PATTERNS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS patterns (
    pattern_id VARCHAR PRIMARY KEY,
    pattern_type VARCHAR NOT NULL,
    pattern_data JSON NOT NULL,
    success_rate DOUBLE NOT NULL,
    context_domain VARCHAR,
    context_language VARCHAR,
    context_tags VARCHAR[],
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the heuristics table.
pub const CREATE_HEURISTICS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS heuristics (
    heuristic_id UUID PRIMARY KEY,
    condition_text TEXT NOT NULL,
    action_text TEXT NOT NULL,
    confidence DOUBLE NOT NULL,
    evidence JSON NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create recommendation sessions table.
pub const CREATE_RECOMMENDATION_SESSIONS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS recommendation_sessions (
    session_id UUID PRIMARY KEY,
    episode_id UUID NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    payload JSON NOT NULL
)
";

/// SQL to create recommendation feedback table.
pub const CREATE_RECOMMENDATION_FEEDBACK_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS recommendation_feedback (
    session_id UUID PRIMARY KEY,
    payload JSON NOT NULL
)
";

/// SQL to create the embeddings table with native vector support.
pub const CREATE_EMBEDDINGS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS embeddings (
    embedding_id VARCHAR PRIMARY KEY,
    item_id VARCHAR NOT NULL,
    item_type VARCHAR NOT NULL,
    embedding_data JSON NOT NULL,
    embedding_vector FLOAT[],
    dimension INTEGER NOT NULL,
    model VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the `episode_relationships` table.
pub const CREATE_EPISODE_RELATIONSHIPS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS episode_relationships (
    relationship_id UUID PRIMARY KEY,
    from_episode_id UUID NOT NULL,
    to_episode_id UUID NOT NULL,
    relationship_type VARCHAR NOT NULL,
    reason TEXT,
    created_by VARCHAR,
    priority INTEGER,
    metadata JSON NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the sequence for `execution_records`.
pub const CREATE_EXECUTION_RECORDS_SEQUENCE: &str = "CREATE SEQUENCE IF NOT EXISTS seq_execution_records_id";

/// SQL to create the `execution_records` table for monitoring.
pub const CREATE_EXECUTION_RECORDS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS execution_records (
    id INTEGER PRIMARY KEY DEFAULT nextval('seq_execution_records_id'),
    agent_name VARCHAR NOT NULL,
    agent_type VARCHAR NOT NULL,
    success BOOLEAN NOT NULL,
    duration_ms BIGINT NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    task_description TEXT,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the `agent_metrics` table for monitoring.
pub const CREATE_AGENT_METRICS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS agent_metrics (
    agent_name VARCHAR PRIMARY KEY,
    agent_type VARCHAR NOT NULL,
    total_executions BIGINT NOT NULL DEFAULT 0,
    successful_executions BIGINT NOT NULL DEFAULT 0,
    total_duration_ms BIGINT NOT NULL DEFAULT 0,
    avg_duration_ms BIGINT NOT NULL DEFAULT 0,
    min_duration_ms BIGINT NOT NULL DEFAULT 0,
    max_duration_ms BIGINT NOT NULL DEFAULT 0,
    last_execution TIMESTAMP WITH TIME ZONE,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the `task_metrics` table for monitoring.
pub const CREATE_TASK_METRICS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS task_metrics (
    task_type VARCHAR PRIMARY KEY,
    total_tasks BIGINT NOT NULL DEFAULT 0,
    completed_tasks BIGINT NOT NULL DEFAULT 0,
    avg_completion_time_ms BIGINT NOT NULL DEFAULT 0,
    agent_success_rates JSON NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the `episode_summaries` table.
pub const CREATE_EPISODE_SUMMARIES_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS episode_summaries (
    episode_id UUID PRIMARY KEY,
    summary_text TEXT NOT NULL,
    key_concepts JSON NOT NULL,
    key_steps JSON NOT NULL,
    summary_embedding FLOAT[],
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL to create the `episode_tags` table.
pub const CREATE_EPISODE_TAGS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS episode_tags (
    episode_id UUID NOT NULL,
    tag VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (episode_id, tag)
)
";
