//! Storage implementation for episode relationships in Turso.
//!
//! Provides CRUD operations for managing relationships between episodes.

use crate::{Result, TursoStorage};
use do_memory_core::episode::{
    Direction, EpisodePatternRelationship, EpisodeRelationship, RelationshipMetadata,
    RelationshipType,
};
use tracing::debug;
use uuid::Uuid;

#[path = "relationships_helpers.rs"]
mod relationships_helpers;

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

        const SQL: &str = "INSERT INTO episode_relationships (relationship_id, from_episode_id, to_episode_id, relationship_type, reason, created_by, priority, weight, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        conn.execute(
            SQL,
            libsql::params![
                relationship_id.to_string(),
                from_episode_id.to_string(),
                to_episode_id.to_string(),
                relationship_type.as_str(),
                metadata.reason,
                metadata.created_by,
                metadata.priority,
                metadata.weight,
                metadata_json,
                created_at
            ],
        )
        .await
        .map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to add relationship: {}", e))
        })?;

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

        const SQL: &str = "INSERT INTO episode_relationships (relationship_id, from_episode_id, to_episode_id, relationship_type, reason, created_by, priority, weight, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        conn.execute(
            SQL,
            libsql::params![
                relationship.id.to_string(),
                relationship.from_episode_id.to_string(),
                relationship.to_episode_id.to_string(),
                relationship.relationship_type.as_str(),
                relationship.metadata.reason.clone(),
                relationship.metadata.created_by.clone(),
                relationship.metadata.priority,
                relationship.metadata.weight,
                metadata_json,
                created_at
            ],
        )
        .await
        .map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to store relationship: {}", e))
        })?;

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

        const SQL: &str = "DELETE FROM episode_relationships WHERE relationship_id = ?";

        conn.execute(SQL, libsql::params![relationship_id.to_string()])
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to remove relationship: {}", e))
            })?;

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

        let mut params: Vec<libsql::Value> = Vec::new();
        let sql = match direction {
            Direction::Outgoing => {
                params.push(episode_id.to_string().into());
                "SELECT * FROM episode_relationships WHERE from_episode_id = ?"
            }
            Direction::Incoming => {
                params.push(episode_id.to_string().into());
                "SELECT * FROM episode_relationships WHERE to_episode_id = ?"
            }
            Direction::Both => {
                params.push(episode_id.to_string().into());
                params.push(episode_id.to_string().into());
                "SELECT * FROM episode_relationships WHERE from_episode_id = ? OR to_episode_id = ?"
            }
        };

        let stmt = conn.prepare(sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt
            .query(libsql::params_from_iter(params))
            .await
            .map_err(|e| {
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

        let mut params: Vec<libsql::Value> = Vec::new();
        let sql = match direction {
            Direction::Outgoing => {
                params.push(episode_id.to_string().into());
                params.push(relationship_type.as_str().into());
                "SELECT * FROM episode_relationships WHERE from_episode_id = ? AND relationship_type = ?"
            }
            Direction::Incoming => {
                params.push(episode_id.to_string().into());
                params.push(relationship_type.as_str().into());
                "SELECT * FROM episode_relationships WHERE to_episode_id = ? AND relationship_type = ?"
            }
            Direction::Both => {
                params.push(episode_id.to_string().into());
                params.push(episode_id.to_string().into());
                params.push(relationship_type.as_str().into());
                "SELECT * FROM episode_relationships WHERE (from_episode_id = ? OR to_episode_id = ?) AND relationship_type = ?"
            }
        };

        let stmt = conn.prepare(sql).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt
            .query(libsql::params_from_iter(params))
            .await
            .map_err(|e| {
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

        const SQL: &str = "SELECT COUNT(*) as count FROM episode_relationships WHERE from_episode_id = ? AND to_episode_id = ? AND relationship_type = ?";

        let stmt = conn.prepare(SQL).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt
            .query(libsql::params![
                from_episode_id.to_string(),
                to_episode_id.to_string(),
                relationship_type.as_str()
            ])
            .await
            .map_err(|e| {
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

    /// Fetch every relationship across the entire store (WG-150 / WG-151, ADR-055).
    ///
    /// Used by [`get_relationship_by_id`](Self::get_relationship_by_id) and by
    /// global cycle validation in the CLI. O(N) in the number of stored
    /// relationships; callers should cap input sizes for very large stores.
    pub async fn get_all_relationships(&self) -> Result<Vec<EpisodeRelationship>> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare("SELECT * FROM episode_relationships")
            .await
            .map_err(|e| {
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
            relationships.push(self.row_to_relationship(&row)?);
        }
        debug!(
            "Loaded {} relationships from durable store",
            relationships.len()
        );
        Ok(relationships)
    }

    /// Look up a single relationship by its ID (WG-150, ADR-055).
    ///
    /// Returns `Ok(None)` when not found rather than an error so callers can
    /// distinguish "missing" from "I/O failure".
    pub async fn get_relationship_by_id(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<EpisodeRelationship>> {
        let conn = self.get_connection().await?;
        const SQL: &str = "SELECT * FROM episode_relationships WHERE relationship_id = ?";

        let stmt = conn.prepare(SQL).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;
        let mut rows = stmt
            .query(libsql::params![relationship_id.to_string()])
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
            })?;
        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            Ok(Some(self.row_to_relationship(&row)?))
        } else {
            Ok(None)
        }
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

        const SQL: &str = "INSERT INTO episode_pattern_relationships (relationship_id, episode_id, pattern_id, relationship_type, weight, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)";

        conn.execute(
            SQL,
            libsql::params![
                relationship.id.to_string(),
                relationship.episode_id.to_string(),
                relationship.pattern_id.to_string(),
                relationship.relationship_type.as_str(),
                relationship.metadata.weight,
                metadata_json,
                created_at
            ],
        )
        .await
        .map_err(|e| {
            do_memory_core::Error::Storage(format!(
                "Failed to store episode-pattern relationship: {}",
                e
            ))
        })?;

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

        const SQL: &str = "SELECT * FROM episode_pattern_relationships WHERE episode_id = ?";

        let stmt = conn.prepare(SQL).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;

        let mut rows = stmt
            .query(libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| {
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

    /// Get weighted neighbors (episodes and patterns) for an episode
    pub async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        let conn = self.get_connection().await?;
        let mut neighbors = Vec::new();

        // 1. Get episode neighbors
        const SQL_EP: &str =
            "SELECT to_episode_id, weight FROM episode_relationships WHERE from_episode_id = ?";
        let stmt_ep = conn.prepare(SQL_EP).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;
        let mut rows_ep = stmt_ep
            .query(libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| {
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
        const SQL_PT: &str =
            "SELECT pattern_id, weight FROM episode_pattern_relationships WHERE episode_id = ?";
        let stmt_pt = conn.prepare(SQL_PT).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
        })?;
        let mut rows_pt = stmt_pt
            .query(libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| {
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
