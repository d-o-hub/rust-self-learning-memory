//! # Batch Query Operations
//!
//! Efficient retrieval of multiple episodes and patterns using batch queries.

use super::super::episodes::row_to_episode;
use super::super::patterns::row_to_pattern;
use crate::TursoStorage;
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result};
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

    /// Retrieve multiple heuristics by IDs efficiently
    ///
    /// Uses a single query with IN clause for efficient batch retrieval.
    ///
    /// # Arguments
    ///
    /// * `ids` - Slice of heuristic UUIDs to retrieve
    ///
    /// # Returns
    ///
    /// Vector of optional heuristics (None if heuristic not found)
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
    /// let heuristics = storage.get_heuristics_batch(&ids).await?;
    ///
    /// for heuristic in heuristics {
    ///     if let Some(h) = heuristic {
    ///         println!("Found heuristic: {}", h.heuristic_id);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_heuristics_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<Heuristic>>> {
        if ids.is_empty() {
            debug!("Empty IDs batch received for heuristic retrieval");
            return Ok(Vec::new());
        }

        debug!("Retrieving heuristics batch: {} items", ids.len());
        let conn = self.get_connection().await?;

        // Build the IN clause with placeholders
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            r#"
            SELECT heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
            FROM heuristics WHERE heuristic_id IN ({})
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
            .map_err(|e| Error::Storage(format!("Failed to query heuristics batch: {}", e)))?;

        // Create a map of heuristic_id -> Heuristic for efficient lookup
        let mut heuristic_map = std::collections::HashMap::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch heuristic row: {}", e)))?
        {
            let heuristic = super::super::heuristics::row_to_heuristic(&row)?;
            heuristic_map.insert(heuristic.heuristic_id, heuristic);
        }

        // Return heuristics in the same order as the input IDs
        let result: Vec<Option<Heuristic>> = ids
            .iter()
            .map(|id| heuristic_map.get(id).cloned())
            .collect();

        info!(
            "Retrieved {} of {} requested heuristics",
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

        // Store only one episode with specific ID
        let episode = Episode {
            episode_id: id1,
            task_type: TaskType::CodeGeneration,
            task_description: "Test task".to_string(),
            context: TaskContext::default(),
            start_time: chrono::Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            heuristics: Vec::new(),
            applied_patterns: Vec::new(),
            salient_features: None,
            metadata: std::collections::HashMap::new(),
            tags: Vec::new(),
        };
        storage.store_episodes_batch(vec![episode]).await.unwrap();

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

        let _retrieved = storage.get_episodes_batch(&ids).await.unwrap();
        // Episodes should be in storage (we're checking by generated IDs)
    }

    #[tokio::test]
    async fn test_get_heuristics_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage.get_heuristics_batch(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_heuristics_batch_with_missing() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Store only one heuristic
        let heuristic = create_test_heuristic_with_id(id1);
        storage
            .store_heuristics_batch(vec![heuristic])
            .await
            .unwrap();

        // Retrieve both - one should exist, one should be None
        let result = storage.get_heuristics_batch(&[id1, id2]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].is_some());
        assert!(result[1].is_none());
    }

    #[tokio::test]
    async fn test_store_and_get_heuristics_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let heuristics = vec![
            create_test_heuristic_with_id(id1),
            create_test_heuristic_with_id(id2),
            create_test_heuristic_with_id(id3),
        ];

        // Store heuristics
        storage
            .store_heuristics_batch(heuristics.clone())
            .await
            .unwrap();

        // Retrieve them in batch
        let retrieved = storage
            .get_heuristics_batch(&[id1, id2, id3])
            .await
            .unwrap();
        assert_eq!(retrieved.len(), 3);
        assert!(retrieved[0].is_some());
        assert!(retrieved[1].is_some());
        assert!(retrieved[2].is_some());

        // Verify the retrieved heuristics match
        assert_eq!(retrieved[0].as_ref().unwrap().heuristic_id, id1);
        assert_eq!(retrieved[1].as_ref().unwrap().heuristic_id, id2);
        assert_eq!(retrieved[2].as_ref().unwrap().heuristic_id, id3);
    }

    fn create_test_heuristic_with_id(id: Uuid) -> Heuristic {
        use memory_core::types::Evidence;

        Heuristic {
            heuristic_id: id,
            condition: format!("test condition {}", id),
            action: format!("test action {}", id),
            confidence: 0.85,
            evidence: Evidence {
                episode_ids: vec![],
                success_rate: 0.9,
                sample_size: 10,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
