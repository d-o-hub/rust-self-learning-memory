//! # Episode CRUD Operations
//!
//! CRUD operations for episodes.

use crate::TursoStorage;
use memory_core::{semantic::EpisodeSummary, Episode, Error, Result};
use tracing::{debug, info};
use uuid::Uuid;

/// Compress JSON data if compression is enabled and data is large enough
#[cfg(feature = "compression")]
pub fn compress_json_field(data: &[u8], threshold: usize) -> Result<Vec<u8>> {
    use crate::compression::CompressedPayload;

    let compressed = CompressedPayload::compress(data, threshold)?;
    if compressed.algorithm == crate::CompressionAlgorithm::None {
        // No compression applied, return original data
        Ok(data.to_vec())
    } else {
        // Store as base64-encoded compressed data with algorithm prefix
        let payload = format!(
            "__compressed__:{}:{}\n{}",
            compressed.algorithm,
            compressed.original_size,
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed.data)
        );
        Ok(payload.into_bytes())
    }
}

/// Decompress JSON data if it's compressed
#[cfg(feature = "compression")]
pub fn decompress_json_field(data: &str) -> Result<Vec<u8>> {
    if data.starts_with("__compressed__:") {
        // Parse the compressed format: __compressed__:<algorithm>:<original_size>\n<base64_data>

        let remainder = &data["__compressed__:".len()..];
        let newline_pos = remainder.find('\n').ok_or_else(|| {
            Error::Storage("Invalid compressed data format: missing newline".to_string())
        })?;
        let header = &remainder[..newline_pos];
        let encoded_data = &remainder[newline_pos + 1..];

        // Parse header: <algorithm>:<original_size>
        let colon_pos = header
            .find(':')
            .ok_or_else(|| Error::Storage("Invalid compressed header format".to_string()))?;
        let algorithm_str = &header[..colon_pos];
        let original_size: usize = header[colon_pos + 1..].parse().map_err(|_| {
            Error::Storage("Invalid original size in compressed header".to_string())
        })?;

        let algorithm = match algorithm_str {
            "lz4" => crate::CompressionAlgorithm::Lz4,
            "zstd" => crate::CompressionAlgorithm::Zstd,
            "gzip" => crate::CompressionAlgorithm::Gzip,
            _ => {
                return Err(Error::Storage(format!(
                    "Unknown compression algorithm: {}",
                    algorithm_str
                )))
            }
        };

        let compressed_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded_data)
                .map_err(|e| {
                    Error::Storage(format!("Failed to decode base64 compressed data: {}", e))
                })?;

        let payload = crate::CompressedPayload {
            original_size,
            compressed_size: compressed_data.len(),
            compression_ratio: compressed_data.len() as f64 / original_size as f64,
            data: compressed_data,
            algorithm,
        };

        payload.decompress()
    } else {
        // Not compressed, return as-is
        Ok(data.as_bytes().to_vec())
    }
}

impl TursoStorage {
    /// Store an episode
    ///
    /// Uses INSERT OR REPLACE for upsert semantics.
    /// When compression is enabled, large payloads are compressed to reduce bandwidth.
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        debug!("Storing episode: {}", episode.episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, heuristics, metadata, domain, language,
                archived_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        // Get compression threshold from config
        #[cfg(feature = "compression")]
        let compression_threshold = self.config.compression_threshold;
        #[cfg(not(feature = "compression"))]
        let _compression_threshold = 0;

        let context_json = serde_json::to_string(&episode.context).map_err(Error::Serialization)?;
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

        // Compress patterns, heuristics, and metadata if they're large enough
        #[cfg(feature = "compression")]
        let should_compress = self.config.compress_episodes;
        #[cfg(not(feature = "compression"))]
        let _should_compress = false;

        #[cfg(feature = "compression")]
        let patterns_json = if should_compress {
            let data = serde_json::to_string(&episode.patterns).map_err(Error::Serialization)?;
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
            let data = serde_json::to_string(&episode.heuristics).map_err(Error::Serialization)?;
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
            let data = serde_json::to_string(&episode.metadata).map_err(Error::Serialization)?;
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

        // Get archived_at from metadata if present
        let archived_at = episode
            .metadata
            .get("archived_at")
            .and_then(|v| v.parse::<i64>().ok());

        // Convert bytes to String for SQL (assuming UTF-8)
        let patterns_str = String::from_utf8(patterns_json)
            .map_err(|e| Error::Storage(format!("Failed to convert patterns to UTF-8: {}", e)))?;
        let heuristics_str = String::from_utf8(heuristics_json)
            .map_err(|e| Error::Storage(format!("Failed to convert heuristics to UTF-8: {}", e)))?;
        let metadata_str = String::from_utf8(metadata_json)
            .map_err(|e| Error::Storage(format!("Failed to convert metadata to UTF-8: {}", e)))?;

        conn.execute(
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
        .map_err(|e| Error::Storage(format!("Failed to store episode: {}", e)))?;

        info!("Successfully stored episode: {}", episode.episode_id);
        Ok(())
    }

    /// Retrieve an episode by ID
    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Option<Episode>> {
        debug!("Retrieving episode: {}", episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes WHERE episode_id = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episode: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = Self::row_to_episode(&row)?;
            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }

    /// Delete an episode by ID
    pub async fn delete_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Deleting episode: {}", episode_id);
        let conn = self.get_connection().await?;

        let sql = "DELETE FROM episodes WHERE episode_id = ?";

        conn.execute(sql, libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;

        info!("Successfully deleted episode: {}", episode_id);
        Ok(())
    }

    /// Store an episode summary
    pub async fn store_episode_summary(&self, summary: &EpisodeSummary) -> Result<()> {
        debug!("Storing episode summary: {}", summary.episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO episode_summaries (
                episode_id, summary_text, key_concepts, key_steps,
                summary_embedding, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let key_concepts_json =
            serde_json::to_string(&summary.key_concepts).map_err(Error::Serialization)?;
        let key_steps_json =
            serde_json::to_string(&summary.key_steps).map_err(Error::Serialization)?;
        let embedding_json = summary
            .summary_embedding
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(Error::Serialization)?;

        conn.execute(
            sql,
            libsql::params![
                summary.episode_id.to_string(),
                summary.summary_text.clone(),
                key_concepts_json,
                key_steps_json,
                embedding_json,
                summary.created_at.timestamp(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store summary: {}", e)))?;

        info!(
            "Successfully stored summary for episode: {}",
            summary.episode_id
        );
        Ok(())
    }

    /// Retrieve an episode summary by episode ID
    pub async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        debug!("Retrieving episode summary: {}", episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, summary_text, key_concepts, key_steps,
                   summary_embedding, created_at
            FROM episode_summaries WHERE episode_id = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query summary: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch summary row: {}", e)))?
        {
            let summary = Self::row_to_summary(&row)?;
            Ok(Some(summary))
        } else {
            Ok(None)
        }
    }

    /// Retrieve an episode by task description
    pub async fn get_episode_by_task_desc(&self, task_desc: &str) -> Result<Option<Episode>> {
        debug!("Retrieving episode by task description: {}", task_desc);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language,
                   archived_at
            FROM episodes WHERE task_description = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![task_desc])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episode: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = Self::row_to_episode(&row)?;
            Ok(Some(episode))
        } else {
            Ok(None)
        }
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
    async fn test_store_and_get_episode() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "Test episode".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        let episode_id = episode.episode_id;
        storage.store_episode(&episode).await.unwrap();

        let retrieved = storage.get_episode(episode_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().task_description, "Test episode");
    }

    #[tokio::test]
    async fn test_delete_episode() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "To delete".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        );

        let episode_id = episode.episode_id;
        storage.store_episode(&episode).await.unwrap();

        storage.delete_episode(episode_id).await.unwrap();

        let retrieved = storage.get_episode(episode_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_episode() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let nonexistent_id = Uuid::new_v4();
        let result = storage.get_episode(nonexistent_id).await.unwrap();
        assert!(result.is_none());
    }
}
