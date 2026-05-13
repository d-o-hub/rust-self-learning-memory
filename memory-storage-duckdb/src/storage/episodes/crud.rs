use crate::DuckDbStorage;
use chrono::DateTime;
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
            let mut conn = conn_arc.lock();
            let tx = conn
                .transaction()
                .map_err(|e| Error::Storage(format!("Failed to start transaction: {e}")))?;

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
            let applied_patterns_json = serde_json::to_string(&episode.applied_patterns)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let salient_features_json = serde_json::to_string(&episode.salient_features)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let checkpoints_json = serde_json::to_string(&episode.checkpoints)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let metadata_json = serde_json::to_string(&episode.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            tx.execute(
                "INSERT OR REPLACE INTO episodes (
                    episode_id, task_type, task_description, context, start_time, end_time,
                    steps, outcome, reward, reflection, patterns, heuristics,
                    applied_patterns, salient_features, checkpoints,
                    metadata, domain, language
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                    applied_patterns_json,
                    salient_features_json,
                    checkpoints_json,
                    metadata_json,
                    episode.context.domain,
                    episode.context.language,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store episode: {e}")))?;

            // Store tags
            for tag in &episode.tags {
                tx.execute(
                    "INSERT OR IGNORE INTO episode_tags (episode_id, tag) VALUES (?, ?)",
                    params![episode.episode_id.to_string(), tag],
                )
                .map_err(|e| Error::Storage(format!("Failed to store episode tag: {e}")))?;
            }

            tx.commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {e}")))?;

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
                CAST(applied_patterns AS VARCHAR), CAST(salient_features AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE episode_id = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let id_str = id.to_string();
            let mut rows = stmt
                .query(params![id_str])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let episode_id_str: String = row
                    .get(0)
                    .map_err(|e| Error::Storage(format!("col 0: {e}")))?;
                let task_type_str: String = row
                    .get(1)
                    .map_err(|e| Error::Storage(format!("col 1: {e}")))?;
                let task_desc: String = row
                    .get(2)
                    .map_err(|e| Error::Storage(format!("col 2: {e}")))?;
                let context_json: String = row
                    .get(3)
                    .map_err(|e| Error::Storage(format!("col 3: {e}")))?;
                let start_time_str: String = row
                    .get(4)
                    .map_err(|e| Error::Storage(format!("col 4: {e}")))?;
                let end_time_str: Option<String> = row
                    .get(5)
                    .map_err(|e| Error::Storage(format!("col 5: {e}")))?;
                let steps_json: String = row
                    .get(6)
                    .map_err(|e| Error::Storage(format!("col 6: {e}")))?;
                let outcome_json: Option<String> = row
                    .get(7)
                    .map_err(|e| Error::Storage(format!("col 7: {e}")))?;
                let reward_json: Option<String> = row
                    .get(8)
                    .map_err(|e| Error::Storage(format!("col 8: {e}")))?;
                let reflection_json: Option<String> = row
                    .get(9)
                    .map_err(|e| Error::Storage(format!("col 9: {e}")))?;
                let patterns_json: String = row
                    .get(10)
                    .map_err(|e| Error::Storage(format!("col 10: {e}")))?;
                let heuristics_json: String = row
                    .get(11)
                    .map_err(|e| Error::Storage(format!("col 11: {e}")))?;
                let applied_patterns_json: Option<String> = row
                    .get(12)
                    .map_err(|e| Error::Storage(format!("col 12: {e}")))?;
                let salient_features_json: Option<String> = row
                    .get(13)
                    .map_err(|e| Error::Storage(format!("col 13: {e}")))?;
                let checkpoints_json: String = row
                    .get(14)
                    .map_err(|e| Error::Storage(format!("col 14: {e}")))?;
                let metadata_json: String = row
                    .get(15)
                    .map_err(|e| Error::Storage(format!("col 15: {e}")))?;

                // Load tags
                let mut tag_stmt = conn
                    .prepare("SELECT tag FROM episode_tags WHERE episode_id = ?")
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let tag_rows = tag_stmt
                    .query_map(params![id_str], |r| r.get::<_, String>(0))
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let mut tags = Vec::new();
                for tag in tag_rows {
                    tags.push(tag.map_err(|e| Error::Storage(e.to_string()))?);
                }

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
                    applied_patterns_json,
                    salient_features_json,
                    &checkpoints_json,
                    &metadata_json,
                    tags,
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

    /// Deletes an episode by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete operation fails.
    pub(crate) async fn delete_episode_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut conn = conn_arc.lock();
            let tx = conn
                .transaction()
                .map_err(|e| Error::Storage(format!("Failed to start transaction: {e}")))?;

            let id_str = id.to_string();
            tx.execute(
                "DELETE FROM episode_tags WHERE episode_id = ?",
                params![id_str],
            )
            .map_err(|e| Error::Storage(format!("Failed to delete episode tags: {e}")))?;

            tx.execute("DELETE FROM episodes WHERE episode_id = ?", params![id_str])
                .map_err(|e| Error::Storage(format!("Failed to delete episode: {e}")))?;

            tx.commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {e}")))?;
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
}
