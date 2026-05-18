//! Database row to Episode conversion operations.

use crate::TursoStorage;
use do_memory_core::types::TaskType;
use do_memory_core::{Episode, Error, Result};
use uuid::Uuid;

use do_memory_core::semantic::EpisodeSummary;

#[cfg(feature = "compression")]
use crate::compression;

impl TursoStorage {
    /// Convert a database row to an Episode
    pub async fn row_to_episode(&self, row: &libsql::Row) -> Result<Episode> {
        row_to_episode(row)
    }
}

/// Convert a database row to an Episode
pub fn row_to_episode(row: &libsql::Row) -> Result<Episode> {
    let id_str: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
    let episode_id = Uuid::parse_str(&id_str).map_err(|e| Error::Storage(e.to_string()))?;

    let task_type_str: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
    let task_type = task_type_str
        .parse::<TaskType>()
        .map_err(|_| Error::Storage(format!("Invalid task type: {}", task_type_str)))?;

    let task_description: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;

    let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
    let context = serde_json::from_str(&context_json).map_err(Error::Serialization)?;

    let start_time_ts: i64 = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
    let start_time = chrono::DateTime::from_timestamp(start_time_ts, 0).unwrap_or_default();

    let end_time_ts: Option<i64> = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
    let end_time = end_time_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));

    let steps_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
    let steps = serde_json::from_str(&steps_json).map_err(Error::Serialization)?;

    let outcome_json: Option<String> = row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
    let outcome = outcome_json
        .and_then(|json| serde_json::from_str(&json).ok());

    let reward_json: Option<String> = row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
    let reward = reward_json
        .and_then(|json| serde_json::from_str(&json).ok());

    let reflection_json: Option<String> = row.get(9).map_err(|e| Error::Storage(e.to_string()))?;
    let reflection = reflection_json
        .and_then(|json| serde_json::from_str(&json).ok());

    let patterns_json: String = row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
    let patterns = serde_json::from_str(&patterns_json).map_err(Error::Serialization)?;

    let heuristics_json: String = row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
    let heuristics = serde_json::from_str(&heuristics_json).map_err(Error::Serialization)?;

    let checkpoints_json: String = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
    let checkpoints = serde_json::from_str(&checkpoints_json).map_err(Error::Serialization)?;

    let metadata_json: String = row.get(13).map_err(|e| Error::Storage(e.to_string()))?;
    let metadata = serde_json::from_str(&metadata_json).map_err(Error::Serialization)?;

    let mut episode = Episode {
        episode_id,
        task_type,
        task_description,
        context,
        start_time,
        end_time,
        steps,
        outcome,
        reward,
        reflection,
        patterns,
        heuristics,
        applied_patterns: Vec::new(),
        salient_features: None,
        metadata,
        tags: Vec::new(),
        checkpoints,
    };

    // Load tags if available (will be handled by caller if needed)
    // episode.tags = ...

    Ok(episode)
}

/// Convert a database row to an EpisodeSummary
pub fn row_to_summary(row: &libsql::Row) -> Result<EpisodeSummary> {
    let episode_id_str: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
    let _episode_id = Uuid::parse_str(&episode_id_str).map_err(|e| Error::Storage(e.to_string()))?;

    let summary_text: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
    let key_concepts_json: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
    let key_steps_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;

    let key_concepts = serde_json::from_str(&key_concepts_json).map_err(Error::Serialization)?;
    let key_steps = serde_json::from_str(&key_steps_json).map_err(Error::Serialization)?;

    let summary_embedding: Option<Vec<u8>> = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
    let embedding = summary_embedding.and_then(|bytes| {
        let mut floats = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            floats.push(f32::from_le_bytes(arr));
        }
        Some(floats)
    });

    Ok(EpisodeSummary {
        summary_text,
        key_concepts,
        key_steps,
        embedding,
    })
}
