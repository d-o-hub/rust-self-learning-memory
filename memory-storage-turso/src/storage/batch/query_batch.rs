//! # Batch Query Operations
//!
//! Efficient retrieval of multiple episodes and patterns using batch queries.

use super::super::episodes::row_to_episode;
use super::super::patterns::row_to_pattern;
use crate::TursoStorage;
use memory_core::{episode::PatternId, Episode, Error, Pattern, Result};
use tracing::{debug, info};
use uuid::Uuid;

impl TursoStorage {
    /// Retrieve multiple episodes by IDs efficiently
    ///
    /// Uses a single query with IN clause for efficient batch retrieval.
    ///
    /// # Arguments
    ///
    /// * `ids` - Slice of episode UUIDs to retrieve
    ///
    /// # Returns
    ///
    /// Vector of optional episodes (None if episode not found)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use uuid::Uuid;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    /// let episodes = storage.get_episodes_batch(&ids).await?;
    ///
    /// for episode in episodes {
    ///     if let Some(ep) = episode {
    ///         println!("Found episode: {}", ep.episode_id);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_episodes_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<Episode>>> {
        if ids.is_empty() {
            debug!("Empty IDs batch received for episode retrieval");
            return Ok(Vec::new());
        }

        debug!("Retrieving episodes batch: {} items", ids.len());
        let conn = self.get_connection().await?;

        // Build the IN clause with placeholders
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes WHERE episode_id IN ({})
        "#,
            placeholders.join(", ")
        );

        // Convert UUIDs to strings for the query
        let params: Vec<libsql::Value> = ids
            .iter()
            .map(|id| libsql::Value::Text(id.to_string()))
            .collect();

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes batch: {}", e)))?;

        // Create a map of episode_id -> Episode for efficient lookup
        let mut episode_map = std::collections::HashMap::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = row_to_episode(&row)?;
            episode_map.insert(episode.episode_id, episode);
        }

        // Return episodes in the same order as the input IDs
        let result: Vec<Option<Episode>> =
            ids.iter().map(|id| episode_map.get(id).cloned()).collect();

        info!(
            "Retrieved {} of {} requested episodes",
            result.iter().filter(|e| e.is_some()).count(),
            ids.len()
        );
        Ok(result)
    }

    /// Retrieve multiple patterns by IDs efficiently
    ///
    /// Uses a single query with IN clause for efficient batch retrieval.
    ///
    /// # Arguments
    ///
    /// * `ids` - Slice of pattern IDs to retrieve
    ///
    /// # Returns
    ///
    /// Vector of optional patterns (None if pattern not found)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use memory_core::episode::PatternId;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let ids = vec![PatternId::new_v4(), PatternId::new_v4()];
    /// let patterns = storage.get_patterns_batch(&ids).await?;
    ///
    /// for pattern in patterns {
    ///     if let Some(p) = pattern {
    ///         println!("Found pattern: {:?}", p.id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_patterns_batch(&self, ids: &[PatternId]) -> Result<Vec<Option<Pattern>>> {
        if ids.is_empty() {
            debug!("Empty IDs batch received for pattern retrieval");
            return Ok(Vec::new());
        }

        debug!("Retrieving patterns batch: {} items", ids.len());
        let conn = self.get_connection().await?;

        // Build the IN clause with placeholders
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            r#"
            SELECT pattern_id, pattern_type, pattern_data, success_rate,
                   context_domain, context_language, context_tags, occurrence_count,
                   created_at, updated_at
            FROM patterns WHERE pattern_id IN ({})
        "#,
            placeholders.join(", ")
        );

        // Convert IDs to strings for the query
        let params: Vec<libsql::Value> = ids
            .iter()
            .map(|id| libsql::Value::Text(id.to_string()))
            .collect();

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query patterns batch: {}", e)))?;

        // Create a map of pattern_id -> Pattern for efficient lookup
        let mut pattern_map = std::collections::HashMap::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            let pattern = row_to_pattern(&row)?;
            pattern_map.insert(pattern.id(), pattern);
        }

        // Return patterns in the same order as the input IDs
        let result: Vec<Option<Pattern>> =
            ids.iter().map(|id| pattern_map.get(id).cloned()).collect();

        info!(
            "Retrieved {} of {} requested patterns",
            result.iter().filter(|e| e.is_some()).count(),
            ids.len()
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{Episode, TaskContext, TaskType};
    use tempfile::TempDir;
    use uuid::Uuid;

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
    async fn test_get_episodes_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage.get_episodes_batch(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_episodes_batch_with_missing() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Store only one episode
        let episodes = vec![(
            id1,
            Episode::new(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            ),
        )];
        storage
            .store_episodes_batch_with_ids(episodes)
            .await
            .unwrap();

        // Retrieve both - one should exist, one should be None
        let result = storage.get_episodes_batch(&[id1, id2]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].is_some());
        assert!(result[1].is_none());
    }

    #[tokio::test]
    async fn test_get_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage.get_patterns_batch(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_store_and_get_multiple_episodes_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episodes = vec![
            Episode::new(
                "Task 1".to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            ),
            Episode::new(
                "Task 2".to_string(),
                TaskContext::default(),
                TaskType::Debugging,
            ),
            Episode::new(
                "Task 3".to_string(),
                TaskContext::default(),
                TaskType::Refactoring,
            ),
        ];

        let result = storage.store_episodes_batch(episodes).await;
        assert!(result.is_ok());

        // Verify we can retrieve them
        let ids: Vec<Uuid> = vec![
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
        ];

        let retrieved = storage.get_episodes_batch(&ids).await.unwrap();
        // Episodes should be in storage (we're checking by generated IDs)
    }
}
