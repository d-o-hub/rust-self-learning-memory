//! CRUD operations for episodes.

use crate::TursoStorage;
use do_memory_core::{Episode, Error, Result};
use tracing::{debug, info};
use uuid::Uuid;

use do_memory_core::semantic::EpisodeSummary;

impl TursoStorage {
    /// Store an episode
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        debug!("Storing episode: {}", episode.episode_id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, heuristics, checkpoints, metadata,
                domain, language
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let context_json = serde_json::to_string(&episode.context).map_err(Error::Serialization)?;
        let steps_json = serde_json::to_string(&episode.steps).map_err(Error::Serialization)?;
        let outcome_json = serde_json::to_string(&episode.outcome).map_err(Error::Serialization)?;
        let reward_json = serde_json::to_string(&episode.reward).map_err(Error::Serialization)?;
        let reflection_json =
            serde_json::to_string(&episode.reflection).map_err(Error::Serialization)?;
        let patterns_json = serde_json::to_string(&episode.patterns).map_err(Error::Serialization)?;
        let heuristics_json =
            serde_json::to_string(&episode.heuristics).map_err(Error::Serialization)?;
        let checkpoints_json =
            serde_json::to_string(&episode.checkpoints).map_err(Error::Serialization)?;
        let metadata_json = serde_json::to_string(&episode.metadata).map_err(Error::Serialization)?;

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![
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
            patterns_json,
            heuristics_json,
            checkpoints_json,
            metadata_json,
            episode.context.domain.clone(),
            episode.context.language.clone(),
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store episode: {}", e)))?;

        // Store tags if any
        if !episode.tags.is_empty() {
            self.store_episode_tags(episode.episode_id, &episode.tags)
                .await?;
        }

        info!("Successfully stored episode: {}", episode.episode_id);
        Ok(())
    }

    /// Retrieve an episode by ID
    pub async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        debug!("Retrieving episode: {}", id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let select_cols = super::raw_query::EPISODE_SELECT_COLUMNS;
        let sql = format!(
            "SELECT {} FROM episodes WHERE episode_id = ?",
            select_cols.join(", ")
        );

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, &sql)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episode: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let mut episode = super::row::row_to_episode(&row)?;

            // Load tags
            episode.tags = self.get_episode_tags(id).await?;

            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }

    /// Delete an episode by ID
    pub async fn delete_episode(&self, id: Uuid) -> Result<()> {
        debug!("Deleting episode: {}", id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = "DELETE FROM episodes WHERE episode_id = ?";

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;

        info!("Successfully deleted episode: {}", id);
        Ok(())
    }

    /// Store a semantic summary for an episode
    pub async fn store_summary(&self, episode_id: Uuid, summary: &EpisodeSummary) -> Result<()> {
        debug!("Storing summary for episode: {}", episode_id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO episode_summaries (
                episode_id, summary_text, key_concepts, key_steps, summary_embedding
            ) VALUES (?, ?, ?, ?, ?)
        "#;

        let key_concepts_json =
            serde_json::to_string(&summary.key_concepts).map_err(Error::Serialization)?;
        let key_steps_json =
            serde_json::to_string(&summary.key_steps).map_err(Error::Serialization)?;

        // Convert f32 vector to bytes for BLOB storage
        let embedding_bytes = summary.embedding.as_ref().map(|vec| {
            let mut bytes = Vec::with_capacity(vec.len() * 4);
            for &f in vec {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            bytes
        });

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![
            episode_id.to_string(),
            summary.summary_text.clone(),
            key_concepts_json,
            key_steps_json,
            embedding_bytes,
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store summary: {}", e)))?;

        info!("Successfully stored summary for episode: {}", episode_id);
        Ok(())
    }

    /// Retrieve a semantic summary by episode ID
    pub async fn get_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        debug!("Retrieving summary for episode: {}", episode_id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT episode_id, summary_text, key_concepts, key_steps, summary_embedding
            FROM episode_summaries WHERE episode_id = ?
        "#;

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query summary: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch summary row: {}", e)))?
        {
            Ok(Some(super::row::row_to_summary(&row)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TursoStorage;
    use do_memory_core::{TaskContext, TaskType};
    use tempfile::TempDir;

    async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
        let dir = tempfile::tempdir().map_err(|e| Error::Storage(e.to_string()))?;
        let db_path = dir.path().join("test.db");
        let path_str = format!("file:{}", db_path.to_string_lossy());
        let storage = TursoStorage::new(&path_str, "").await?;
        storage.initialize_schema().await?;
        Ok((storage, dir))
    }

    #[tokio::test]
    async fn test_store_and_get_episode() -> Result<()> {
        let (storage, _dir) = create_test_storage().await?;
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        episode.add_tag("test".to_string()).unwrap();

        storage.store_episode(&episode).await?;

        let retrieved = storage.get_episode(episode.episode_id).await?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.episode_id, episode.episode_id);
        assert_eq!(retrieved.task_description, episode.task_description);
        assert!(retrieved.has_tag("test"));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_episode() -> Result<()> {
        let (storage, _dir) = create_test_storage().await?;
        let episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        storage.store_episode(&episode).await?;
        storage.delete_episode(episode.episode_id).await?;

        let retrieved = storage.get_episode(episode.episode_id).await?;
        assert!(retrieved.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_get_summary() -> Result<()> {
        let (storage, _dir) = create_test_storage().await?;
        let episode_id = Uuid::new_v4();
        let summary = EpisodeSummary {
            summary_text: "Test summary".to_string(),
            key_concepts: vec!["concept1".to_string()],
            key_steps: vec!["step1".to_string()],
            embedding: Some(vec![0.1, 0.2, 0.3]),
        };

        // Note: foreign key constraint requires episode to exist
        let episode = Episode {
            episode_id,
            ..Episode::new(
                "Test".to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            )
        };
        storage.store_episode(&episode).await?;

        storage.store_summary(episode_id, &summary).await?;

        let retrieved = storage.get_summary(episode_id).await?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.summary_text, summary.summary_text);
        assert_eq!(retrieved.key_concepts, summary.key_concepts);
        assert_eq!(retrieved.key_steps, summary.key_steps);
        assert!(retrieved.embedding.is_some());
        let emb = retrieved.embedding.unwrap();
        assert_eq!(emb.len(), 3);
        assert!((emb[0] - 0.1).abs() < f32::EPSILON);

        Ok(())
    }
}
