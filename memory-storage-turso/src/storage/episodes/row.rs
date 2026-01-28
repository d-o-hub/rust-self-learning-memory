//! # Episode Row Conversion
//!
//! Database row to Episode conversion operations.

use crate::TursoStorage;
use memory_core::{semantic::EpisodeSummary, Episode, Error, Result, TaskType};
use uuid::Uuid;

/// Convert a database row to an Episode
impl TursoStorage {
    pub(crate) fn row_to_episode(row: &libsql::Row) -> Result<Episode> {
        let episode_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let task_type: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let task_description: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
        let start_time_timestamp: i64 = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
        let end_time_timestamp: Option<i64> = row.get(5).ok();
        let steps_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
        let outcome_json: Option<String> = row.get(7).ok();
        let reward_json: Option<String> = row.get(8).ok();
        let reflection_json: Option<String> = row.get(9).ok();
        let patterns_json: String = row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
        let heuristics_json: String = row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
        let metadata_json: String = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
        let _domain: String = row.get(13).map_err(|e| Error::Storage(e.to_string()))?;
        let _language: Option<String> = row.get(14).ok();
        let archived_at: Option<i64> = row.get(15).ok();

        let context: memory_core::TaskContext = serde_json::from_str(&context_json)
            .map_err(|e| Error::Storage(format!("Failed to parse context: {}", e)))?;
        let steps: Vec<memory_core::episode::ExecutionStep> = serde_json::from_str(&steps_json)
            .map_err(|e| Error::Storage(format!("Failed to parse steps: {}", e)))?;
        let outcome = outcome_json
            .map(|s| serde_json::from_str::<memory_core::TaskOutcome>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse outcome: {}", e)))?;
        let reward = reward_json
            .map(|s| serde_json::from_str::<memory_core::types::RewardScore>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse reward: {}", e)))?;
        let reflection = reflection_json
            .map(|s| serde_json::from_str::<memory_core::Reflection>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse reflection: {}", e)))?;

        // Parse patterns (with decompression if compression is enabled)
        #[cfg(feature = "compression")]
        use super::crud::decompress_json_field;
        #[cfg(feature = "compression")]
        let patterns_bytes = decompress_json_field(&patterns_json)?;
        #[cfg(not(feature = "compression"))]
        let patterns_bytes = patterns_json.as_bytes().to_vec();

        let patterns_str = String::from_utf8(patterns_bytes)
            .map_err(|e| Error::Storage(format!("Failed to convert patterns from UTF-8: {}", e)))?;
        let patterns: Vec<memory_core::episode::PatternId> = serde_json::from_str(&patterns_str)
            .map_err(|e| Error::Storage(format!("Failed to parse patterns: {}", e)))?;

        // Parse heuristics (with decompression if compression is enabled)
        #[cfg(feature = "compression")]
        let heuristics_bytes = decompress_json_field(&heuristics_json)?;
        #[cfg(not(feature = "compression"))]
        let heuristics_bytes = heuristics_json.as_bytes().to_vec();

        let heuristics_str = String::from_utf8(heuristics_bytes).map_err(|e| {
            Error::Storage(format!("Failed to convert heuristics from UTF-8: {}", e))
        })?;
        let heuristics: Vec<Uuid> = serde_json::from_str(&heuristics_str)
            .map_err(|e| Error::Storage(format!("Failed to parse heuristics: {}", e)))?;

        // Parse metadata (with decompression if compression is enabled)
        #[cfg(feature = "compression")]
        let metadata_bytes = decompress_json_field(&metadata_json)?;
        #[cfg(not(feature = "compression"))]
        let metadata_bytes = metadata_json.as_bytes().to_vec();

        let metadata_str = String::from_utf8(metadata_bytes)
            .map_err(|e| Error::Storage(format!("Failed to convert metadata from UTF-8: {}", e)))?;
        let mut metadata: std::collections::HashMap<String, String> =
            serde_json::from_str(&metadata_str)
                .map_err(|e| Error::Storage(format!("Failed to parse metadata: {}", e)))?;

        // Add archived_at to metadata if present in database
        if let Some(ts) = archived_at {
            metadata.insert("archived_at".to_string(), ts.to_string());
        }

        Ok(Episode {
            episode_id: uuid::Uuid::parse_str(&episode_id)
                .map_err(|e| Error::Storage(format!("Invalid episode ID: {}", e)))?,
            task_type: task_type
                .parse::<TaskType>()
                .map_err(|e| Error::Storage(e.to_string()))?,
            task_description,
            context,
            steps,
            outcome,
            reward,
            reflection,
            patterns,
            heuristics,
            applied_patterns: Vec::new(),
            salient_features: None,
            tags: vec![],
            start_time: chrono::DateTime::from_timestamp(start_time_timestamp, 0)
                .unwrap_or_default(),
            end_time: end_time_timestamp.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            metadata,
        })
    }
}

/// Convert a database row to an EpisodeSummary
impl TursoStorage {
    pub(crate) fn row_to_summary(row: &libsql::Row) -> Result<EpisodeSummary> {
        let episode_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let summary_text: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let key_concepts_json: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let key_steps_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
        let embedding_json: Option<String> = row.get(4).ok();
        let created_at_timestamp: i64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;

        let key_concepts: Vec<String> = serde_json::from_str(&key_concepts_json)
            .map_err(|e| Error::Storage(format!("Failed to parse key concepts: {}", e)))?;
        let key_steps: Vec<String> = serde_json::from_str(&key_steps_json)
            .map_err(|e| Error::Storage(format!("Failed to parse key steps: {}", e)))?;
        let summary_embedding = embedding_json
            .as_ref()
            .map(|s| serde_json::from_str::<Vec<f32>>(s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse embedding: {}", e)))?;

        Ok(EpisodeSummary {
            episode_id: uuid::Uuid::parse_str(&episode_id)
                .map_err(|e| Error::Storage(format!("Invalid episode ID: {}", e)))?,
            summary_text,
            key_concepts,
            key_steps,
            summary_embedding,
            created_at: chrono::DateTime::from_timestamp(created_at_timestamp, 0)
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{ComplexityLevel, Episode, TaskContext, TaskType};
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
    async fn test_row_to_episode_roundtrip() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let episode = Episode::new(
            "Roundtrip test".to_string(),
            TaskContext {
                domain: "test-domain".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration,
        );

        let episode_id = episode.episode_id;
        storage.store_episode(&episode).await.unwrap();

        let retrieved = storage.get_episode(episode_id).await.unwrap().unwrap();
        assert_eq!(retrieved.task_description, "Roundtrip test");
        assert_eq!(retrieved.context.domain, "test-domain");
        assert_eq!(retrieved.context.language, Some("rust".to_string()));
    }

    #[tokio::test]
    async fn test_row_to_episode_with_metadata() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let mut episode = Episode::new(
            "With metadata".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        );
        episode
            .metadata
            .insert("priority".to_string(), "high".to_string());
        episode
            .metadata
            .insert("assigned_to".to_string(), "developer".to_string());

        let episode_id = episode.episode_id;
        storage.store_episode(&episode).await.unwrap();

        let retrieved = storage.get_episode(episode_id).await.unwrap().unwrap();
        assert_eq!(
            retrieved.metadata.get("priority"),
            Some(&"high".to_string())
        );
        assert_eq!(
            retrieved.metadata.get("assigned_to"),
            Some(&"developer".to_string())
        );
    }
}
