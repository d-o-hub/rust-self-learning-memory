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
