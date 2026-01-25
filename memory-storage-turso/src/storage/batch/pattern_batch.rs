//! # Batch Pattern Operations
//!
//! Optimized batch operations for patterns using transactions.

use crate::TursoStorage;
use memory_core::{Episode, Error, Pattern, Result};
use tracing::{debug, error, info};
use uuid::Uuid;

#[cfg(feature = "compression")]
use crate::storage::episodes::compress_json_field;

impl TursoStorage {
    /// Store multiple episodes with custom IDs
    ///
    /// Allows specifying custom episode IDs instead of generating them.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Vector of (UUID, Episode) tuples
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use memory_core::{Episode, TaskContext, TaskType};
    /// # use uuid::Uuid;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let custom_id = Uuid::new_v4();
    /// let episodes = vec![(
    ///     custom_id,
    ///     Episode::new("Task".to_string(), TaskContext::default(), TaskType::CodeGeneration)
    /// )];
    ///
    /// storage.store_episodes_batch_with_ids(episodes).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_episodes_batch_with_ids(
        &self,
        episodes: Vec<(Uuid, Episode)>,
    ) -> Result<()> {
        if episodes.is_empty() {
            debug!("Empty episodes batch with IDs received, skipping");
            return Ok(());
        }

        debug!(
            "Storing episodes batch with custom IDs: {} items",
            episodes.len()
        );
        let conn = self.get_connection().await?;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for episodes batch with IDs: {}",
                e
            ))
        })?;

        let sql = r#"
            INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, heuristics, metadata, domain, language,
                archived_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        #[cfg(feature = "compression")]
        let compression_threshold = self.config.compression_threshold;
        #[cfg(not(feature = "compression"))]
        let _compression_threshold = 0;

        #[cfg(feature = "compression")]
        let should_compress = self.config.compress_episodes;
        #[cfg(not(feature = "compression"))]
        let _should_compress = false;

        for (episode_id, episode) in &episodes {
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

            #[cfg(feature = "compression")]
            let patterns_json = if should_compress {
                let data =
                    serde_json::to_string(&episode.patterns).map_err(Error::Serialization)?;
                compress_json_field(data.as_bytes(), compression_threshold)?
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
                compress_json_field(data.as_bytes(), compression_threshold)?
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
                compress_json_field(data.as_bytes(), compression_threshold)?
            } else {
                serde_json::to_string(&episode.metadata)
                    .map_err(Error::Serialization)?
                    .into_bytes()
            };

            #[cfg(not(feature = "compression"))]
            let metadata_json: Vec<u8> = serde_json::to_string(&episode.metadata)
                .map_err(Error::Serialization)?
                .into_bytes();

            let archived_at = episode
                .metadata
                .get("archived_at")
                .and_then(|v| v.parse::<i64>().ok());

            let patterns_str = String::from_utf8(patterns_json).map_err(|e| {
                Error::Storage(format!("Failed to convert patterns to UTF-8: {}", e))
            })?;
            let heuristics_str = String::from_utf8(heuristics_json).map_err(|e| {
                Error::Storage(format!("Failed to convert heuristics to UTF-8: {}", e))
            })?;
            let metadata_str = String::from_utf8(metadata_json).map_err(|e| {
                Error::Storage(format!("Failed to convert metadata to UTF-8: {}", e))
            })?;

            if let Err(e) = conn
                .execute(
                    sql,
                    libsql::params![
                        episode_id.to_string(),
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
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Failed to store episode in batch with ID: {}",
                    e
                )));
            }
        }

        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit episodes batch with IDs transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully stored episodes batch with IDs: {} items",
            episodes.len()
        );
        Ok(())
    }

    /// Store multiple patterns in a single transaction
    ///
    /// # Arguments
    ///
    /// * `patterns` - Vector of patterns to store
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use memory_core::Pattern;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let patterns = vec![
    ///     // ... patterns to store
    /// ];
    ///
    /// storage.store_patterns_batch(patterns).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_patterns_batch(&self, patterns: Vec<Pattern>) -> Result<()> {
        if patterns.is_empty() {
            debug!("Empty patterns batch received, skipping");
            return Ok(());
        }

        debug!("Storing patterns batch: {} items", patterns.len());
        let conn = self.get_connection().await?;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for patterns batch: {}",
                e
            ))
        })?;

        let sql = r#"
            INSERT OR REPLACE INTO patterns (
                pattern_id, pattern_type, pattern_data, success_rate,
                context_domain, context_language, context_tags, occurrence_count,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        for pattern in &patterns {
            // Extract data from Pattern enum using the existing logic
            let (description, context, heuristic, success_rate, occurrence_count) = match &pattern {
                Pattern::ToolSequence {
                    id: _,
                    tools,
                    context,
                    success_rate,
                    avg_latency: _,
                    occurrence_count,
                    effectiveness: _,
                } => {
                    // Use tools directly without cloning - join() accepts IntoIterator
                    let desc = format!("Tool sequence: {}", tools.join(" -> "));
                    let heur = memory_core::Heuristic::new(
                        format!("When need tools: {}", tools.join(", ")),
                        format!("Use sequence: {}", tools.join(" -> ")),
                        *success_rate,
                    );
                    (
                        desc,
                        context.clone(),
                        heur,
                        *success_rate,
                        *occurrence_count,
                    )
                }
                Pattern::DecisionPoint {
                    id: _,
                    condition,
                    action,
                    outcome_stats,
                    context,
                    effectiveness: _,
                } => {
                    let desc = format!("Decision: {} -> {}", condition, action);
                    let heur = memory_core::Heuristic::new(
                        condition.clone(),
                        action.clone(),
                        outcome_stats.success_rate(),
                    );
                    (
                        desc,
                        context.clone(),
                        heur,
                        outcome_stats.success_rate(),
                        outcome_stats.total_count,
                    )
                }
                Pattern::ErrorRecovery {
                    id: _,
                    error_type,
                    recovery_steps,
                    success_rate,
                    context,
                    effectiveness: _,
                } => {
                    let desc = format!("Error recovery for: {}", error_type);
                    let heur = memory_core::Heuristic::new(
                        format!("Error: {}", error_type),
                        format!("Recovery: {}", recovery_steps.join(" -> ")),
                        *success_rate,
                    );
                    (
                        desc,
                        context.clone(),
                        heur,
                        *success_rate,
                        recovery_steps.len(),
                    )
                }
                Pattern::ContextPattern {
                    id: _,
                    context_features,
                    recommended_approach,
                    evidence: _,
                    success_rate,
                    effectiveness: _,
                } => {
                    let desc = format!("Context pattern: {}", recommended_approach);
                    let heur = memory_core::Heuristic::new(
                        format!("Features: {}", context_features.join(", ")),
                        recommended_approach.clone(),
                        *success_rate,
                    );
                    (
                        desc,
                        memory_core::TaskContext::default(),
                        heur,
                        *success_rate,
                        context_features.len(),
                    )
                }
            };

            // Create pattern_data JSON blob
            let pattern_data = crate::storage::patterns::PatternDataJson {
                description,
                context: context.clone(),
                heuristic,
            };
            let pattern_data_json =
                serde_json::to_string(&pattern_data).map_err(Error::Serialization)?;

            let context_tags_json =
                serde_json::to_string(&context.tags).map_err(Error::Serialization)?;

            let now = chrono::Utc::now();

            if let Err(e) = conn
                .execute(
                    sql,
                    libsql::params![
                        pattern.id().to_string(),
                        format!("{:?}", pattern),
                        pattern_data_json,
                        success_rate,
                        context.domain.clone(),
                        context.language.clone(),
                        context_tags_json,
                        occurrence_count as i64,
                        now.timestamp(),
                        now.timestamp(),
                    ],
                )
                .await
            {
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Failed to store pattern in batch: {}",
                    e
                )));
            }
        }

        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit patterns batch transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully stored patterns batch: {} items",
            patterns.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{Episode, Pattern, PatternId, TaskContext, TaskType};
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
    async fn test_store_episodes_batch_with_ids() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id1 = Uuid::new_v4();
        let _id2 = Uuid::new_v4(); // Reserved for future batch testing

        let episodes = vec![(
            id1,
            Episode::new(
                "Task with custom ID 1".to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            ),
        )];

        let result = storage.store_episodes_batch_with_ids(episodes).await;
        assert!(result.is_ok());

        // Verify retrieval
        let retrieved = storage.get_episode(id1).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().task_description, "Task with custom ID 1");
    }

    #[tokio::test]
    async fn test_store_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage.store_patterns_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_patterns_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let patterns = vec![Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: "test condition".to_string(),
            action: "test action".to_string(),
            outcome_stats: memory_core::types::OutcomeStats {
                success_count: 5,
                failure_count: 1,
                total_count: 6,
                avg_duration_secs: 0.0,
            },
            context: memory_core::TaskContext::default(),
            effectiveness: memory_core::pattern::PatternEffectiveness::default(),
        }];

        let result = storage.store_patterns_batch(patterns).await;
        assert!(result.is_ok());
    }
}
