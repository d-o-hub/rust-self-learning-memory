//! # Raw Episode Query Operations
//!
//! Execute raw SQL queries for episodes with proper parsing.
//! Used by the cache integration layer for flexible query caching.

use crate::TursoStorage;
use do_memory_core::{Episode, Error, Result};
use libsql::params;
use libsql::params::IntoParams;
use tracing::{debug, info, warn};

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
    /// # Security
    ///
    /// SQL injection risk: The SQL string is executed directly without sanitization.
    /// Callers must ensure SQL comes from trusted sources or use parameterized queries.
    /// Use `query_with_params` for safe parameterized execution.
    pub async fn query(&self, sql: &str) -> Result<Vec<Episode>> {
        debug!("Executing raw episode query: {}", sql);
        let (conn, _conn_id) = self.storage.get_connection_with_id().await?;

        let mut rows = conn
            .query(sql, params![])
            .await
            .map_err(|e| Error::Storage(format!("Failed to execute episode query: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            match super::row::row_to_episode(&row) {
                Ok(episode) => episodes.push(episode),
                Err(e) => {
                    warn!("Failed to parse episode row: {}", e);
                    // Continue processing other rows
                }
            }
        }

        info!("Raw query returned {} episodes", episodes.len());
        Ok(episodes)
    }

    /// Execute a parameterized SQL query and parse episodes
    ///
    /// This is the safe way to execute queries with user input.
    /// Parameters are properly escaped to prevent SQL injection.
    ///
    /// # Arguments
    ///
    /// * `sql` - SQL query with ? placeholders
    /// * `params` - Parameters to bind to placeholders
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
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            match super::row::row_to_episode(&row) {
                Ok(episode) => episodes.push(episode),
                Err(e) => {
                    warn!("Failed to parse episode row: {}", e);
                    // Continue processing other rows
                }
            }
        }

        info!("Parameterized query returned {} episodes", episodes.len());
        Ok(episodes)
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
        assert!(result.is_empty());
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
}
