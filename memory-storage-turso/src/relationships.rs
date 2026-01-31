//! Storage implementation for episode relationships in Turso.
//!
//! Provides CRUD operations for managing relationships between episodes.

use crate::{Result, TursoStorage};
use memory_core::episode::{Direction, EpisodeRelationship, RelationshipMetadata, RelationshipType};
use serde_json;
use std::collections::HashMap;
use tracing::{debug, warn};
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
            .map_err(|e| memory_core::Error::SerializationError(e.to_string()))?;

        let sql = r#"
            INSERT INTO episode_relationships (
                relationship_id, from_episode_id, to_episode_id, 
                relationship_type, reason, created_by, priority, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.execute_with_retry(
            &conn,
            &format!(
                "{} VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, '{}', {})",
                "INSERT INTO episode_relationships (relationship_id, from_episode_id, to_episode_id, relationship_type, reason, created_by, priority, metadata, created_at)",
                relationship_id,
                from_episode_id,
                to_episode_id,
                relationship_type.as_str(),
                metadata.reason.as_ref().map(|r| format!("'{}'", r.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                metadata.created_by.as_ref().map(|c| format!("'{}'", c.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                metadata.priority.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string()),
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

        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(())
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to execute query: {}", e)))?;

        let mut relationships = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to fetch row: {}", e)))?
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
                episode_id, relationship_type.as_str()
            ),
            Direction::Incoming => format!(
                "SELECT * FROM episode_relationships WHERE to_episode_id = '{}' AND relationship_type = '{}'",
                episode_id, relationship_type.as_str()
            ),
            Direction::Both => format!(
                "SELECT * FROM episode_relationships WHERE (from_episode_id = '{}' OR to_episode_id = '{}') AND relationship_type = '{}'",
                episode_id, episode_id, relationship_type.as_str()
            ),
        };

        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(())
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to execute query: {}", e)))?;

        let mut relationships = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to fetch row: {}", e)))?
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
            from_episode_id, to_episode_id, relationship_type.as_str()
        );

        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(())
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to execute query: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to fetch row: {}", e)))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| memory_core::Error::StorageError(format!("Failed to get count: {}", e)))?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Helper to convert a database row to an EpisodeRelationship
    fn row_to_relationship(&self, row: &libsql::Row) -> Result<EpisodeRelationship> {
        let relationship_id_str: String = row
            .get(0)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get relationship_id: {}", e)))?;
        let from_episode_id_str: String = row
            .get(1)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get from_episode_id: {}", e)))?;
        let to_episode_id_str: String = row
            .get(2)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get to_episode_id: {}", e)))?;
        let relationship_type_str: String = row
            .get(3)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get relationship_type: {}", e)))?;
        
        let reason: Option<String> = row.get(4).ok();
        let created_by: Option<String> = row.get(5).ok();
        let priority: Option<i64> = row.get(6).ok();
        let metadata_json: String = row
            .get(7)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get metadata: {}", e)))?;
        let created_at_timestamp: i64 = row
            .get(8)
            .map_err(|e| memory_core::Error::StorageError(format!("Failed to get created_at: {}", e)))?;

        let relationship_id = Uuid::parse_str(&relationship_id_str)
            .map_err(|e| memory_core::Error::StorageError(format!("Invalid relationship_id UUID: {}", e)))?;
        let from_episode_id = Uuid::parse_str(&from_episode_id_str)
            .map_err(|e| memory_core::Error::StorageError(format!("Invalid from_episode_id UUID: {}", e)))?;
        let to_episode_id = Uuid::parse_str(&to_episode_id_str)
            .map_err(|e| memory_core::Error::StorageError(format!("Invalid to_episode_id UUID: {}", e)))?;

        let relationship_type = RelationshipType::from_str(&relationship_type_str)
            .map_err(|e| memory_core::Error::StorageError(e))?;

        let custom_fields: HashMap<String, String> = serde_json::from_str(&metadata_json)
            .map_err(|e| memory_core::Error::DeserializationError(e.to_string()))?;

        let metadata = RelationshipMetadata {
            reason,
            created_by,
            priority: priority.map(|p| p as u8),
            custom_fields,
        };

        let created_at = chrono::DateTime::from_timestamp(created_at_timestamp, 0)
            .ok_or_else(|| memory_core::Error::StorageError("Invalid timestamp".to_string()))?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::episode::Episode;
    use memory_core::{TaskContext, TaskType};

    async fn create_test_storage() -> TursoStorage {
        let storage = TursoStorage::new_in_memory()
            .await
            .expect("Failed to create in-memory storage");
        storage
            .initialize_schema()
            .await
            .expect("Failed to initialize schema");
        storage
    }

    async fn create_test_episode(storage: &TursoStorage) -> Uuid {
        let episode = Episode::new(
            "Test episode".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );
        let episode_id = episode.episode_id;
        storage
            .store_episode(episode)
            .await
            .expect("Failed to store episode");
        episode_id
    }

    #[tokio::test]
    async fn test_add_relationship() {
        let storage = create_test_storage().await;
        let from_id = create_test_episode(&storage).await;
        let to_id = create_test_episode(&storage).await;

        let metadata = RelationshipMetadata::with_reason("Test relationship".to_string());
        let rel_id = storage
            .add_relationship(from_id, to_id, RelationshipType::ParentChild, metadata)
            .await
            .expect("Failed to add relationship");

        assert_ne!(rel_id, Uuid::nil());
    }

    #[tokio::test]
    async fn test_get_relationships() {
        let storage = create_test_storage().await;
        let from_id = create_test_episode(&storage).await;
        let to_id = create_test_episode(&storage).await;

        let metadata = RelationshipMetadata::with_reason("Test".to_string());
        storage
            .add_relationship(from_id, to_id, RelationshipType::DependsOn, metadata)
            .await
            .expect("Failed to add relationship");

        let outgoing = storage
            .get_relationships(from_id, Direction::Outgoing)
            .await
            .expect("Failed to get outgoing relationships");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].from_episode_id, from_id);
        assert_eq!(outgoing[0].to_episode_id, to_id);

        let incoming = storage
            .get_relationships(to_id, Direction::Incoming)
            .await
            .expect("Failed to get incoming relationships");
        assert_eq!(incoming.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_relationship() {
        let storage = create_test_storage().await;
        let from_id = create_test_episode(&storage).await;
        let to_id = create_test_episode(&storage).await;

        let metadata = RelationshipMetadata::with_reason("Test".to_string());
        let rel_id = storage
            .add_relationship(from_id, to_id, RelationshipType::ParentChild, metadata)
            .await
            .expect("Failed to add relationship");

        storage
            .remove_relationship(rel_id)
            .await
            .expect("Failed to remove relationship");

        let relationships = storage
            .get_relationships(from_id, Direction::Outgoing)
            .await
            .expect("Failed to get relationships");
        assert_eq!(relationships.len(), 0);
    }

    #[tokio::test]
    async fn test_relationship_exists() {
        let storage = create_test_storage().await;
        let from_id = create_test_episode(&storage).await;
        let to_id = create_test_episode(&storage).await;

        let exists_before = storage
            .relationship_exists(from_id, to_id, RelationshipType::DependsOn)
            .await
            .expect("Failed to check relationship existence");
        assert!(!exists_before);

        let metadata = RelationshipMetadata::with_reason("Test".to_string());
        storage
            .add_relationship(from_id, to_id, RelationshipType::DependsOn, metadata)
            .await
            .expect("Failed to add relationship");

        let exists_after = storage
            .relationship_exists(from_id, to_id, RelationshipType::DependsOn)
            .await
            .expect("Failed to check relationship existence");
        assert!(exists_after);
    }

    #[tokio::test]
    async fn test_get_dependencies() {
        let storage = create_test_storage().await;
        let ep1 = create_test_episode(&storage).await;
        let ep2 = create_test_episode(&storage).await;
        let ep3 = create_test_episode(&storage).await;

        // ep1 depends on ep2 and ep3
        let metadata = RelationshipMetadata::with_reason("Dependency".to_string());
        storage
            .add_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
            .await
            .expect("Failed to add relationship");
        storage
            .add_relationship(ep1, ep3, RelationshipType::DependsOn, metadata)
            .await
            .expect("Failed to add relationship");

        let deps = storage
            .get_dependencies(ep1)
            .await
            .expect("Failed to get dependencies");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&ep2));
        assert!(deps.contains(&ep3));
    }
}
