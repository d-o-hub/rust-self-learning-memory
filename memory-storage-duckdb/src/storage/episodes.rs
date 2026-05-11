use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    // ========== Internal CRUD Operations ==========

    /// Stores an episode in the `DuckDB` database.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or database execution fails.
    pub(crate) async fn store_episode_internal(
        &self,
        episode: &do_memory_core::Episode,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let episode = episode.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let context_json = serde_json::to_string(&episode.context)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let steps_json = serde_json::to_string(&episode.steps)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let outcome_json = serde_json::to_string(&episode.outcome)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let reward_json = serde_json::to_string(&episode.reward)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let reflection_json = serde_json::to_string(&episode.reflection)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let patterns_json = serde_json::to_string(&episode.patterns)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let heuristics_json = serde_json::to_string(&episode.heuristics)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let checkpoints_json = serde_json::to_string(&episode.checkpoints)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let metadata_json = serde_json::to_string(&episode.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT INTO episodes (
                    episode_id, task_type, task_description, context, start_time, end_time,
                    steps, outcome, reward, reflection, patterns, heuristics, checkpoints,
                    metadata, domain, language
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    episode.episode_id.to_string(),
                    episode.task_type.to_string(),
                    episode.task_description,
                    context_json,
                    episode.start_time.to_rfc3339(),
                    episode.end_time.as_ref().map(DateTime::to_rfc3339),
                    steps_json,
                    outcome_json,
                    reward_json,
                    reflection_json,
                    patterns_json,
                    heuristics_json,
                    checkpoints_json,
                    metadata_json,
                    episode.context.domain,
                    episode.context.language,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store episode: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;

        // Emit standardized event
        self.emit_event(do_memory_core::types::event::MemoryEvent::EpisodeStored {
            episode_id: episode.episode_id.to_string(),
            backend: "duckdb".to_string(),
            timestamp: do_memory_core::types::event::unix_now_secs(),
        })
        .await;

        Ok(())
    }

    /// Retrieves an episode by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub(crate) async fn get_episode_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT
                episode_id, task_type, task_description, CAST(context AS VARCHAR),
                strftime(CAST(start_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                strftime(CAST(end_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                CAST(steps AS VARCHAR), CAST(outcome AS VARCHAR), CAST(reward AS VARCHAR),
                CAST(reflection AS VARCHAR), CAST(patterns AS VARCHAR), CAST(heuristics AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE episode_id = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let episode_id_str: String =
                    row.get(0).map_err(|e| Error::Storage(format!("col 0: {e}")))?;
                let task_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(format!("col 1: {e}")))?;
                let task_desc: String =
                    row.get(2).map_err(|e| Error::Storage(format!("col 2: {e}")))?;
                let context_json: String =
                    row.get(3).map_err(|e| Error::Storage(format!("col 3: {e}")))?;
                let start_time_str: String =
                    row.get(4).map_err(|e| Error::Storage(format!("col 4: {e}")))?;
                let end_time_str: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(format!("col 5: {e}")))?;
                let steps_json: String =
                    row.get(6).map_err(|e| Error::Storage(format!("col 6: {e}")))?;
                let outcome_json: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(format!("col 7: {e}")))?;
                let reward_json: Option<String> =
                    row.get(8).map_err(|e| Error::Storage(format!("col 8: {e}")))?;
                let reflection_json: Option<String> =
                    row.get(9).map_err(|e| Error::Storage(format!("col 9: {e}")))?;
                let patterns_json: String =
                    row.get(10).map_err(|e| Error::Storage(format!("col 10: {e}")))?;
                let heuristics_json: String =
                    row.get(11).map_err(|e| Error::Storage(format!("col 11: {e}")))?;
                let checkpoints_json: String =
                    row.get(12).map_err(|e| Error::Storage(format!("col 12: {e}")))?;
                let metadata_json: String =
                    row.get(13).map_err(|e| Error::Storage(format!("col 13: {e}")))?;

                let episode = Self::map_row_to_episode(
                    &episode_id_str,
                    &task_type_str,
                    task_desc,
                    &context_json,
                    &start_time_str,
                    end_time_str,
                    &steps_json,
                    outcome_json,
                    reward_json,
                    reflection_json,
                    &patterns_json,
                    &heuristics_json,
                    &checkpoints_json,
                    &metadata_json,
                )?;
                Ok::<Option<do_memory_core::Episode>, Error>(Some(episode))
            } else {
                Ok::<Option<do_memory_core::Episode>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    #[allow(clippy::too_many_arguments)]
    fn map_row_to_episode(
        episode_id_str: &str,
        task_type_str: &str,
        task_desc: String,
        context_json: &str,
        start_time_str: &str,
        end_time_str: Option<String>,
        steps_json: &str,
        outcome_json: Option<String>,
        reward_json: Option<String>,
        reflection_json: Option<String>,
        patterns_json: &str,
        heuristics_json: &str,
        checkpoints_json: &str,
        metadata_json: &str,
    ) -> Result<do_memory_core::Episode> {
        let episode = do_memory_core::Episode {
            episode_id: Uuid::parse_str(episode_id_str)
                .map_err(|e| Error::Storage(format!("episode_id parse: {e}")))?,
            task_type: task_type_str
                .parse()
                .map_err(|e| Error::Storage(format!("task_type parse: {e}")))?,
            task_description: task_desc,
            context: serde_json::from_str(context_json)
                .map_err(|e| Error::Storage(format!("context parse: {e}")))?,
            start_time: DateTime::parse_from_rfc3339(start_time_str)
                .or_else(|_| DateTime::parse_from_str(start_time_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                .map_err(|e| Error::Storage(format!("start_time parse: {e}")))?
                .with_timezone(&Utc),
            end_time: end_time_str
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map(|t| t.with_timezone(&Utc))
                        .map_err(|e| Error::Storage(format!("end_time parse: {e}")))
                })
                .transpose()?,
            steps: serde_json::from_str(steps_json)
                .map_err(|e| Error::Storage(format!("steps parse: {e}")))?,
            outcome: outcome_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("outcome parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            reward: reward_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("reward parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            reflection: reflection_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("reflection parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            patterns: serde_json::from_str(patterns_json)
                .map_err(|e| Error::Storage(format!("patterns parse: {e}")))?,
            heuristics: serde_json::from_str(heuristics_json)
                .map_err(|e| Error::Storage(format!("heuristics parse: {e}")))?,
            applied_patterns: Vec::new(),
            salient_features: None,
            checkpoints: serde_json::from_str(checkpoints_json)
                .map_err(|e| Error::Storage(format!("checkpoints parse: {e}")))?,
            metadata: serde_json::from_str(metadata_json)
                .map_err(|e| Error::Storage(format!("metadata parse: {e}")))?,
            tags: Vec::new(),
        };
        Ok(episode)
    }

    /// Deletes an episode by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete operation fails.
    pub(crate) async fn delete_episode_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "DELETE FROM episodes WHERE episode_id = ?",
                params![id.to_string()],
            )
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;

        // Emit standardized event
        self.emit_event(
            do_memory_core::types::event::MemoryEvent::EpisodeGarbageCollected {
                id: id.to_string(),
                reason: "manual".to_string(),
                timestamp: do_memory_core::types::event::unix_now_secs(),
            },
        )
        .await;

        Ok(())
    }

    /// Stores a pattern in the `DuckDB` database.
    ///
    /// # Errors
    ///
    pub(crate) async fn query_episodes_since_internal(
        &self,
        since: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        let limit = do_memory_core::storage::apply_query_limit(limit);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT
                episode_id, task_type, task_description, CAST(context AS VARCHAR),
                strftime(CAST(start_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                strftime(CAST(end_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                CAST(steps AS VARCHAR), CAST(outcome AS VARCHAR), CAST(reward AS VARCHAR),
                CAST(reflection AS VARCHAR), CAST(patterns AS VARCHAR), CAST(heuristics AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE start_time >= ?
                ORDER BY start_time ASC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![since.to_rfc3339(), i64::try_from(limit).unwrap_or(1000)], |row| {
                    Ok(Self::map_row_to_episode(
                        &row.get::<_, String>(0)?,
                        &row.get::<_, String>(1)?,
                        row.get(2)?,
                        &row.get::<_, String>(3)?,
                        &row.get::<_, String>(4)?,
                        row.get(5)?,
                        &row.get::<_, String>(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        &row.get::<_, String>(10)?,
                        &row.get::<_, String>(11)?,
                        &row.get::<_, String>(12)?,
                        &row.get::<_, String>(13)?,
                    ))
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            for row in rows {
                episodes.push(row.map_err(|e| Error::Storage(e.to_string()))??);
            }
            Ok::<Vec<do_memory_core::Episode>, Error>(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn query_episodes_by_metadata_internal(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        let limit = do_memory_core::storage::apply_query_limit(limit);
        let key = key.to_string();
        let value = value.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT
                episode_id, task_type, task_description, CAST(context AS VARCHAR),
                strftime(CAST(start_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                strftime(CAST(end_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                CAST(steps AS VARCHAR), CAST(outcome AS VARCHAR), CAST(reward AS VARCHAR),
                CAST(reflection AS VARCHAR), CAST(patterns AS VARCHAR), CAST(heuristics AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE metadata->>? = ?
                ORDER BY start_time DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![key, value, i64::try_from(limit).unwrap_or(1000)], |row| {
                    Ok(Self::map_row_to_episode(
                        &row.get::<_, String>(0)?,
                        &row.get::<_, String>(1)?,
                        row.get(2)?,
                        &row.get::<_, String>(3)?,
                        &row.get::<_, String>(4)?,
                        row.get(5)?,
                        &row.get::<_, String>(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        &row.get::<_, String>(10)?,
                        &row.get::<_, String>(11)?,
                        &row.get::<_, String>(12)?,
                        &row.get::<_, String>(13)?,
                    ))
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            for row in rows {
                episodes.push(row.map_err(|e| Error::Storage(e.to_string()))??);
            }
            Ok::<Vec<do_memory_core::Episode>, Error>(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Embedding Storage Methods ==========
}
