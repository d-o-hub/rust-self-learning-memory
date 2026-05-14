use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    pub(crate) async fn store_episode_summary_internal(
        &self,
        summary: &do_memory_core::types::EpisodeSummary,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let summary = summary.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let key_concepts_json = serde_json::to_string(&summary.key_concepts)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let key_steps_json = serde_json::to_string(&summary.key_steps)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO episode_summaries (
                    episode_id, summary_text, key_concepts, key_steps, summary_embedding
                ) VALUES (?, ?, ?, ?, ?)",
                params![
                    summary.episode_id.to_string(),
                    summary.summary_text,
                    key_concepts_json,
                    key_steps_json,
                    summary.summary_embedding,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store summary: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_episode_summary_internal(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<do_memory_core::types::EpisodeSummary>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT summary_text, CAST(key_concepts AS VARCHAR), CAST(key_steps AS VARCHAR), summary_embedding FROM episode_summaries WHERE episode_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![episode_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let summary_text: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let key_concepts_json: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let key_steps_json: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let summary_embedding: Option<Vec<f32>> = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;

                Ok::<Option<do_memory_core::types::EpisodeSummary>, Error>(Some(do_memory_core::types::EpisodeSummary {
                    episode_id,
                    summary_text,
                    key_concepts: serde_json::from_str(&key_concepts_json).map_err(|e| Error::Storage(e.to_string()))?,
                    key_steps: serde_json::from_str(&key_steps_json).map_err(|e| Error::Storage(e.to_string()))?,
                    summary_embedding,
                }))
            } else {
                Ok::<Option<do_memory_core::types::EpisodeSummary>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn add_tags_to_episode_internal(
        &self,
        episode_id: Uuid,
        tags: &[String],
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let tags = tags.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            for tag in tags {
                conn.execute(
                    "INSERT OR IGNORE INTO episode_tags (episode_id, tag) VALUES (?, ?)",
                    params![episode_id.to_string(), tag],
                )
                .map_err(|e| Error::Storage(format!("Failed to add tag: {e}")))?;
            }
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_tags_for_episode_internal(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<String>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT tag FROM episode_tags WHERE episode_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![episode_id.to_string()], |row| row.get(0))
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut tags = Vec::new();
            for tag in rows {
                tags.push(tag.map_err(|e| Error::Storage(e.to_string()))?);
            }
            Ok::<Vec<String>, Error>(tags)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}
