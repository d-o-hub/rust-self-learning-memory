//! # Raw Pattern Query Operations
//!
//! Execute raw SQL queries for patterns with proper parsing.
//! Used by the cache integration layer for flexible query caching.

use super::row::row_to_pattern_at;
use crate::TursoStorage;
use crate::storage::query_builder::PatternQueryBuilder;
use do_memory_core::{Error, Pattern, Result};
use libsql::params;
use libsql::params::IntoParams;
use tracing::{debug, info};

/// Raw pattern query executor
///
/// Executes SQL queries against the patterns table and parses results.
/// The SQL must return columns in the standard pattern query order:
/// pattern_id, pattern_type, pattern_data, success_rate,
/// context_domain, context_language, context_tags, occurrence_count,
/// created_at, updated_at
pub struct RawPatternQuery<'a> {
    storage: &'a TursoStorage,
}

impl<'a> RawPatternQuery<'a> {
    /// Create a new raw pattern query executor
    pub fn new(storage: &'a TursoStorage) -> Self {
        Self { storage }
    }

    /// Execute a raw SQL query and parse patterns
    ///
    /// The SQL must return columns in the order expected by `row_to_pattern`:
    /// - pattern_id, pattern_type, pattern_data, success_rate
    /// - context_domain, context_language, context_tags, occurrence_count
    /// - created_at, updated_at
    ///
    /// Use the `PATTERN_SELECT_COLUMNS` constant for correct column ordering.
    ///
    /// # Errors
    ///
    /// Any row that fails to parse aborts the query with a contextual
    /// [`Error::Storage`] naming the column, the result row index, and this
    /// surface. Rows are never silently skipped.
    ///
    /// # Security
    ///
    /// SQL injection risk: The SQL string is executed directly without
    /// sanitization. Callers must ensure SQL comes from trusted sources.
    /// Prefer [`Self::query_with_params`] (the supported path) or
    /// [`Self::query_built`] for parameterized execution.
    pub async fn query(&self, sql: &str) -> Result<Vec<Pattern>> {
        debug!("Executing raw pattern query: {}", sql);
        self.storage
            .with_connection_with_id(async |conn, _conn_id| {
                let mut rows = conn.query(sql, params![]).await.map_err(|e| {
                    Error::Storage(format!("Failed to execute pattern query: {}", e))
                })?;

                let mut patterns = Vec::new();
                let mut row_index = 0usize;
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
                {
                    patterns.push(row_to_pattern_at(
                        &row,
                        "RawPatternQuery::query",
                        row_index,
                    )?);
                    row_index += 1;
                }

                info!("Raw query returned {} patterns", patterns.len());
                Ok(patterns)
            })
            .await
    }

    /// Execute a parameterized SQL query and parse patterns
    ///
    /// This is the supported way to execute queries, including with user input.
    /// Parameters are bound through libSQL placeholders to prevent SQL
    /// injection.
    ///
    /// # Arguments
    ///
    /// * `sql` - SQL query with ? placeholders
    /// * `params` - Parameters to bind to placeholders
    ///
    /// # Errors
    ///
    /// Any row that fails to parse aborts the query with a contextual
    /// [`Error::Storage`] naming the column, the result row index, and this
    /// surface. Rows are never silently skipped.
    pub async fn query_with_params<P: IntoParams>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<Pattern>> {
        debug!("Executing parameterized pattern query: {}", sql);
        self.storage
            .with_connection_with_id(async |conn, _conn_id| {
                let mut rows = conn.query(sql, params).await.map_err(|e| {
                    Error::Storage(format!("Failed to execute pattern query: {}", e))
                })?;

                let mut patterns = Vec::new();
                let mut row_index = 0usize;
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
                {
                    patterns.push(row_to_pattern_at(
                        &row,
                        "RawPatternQuery::query_with_params",
                        row_index,
                    )?);
                    row_index += 1;
                }

                info!("Parameterized query returned {} patterns", patterns.len());
                Ok(patterns)
            })
            .await
    }

    /// Execute an allowlisted, parameterized query built via
    /// [`PatternQueryBuilder`].
    ///
    /// This is the safest entry point: the builder can only emit allowlisted
    /// column names and binds every value as a `?` placeholder.
    pub async fn query_built(&self, builder: PatternQueryBuilder) -> Result<Vec<Pattern>> {
        let (sql, params) = builder.into_parts();
        self.query_with_params(&sql, params).await
    }
}

/// Standard SELECT columns for pattern queries
///
/// Use this constant to ensure correct column ordering for `row_to_pattern`.
pub const PATTERN_SELECT_COLUMNS: &str = r#"
    pattern_id, pattern_type, pattern_data, success_rate,
    context_domain, context_language, context_tags, occurrence_count,
    created_at, updated_at
"#;

impl TursoStorage {
    /// Execute a raw SQL query for patterns
    ///
    /// Convenience method for cache integration.
    /// See `RawPatternQuery` for details.
    pub async fn query_patterns_raw(&self, sql: &str) -> Result<Vec<Pattern>> {
        RawPatternQuery::new(self).query(sql).await
    }

    /// Execute a parameterized SQL query for patterns
    ///
    /// Convenience method for cache integration with safe parameters.
    pub async fn query_patterns_raw_with_params<P: IntoParams>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<Pattern>> {
        RawPatternQuery::new(self)
            .query_with_params(sql, params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::{Pattern, TaskContext};
    use tempfile::TempDir;

    async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

        let storage = TursoStorage::from_database(db)?;
        storage.initialize_schema().await?;

        Ok((storage, dir))
    }

    #[tokio::test]
    async fn test_raw_pattern_query_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let raw_query = RawPatternQuery::new(&storage);

        let sql = format!(
            "SELECT {} FROM patterns WHERE context_domain = 'nonexistent'",
            PATTERN_SELECT_COLUMNS
        );
        let result = raw_query.query(&sql).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_raw_pattern_query_with_data() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create test pattern
        let pattern = Pattern::DecisionPoint {
            id: uuid::Uuid::new_v4(),
            condition: "test condition".to_string(),
            action: "test action".to_string(),
            outcome_stats: do_memory_core::types::OutcomeStats {
                success_count: 10,
                failure_count: 2,
                total_count: 12,
                avg_duration_secs: 0.5,
            },
            context: TaskContext {
                domain: "test-domain".to_string(),
                ..Default::default()
            },
            effectiveness: do_memory_core::patterns::PatternEffectiveness::default(),
        };
        storage.store_pattern(&pattern).await.unwrap();

        // Query with raw SQL
        let raw_query = RawPatternQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM patterns WHERE context_domain = ?",
            PATTERN_SELECT_COLUMNS
        );
        let result = raw_query
            .query_with_params(&sql, ["test-domain".to_string()])
            .await
            .unwrap();

        assert_ne!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_raw_pattern_query_rejects_corrupt_row_with_field_and_row() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let pattern = Pattern::DecisionPoint {
            id: uuid::Uuid::new_v4(),
            condition: "test condition".to_string(),
            action: "test action".to_string(),
            outcome_stats: do_memory_core::types::OutcomeStats {
                success_count: 1,
                failure_count: 0,
                total_count: 1,
                avg_duration_secs: 0.5,
            },
            context: TaskContext {
                domain: "corrupt-domain".to_string(),
                ..Default::default()
            },
            effectiveness: do_memory_core::patterns::PatternEffectiveness::default(),
        };
        storage.store_pattern(&pattern).await.unwrap();

        {
            let (conn, _conn_id) = storage.get_connection_with_id().await.unwrap();
            conn.execute(
                "UPDATE patterns SET pattern_data = 'not-json' WHERE pattern_id = ?",
                libsql::params![pattern.id().to_string()],
            )
            .await
            .unwrap();
        }

        let raw_query = RawPatternQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM patterns WHERE context_domain = ?",
            PATTERN_SELECT_COLUMNS
        );
        let err = raw_query
            .query_with_params(&sql, ["corrupt-domain".to_string()])
            .await
            .expect_err("corrupt pattern row must not be silently skipped");

        let msg = err.to_string();
        assert!(msg.contains("pattern_data"), "field not named: {msg}");
        assert!(msg.contains("result row 0"), "row not named: {msg}");
        assert!(
            msg.contains("query_with_params"),
            "surface not named: {msg}"
        );
    }

    #[tokio::test]
    async fn test_query_built_returns_patterns() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let pattern = Pattern::DecisionPoint {
            id: uuid::Uuid::new_v4(),
            condition: "built condition".to_string(),
            action: "built action".to_string(),
            outcome_stats: do_memory_core::types::OutcomeStats {
                success_count: 1,
                failure_count: 0,
                total_count: 1,
                avg_duration_secs: 0.1,
            },
            context: TaskContext {
                domain: "built-domain".to_string(),
                ..Default::default()
            },
            effectiveness: do_memory_core::patterns::PatternEffectiveness::default(),
        };
        storage.store_pattern(&pattern).await.unwrap();

        let raw_query = RawPatternQuery::new(&storage);
        let builder = crate::storage::query_builder::PatternQueryBuilder::patterns().filter(
            crate::storage::query_builder::PatternColumn::ContextDomain,
            crate::storage::query_builder::FilterOp::Eq,
            "built-domain",
        );
        let result = raw_query.query_built(builder).await.unwrap();

        assert_eq!(result.len(), 1);
    }
}
