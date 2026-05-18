//! Storage implementation for episode relationships in Turso.
//!
//! Provides CRUD operations for managing relationships between episodes.

use crate::{Result, TursoStorage};
use do_memory_core::episode::{
    Direction, EpisodePatternRelationship, EpisodeRelationship, RelationshipMetadata,
    RelationshipType,
};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

impl TursoStorage {
    /// Add a relationship between two episodes
    pub async fn add_relationship(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        metadata: RelationshipMetadata,
    ) -> Result<Uuid> {
        let conn = self.get_connection().await?;
        let relationship_id = Uuid::new_v4();
        let created_at = chrono::Utc::now().timestamp();

        let metadata_json = serde_json::to_string(&metadata.custom_fields)
            .map_err(|e| do_memory_core::Error::Storage(format!("Serialization error: {}", e)))?;

        self.execute_with_retry(
            &conn,
            &format!(
                "{} VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, '{}', {})",
                "INSERT INTO episode_relationships (relationship_id, from_episode_id, to_episode_id, relationship_type, reason, created_by, priority, weight, metadata, created_at)",
                relationship_id,
                from_episode_id,
                to_episode_id,
                relationship_type.as_str(),
                metadata.reason.as_ref().map(|r| format!("'{}'", r.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                metadata.created_by.as_ref().map(|c| format!("'{}'", c.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                metadata.priority.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string()),
                metadata.weight.map(|w| w.to_string()).unwrap_or_else(|| "NULL".to_string()),
                metadata_json.replace('\'', "''"),
                created_at
            ),
        )
        .await?;

        debug!(
            "Added relationship {} from {} to {} (type: {:?})",
            relationship_id, from_episode_id, to_episode_id, relationship_type
        );

        Ok(relationship_id)
    }

    /// Store a relationship between two episodes
    ///
    /// This is the StorageBackend trait implementation that takes a pre-built EpisodeRelationship.
    pub async fn store_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        let conn = self.get_connection().await?;
        let created_at = relationship.created_at.timestamp();

        let metadata_json = serde_json::to_string(&relationship.metadata.custom_fields)
            .map_err(|e| do_memory_core::Error::Storage(format!("Serialization error: {}", e)))?;

        self.execute_with_retry(
            &conn,
            &format!(
                "{} VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, '{}', {})",
                "INSERT INTO episode_relationships (relationship_id, from_episode_id, to_episode_id, relationship_type, reason, created_by, priority, weight, metadata, created_at)",
                relationship.id,
                relationship.from_episode_id,
                relationship.to_episode_id,
                relationship.relationship_type.as_str(),
                relationship.metadata.reason.as_ref().map(|r| format!("'{}'", r.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                relationship.metadata.created_by.as_ref().map(|c| format!("'{}'", c.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                relationship.metadata.priority.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string()),
                relationship.metadata.weight.map(|w| w.to_string()).unwrap_or_else(|| "NULL".to_string()),
                metadata_json.replace('\'', "''"),
                created_at
            ),
        )
        .await?;

        debug!(
            "Stored relationship {} from {} to {} (type: {:?})",
            relationship.id,
            relationship.from_episode_id,
            relationship.to_episode_id,
            relationship.relationship_type
        );

        Ok(())
    }

    /// Remove a relationship by ID
    pub async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        let conn = self.get_connection().await?;

        let sql = format!(
            "DELETE FROM episode_relationships WHERE relationship_id = '{}'",
            relationship_id
        );

        self.execute_with_retry(&conn, &sql).await?;

        debug!("Removed relationship {}", relationship_id);
        Ok(())
    }

    /// Get relationships for an episode
    pub async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        let conn = self.get_connection().await?;

        let sql = match direction {
            Direction::Outgoing => format!(
                "SELECT * FROM episode_relationships WHERE from_episode_id = '{}'",
                episode_id
            ),
            Direction::Incoming => format!(
                "SELECT * FROM episode_relationships WHERE to_episode_id = '{}'",
                episode_id
            ),
            Direction::Both => format!(
                "SELECT * FROM episode_relationships WHERE from_episode_id = '{}' OR to_episode_id = '{}'",
                episode_id, episode_id
            ),
        };

        let stmt = conn.prepare(&sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;

        let mut relationships = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let relationship = self.row_to_relationship(&row)?;
            relationships.push(relationship);
        }

        debug!(
            "Found {} relationships for episode {} (direction: {:?})",
            relationships.len(),
            episode_id,
            direction
        );

        Ok(relationships)
    }

    /// Get relationships by type
    pub async fn get_relationships_by_type(
        &self,
        episode_id: Uuid,
        relationship_type: RelationshipType,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        let conn = self.get_connection().await?;

        let sql = match direction {
            Direction::Outgoing => format!(
                "SELECT * FROM episode_relationships WHERE from_episode_id = '{}' AND relationship_type = '{}'",
                episode_id,
                relationship_type.as_str()
            ),
            Direction::Incoming => format!(
                "SELECT * FROM episode_relationships WHERE to_episode_id = '{}' AND relationship_type = '{}'",
                episode_id,
                relationship_type.as_str()
            ),
            Direction::Both => format!(
                "SELECT * FROM episode_relationships WHERE (from_episode_id = '{}' OR to_episode_id = '{}') AND relationship_type = '{}'",
                episode_id,
                episode_id,
                relationship_type.as_str()
            ),
        };

        let stmt = conn.prepare(&sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;

        let mut relationships = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let relationship = self.row_to_relationship(&row)?;
            relationships.push(relationship);
        }

        Ok(relationships)
    }

    /// Check if a relationship exists
    pub async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        let conn = self.get_connection().await?;

        let sql = format!(
            "SELECT COUNT(*) as count FROM episode_relationships WHERE from_episode_id = '{}' AND to_episode_id = '{}' AND relationship_type = '{}'",
            from_episode_id,
            to_episode_id,
            relationship_type.as_str()
        );

        let stmt = conn.prepare(&sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let count: i64 = row.get(0).map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to get count: {}", e))
            })?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Helper to convert a database row to an EpisodeRelationship
    fn row_to_relationship(&self, row: &libsql::Row) -> Result<EpisodeRelationship> {
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

    /// Get all episodes that depend on the given episode (blocking it)
    pub async fn get_dependent_episodes(&self, episode_id: Uuid) -> Result<Vec<Uuid>> {
        let relationships = self
            .get_relationships_by_type(episode_id, RelationshipType::DependsOn, Direction::Incoming)
            .await?;

        Ok(relationships
            .into_iter()
            .map(|r| r.from_episode_id)
            .collect())
    }

    /// Get all episodes that the given episode depends on
    pub async fn get_dependencies(&self, episode_id: Uuid) -> Result<Vec<Uuid>> {
        let relationships = self
            .get_relationships_by_type(episode_id, RelationshipType::DependsOn, Direction::Outgoing)
            .await?;

        Ok(relationships.into_iter().map(|r| r.to_episode_id).collect())
    }

    /// Store a relationship between an episode and a pattern
    pub async fn store_episode_pattern_relationship(
        &self,
        relationship: &EpisodePatternRelationship,
    ) -> Result<()> {
        let conn = self.get_connection().await?;
        let created_at = relationship.created_at.timestamp();

        let metadata_json = serde_json::to_string(&relationship.metadata.custom_fields)
            .map_err(|e| do_memory_core::Error::Storage(format!("Serialization error: {}", e)))?;

        self.execute_with_retry(
            &conn,
            &format!(
                "INSERT INTO episode_pattern_relationships (relationship_id, episode_id, pattern_id, relationship_type, weight, metadata, created_at) VALUES ('{}', '{}', '{}', '{}', {}, '{}', {})",
                relationship.id,
                relationship.episode_id,
                relationship.pattern_id,
                relationship.relationship_type.as_str(),
                relationship.metadata.weight.map(|w| w.to_string()).unwrap_or_else(|| "NULL".to_string()),
                metadata_json.replace('\'', "''"),
                created_at
            ),
        )
        .await?;

        debug!(
            "Stored episode-pattern relationship {} from episode {} to pattern {} (type: {:?})",
            relationship.id,
            relationship.episode_id,
            relationship.pattern_id,
            relationship.relationship_type
        );

        Ok(())
    }

    /// Get pattern relationships for an episode
    pub async fn get_episode_pattern_relationships(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<EpisodePatternRelationship>> {
        let conn = self.get_connection().await?;

        let sql = format!(
            "SELECT * FROM episode_pattern_relationships WHERE episode_id = '{}'",
            episode_id
        );

        let stmt = conn.prepare(&sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;

        let mut relationships = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let rel = self.row_to_episode_pattern_relationship(&row)?;
            relationships.push(rel);
        }

        Ok(relationships)
    }

    /// Helper to convert a database row to an EpisodePatternRelationship
    fn row_to_episode_pattern_relationship(
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

    /// Get weighted neighbors (episodes and patterns) for an episode
    pub async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        let conn = self.get_connection().await?;
        let mut neighbors = Vec::new();

        // 1. Get episode neighbors
        let sql_ep = format!(
            "SELECT to_episode_id, weight FROM episode_relationships WHERE from_episode_id = '{}'",
            episode_id
        );
        let stmt_ep = conn.prepare(&sql_ep).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;
        let mut rows_ep = stmt_ep.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;
        while let Some(row) = rows_ep
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let id_str: String = row.get(0).unwrap();
            let weight: Option<f64> = row.get(1).ok();
            let id = Uuid::parse_str(&id_str).unwrap();
            neighbors.push((id, weight.map(|w| w as f32).unwrap_or(1.0), false));
        }

        // 2. Get pattern neighbors
        let sql_pt = format!(
            "SELECT pattern_id, weight FROM episode_pattern_relationships WHERE episode_id = '{}'",
            episode_id
        );
        let stmt_pt = conn.prepare(&sql_pt).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;
        let mut rows_pt = stmt_pt.query(()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
        })?;
        while let Some(row) = rows_pt
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            let id_str: String = row.get(0).unwrap();
            let weight: Option<f64> = row.get(1).ok();
            let id = Uuid::parse_str(&id_str).unwrap();
            neighbors.push((id, weight.map(|w| w as f32).unwrap_or(1.0), true));
        }

        Ok(neighbors)
    }
}

#[cfg(test)]
#[path = "relationships_tests.rs"]
mod tests;
