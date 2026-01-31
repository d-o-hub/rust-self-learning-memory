//! Relationship manager for episode relationships.
//!
//! This module provides the `RelationshipManager` struct for managing episode
//! relationships with validation, cycle detection, and graph operations.

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::graph_algorithms::{find_path_dfs, has_path_dfs, topological_sort};
use super::relationship_errors::{GraphError, RemovalError, ValidationError};
use super::{EpisodeRelationship, RelationshipMetadata, RelationshipType};

/// Manages episode relationships with validation and graph operations.
///
/// The `RelationshipManager` maintains an in-memory graph representation
/// for fast traversal and validation of episode relationships.
///
/// # Examples
///
/// ```
/// use memory_core::episode::relationship_manager::RelationshipManager;
/// use memory_core::episode::{RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// let mut manager = RelationshipManager::new();
/// let from_id = Uuid::new_v4();
/// let to_id = Uuid::new_v4();
///
/// let result = manager.add_with_validation(
///     from_id,
///     to_id,
///     RelationshipType::DependsOn,
///     RelationshipMetadata::default(),
/// );
///
/// assert!(result.is_ok());
/// assert!(manager.relationship_exists(from_id, to_id, RelationshipType::DependsOn));
/// ```
#[derive(Debug, Clone)]
pub struct RelationshipManager {
    /// In-memory graph representation for fast traversal
    adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>>,
    /// Reverse index for incoming relationships
    reverse_adjacency: HashMap<Uuid, Vec<EpisodeRelationship>>,
    /// Cache of relationship IDs for quick existence checks
    relationship_cache: HashSet<(Uuid, Uuid, RelationshipType)>,
}

impl Default for RelationshipManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipManager {
    /// Create a new relationship manager.
    ///
    /// Initializes empty data structures for storing relationships.
    #[must_use]
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            relationship_cache: HashSet::new(),
        }
    }

    /// Load relationships from storage into memory.
    ///
    /// Clears any existing state and rebuilds the graph from the provided
    /// relationships.
    ///
    /// # Arguments
    ///
    /// * `relationships` - Vector of relationships to load
    pub fn load_relationships(&mut self, relationships: Vec<EpisodeRelationship>) {
        // Clear existing state
        self.adjacency_list.clear();
        self.reverse_adjacency.clear();
        self.relationship_cache.clear();

        // Build graph
        for rel in relationships {
            self.add_to_graph(rel);
        }
    }

    /// Add a relationship with full validation.
    ///
    /// Validates the relationship according to the following rules:
    /// 1. No self-relationships (episode cannot relate to itself)
    /// 2. No duplicate relationships
    /// 3. No cycles for relationship types that require acyclicity
    /// 4. Priority must be in range 1-10 (if specified)
    ///
    /// # Arguments
    ///
    /// * `from_episode_id` - Source episode ID
    /// * `to_episode_id` - Target episode ID
    /// * `relationship_type` - Type of relationship
    /// * `metadata` - Additional relationship metadata
    ///
    /// # Returns
    ///
    /// `Ok(EpisodeRelationship)` on success, or `Err(ValidationError)` on validation failure.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if:
    /// - Self-relationship is attempted
    /// - Duplicate relationship exists
    /// - Cycle would be created (for acyclic relationship types)
    /// - Priority is outside valid range (1-10)
    pub fn add_with_validation(
        &mut self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        metadata: RelationshipMetadata,
    ) -> Result<EpisodeRelationship, ValidationError> {
        // Validation step 1: Prevent self-relationships
        if from_episode_id == to_episode_id {
            return Err(ValidationError::SelfRelationship {
                episode_id: from_episode_id,
            });
        }

        // Validation step 2: Check if already exists
        if self.relationship_exists(from_episode_id, to_episode_id, relationship_type) {
            return Err(ValidationError::DuplicateRelationship {
                from: from_episode_id,
                to: to_episode_id,
                rel_type: relationship_type,
            });
        }

        // Validation step 3: Check for cycles (if applicable)
        if relationship_type.requires_acyclic() {
            let would_create = self
                .would_create_cycle(from_episode_id, to_episode_id)
                .map_err(|_e| ValidationError::EpisodeNotFound {
                    episode_id: from_episode_id,
                })?;
            if would_create {
                let cycle = self
                    .find_cycle_path(from_episode_id, to_episode_id)
                    .map_err(|_e| ValidationError::EpisodeNotFound {
                        episode_id: from_episode_id,
                    })?;
                return Err(ValidationError::CycleDetected {
                    from: from_episode_id,
                    to: to_episode_id,
                    cycle_path: cycle,
                });
            }
        }

        // Validation step 4: Validate priority (if present)
        if let Some(priority) = metadata.priority {
            if !(1..=10).contains(&priority) {
                return Err(ValidationError::InvalidPriority {
                    priority,
                    valid_range: (1, 10),
                });
            }
        }

        // Create the relationship
        let relationship =
            EpisodeRelationship::new(from_episode_id, to_episode_id, relationship_type, metadata);

        // Update internal state
        self.add_to_graph(relationship.clone());

        Ok(relationship)
    }

    /// Remove a relationship by ID.
    ///
    /// # Arguments
    ///
    /// * `relationship_id` - The UUID of the relationship to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(RemovalError)` if relationship not found.
    pub fn remove_relationship(&mut self, relationship_id: Uuid) -> Result<(), RemovalError> {
        // Find the relationship
        let relationship =
            self.find_relationship_by_id(relationship_id)
                .ok_or(RemovalError::NotFound {
                    id: relationship_id,
                })?;

        // Remove from forward adjacency
        if let Some(rels) = self.adjacency_list.get_mut(&relationship.from_episode_id) {
            rels.retain(|r| r.id != relationship_id);
        }

        // Remove from reverse adjacency
        if let Some(rels) = self.reverse_adjacency.get_mut(&relationship.to_episode_id) {
            rels.retain(|r| r.id != relationship_id);
        }

        // Remove from cache
        self.relationship_cache.remove(&(
            relationship.from_episode_id,
            relationship.to_episode_id,
            relationship.relationship_type,
        ));

        Ok(())
    }

    /// Check if a relationship exists.
    ///
    /// # Arguments
    ///
    /// * `from_id` - Source episode ID
    /// * `to_id` - Target episode ID
    /// * `rel_type` - Relationship type
    ///
    /// # Returns
    ///
    /// `true` if the relationship exists, `false` otherwise.
    #[must_use]
    pub fn relationship_exists(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        rel_type: RelationshipType,
    ) -> bool {
        self.relationship_cache
            .contains(&(from_id, to_id, rel_type))
    }

    /// Get all outgoing relationships for an episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode ID to query
    ///
    /// # Returns
    ///
    /// Vector of outgoing relationships (empty if none exist).
    #[must_use]
    pub fn get_outgoing(&self, episode_id: Uuid) -> Vec<EpisodeRelationship> {
        self.adjacency_list
            .get(&episode_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all incoming relationships for an episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode ID to query
    ///
    /// # Returns
    ///
    /// Vector of incoming relationships (empty if none exist).
    #[must_use]
    pub fn get_incoming(&self, episode_id: Uuid) -> Vec<EpisodeRelationship> {
        self.reverse_adjacency
            .get(&episode_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get relationships by type for an episode.
    ///
    /// Returns both outgoing and incoming relationships of the specified type.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode ID to query
    /// * `rel_type` - The relationship type to filter by
    ///
    /// # Returns
    ///
    /// Vector of matching relationships.
    #[must_use]
    pub fn get_by_type(
        &self,
        episode_id: Uuid,
        rel_type: RelationshipType,
    ) -> Vec<EpisodeRelationship> {
        let mut results = Vec::new();

        // Get outgoing
        if let Some(rels) = self.adjacency_list.get(&episode_id) {
            results.extend(
                rels.iter()
                    .filter(|r| r.relationship_type == rel_type)
                    .cloned(),
            );
        }

        // Get incoming
        if let Some(rels) = self.reverse_adjacency.get(&episode_id) {
            results.extend(
                rels.iter()
                    .filter(|r| r.relationship_type == rel_type)
                    .cloned(),
            );
        }

        results
    }

    /// Check if adding a relationship would create a cycle.
    ///
    /// Performs a pre-flight check to determine if adding a relationship
    /// from `from_id` to `to_id` would create a cycle in the graph.
    ///
    /// # Arguments
    ///
    /// * `from_id` - Source episode ID
    /// * `to_id` - Target episode ID
    ///
    /// # Returns
    ///
    /// `Ok(true)` if a cycle would be created, `Ok(false)` otherwise,
    /// or `Err(GraphError)` on error.
    pub fn would_create_cycle(&self, from_id: Uuid, to_id: Uuid) -> Result<bool, GraphError> {
        // Use DFS to check if path exists from to_id to from_id
        // If such a path exists, adding from_id -> to_id would create a cycle
        has_path_dfs(&self.adjacency_list, to_id, from_id)
    }

    /// Find the cycle path if adding a relationship would create one.
    ///
    /// # Arguments
    ///
    /// * `from_id` - Source episode ID
    /// * `to_id` - Target episode ID
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Uuid>)` containing the cycle path (from `to_id` back to `from_id`),
    /// or `Err(GraphError)` if no cycle would be created.
    pub fn find_cycle_path(&self, from_id: Uuid, to_id: Uuid) -> Result<Vec<Uuid>, GraphError> {
        // Find path from to_id to from_id (which would complete the cycle)
        find_path_dfs(&self.adjacency_list, to_id, from_id)
    }

    /// Get all relationships in the manager.
    ///
    /// # Returns
    ///
    /// Vector of all stored relationships.
    #[must_use]
    pub fn get_all_relationships(&self) -> Vec<EpisodeRelationship> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for rels in self.adjacency_list.values() {
            for rel in rels {
                if seen.insert(rel.id) {
                    result.push(rel.clone());
                }
            }
        }

        result
    }

    /// Get the count of relationships.
    ///
    /// # Returns
    ///
    /// Number of relationships stored.
    #[must_use]
    pub fn relationship_count(&self) -> usize {
        self.relationship_cache.len()
    }

    /// Get the count of episodes that have relationships.
    ///
    /// # Returns
    ///
    /// Number of unique episode IDs in the graph.
    #[must_use]
    pub fn episode_count(&self) -> usize {
        let mut episodes = HashSet::new();

        for (from_id, rels) in &self.adjacency_list {
            episodes.insert(*from_id);
            for rel in rels {
                episodes.insert(rel.to_episode_id);
            }
        }

        episodes.len()
    }

    /// Perform topological sort on the relationship graph.
    ///
    /// Returns episodes in dependency order (dependencies before dependents).
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Uuid>)` containing sorted episode IDs, or `Err(GraphError)`
    /// if the graph contains cycles.
    pub fn topological_order(&self) -> Result<Vec<Uuid>, GraphError> {
        topological_sort(&self.adjacency_list)
    }

    /// Clear all relationships.
    pub fn clear(&mut self) {
        self.adjacency_list.clear();
        self.reverse_adjacency.clear();
        self.relationship_cache.clear();
    }

    /// Internal method to add relationship to graph structures.
    fn add_to_graph(&mut self, relationship: EpisodeRelationship) {
        // Add to forward adjacency
        self.adjacency_list
            .entry(relationship.from_episode_id)
            .or_default()
            .push(relationship.clone());

        // Add to reverse adjacency
        self.reverse_adjacency
            .entry(relationship.to_episode_id)
            .or_default()
            .push(relationship.clone());

        // Add to cache
        self.relationship_cache.insert((
            relationship.from_episode_id,
            relationship.to_episode_id,
            relationship.relationship_type,
        ));
    }

    /// Find relationship by ID (internal helper).
    fn find_relationship_by_id(&self, id: Uuid) -> Option<EpisodeRelationship> {
        for rels in self.adjacency_list.values() {
            if let Some(rel) = rels.iter().find(|r| r.id == id) {
                return Some(rel.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let manager = RelationshipManager::new();
        assert_eq!(manager.relationship_count(), 0);
        assert_eq!(manager.episode_count(), 0);
    }

    #[test]
    fn test_default_manager() {
        let manager = RelationshipManager::default();
        assert_eq!(manager.relationship_count(), 0);
    }

    #[test]
    fn test_add_valid_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
        assert!(manager.relationship_exists(from_id, to_id, RelationshipType::DependsOn));
        assert_eq!(manager.relationship_count(), 1);
    }

    #[test]
    fn test_prevent_self_relationship() {
        let mut manager = RelationshipManager::new();
        let id = Uuid::new_v4();

        let result = manager.add_with_validation(
            id,
            id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(matches!(
            result,
            Err(ValidationError::SelfRelationship { .. })
        ));
        assert_eq!(manager.relationship_count(), 0);
    }

    #[test]
    fn test_prevent_duplicate_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Add first time - should succeed
        manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Add second time - should fail
        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(matches!(
            result,
            Err(ValidationError::DuplicateRelationship { .. })
        ));
        assert_eq!(manager.relationship_count(), 1);
    }

    #[test]
    fn test_allow_different_types_same_pair() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Add DependsOn relationship
        manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Add RelatedTo relationship (same pair, different type)
        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 2);
    }

    #[test]
    fn test_detect_cycle() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // Create chain: A -> B -> C
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Try to add C -> A (would create cycle)
        let result = manager.add_with_validation(
            c,
            a,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(matches!(result, Err(ValidationError::CycleDetected { .. })));
    }

    #[test]
    fn test_allow_non_cyclic_relationships() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // Create: A -> B, C -> B (no cycle)
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        let result = manager.add_with_validation(
            c,
            b,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 2);
    }

    #[test]
    fn test_no_cycle_check_for_non_acyclic_types() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // Create chain with RelatedTo (doesn't require acyclic)
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Adding C -> A should succeed (RelatedTo doesn't require acyclic)
        let result = manager.add_with_validation(
            c,
            a,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_priority_validation() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Valid priority (1)
        let mut metadata = RelationshipMetadata::default();
        metadata.priority = Some(1);
        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            metadata.clone(),
        );
        assert!(result.is_ok());

        // Valid priority (10)
        let from_id2 = Uuid::new_v4();
        let to_id2 = Uuid::new_v4();
        metadata.priority = Some(10);
        let result = manager.add_with_validation(
            from_id2,
            to_id2,
            RelationshipType::DependsOn,
            metadata.clone(),
        );
        assert!(result.is_ok());

        // Invalid priority (0)
        let from_id3 = Uuid::new_v4();
        let to_id3 = Uuid::new_v4();
        metadata.priority = Some(0);
        let result = manager.add_with_validation(
            from_id3,
            to_id3,
            RelationshipType::DependsOn,
            metadata.clone(),
        );
        assert!(matches!(
            result,
            Err(ValidationError::InvalidPriority { .. })
        ));

        // Invalid priority (11)
        let from_id4 = Uuid::new_v4();
        let to_id4 = Uuid::new_v4();
        metadata.priority = Some(11);
        let result =
            manager.add_with_validation(from_id4, to_id4, RelationshipType::DependsOn, metadata);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidPriority { .. })
        ));
    }

    #[test]
    fn test_remove_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.relationship_count(), 1);

        // Remove the relationship
        let result = manager.remove_relationship(rel.id);
        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 0);
        assert!(!manager.relationship_exists(from_id, to_id, RelationshipType::DependsOn));
    }

    #[test]
    fn test_remove_nonexistent_relationship() {
        let mut manager = RelationshipManager::new();
        let fake_id = Uuid::new_v4();

        let result = manager.remove_relationship(fake_id);
        assert!(matches!(result, Err(RemovalError::NotFound { .. })));
    }

    #[test]
    fn test_get_outgoing() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                a,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let outgoing = manager.get_outgoing(a);
        assert_eq!(outgoing.len(), 2);
    }

    #[test]
    fn test_get_incoming() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                c,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let incoming = manager.get_incoming(b);
        assert_eq!(incoming.len(), 2);
    }

    #[test]
    fn test_get_by_type() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                a,
                c,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let depends_on = manager.get_by_type(a, RelationshipType::DependsOn);
        assert_eq!(depends_on.len(), 1);

        let related_to = manager.get_by_type(a, RelationshipType::RelatedTo);
        assert_eq!(related_to.len(), 1);
    }

    #[test]
    fn test_would_create_cycle() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // C -> A would create cycle
        assert!(manager.would_create_cycle(c, a).unwrap());

        // A -> C would not create cycle (already exists indirectly)
        // Actually, this would be a duplicate, not a cycle check
        // Let's check a new node
        let d = Uuid::new_v4();
        assert!(!manager.would_create_cycle(c, d).unwrap());
    }

    #[test]
    fn test_find_cycle_path() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Path from A to C would be: A -> B -> C
        // This is the path that would complete the cycle if we added C -> A
        let path = manager.find_cycle_path(c, a).unwrap();
        assert_eq!(path, vec![a, b, c]);
    }

    #[test]
    fn test_load_relationships() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        let rel1 = EpisodeRelationship::new(
            a,
            b,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        let rel2 = EpisodeRelationship::new(
            b,
            c,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        manager.load_relationships(vec![rel1.clone(), rel2.clone()]);

        assert_eq!(manager.relationship_count(), 2);
        assert!(manager.relationship_exists(a, b, RelationshipType::DependsOn));
        assert!(manager.relationship_exists(b, c, RelationshipType::DependsOn));
    }

    #[test]
    fn test_load_relationships_clears_existing() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // Add initial relationship
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        assert_eq!(manager.relationship_count(), 1);

        // Load new relationships (should clear old ones)
        let rel = EpisodeRelationship::new(
            c,
            d,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        manager.load_relationships(vec![rel]);

        assert_eq!(manager.relationship_count(), 1);
        assert!(!manager.relationship_exists(a, b, RelationshipType::DependsOn));
        assert!(manager.relationship_exists(c, d, RelationshipType::DependsOn));
    }

    #[test]
    fn test_get_all_relationships() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let all = manager.get_all_relationships();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_episode_count() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B, B -> C (3 unique episodes)
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.episode_count(), 3);
    }

    #[test]
    fn test_topological_order() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let order = manager.topological_order().unwrap();
        let a_pos = order.iter().position(|&x| x == a).unwrap();
        let b_pos = order.iter().position(|&x| x == b).unwrap();
        let c_pos = order.iter().position(|&x| x == c).unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_topological_order_fails_with_cycle() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        // A -> B -> A (cycle)
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                a,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Note: topological_order uses the full graph which includes the cycle
        // But since RelatedTo doesn't require acyclic, the cycle exists
        let result = manager.topological_order();
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        assert_eq!(manager.relationship_count(), 1);

        manager.clear();
        assert_eq!(manager.relationship_count(), 0);
        assert_eq!(manager.episode_count(), 0);
    }

    #[test]
    fn test_complex_graph_operations() {
        let mut manager = RelationshipManager::new();

        // Create a complex DAG:
        // A -> B, A -> C
        // B -> D
        // C -> D
        // D -> E
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();
        let e = Uuid::new_v4();

        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                a,
                c,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                b,
                d,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                c,
                d,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();
        manager
            .add_with_validation(
                d,
                e,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.relationship_count(), 5);
        assert_eq!(manager.episode_count(), 5);

        // Check outgoing from A
        let outgoing_a = manager.get_outgoing(a);
        assert_eq!(outgoing_a.len(), 2);

        // Check incoming to D
        let incoming_d = manager.get_incoming(d);
        assert_eq!(incoming_d.len(), 2);

        // Verify topological order
        let order = manager.topological_order().unwrap();
        let a_pos = order.iter().position(|&x| x == a).unwrap();
        let e_pos = order.iter().position(|&x| x == e).unwrap();
        assert!(a_pos < e_pos);
    }

    #[test]
    fn test_all_relationship_types() {
        let mut manager = RelationshipManager::new();
        let _a = Uuid::new_v4();
        let _b = Uuid::new_v4();

        // Test all relationship types can be added
        for rel_type in RelationshipType::all() {
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();

            let result = manager.add_with_validation(
                from_id,
                to_id,
                rel_type,
                RelationshipMetadata::default(),
            );

            // Acyclic types should work on fresh nodes
            assert!(result.is_ok(), "Failed to add {:?} relationship", rel_type);
        }
    }

    #[test]
    fn test_cycle_detection_with_different_types() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B (DependsOn, requires acyclic)
        manager
            .add_with_validation(
                a,
                b,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // B -> C (ParentChild, requires acyclic)
        manager
            .add_with_validation(
                b,
                c,
                RelationshipType::ParentChild,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // C -> A (DependsOn, would create cycle)
        let result = manager.add_with_validation(
            c,
            a,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        assert!(matches!(result, Err(ValidationError::CycleDetected { .. })));

        // C -> A (RelatedTo, does NOT require acyclic, should succeed)
        let result = manager.add_with_validation(
            c,
            a,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_queries() {
        let manager = RelationshipManager::new();
        let id = Uuid::new_v4();

        assert!(manager.get_outgoing(id).is_empty());
        assert!(manager.get_incoming(id).is_empty());
        assert!(manager
            .get_by_type(id, RelationshipType::DependsOn)
            .is_empty());
        assert!(!manager.relationship_exists(id, Uuid::new_v4(), RelationshipType::DependsOn));
    }

    #[test]
    fn test_relationship_metadata_preservation() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let mut metadata = RelationshipMetadata::new();
        metadata.reason = Some("Test reason".to_string());
        metadata.created_by = Some("test_user".to_string());
        metadata.priority = Some(5);
        metadata.set_field("key1".to_string(), "value1".to_string());

        let rel = manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                metadata.clone(),
            )
            .unwrap();

        assert_eq!(rel.metadata.reason, metadata.reason);
        assert_eq!(rel.metadata.created_by, metadata.created_by);
        assert_eq!(rel.metadata.priority, metadata.priority);
        assert_eq!(rel.metadata.get_field("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_large_graph_performance() {
        let mut manager = RelationshipManager::new();
        let nodes: Vec<Uuid> = (0..1000).map(|_| Uuid::new_v4()).collect();

        // Create a chain: 0 -> 1 -> 2 -> ... -> 999
        let start = std::time::Instant::now();
        for i in 0..nodes.len() - 1 {
            manager
                .add_with_validation(
                    nodes[i],
                    nodes[i + 1],
                    RelationshipType::DependsOn,
                    RelationshipMetadata::default(),
                )
                .unwrap();
        }
        let duration = start.elapsed();

        assert_eq!(manager.relationship_count(), 999);
        assert!(duration.as_millis() < 1000); // Should complete in under 1 second

        // Test cycle detection performance
        let start = std::time::Instant::now();
        let would_cycle = manager.would_create_cycle(nodes[999], nodes[0]).unwrap();
        let duration = start.elapsed();

        assert!(would_cycle);
        assert!(duration.as_millis() < 100); // Should be fast
    }
}
