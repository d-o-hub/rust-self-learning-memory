//! # Combined Batch Operations
//!
//! Batch operations for storing episodes with their associated patterns.

use crate::TursoStorage;
use memory_core::{Episode, Error, Pattern, Result};
use tracing::{debug, error, info};

impl TursoStorage {
    /// Store episodes with their associated patterns in a single transaction
    ///
    /// This is more efficient than storing episodes and patterns separately
    /// when they are related, as it ensures atomicity and reduces round-trips.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Episodes to store
    /// * `patterns` - Patterns to store (may be associated with episodes)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use memory_core::{Episode, Pattern, TaskContext, TaskType};
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    ///
    /// let episodes = vec![/* ... */];
    /// let patterns = vec![/* ... */];
    ///
    /// storage.store_episodes_with_patterns_batch(episodes, patterns).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_episodes_with_patterns_batch(
        &self,
        episodes: Vec<Episode>,
        patterns: Vec<Pattern>,
    ) -> Result<()> {
        if episodes.is_empty() && patterns.is_empty() {
            debug!("Empty combined batch received, skipping");
            return Ok(());
        }

        debug!(
            "Storing combined batch: {} episodes, {} patterns",
            episodes.len(),
            patterns.len()
        );
        let conn = self.get_connection().await?;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for combined batch: {}",
                e
            ))
        })?;

        // Store episodes first
        if !episodes.is_empty() {
            let episode_sql = r#"
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

            for episode in &episodes {
                let context_json =
                    serde_json::to_string(&episode.context).map_err(Error::Serialization)?;
                let steps_json =
                    serde_json::to_string(&episode.steps).map_err(Error::Serialization)?;
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
                        episode_sql,
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
                    if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                        error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    return Err(Error::Storage(format!(
                        "Failed to store episode in combined batch: {}",
                        e
                    )));
                }
            }
        }

        // Store patterns
        if !patterns.is_empty() {
            let pattern_sql = r#"
                INSERT OR REPLACE INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, context_tags, occurrence_count,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            for pattern in &patterns {
                let (description, context, heuristic, success_rate, occurrence_count) =
                    match &pattern {
                        Pattern::ToolSequence {
                            id: _,
                            tools,
                            context,
                            success_rate,
                            avg_latency: _,
                            occurrence_count,
                            effectiveness: _,
                        } => {
                            let tools_vec = tools.clone();
                            let desc = format!("Tool sequence: {}", tools_vec.join(" -> "));
                            let heur = memory_core::Heuristic::new(
                                format!("When need tools: {}", tools_vec.join(", ")),
                                format!("Use sequence: {}", tools_vec.join(" -> ")),
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

                let pattern_data = crate::storage::patterns::PatternDataJson {
                    description: description.clone(),
                    context: context.clone(),
                    heuristic: heuristic.clone(),
                };
                let pattern_data_json =
                    serde_json::to_string(&pattern_data).map_err(Error::Serialization)?;

                let context_tags_json =
                    serde_json::to_string(&context.tags).map_err(Error::Serialization)?;

                let now = chrono::Utc::now();

                if let Err(e) = conn
                    .execute(
                        pattern_sql,
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
                        "Failed to store pattern in combined batch: {}",
                        e
                    )));
                }
            }
        }

        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit combined batch transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully stored combined batch: {} episodes, {} patterns",
            episodes.len(),
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
    async fn test_store_episodes_with_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let result = storage
            .store_episodes_with_patterns_batch(vec![], vec![])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_episodes_with_patterns_batch_episodes_only() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episodes = vec![Episode::new(
            "Task with patterns".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        )];

        let result = storage
            .store_episodes_with_patterns_batch(episodes, vec![])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_episodes_with_patterns_batch_patterns_only() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let patterns = vec![Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: "refactoring needed".to_string(),
            action: "create tests first".to_string(),
            outcome_stats: memory_core::types::OutcomeStats {
                success_count: 10,
                failure_count: 2,
                total_count: 12,
            },
            context: TaskContext::default(),
            effectiveness: None,
        }];

        let result = storage
            .store_episodes_with_patterns_batch(vec![], patterns)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_episodes_with_patterns_batch_both() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episodes = vec![Episode::new(
            "Complex task".to_string(),
            TaskContext::default(),
            TaskType::Analysis,
        )];

        let patterns = vec![Pattern::ContextPattern {
            id: PatternId::new_v4(),
            context_features: vec!["analysis".to_string()],
            recommended_approach: "Break down into smaller parts".to_string(),
            evidence: vec![],
            success_rate: 0.85,
            effectiveness: None,
        }];

        let result = storage
            .store_episodes_with_patterns_batch(episodes, patterns)
            .await;
        assert!(result.is_ok());
    }
}
