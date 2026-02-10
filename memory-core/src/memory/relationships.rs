//! Relationship API for [`SelfLearningMemory`].
//!
//! This module extends [`SelfLearningMemory`] with episode relationship management,
//! providing methods to add, remove, and query relationships between episodes.

use crate::episode::{
    Direction, EpisodeRelationship, RelationshipManager, RelationshipMetadata, RelationshipType,
};
use crate::error::Result;
use crate::security::audit::{relationship_added, relationship_removed, AuditContext};
use uuid::Uuid;

use super::relationship_query::{EpisodeWithRelationships, RelationshipFilter, RelationshipGraph};
use super::types::SelfLearningMemory;

impl SelfLearningMemory {
    /// Add a relationship between two episodes.
    ///
    /// Creates a directed relationship from one episode to another with optional
    /// metadata. The relationship is validated before being stored.
    ///
    /// # Arguments
    ///
    /// * `from_episode_id` - Source episode UUID
    /// * `to_episode_id` - Target episode UUID
    /// * `relationship_type` - Type of relationship (`DependsOn`, `ParentChild`, etc.)
    /// * `metadata` - Optional metadata (reason, priority, custom fields)
    ///
    /// # Returns
    ///
    /// The UUID of the created relationship on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Either episode does not exist
    /// - Self-relationship is attempted
    /// - Duplicate relationship exists
    /// - Cycle would be created (for acyclic relationship types)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_core::memory::SelfLearningMemory;
    /// use memory_core::episode::{RelationshipType, RelationshipMetadata};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let parent_id = Uuid::new_v4();
    /// let child_id = Uuid::new_v4();
    ///
    /// let rel_id = memory.add_episode_relationship(
    ///     parent_id,
    ///     child_id,
    ///     RelationshipType::ParentChild,
    ///     RelationshipMetadata::with_reason("Child task spawned from parent".to_string()),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_episode_relationship(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        metadata: RelationshipMetadata,
    ) -> Result<Uuid> {
        // Validate both episodes exist (get_episode returns Error::NotFound if missing)
        self.get_episode(from_episode_id).await?;
        self.get_episode(to_episode_id).await?;

        // Use RelationshipManager for validation
        let mut manager = RelationshipManager::new();

        // Load existing relationships for cycle detection
        let existing = self.get_all_relationships_internal().await?;
        manager.load_relationships(existing);

        // Add with validation (handles self-ref, duplicates, cycles)
        let relationship = manager
            .add_with_validation(from_episode_id, to_episode_id, relationship_type, metadata)
            .map_err(|e| crate::error::Error::ValidationFailed(e.to_string()))?;

        let relationship_id = relationship.id;

        // Store to durable storage if available
        if let Some(storage) = &self.turso_storage {
            storage.store_relationship(&relationship).await?;
        }

        // Store to cache if available
        if let Some(cache) = &self.cache_storage {
            let _ = cache.store_relationship(&relationship).await;
        }

        // In-memory fallback storage (when no backends configured)
        if self.turso_storage.is_none() && self.cache_storage.is_none() {
            let mut relationships = self.relationships_fallback.write().await;
            relationships.insert(relationship_id, relationship.clone());
        }

        // Audit log: relationship added
        let context = AuditContext::system();
        let audit_entry = relationship_added(
            &context,
            relationship_id,
            from_episode_id,
            to_episode_id,
            relationship_type.as_str(),
        );
        self.audit_logger.log(audit_entry);

        Ok(relationship_id)
    }

    /// Remove a relationship by its ID.
    ///
    /// # Arguments
    ///
    /// * `relationship_id` - The UUID of the relationship to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the relationship was not found.
    pub async fn remove_episode_relationship(&self, relationship_id: Uuid) -> Result<()> {
        // Remove from durable storage
        if let Some(storage) = &self.turso_storage {
            storage.remove_relationship(relationship_id).await?;
        }

        // Remove from cache
        if let Some(cache) = &self.cache_storage {
            let _ = cache.remove_relationship(relationship_id).await;
        }

        // In-memory fallback removal (when no backends configured)
        if self.turso_storage.is_none() && self.cache_storage.is_none() {
            let mut relationships = self.relationships_fallback.write().await;
            relationships.remove(&relationship_id);
        }

        // Audit log: relationship removed
        let context = AuditContext::system();
        let audit_entry = relationship_removed(&context, relationship_id);
        self.audit_logger.log(audit_entry);

        Ok(())
    }

    /// Get all relationships for an episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode to query
    /// * `direction` - Which relationships to return (Outgoing, Incoming, or Both)
    ///
    /// # Returns
    ///
    /// A vector of relationships matching the query.
    pub async fn get_episode_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        // Try cache first
        if let Some(cache) = &self.cache_storage {
            if let Ok(rels) = cache.get_relationships(episode_id, direction).await {
                if !rels.is_empty() {
                    return Ok(rels);
                }
            }
        }

        // Fall back to durable storage
        if let Some(storage) = &self.turso_storage {
            return storage.get_relationships(episode_id, direction).await;
        }

        // In-memory fallback (no storage configured)
        if self.turso_storage.is_none() && self.cache_storage.is_none() {
            let relationships = self.relationships_fallback.read().await;
            let filtered = relationships
                .values()
                .filter(|rel| match direction {
                    Direction::Outgoing => rel.from_episode_id == episode_id,
                    Direction::Incoming => rel.to_episode_id == episode_id,
                    Direction::Both => {
                        rel.from_episode_id == episode_id || rel.to_episode_id == episode_id
                    }
                })
                .cloned()
                .collect();
            return Ok(filtered);
        }

        Ok(Vec::new())
    }

    /// Find related episodes with filtering options.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode to find relationships for
    /// * `filter` - Optional filter criteria (type, direction, limit, priority)
    ///
    /// # Returns
    ///
    /// A vector of related episode IDs matching the filter.
    pub async fn find_related_episodes(
        &self,
        episode_id: Uuid,
        filter: RelationshipFilter,
    ) -> Result<Vec<Uuid>> {
        let direction = filter.direction.unwrap_or(Direction::Both);
        let relationships = self
            .get_episode_relationships(episode_id, direction)
            .await?;

        let mut related: Vec<Uuid> = relationships
            .into_iter()
            .filter(|rel| Self::matches_filter(rel, &filter))
            .map(|rel| {
                if rel.from_episode_id == episode_id {
                    rel.to_episode_id
                } else {
                    rel.from_episode_id
                }
            })
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            related.truncate(limit);
        }

        Ok(related)
    }

    /// Get an episode with all its relationships.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode to retrieve
    ///
    /// # Returns
    ///
    /// The episode along with its incoming and outgoing relationships.
    pub async fn get_episode_with_relationships(
        &self,
        episode_id: Uuid,
    ) -> Result<EpisodeWithRelationships> {
        let episode = self.get_episode(episode_id).await?;

        let outgoing = self
            .get_episode_relationships(episode_id, Direction::Outgoing)
            .await?;
        let incoming = self
            .get_episode_relationships(episode_id, Direction::Incoming)
            .await?;

        Ok(EpisodeWithRelationships {
            episode,
            outgoing,
            incoming,
        })
    }

    /// Build a relationship graph starting from an episode.
    ///
    /// Traverses relationships up to a specified depth to build a graph
    /// that can be visualized or analyzed.
    ///
    /// # Arguments
    ///
    /// * `root_episode_id` - The starting episode
    /// * `max_depth` - Maximum traversal depth (default: 3)
    ///
    /// # Returns
    ///
    /// A `RelationshipGraph` containing nodes and edges.
    pub async fn build_relationship_graph(
        &self,
        root_episode_id: Uuid,
        max_depth: usize,
    ) -> Result<RelationshipGraph> {
        let mut graph = RelationshipGraph::new(root_episode_id);
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        queue.push_back((root_episode_id, 0));

        while let Some((episode_id, depth)) = queue.pop_front() {
            if visited.contains(&episode_id) || depth > max_depth {
                continue;
            }
            visited.insert(episode_id);

            // Add episode node (ignore if not found)
            if let Ok(episode) = self.get_episode(episode_id).await {
                graph.add_node(episode);
            }

            // Get relationships and add edges
            let relationships = self
                .get_episode_relationships(episode_id, Direction::Both)
                .await?;
            for rel in relationships {
                graph.add_edge(rel.clone());

                // Queue connected episodes
                let connected = if rel.from_episode_id == episode_id {
                    rel.to_episode_id
                } else {
                    rel.from_episode_id
                };
                if !visited.contains(&connected) {
                    queue.push_back((connected, depth + 1));
                }
            }
        }

        Ok(graph)
    }

    /// Check if a relationship exists between two episodes.
    ///
    /// # Arguments
    ///
    /// * `from_episode_id` - Source episode
    /// * `to_episode_id` - Target episode
    /// * `relationship_type` - Type of relationship to check
    ///
    /// # Returns
    ///
    /// `true` if the relationship exists, `false` otherwise.
    pub async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        if let Some(storage) = &self.turso_storage {
            return storage
                .relationship_exists(from_episode_id, to_episode_id, relationship_type)
                .await;
        }
        if self.turso_storage.is_none() && self.cache_storage.is_none() {
            let relationships = self.relationships_fallback.read().await;
            let exists = relationships.values().any(|rel| {
                rel.from_episode_id == from_episode_id
                    && rel.to_episode_id == to_episode_id
                    && rel.relationship_type == relationship_type
            });
            return Ok(exists);
        }
        Ok(false)
    }

    /// Get all dependencies for an episode (episodes it depends on).
    ///
    /// Convenience method for getting `DependsOn` relationships.
    pub async fn get_episode_dependencies(&self, episode_id: Uuid) -> Result<Vec<Uuid>> {
        self.find_related_episodes(
            episode_id,
            RelationshipFilter::new()
                .with_type(RelationshipType::DependsOn)
                .with_direction(Direction::Outgoing),
        )
        .await
    }

    /// Get all dependents for an episode (episodes that depend on it).
    ///
    /// Convenience method for getting reverse `DependsOn` relationships.
    pub async fn get_episode_dependents(&self, episode_id: Uuid) -> Result<Vec<Uuid>> {
        self.find_related_episodes(
            episode_id,
            RelationshipFilter::new()
                .with_type(RelationshipType::DependsOn)
                .with_direction(Direction::Incoming),
        )
        .await
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    /// Check if a relationship matches the given filter criteria.
    fn matches_filter(rel: &EpisodeRelationship, filter: &RelationshipFilter) -> bool {
        // Filter by type
        if let Some(ref rel_type) = filter.relationship_type {
            if rel.relationship_type != *rel_type {
                return false;
            }
        }
        // Filter by priority
        if let Some(min_priority) = filter.min_priority {
            if let Some(priority) = rel.metadata.priority {
                if priority < min_priority {
                    return false;
                }
            }
        }
        true
    }

    #[allow(clippy::unused_async)]
    async fn get_all_relationships_internal(&self) -> Result<Vec<EpisodeRelationship>> {
        // For cycle detection, we would need to query all relationships
        // For now, return empty - full implementation would query storage
        if let Some(_storage) = &self.turso_storage {
            // This would need a method to get all relationships
            // For now, return empty vec
        }
        if self.turso_storage.is_none() && self.cache_storage.is_none() {
            let relationships = self.relationships_fallback.read().await;
            return Ok(relationships.values().cloned().collect());
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};

    #[tokio::test]
    async fn test_add_relationship_validates_episodes() {
        let memory = SelfLearningMemory::new();
        let fake_from = Uuid::new_v4();
        let fake_to = Uuid::new_v4();

        let result = memory
            .add_episode_relationship(
                fake_from,
                fake_to,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_relationships_empty() {
        let memory = SelfLearningMemory::new();
        let episode_id = Uuid::new_v4();

        let result = memory
            .get_episode_relationships(episode_id, Direction::Both)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_find_related_empty() {
        let memory = SelfLearningMemory::new();
        let episode_id = Uuid::new_v4();

        let result = memory
            .find_related_episodes(episode_id, RelationshipFilter::default())
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_relationship_exists_no_storage() {
        let memory = SelfLearningMemory::new();

        let result = memory
            .relationship_exists(Uuid::new_v4(), Uuid::new_v4(), RelationshipType::DependsOn)
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_get_dependencies_empty() {
        let memory = SelfLearningMemory::new();
        let episode_id = Uuid::new_v4();

        let deps = memory.get_episode_dependencies(episode_id).await.unwrap();
        assert!(deps.is_empty());
    }

    #[tokio::test]
    async fn test_get_dependents_empty() {
        let memory = SelfLearningMemory::new();
        let episode_id = Uuid::new_v4();

        let deps = memory.get_episode_dependents(episode_id).await.unwrap();
        assert!(deps.is_empty());
    }

    #[tokio::test]
    async fn test_build_relationship_graph_single_node() {
        let memory = SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let graph = memory
            .build_relationship_graph(episode_id, 3)
            .await
            .unwrap();

        assert_eq!(graph.root, episode_id);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[tokio::test]
    async fn test_get_episode_with_relationships() {
        let memory = SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let result = memory
            .get_episode_with_relationships(episode_id)
            .await
            .unwrap();

        assert_eq!(result.episode.episode_id, episode_id);
        assert!(result.outgoing.is_empty());
        assert!(result.incoming.is_empty());
        assert_eq!(result.total_relationships(), 0);
    }
}
