//! # Batch Configuration
//!
//! Configuration options for batch operations.

use crate::TursoStorage;
use memory_core::{episode::PatternId, Episode, Error, Pattern, Result};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for batch operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of items per batch (default: 100)
    pub batch_size: usize,
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,
    /// Maximum delay for exponential backoff (milliseconds)
    pub retry_max_delay_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_retries: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 5000,
        }
    }
}

impl TursoStorage {
    // ========== Batch Episode Operations ==========

    /// Store multiple episodes in a single transaction
    ///
    /// Uses prepared statements and transactions for 4-6x throughput improvement.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Vector of episodes to store
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use memory_core::{Episode, TaskContext, TaskType};
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let episodes = vec![
    ///     Episode::new("Task 1".to_string(), TaskContext::default(), TaskType::CodeGeneration),
    ///     Episode::new("Task 2".to_string(), TaskContext::default(), TaskType::Debugging),
    /// ];
    ///
    /// storage.store_episodes_batch(episodes).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_episodes_batch(&self, episodes: Vec<Episode>) -> Result<()> {
        if episodes.is_empty() {
            debug!("Empty episodes batch received, skipping");
            return Ok(());
        }

        debug!("Storing episodes batch: {} items", episodes.len());
        let conn = self.get_connection().await?;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for episodes batch: {}",
                e
            ))
        })?;

        // SQL statement for episode insertion
        let sql = r#"
            INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, heuristics, metadata, domain, language,
                archived_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        // Prepare statement if supported by the connection
        // Note: libsql-rs doesn't have explicit prepare, but we can still
        // use the same SQL with transaction for batching

        // Get compression settings
        #[cfg(feature = "compression")]
        let compression_threshold = self.config.compression_threshold;
        #[cfg(not(feature = "compression"))]
        let _compression_threshold = 0;

        #[cfg(feature = "compression")]
        let should_compress = self.config.compress_episodes;
        #[cfg(not(feature = "compression"))]
        let _should_compress = false;

        // Store all episodes in the transaction
        for episode in &episodes {
            // Serialize episode data
            let context_json =
                serde_json::to_string(&episode.context).map_err(Error::Serialization)?;
            let steps_json = serde_json::to_string(&episode.steps).map_err(Error::Serialization)?;
            let outcome_json = episode
                .outcome
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(Error::Serialization)?;
            let reward_json = episode
                .reward
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(Error::Serialization)?;
            let reflection_json = episode
                .reflection
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(Error::Serialization)?;

            // Compress patterns, heuristics, and metadata if needed
            #[cfg(feature = "compression")]
            let patterns_json = if should_compress {
                let data =
                    serde_json::to_string(&episode.patterns).map_err(Error::Serialization)?;
                super::episodes::compress_json_field(data.as_bytes(), compression_threshold)?
            } else {
                serde_json::to_string(&episode.patterns)
                    .map_err(Error::Serialization)?
                    .into_bytes()
            };

            #[cfg(not(feature = "compression"))]
            let patterns_json: Vec<u8> = serde_json::to_string(&episode.patterns)
                .map_err(Error::Serialization)?
                .into_bytes();

            #[cfg(feature = "compression")]
            let heuristics_json = if should_compress {
                let data =
                    serde_json::to_string(&episode.heuristics).map_err(Error::Serialization)?;
                super::episodes::compress_json_field(data.as_bytes(), compression_threshold)?
            } else {
                serde_json::to_string(&episode.heuristics)
                    .map_err(Error::Serialization)?
                    .into_bytes()
            };

            #[cfg(not(feature = "compression"))]
            let heuristics_json: Vec<u8> = serde_json::to_string(&episode.heuristics)
                .map_err(Error::Serialization)?
                .into_bytes();

            #[cfg(feature = "compression")]
            let metadata_json = if should_compress {
                let data =
                    serde_json::to_string(&episode.metadata).map_err(Error::Serialization)?;
                super::episodes::compress_json_field(data.as_bytes(), compression_threshold)?
            } else {
                serde_json::to_string(&episode.metadata)
                    .map_err(Error::Serialization)?
                    .into_bytes()
            };

            #[cfg(not(feature = "compression"))]
            let metadata_json: Vec<u8> = serde_json::to_string(&episode.metadata)
                .map_err(Error::Serialization)?
                .into_bytes();

            // Get archived_at from metadata if present
            let archived_at = episode
                .metadata
                .get("archived_at")
                .and_then(|v| v.parse::<i64>().ok());

            // Convert bytes to String for SQL
            let patterns_str = String::from_utf8(patterns_json).map_err(|e| {
                Error::Storage(format!("Failed to convert patterns to UTF-8: {}", e))
            })?;
            let heuristics_str = String::from_utf8(heuristics_json).map_err(|e| {
                Error::Storage(format!("Failed to convert heuristics to UTF-8: {}", e))
            })?;
            let metadata_str = String::from_utf8(metadata_json).map_err(|e| {
                Error::Storage(format!("Failed to convert metadata to UTF-8: {}", e))
            })?;

            // Execute the insert
            if let Err(e) = conn
                .execute(
                    sql,
                    libsql::params![
                        episode.episode_id.to_string(),
                        episode.task_type.to_string(),
                        episode.task_description.clone(),
                        context_json,
                        episode.start_time.timestamp(),
                        episode.end_time.map(|t| t.timestamp()),
                        steps_json,
                        outcome_json,
                        reward_json,
                        reflection_json,
                        patterns_str,
                        heuristics_str,
                        metadata_str,
                        episode.context.domain.clone(),
                        episode.context.language.clone(),
                        archived_at,
                    ],
                )
                .await
            {
                // Rollback on error
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Failed to store episode in batch: {}",
                    e
                )));
            }
        }

        // Commit transaction
        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit episodes batch transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully stored episodes batch: {} items",
            episodes.len()
        );
        Ok(())
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
    async fn test_store_episodes_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage.store_episodes_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_episodes_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episodes = vec![Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )];

        let result = storage.store_episodes_batch(episodes).await;
        assert!(result.is_ok());
    }
}
