//! # Episode Storage Module
//!
//! Episode CRUD operations and query functionality.

pub mod crud;
pub mod query;
pub mod row;

use memory_core::{semantic::EpisodeSummary, Episode, Error, Result, TaskType};
use uuid::Uuid;

/// Query builder for episodes
#[derive(Debug, Clone, Default)]
pub struct EpisodeQuery {
    pub task_type: Option<TaskType>,
    pub domain: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
    pub completed_only: bool,
}

/// Compress JSON data if compression is enabled and data is large enough
#[cfg(feature = "compression")]
pub fn compress_json_field(data: &[u8], threshold: usize) -> Result<Vec<u8>> {
    use crate::compression::CompressedPayload;

    let compressed = CompressedPayload::compress(data, threshold)?;
    if compressed.algorithm == crate::CompressionAlgorithm::None {
        Ok(data.to_vec())
    } else {
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
    if let Some(remainder) = data.strip_prefix("__compressed__:") {
        let newline_pos = remainder.find('\n').ok_or_else(|| {
            Error::Storage("Invalid compressed data format: missing newline".to_string())
        })?;
        let header = &remainder[..newline_pos];
        let encoded_data = &remainder[newline_pos + 1..];

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
        Ok(data.as_bytes().to_vec())
    }
}

/// Convert a database row to an Episode (pub(crate) for internal use)
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

    #[cfg(feature = "compression")]
    let patterns_bytes = decompress_json_field(&patterns_json)?;
    #[cfg(not(feature = "compression"))]
    let patterns_bytes = patterns_json.as_bytes().to_vec();

    let patterns_str = String::from_utf8(patterns_bytes)
        .map_err(|e| Error::Storage(format!("Failed to convert patterns from UTF-8: {}", e)))?;
    let patterns: Vec<memory_core::episode::PatternId> = serde_json::from_str(&patterns_str)
        .map_err(|e| Error::Storage(format!("Failed to parse patterns: {}", e)))?;

    #[cfg(feature = "compression")]
    let heuristics_bytes = decompress_json_field(&heuristics_json)?;
    #[cfg(not(feature = "compression"))]
    let heuristics_bytes = heuristics_json.as_bytes().to_vec();

    let heuristics_str = String::from_utf8(heuristics_bytes)
        .map_err(|e| Error::Storage(format!("Failed to convert heuristics from UTF-8: {}", e)))?;
    let heuristics: Vec<Uuid> = serde_json::from_str(&heuristics_str)
        .map_err(|e| Error::Storage(format!("Failed to parse heuristics: {}", e)))?;

    #[cfg(feature = "compression")]
    let metadata_bytes = decompress_json_field(&metadata_json)?;
    #[cfg(not(feature = "compression"))]
    let metadata_bytes = metadata_json.as_bytes().to_vec();

    let metadata_str = String::from_utf8(metadata_bytes)
        .map_err(|e| Error::Storage(format!("Failed to convert metadata from UTF-8: {}", e)))?;
    let mut metadata: std::collections::HashMap<String, String> =
        serde_json::from_str(&metadata_str)
            .map_err(|e| Error::Storage(format!("Failed to parse metadata: {}", e)))?;

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
        start_time: chrono::DateTime::from_timestamp(start_time_timestamp, 0).unwrap_or_default(),
        end_time: end_time_timestamp.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
        metadata,
    })
}

/// Convert a database row to an EpisodeSummary (pub(crate) for internal use)
#[allow(dead_code)]
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
        created_at: chrono::DateTime::from_timestamp(created_at_timestamp, 0).unwrap_or_default(),
    })
}
