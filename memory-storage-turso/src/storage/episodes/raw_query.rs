//! # Raw Episode Query Operations
//!
//! Execute raw SQL queries for episodes with proper parsing.
//! Used by the cache integration layer for flexible query caching.

use super::row::row_to_episode_at;
use crate::TursoStorage;
use crate::storage::query_builder::EpisodeQueryBuilder;
use do_memory_core::{Episode, Error, Result};
use libsql::params;
use libsql::params::IntoParams;
use tracing::{debug, info};

/// Raw episode query executor
///
/// Executes SQL queries against the episodes table and parses results.
/// The SQL must return columns in the standard episode query order:
/// episode_id, task_type, task_description, context, start_time, end_time,
/// steps, outcome, reward, reflection, patterns, heuristics, checkpoints,
/// metadata, domain, language, archived_at
pub struct RawEpisodeQuery<'a> {
    storage: &'a TursoStorage,
}

impl<'a> RawEpisodeQuery<'a> {
    /// Create a new raw episode query executor
    pub fn new(storage: &'a TursoStorage) -> Self {
        Self { storage }
    }

    /// Execute a raw SQL query and parse episodes
    ///
    /// The SQL must return columns in the order expected by `row_to_episode`:
    /// - episode_id, task_type, task_description, context
    /// - start_time, end_time, steps, outcome, reward
    /// - reflection, patterns, heuristics, checkpoints, metadata
    /// - domain, language, archived_at
    ///
    /// Use the `EPISODE_SELECT_COLUMNS` constant for correct column ordering.
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
    pub async fn query(&self, sql: &str) -> Result<Vec<Episode>> {
        debug!("Executing raw episode query: {}", sql);
        let (conn, _conn_id) = self.storage.get_connection_with_id().await?;

        let mut rows = conn
            .query(sql, params![])
            .await
            .map_err(|e| Error::Storage(format!("Failed to execute episode query: {}", e)))?;

        let mut episodes = Vec::new();
        let mut row_index = 0usize;
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(row_to_episode_at(
                &row,
                "RawEpisodeQuery::query",
                row_index,
            )?);
            row_index += 1;
        }

        info!("Raw query returned {} episodes", episodes.len());
        Ok(episodes)
    }

    /// Execute a parameterized SQL query and parse episodes
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use do_memory_storage_turso::TursoStorage;
    /// # async fn example(storage: &TursoStorage) -> anyhow::Result<()> {
    /// use do_memory_storage_turso::storage::episodes::RawEpisodeQuery;
    /// let raw_query = RawEpisodeQuery::new(storage);
    /// let episodes = raw_query.query_with_params(
    ///     "SELECT * FROM episodes WHERE domain = ? LIMIT 100",
    ///     &["test_domain".to_string()]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_with_params<P: IntoParams>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<Episode>> {
        debug!("Executing parameterized episode query: {}", sql);
        let (conn, _conn_id) = self.storage.get_connection_with_id().await?;

        let mut rows = conn
            .query(sql, params)
            .await
            .map_err(|e| Error::Storage(format!("Failed to execute episode query: {}", e)))?;

        let mut episodes = Vec::new();
        let mut row_index = 0usize;
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(row_to_episode_at(
                &row,
                "RawEpisodeQuery::query_with_params",
                row_index,
            )?);
            row_index += 1;
        }

        info!("Parameterized query returned {} episodes", episodes.len());
        Ok(episodes)
    }

    /// Execute an allowlisted, parameterized query built via
    /// [`EpisodeQueryBuilder`].
    ///
    /// This is the safest entry point: the builder can only emit allowlisted
    /// column names and binds every value as a `?` placeholder.
    pub async fn query_built(&self, builder: EpisodeQueryBuilder) -> Result<Vec<Episode>> {
        let (sql, params) = builder.into_parts();
        self.query_with_params(&sql, params).await
    }
}

/// Standard SELECT columns for episode queries
///
/// Use this constant to ensure correct column ordering for `row_to_episode`.
pub const EPISODE_SELECT_COLUMNS: &str = r#"
    episode_id, task_type, task_description, context,
    start_time, end_time, steps, outcome, reward,
    reflection, patterns, heuristics,
    COALESCE(checkpoints, '[]') AS checkpoints,
    metadata, domain, language,
    archived_at
"#;

impl TursoStorage {
    /// Execute a raw SQL query for episodes
    ///
    /// Convenience method for cache integration.
    /// See `RawEpisodeQuery` for details.
    pub async fn query_episodes_raw(&self, sql: &str) -> Result<Vec<Episode>> {
        RawEpisodeQuery::new(self).query(sql).await
    }

    /// Execute a parameterized SQL query for episodes
    ///
    /// Convenience method for cache integration with safe parameters.
    pub async fn query_episodes_raw_with_params<P: IntoParams>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<Episode>> {
        RawEpisodeQuery::new(self)
            .query_with_params(sql, params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::{Episode, TaskContext, TaskType};
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
    async fn test_raw_episode_query_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let raw_query = RawEpisodeQuery::new(&storage);

        let sql = format!(
            "SELECT {} FROM episodes WHERE domain = 'nonexistent'",
            EPISODE_SELECT_COLUMNS
        );
        let result = raw_query.query(&sql).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_raw_episode_query_with_data() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create test episode
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext {
                domain: "test-domain".to_string(),
                ..Default::default()
            },
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();

        // Query with raw SQL
        let raw_query = RawEpisodeQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM episodes WHERE domain = ?",
            EPISODE_SELECT_COLUMNS
        );
        let result = raw_query
            .query_with_params(&sql, ["test-domain".to_string()])
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].task_description, "Test task");
        assert_eq!(result[0].context.domain, "test-domain");
    }

    #[tokio::test]
    async fn test_raw_episode_query_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create multiple episodes
        for i in 0..5 {
            let episode = Episode::new(
                format!("Task {}", i),
                TaskContext {
                    domain: "batch-domain".to_string(),
                    ..Default::default()
                },
                TaskType::CodeGeneration,
            );
            storage.store_episode(&episode).await.unwrap();
        }

        // Query all
        let raw_query = RawEpisodeQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM episodes WHERE domain = ? ORDER BY start_time DESC LIMIT 3",
            EPISODE_SELECT_COLUMNS
        );
        let result = raw_query
            .query_with_params(&sql, ["batch-domain".to_string()])
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_raw_episode_query_rejects_corrupt_timestamp_with_field_and_row() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "Corrupt timestamp".to_string(),
            TaskContext {
                domain: "corrupt-ts-domain".to_string(),
                ..Default::default()
            },
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();

        // Push start_time outside chrono's representable range.
        let (conn, _conn_id) = storage.get_connection_with_id().await.unwrap();
        conn.execute(
            "UPDATE episodes SET start_time = ? WHERE episode_id = ?",
            libsql::params![i64::MAX, episode.episode_id.to_string()],
        )
        .await
        .unwrap();

        let raw_query = RawEpisodeQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM episodes WHERE domain = ?",
            EPISODE_SELECT_COLUMNS
        );
        let err = raw_query
            .query_with_params(&sql, ["corrupt-ts-domain".to_string()])
            .await
            .expect_err("corrupt timestamp must not silently decode");

        let msg = err.to_string();
        assert!(msg.contains("start_time"), "field not named: {msg}");
        assert!(msg.contains("result row 0"), "row not named: {msg}");
        assert!(
            msg.contains("query_with_params"),
            "surface not named: {msg}"
        );
    }

    #[tokio::test]
    async fn test_raw_episode_query_rejects_corrupt_json_and_keeps_nullable_null() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "Corrupt json".to_string(),
            TaskContext {
                domain: "corrupt-json-domain".to_string(),
                ..Default::default()
            },
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();

        // Nullable outcome absent stays absent; corrupt required context JSON errors.
        let (conn, _conn_id) = storage.get_connection_with_id().await.unwrap();
        conn.execute(
            "UPDATE episodes SET context = 'not-json' WHERE episode_id = ?",
            libsql::params![episode.episode_id.to_string()],
        )
        .await
        .unwrap();

        let raw_query = RawEpisodeQuery::new(&storage);
        let sql = format!(
            "SELECT {} FROM episodes WHERE domain = ?",
            EPISODE_SELECT_COLUMNS
        );
        let err = raw_query
            .query_with_params(&sql, ["corrupt-json-domain".to_string()])
            .await
            .expect_err("corrupt JSON must not be dropped");

        let msg = err.to_string();
        assert!(msg.contains("context"), "field not named: {msg}");
        assert!(msg.contains("result row 0"), "row not named: {msg}");
    }

    #[tokio::test]
    async fn test_query_built_returns_episodes() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "Built query".to_string(),
            TaskContext {
                domain: "built-domain".to_string(),
                ..Default::default()
            },
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();

        let raw_query = RawEpisodeQuery::new(&storage);
        let builder = crate::storage::query_builder::EpisodeQueryBuilder::episodes()
            .filter(
                crate::storage::query_builder::EpisodeColumn::Domain,
                crate::storage::query_builder::FilterOp::Eq,
                "built-domain",
            )
            .limit(10);
        let result = raw_query.query_built(builder).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].context.domain, "built-domain");
    }
}
