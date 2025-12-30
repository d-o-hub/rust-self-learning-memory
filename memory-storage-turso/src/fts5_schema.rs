//! FTS5 virtual tables for hybrid search
//!
//! This module provides SQL schema definitions for FTS5 virtual tables
//! that enable full-text search capabilities alongside vector search.
//! The FTS5 tables are synchronized with the main tables via triggers
//! to maintain data consistency.

/// SQL to create FTS5 virtual table for episodes
///
/// Creates a virtual table that indexes task_description, context, and domain
/// for full-text search. The episode_id column is UNINDEXED to prevent
/// duplicate indexing while still being available for joins.
///
/// Tokenizer: porter unicode61 - provides stemming and Unicode-aware tokenization
pub const CREATE_EPISODES_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS episodes_fts USING fts5(
    episode_id UNINDEXED,
    task_description,
    context,
    domain,
    tokenize='porter unicode61'
)
"#;

/// SQL to create FTS5 virtual table for patterns
///
/// Creates a virtual table that indexes pattern_data, context_domain,
/// and context_language for full-text search. The pattern_id column is
/// UNINDEXED for join purposes.
pub const CREATE_PATTERNS_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS patterns_fts USING fts5(
    pattern_id UNINDEXED,
    pattern_data,
    context_domain,
    context_language,
    tokenize='porter unicode61'
)
"#;

/// SQL to create synchronization triggers for episodes FTS table
///
/// These triggers automatically maintain the episodes_fts table
/// in sync with the main episodes table:
/// - AFTER INSERT: Copies new episodes to FTS table
/// - AFTER UPDATE: Updates existing FTS entries
/// - AFTER DELETE: Removes deleted episodes from FTS table
pub const CREATE_EPISODES_FTS_TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS episodes_ai AFTER INSERT ON episodes BEGIN
    INSERT INTO episodes_fts(episode_id, task_description, context, domain)
    VALUES (new.episode_id, new.task_description, new.context, new.domain);
END;

CREATE TRIGGER IF NOT EXISTS episodes_au AFTER UPDATE ON episodes BEGIN
    UPDATE episodes_fts 
    SET task_description = new.task_description,
        context = new.context,
        domain = new.domain
    WHERE episode_id = new.episode_id;
END;

CREATE TRIGGER IF NOT EXISTS episodes_ad AFTER DELETE ON episodes BEGIN
    DELETE FROM episodes_fts WHERE episode_id = old.episode_id;
END;
"#;

/// SQL to create synchronization triggers for patterns FTS table
///
/// These triggers automatically maintain the patterns_fts table
/// in sync with the main patterns table:
/// - AFTER INSERT: Copies new patterns to FTS table
/// - AFTER UPDATE: Updates existing FTS entries
/// - AFTER DELETE: Removes deleted patterns from FTS table
pub const CREATE_PATTERNS_FTS_TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS patterns_ai AFTER INSERT ON patterns BEGIN
    INSERT INTO patterns_fts(pattern_id, pattern_data, context_domain, context_language)
    VALUES (new.pattern_id, new.pattern_data, new.context_domain, new.context_language);
END;

CREATE TRIGGER IF NOT EXISTS patterns_au AFTER UPDATE ON patterns BEGIN
    UPDATE patterns_fts 
    SET pattern_data = new.pattern_data,
        context_domain = new.context_domain,
        context_language = new.context_language
    WHERE pattern_id = new.pattern_id;
END;

CREATE TRIGGER IF NOT EXISTS patterns_ad AFTER DELETE ON patterns BEGIN
    DELETE FROM patterns_fts WHERE pattern_id = old.pattern_id;
END;
"#;

/// SQL to create index on FTS5 tables for better search performance
///
/// Creates auxiliary indexes for the FTS5 tables to improve search speed
/// and reduce storage overhead.
#[allow(dead_code)]
pub const CREATE_EPISODES_FTS_INDEX: &str = r#"
INSERT INTO episodes_fts(episodes_fts) VALUES('optimize')
"#;

/// SQL to create index on patterns FTS5 tables
#[allow(dead_code)]
pub const CREATE_PATTERNS_FTS_INDEX: &str = r#"
INSERT INTO patterns_fts(patterns_fts) VALUES('optimize')
"#;

/// SQL to drop all FTS5 tables and triggers (for migration/cleanup)
#[allow(dead_code)]
pub const DROP_FTS5_SCHEMA: &str = r#"
DROP TRIGGER IF EXISTS episodes_ai;
DROP TRIGGER IF EXISTS episodes_au;
DROP TRIGGER IF EXISTS episodes_ad;
DROP TRIGGER IF EXISTS patterns_ai;
DROP TRIGGER IF EXISTS patterns_au;
DROP TRIGGER IF EXISTS patterns_ad;
DROP TABLE IF EXISTS episodes_fts;
DROP TABLE IF EXISTS patterns_fts;
"#;
