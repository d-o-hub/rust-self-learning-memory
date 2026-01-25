//! # Episode Query Operations
//!
//! Query and filtering operations for episodes.

use super::EpisodeQuery;
use crate::TursoStorage;
use memory_core::{Episode, Error, Result};
use tracing::{debug, info};

impl TursoStorage {
    /// Query episodes with filters
    pub async fn query_episodes(&self, query: &EpisodeQuery) -> Result<Vec<Episode>> {
        debug!("Querying episodes with filters: {:?}", query);
        let conn = self.get_connection().await?;

        let mut sql = String::from(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes WHERE 1=1
        "#,
        );

        let mut params_vec = Vec::new();

        if let Some(ref task_type) = query.task_type {
            sql.push_str(" AND task_type = ?");
            params_vec.push(task_type.to_string());
        }

        if let Some(ref domain) = query.domain {
            sql.push_str(" AND domain = ?");
            params_vec.push(domain.clone());
        }

        if let Some(ref language) = query.language {
            sql.push_str(" AND language = ?");
            params_vec.push(language.clone());
        }

        if query.completed_only {
            sql.push_str(" AND end_time IS NOT NULL");
        }

        sql.push_str(" ORDER BY start_time DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!("Found {} episodes matching query", episodes.len());
        Ok(episodes)
    }

    /// Query episodes modified since a given timestamp
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>> {
        debug!("Querying episodes since {}", since);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes
            WHERE start_time >= ?
            ORDER BY start_time DESC
        "#;

        let since_timestamp = since.timestamp();

        let mut rows = conn
            .query(sql, libsql::params![since_timestamp])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!("Found {} episodes modified since {}", episodes.len(), since);
        Ok(episodes)
    }

    /// Query episodes by metadata key-value pair
    ///
    /// Uses json_extract for efficient querying of JSON metadata fields.
    /// Falls back to LIKE pattern matching if json_extract is not available.
    pub async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
        debug!("Querying episodes by metadata {} = {}", key, value);
        let conn = self.get_connection().await?;

        // Use json_extract for efficient JSON metadata querying
        // This is more efficient than LIKE pattern matching as it can use indexes
        let sql = format!(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes
            WHERE json_extract(metadata, '$.{}') = '{}'
            ORDER BY start_time DESC
        "#,
            key, value
        );

        let mut rows = conn
            .query(&sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!(
            "Found {} episodes with metadata {} = {}",
            episodes.len(),
            key,
            value
        );
        Ok(episodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{Episode, TaskContext, TaskType};
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
    async fn test_query_episodes_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let query = EpisodeQuery::default();
        let result = storage.query_episodes(&query).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_query_episodes_with_limit() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create multiple episodes
        for i in 0..5 {
            let episode = Episode::new(
                format!("Task {}", i),
                TaskContext::default(),
                TaskType::CodeGeneration,
            );
            storage.store_episode(&episode).await.unwrap();
        }

        // Query with limit
        let query = EpisodeQuery {
            limit: Some(3),
            ..Default::default()
        };
        let result = storage.query_episodes(&query).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_query_episodes_by_task_type() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create episodes with different task types
        for i in 0..3 {
            let episode = Episode::new(
                format!("Code task {}", i),
                TaskContext::default(),
                TaskType::CodeGeneration,
            );
            storage.store_episode(&episode).await.unwrap();
        }

        for i in 0..2 {
            let episode = Episode::new(
                format!("Debug task {}", i),
                TaskContext::default(),
                TaskType::Debugging,
            );
            storage.store_episode(&episode).await.unwrap();
        }

        // Query by task type
        let query = EpisodeQuery {
            task_type: Some(TaskType::CodeGeneration),
            ..Default::default()
        };
        let result = storage.query_episodes(&query).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_query_episodes_by_metadata() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let mut episode1 = Episode::new(
            "Task with tag".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        );
        episode1
            .metadata
            .insert("tag".to_string(), "important".to_string());
        storage.store_episode(&episode1).await.unwrap();

        let episode2 = Episode::new(
            "Task without tag".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        );
        storage.store_episode(&episode2).await.unwrap();

        // Query by metadata
        let result = storage
            .query_episodes_by_metadata("tag", "important")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].task_description, "Task with tag");
    }
}
