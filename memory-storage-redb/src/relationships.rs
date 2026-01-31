//! Redb cache layer for episode relationships.

use crate::{RedbStorage, Result, RELATIONSHIPS_TABLE};
use memory_core::episode::{Direction, EpisodeRelationship};
use redb::{ReadableTable, ReadableTableMetadata};
use tracing::debug;
use uuid::Uuid;

impl RedbStorage {
    /// Cache a relationship
    pub fn cache_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            let key = relationship.id.to_string();
            let value = postcard::to_allocvec(relationship)
                .map_err(|e| memory_core::Error::Storage(format!("Serialization error: {}", e)))?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(|e| memory_core::Error::Storage(format!("Insert failed: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!("Cached relationship {} in redb", relationship.id);
        Ok(())
    }

    /// Get a cached relationship by ID
    pub fn get_cached_relationship(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<EpisodeRelationship>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
        let key = relationship_id.to_string();

        match table
            .get(key.as_str())
            .map_err(|e| memory_core::Error::Storage(format!("Get failed: {}", e)))?
        {
            Some(value) => {
                let bytes = value.value();
                let relationship: EpisodeRelationship =
                    postcard::from_bytes(bytes).map_err(|e| {
                        memory_core::Error::Storage(format!("Deserialization error: {}", e))
                    })?;
                Ok(Some(relationship))
            }
            None => Ok(None),
        }
    }

    /// Remove a relationship from cache
    pub fn remove_cached_relationship(&self, relationship_id: Uuid) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            let key = relationship_id.to_string();
            table
                .remove(key.as_str())
                .map_err(|e| memory_core::Error::Storage(format!("Remove failed: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!("Removed relationship {} from cache", relationship_id);
        Ok(())
    }

    /// Get all cached relationships for an episode
    pub fn get_cached_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;

        let mut relationships = Vec::new();
        let iter = table
            .iter()
            .map_err(|e| memory_core::Error::Storage(format!("Iterator creation failed: {}", e)))?;

        for item in iter {
            let (_, value) = item
                .map_err(|e| memory_core::Error::Storage(format!("Iterator next failed: {}", e)))?;
            let bytes = value.value();
            let relationship: EpisodeRelationship = postcard::from_bytes(bytes).map_err(|e| {
                memory_core::Error::Storage(format!("Deserialization error: {}", e))
            })?;

            let matches = match direction {
                Direction::Outgoing => relationship.from_episode_id == episode_id,
                Direction::Incoming => relationship.to_episode_id == episode_id,
                Direction::Both => {
                    relationship.from_episode_id == episode_id
                        || relationship.to_episode_id == episode_id
                }
            };

            if matches {
                relationships.push(relationship);
            }
        }

        debug!(
            "Found {} cached relationships for episode {} (direction: {:?})",
            relationships.len(),
            episode_id,
            direction
        );

        Ok(relationships)
    }

    /// Clear all cached relationships
    pub fn clear_relationships_cache(&self) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            // Remove all entries
            let keys: Vec<String> = {
                let iter = table.iter().map_err(|e| {
                    memory_core::Error::Storage(format!("Iterator creation failed: {}", e))
                })?;
                let mut keys = Vec::new();
                for item in iter {
                    let (key, _) = item.map_err(|e| {
                        memory_core::Error::Storage(format!("Iterator next failed: {}", e))
                    })?;
                    keys.push(key.value().to_string());
                }
                keys
            };

            for key in keys {
                table
                    .remove(key.as_str())
                    .map_err(|e| memory_core::Error::Storage(format!("Remove failed: {}", e)))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!("Cleared all cached relationships");
        Ok(())
    }

    /// Get count of cached relationships
    pub fn count_cached_relationships(&self) -> Result<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
        let count = table.len().map_err(|e| {
            memory_core::Error::Storage(format!("Failed to get table length: {}", e))
        })? as usize;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::episode::RelationshipType;
    use tempfile::TempDir;

    async fn create_test_storage() -> (RedbStorage, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("test.redb");
        let storage = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage");
        (storage, dir)
    }

    fn create_test_relationship(from_id: Uuid, to_id: Uuid) -> EpisodeRelationship {
        EpisodeRelationship::with_reason(
            from_id,
            to_id,
            RelationshipType::ParentChild,
            "Test relationship".to_string(),
        )
    }

    #[tokio::test]
    async fn test_cache_and_get_relationship() {
        let (storage, _dir) = create_test_storage().await;
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let relationship = create_test_relationship(from_id, to_id);
        let rel_id = relationship.id;

        // Cache the relationship
        storage
            .cache_relationship(&relationship)
            .expect("Failed to cache relationship");

        // Retrieve it
        let cached = storage
            .get_cached_relationship(rel_id)
            .expect("Failed to get relationship");
        assert!(cached.is_some());
        let cached_rel = cached.unwrap();
        assert_eq!(cached_rel.id, rel_id);
        assert_eq!(cached_rel.from_episode_id, from_id);
        assert_eq!(cached_rel.to_episode_id, to_id);
    }

    #[tokio::test]
    async fn test_remove_cached_relationship() {
        let (storage, _dir) = create_test_storage().await;
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let relationship = create_test_relationship(from_id, to_id);
        let rel_id = relationship.id;

        storage
            .cache_relationship(&relationship)
            .expect("Failed to cache relationship");

        // Remove it
        storage
            .remove_cached_relationship(rel_id)
            .expect("Failed to remove relationship");

        // Verify it's gone
        let cached = storage
            .get_cached_relationship(rel_id)
            .expect("Failed to get relationship");
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_get_cached_relationships_outgoing() {
        let (storage, _dir) = create_test_storage().await;
        let from_id = Uuid::new_v4();
        let to_id1 = Uuid::new_v4();
        let to_id2 = Uuid::new_v4();

        let rel1 = create_test_relationship(from_id, to_id1);
        let rel2 = create_test_relationship(from_id, to_id2);

        storage.cache_relationship(&rel1).expect("Failed to cache");
        storage.cache_relationship(&rel2).expect("Failed to cache");

        let relationships = storage
            .get_cached_relationships(from_id, Direction::Outgoing)
            .expect("Failed to get relationships");

        assert_eq!(relationships.len(), 2);
        assert!(relationships.iter().any(|r| r.to_episode_id == to_id1));
        assert!(relationships.iter().any(|r| r.to_episode_id == to_id2));
    }

    #[tokio::test]
    async fn test_get_cached_relationships_incoming() {
        let (storage, _dir) = create_test_storage().await;
        let to_id = Uuid::new_v4();
        let from_id1 = Uuid::new_v4();
        let from_id2 = Uuid::new_v4();

        let rel1 = create_test_relationship(from_id1, to_id);
        let rel2 = create_test_relationship(from_id2, to_id);

        storage.cache_relationship(&rel1).expect("Failed to cache");
        storage.cache_relationship(&rel2).expect("Failed to cache");

        let relationships = storage
            .get_cached_relationships(to_id, Direction::Incoming)
            .expect("Failed to get relationships");

        assert_eq!(relationships.len(), 2);
        assert!(relationships.iter().any(|r| r.from_episode_id == from_id1));
        assert!(relationships.iter().any(|r| r.from_episode_id == from_id2));
    }

    #[tokio::test]
    async fn test_clear_relationships_cache() {
        let (storage, _dir) = create_test_storage().await;
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        for _ in 0..5 {
            let rel = create_test_relationship(from_id, to_id);
            storage.cache_relationship(&rel).expect("Failed to cache");
        }

        let count_before = storage
            .count_cached_relationships()
            .expect("Failed to count");
        assert_eq!(count_before, 5);

        storage
            .clear_relationships_cache()
            .expect("Failed to clear cache");

        let count_after = storage
            .count_cached_relationships()
            .expect("Failed to count");
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_count_cached_relationships() {
        let (storage, _dir) = create_test_storage().await;

        // Add at least one relationship to ensure table exists
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let rel = create_test_relationship(from_id, to_id);
        storage.cache_relationship(&rel).expect("Failed to cache");

        let count_initial = storage
            .count_cached_relationships()
            .expect("Failed to count");
        assert_eq!(count_initial, 1);

        // Add 2 more relationships (total should be 3)
        for i in 0..2 {
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();
            let rel = create_test_relationship(from_id, to_id);
            storage.cache_relationship(&rel).expect("Failed to cache");

            let count = storage
                .count_cached_relationships()
                .expect("Failed to count");
            assert_eq!(count, i + 2); // +2 because we start at 1
        }
    }
}
