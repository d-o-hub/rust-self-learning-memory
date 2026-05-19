use crate::{Result, TursoStorage};
use do_memory_core::episode::{
    EpisodePatternRelationship, EpisodeRelationship, RelationshipMetadata, RelationshipType,
};
use std::collections::HashMap;
use uuid::Uuid;

impl TursoStorage {
    /// Helper to convert a database row to an EpisodeRelationship
    pub(crate) fn row_to_relationship(&self, row: &libsql::Row) -> Result<EpisodeRelationship> {
        let relationship_id_str: String = row.get(0).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get relationship_id: {}", e))
        })?;
        let from_episode_id_str: String = row.get(1).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get from_episode_id: {}", e))
        })?;
        let to_episode_id_str: String = row.get(2).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get to_episode_id: {}", e))
        })?;
        let relationship_type_str: String = row.get(3).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get relationship_type: {}", e))
        })?;

        let reason: Option<String> = row.get(4).ok();
        let created_by: Option<String> = row.get(5).ok();
        let priority: Option<i64> = row.get(6).ok();
        let weight: Option<f64> = row.get(7).ok();
        let metadata_json: String = row.get(8).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get metadata: {}", e))
        })?;
        let created_at_timestamp: i64 = row.get(9).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get created_at: {}", e))
        })?;

        let relationship_id = Uuid::parse_str(&relationship_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid relationship_id UUID: {}", e))
        })?;
        let from_episode_id = Uuid::parse_str(&from_episode_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid from_episode_id UUID: {}", e))
        })?;
        let to_episode_id = Uuid::parse_str(&to_episode_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid to_episode_id UUID: {}", e))
        })?;

        let relationship_type = RelationshipType::parse(&relationship_type_str)
            .map_err(do_memory_core::Error::Storage)?;

        let custom_fields: HashMap<String, String> = serde_json::from_str(&metadata_json)
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;

        let metadata = RelationshipMetadata {
            reason,
            created_by,
            priority: priority.map(|p| p as u8),
            weight: weight.map(|w| w as f32),
            custom_fields,
        };

        let created_at = chrono::DateTime::from_timestamp(created_at_timestamp, 0)
            .ok_or_else(|| do_memory_core::Error::Storage("Invalid timestamp".to_string()))?;

        Ok(EpisodeRelationship {
            id: relationship_id,
            from_episode_id,
            to_episode_id,
            relationship_type,
            metadata,
            created_at,
        })
    }

    /// Helper to convert a database row to an EpisodePatternRelationship
    pub(crate) fn row_to_episode_pattern_relationship(
        &self,
        row: &libsql::Row,
    ) -> Result<EpisodePatternRelationship> {
        let relationship_id_str: String = row.get(0).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get relationship_id: {}", e))
        })?;
        let episode_id_str: String = row.get(1).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get episode_id: {}", e))
        })?;
        let pattern_id_str: String = row.get(2).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get pattern_id: {}", e))
        })?;
        let relationship_type_str: String = row.get(3).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get relationship_type: {}", e))
        })?;

        let weight: Option<f64> = row.get(4).ok();
        let metadata_json: String = row.get(5).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get metadata: {}", e))
        })?;
        let created_at_timestamp: i64 = row.get(6).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to get created_at: {}", e))
        })?;

        let relationship_id = Uuid::parse_str(&relationship_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid relationship_id UUID: {}", e))
        })?;
        let episode_id = Uuid::parse_str(&episode_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid episode_id UUID: {}", e))
        })?;
        let pattern_id = Uuid::parse_str(&pattern_id_str).map_err(|e| {
            do_memory_core::Error::Storage(format!("Invalid pattern_id UUID: {}", e))
        })?;

        let relationship_type = RelationshipType::parse(&relationship_type_str)
            .map_err(do_memory_core::Error::Storage)?;

        let custom_fields: HashMap<String, String> = serde_json::from_str(&metadata_json)
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;

        let metadata = RelationshipMetadata {
            reason: None,
            created_by: None,
            priority: None,
            weight: weight.map(|w| w as f32),
            custom_fields,
        };

        let created_at = chrono::DateTime::from_timestamp(created_at_timestamp, 0)
            .ok_or_else(|| do_memory_core::Error::Storage("Invalid timestamp".to_string()))?;

        Ok(EpisodePatternRelationship {
            id: relationship_id,
            episode_id,
            pattern_id,
            relationship_type,
            metadata,
            created_at,
        })
    }
}
