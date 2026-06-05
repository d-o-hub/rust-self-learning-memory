//! Redb cache layer for episode relationships.

use crate::{EPISODE_PATTERN_RELATIONSHIPS_TABLE, RELATIONSHIPS_TABLE, RedbStorage, Result};
#[allow(unused_imports)] // False positive - import is used in error mapping
use do_memory_core::Error;
use do_memory_core::episode::{
    Direction, EpisodePatternRelationship, EpisodeRelationship, RelationshipType,
};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};
use tracing::debug;
use uuid::Uuid;

impl RedbStorage {
    // StorageBackend trait implementations

    /// Store a relationship (StorageBackend trait)
    pub async fn store_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        self.cache_relationship(relationship)
    }

    /// Remove a relationship (StorageBackend trait)
    pub async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        self.remove_cached_relationship(relationship_id)
    }

    /// Get relationships (StorageBackend trait)
    pub async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        self.get_cached_relationships(episode_id, direction)
    }

    /// Check if relationship exists (StorageBackend trait)
    pub async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        let relationships = self.get_cached_relationships(from_episode_id, Direction::Outgoing)?;
        Ok(relationships
            .iter()
            .any(|r| r.to_episode_id == to_episode_id && r.relationship_type == relationship_type))
    }

    /// Store a relationship between an episode and a pattern (StorageBackend trait)
    pub async fn store_episode_pattern_relationship(
        &self,
        relationship: &EpisodePatternRelationship,
    ) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(EPISODE_PATTERN_RELATIONSHIPS_TABLE)
                .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            let key = relationship.id.to_string();
            let value = postcard::to_allocvec(relationship).map_err(|e| {
                do_memory_core::Error::Storage(format!("Serialization error: {}", e))
            })?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(|e| do_memory_core::Error::Storage(format!("Insert failed: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| do_memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!(
            "Cached episode-pattern relationship {} in redb",
            relationship.id
        );
        Ok(())
    }

    /// Get pattern relationships for an episode (StorageBackend trait)
    pub async fn get_episode_pattern_relationships(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<EpisodePatternRelationship>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(EPISODE_PATTERN_RELATIONSHIPS_TABLE)
            .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;

        let mut relationships = Vec::new();
        let iter = table.iter().map_err(|e| {
            do_memory_core::Error::Storage(format!("Iterator creation failed: {}", e))
        })?;

        for item in iter {
            let (_, value) = item.map_err(|e| {
                do_memory_core::Error::Storage(format!("Iterator next failed: {}", e))
            })?;
            let bytes = value.value();
            let relationship: EpisodePatternRelationship =
                postcard::from_bytes(bytes).map_err(|e| {
                    do_memory_core::Error::Storage(format!("Deserialization error: {}", e))
                })?;

            if relationship.episode_id == episode_id {
                relationships.push(relationship);
            }
        }

        Ok(relationships)
    }

    /// Get weighted neighbors (episodes and patterns) for an episode (StorageBackend trait)
    pub async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        let mut neighbors = Vec::new();

        // 1. Episode neighbors
        let ep_rels = self.get_cached_relationships(episode_id, Direction::Outgoing)?;
        for rel in ep_rels {
            neighbors.push((rel.to_episode_id, rel.metadata.weight.unwrap_or(1.0), false));
        }

        // 2. Pattern neighbors
        let pt_rels = self.get_episode_pattern_relationships(episode_id).await?;
        for rel in pt_rels {
            neighbors.push((rel.pattern_id, rel.metadata.weight.unwrap_or(1.0), true));
        }

        Ok(neighbors)
    }

    // Original redb-specific methods

    /// Cache a relationship
    pub fn cache_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            let key = relationship.id.to_string();
            let value = postcard::to_allocvec(relationship).map_err(|e| {
                do_memory_core::Error::Storage(format!("Serialization error: {}", e))
            })?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(|e| do_memory_core::Error::Storage(format!("Insert failed: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| do_memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!("Cached relationship {} in redb", relationship.id);
        Ok(())
    }

    /// Get all relationships (StorageBackend trait)
    pub async fn get_all_relationships(&self) -> Result<Vec<EpisodeRelationship>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;

        let mut relationships = Vec::new();
        let iter = table.iter().map_err(|e| {
            do_memory_core::Error::Storage(format!("Iterator creation failed: {}", e))
        })?;

        for item in iter {
            let (_, value) = item.map_err(|e| {
                do_memory_core::Error::Storage(format!("Iterator next failed: {}", e))
            })?;
            let bytes = value.value();
            let relationship: EpisodeRelationship = postcard::from_bytes(bytes).map_err(|e| {
                do_memory_core::Error::Storage(format!("Deserialization error: {}", e))
            })?;
            relationships.push(relationship);
        }

        Ok(relationships)
    }

    /// Get a relationship by its ID (StorageBackend trait)
    pub async fn get_relationship_by_id(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<EpisodeRelationship>> {
        self.get_cached_relationship(relationship_id)
    }

    /// Get a cached relationship by ID
    pub fn get_cached_relationship(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<EpisodeRelationship>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
        let key = relationship_id.to_string();

        match table
            .get(key.as_str())
            .map_err(|e| do_memory_core::Error::Storage(format!("Get failed: {}", e)))?
        {
            Some(value) => {
                let bytes = value.value();
                let relationship: EpisodeRelationship =
                    postcard::from_bytes(bytes).map_err(|e| {
                        do_memory_core::Error::Storage(format!("Deserialization error: {}", e))
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
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            let key = relationship_id.to_string();
            table
                .remove(key.as_str())
                .map_err(|e| do_memory_core::Error::Storage(format!("Remove failed: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| do_memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

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
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;

        let mut relationships = Vec::new();
        let iter = table.iter().map_err(|e| {
            do_memory_core::Error::Storage(format!("Iterator creation failed: {}", e))
        })?;

        for item in iter {
            let (_, value) = item.map_err(|e| {
                do_memory_core::Error::Storage(format!("Iterator next failed: {}", e))
            })?;
            let bytes = value.value();
            let relationship: EpisodeRelationship = postcard::from_bytes(bytes).map_err(|e| {
                do_memory_core::Error::Storage(format!("Deserialization error: {}", e))
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
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin write failed: {}", e)))?;
        {
            let mut table = write_txn
                .open_table(RELATIONSHIPS_TABLE)
                .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
            // Remove all entries
            let keys: Vec<String> = {
                let iter = table.iter().map_err(|e| {
                    do_memory_core::Error::Storage(format!("Iterator creation failed: {}", e))
                })?;
                let mut keys = Vec::new();
                for item in iter {
                    let (key, _) = item.map_err(|e| {
                        do_memory_core::Error::Storage(format!("Iterator next failed: {}", e))
                    })?;
                    keys.push(key.value().to_string());
                }
                keys
            };

            for key in keys {
                table
                    .remove(key.as_str())
                    .map_err(|e| do_memory_core::Error::Storage(format!("Remove failed: {}", e)))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| do_memory_core::Error::Storage(format!("Commit failed: {}", e)))?;

        debug!("Cleared all cached relationships");
        Ok(())
    }

    /// Get count of cached relationships
    pub fn count_cached_relationships(&self) -> Result<usize> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| do_memory_core::Error::Storage(format!("Begin read failed: {}", e)))?;
        let table = read_txn
            .open_table(RELATIONSHIPS_TABLE)
            .map_err(|e| do_memory_core::Error::Storage(format!("Open table failed: {}", e)))?;
        let count = table
            .len()
            .map_err(|e| do_memory_core::Error::Storage(format!("Count failed: {}", e)))?;
        let count = u32::try_from(count).map_err(|e| {
            do_memory_core::Error::Storage(format!("Count conversion failed: {}", e))
        })? as usize;
        Ok(count)
    }
}

#[cfg(test)]
#[path = "relationships_tests.rs"]
mod relationships_tests;
