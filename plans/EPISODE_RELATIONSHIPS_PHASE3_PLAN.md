# Episode Relationships Phase 3: Memory Layer Integration

**Phase**: 3 of 6  
**Status**: ⏳ Not Started  
**Estimated Effort**: 2 days  
**Estimated LOC**: ~400  
**Estimated Tests**: 15+  
**Dependencies**: Phase 1 (Complete ✅), Phase 2 (Required ⏳)

---

## Overview

Phase 3 integrates the relationship functionality into the memory layer, providing high-level APIs that combine storage operations with business logic validation. This phase bridges the gap between low-level storage (Phase 1) and user-facing interfaces (Phases 4-5).

**Key Goals**:
- Extend memory management API with relationship operations
- Implement cache-aware relationship queries
- Add relationship-based episode filtering
- Ensure transaction consistency across layers

---

## Architecture

### Integration Points

```
┌──────────────────────────────────────────────────────┐
│              Memory Layer (Phase 3)                   │
│  ┌────────────────────────────────────────────────┐ │
│  │     MemoryManager Extensions                   │ │
│  │  - add_episode_relationship()                  │ │
│  │  - remove_episode_relationship()               │ │
│  │  - get_episode_relationships()                 │ │
│  │  - find_related_episodes()                     │ │
│  └─────────────┬──────────────────────────────────┘ │
│                │                                      │
└────────────────┼──────────────────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼────────┐ ┌▼─────────────▼───┐ ┌──────────────────┐
│ Phase 2    │ │ Phase 1          │ │ Cache Layer      │
│ Validation │ │ Storage (Turso)  │ │ (Redb)           │
│ & Graph    │ │ - CRUD ops       │ │ - Write-through  │
└────────────┘ └──────────────────┘ └──────────────────┘
```

---

## Component 1: MemoryManager Extensions

**File**: `memory-core/src/memory/mod.rs` (extend existing)  
**Estimated LOC**: ~250  
**Tests**: 10+

### Current MemoryManager Structure

First, let me check the existing MemoryManager to understand the pattern:

```rust
// Expected existing structure (to be verified):
pub struct MemoryManager {
    storage: Arc<dyn Storage>,
    cache: Option<Arc<dyn Cache>>,
    relationship_manager: Option<RelationshipManager>, // NEW
}
```

### New Methods to Add

#### 1. Add Relationship (with validation)

```rust
impl MemoryManager {
    /// Add a relationship between two episodes with full validation
    /// 
    /// This method:
    /// 1. Validates both episodes exist
    /// 2. Checks for cycles (if applicable)
    /// 3. Prevents duplicates
    /// 4. Stores in database
    /// 5. Updates cache
    /// 6. Updates in-memory graph
    ///
    /// # Arguments
    /// * `from_episode_id` - Source episode UUID
    /// * `to_episode_id` - Target episode UUID
    /// * `relationship_type` - Type of relationship
    /// * `metadata` - Optional metadata (reason, priority, etc.)
    ///
    /// # Returns
    /// * `Ok(Uuid)` - The relationship ID
    /// * `Err(Error)` - Validation or storage error
    ///
    /// # Example
    /// ```rust
    /// let rel_id = memory_manager.add_episode_relationship(
    ///     episode_a_id,
    ///     episode_b_id,
    ///     RelationshipType::DependsOn,
    ///     RelationshipMetadata {
    ///         reason: Some("Prerequisite".to_string()),
    ///         priority: Some(8),
    ///         ..Default::default()
    ///     }
    /// ).await?;
    /// ```
    pub async fn add_episode_relationship(
        &mut self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        metadata: RelationshipMetadata,
    ) -> Result<Uuid> {
        // Step 1: Validate both episodes exist
        let from_episode = self.get_episode(from_episode_id).await?
            .ok_or_else(|| Error::NotFound(format!("Episode {} not found", from_episode_id)))?;
        
        let to_episode = self.get_episode(to_episode_id).await?
            .ok_or_else(|| Error::NotFound(format!("Episode {} not found", to_episode_id)))?;
        
        // Step 2: Get or initialize relationship manager
        if self.relationship_manager.is_none() {
            self.load_relationship_manager().await?;
        }
        
        let manager = self.relationship_manager.as_mut()
            .ok_or_else(|| Error::Internal("Relationship manager not initialized".to_string()))?;
        
        // Step 3: Validate with business logic (includes cycle detection)
        let relationship = manager.add_with_validation(
            from_episode_id,
            to_episode_id,
            relationship_type,
            metadata.clone(),
        ).map_err(|e| Error::Validation(format!("Validation failed: {}", e)))?;
        
        // Step 4: Store in database
        let relationship_id = self.storage
            .add_relationship(from_episode_id, to_episode_id, relationship_type, metadata)
            .await?;
        
        // Step 5: Update cache (if enabled)
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.cache_relationship(&relationship).await {
                tracing::warn!("Failed to cache relationship: {}", e);
                // Non-fatal - continue
            }
        }
        
        tracing::info!(
            "Added relationship {} from {} to {} (type: {:?})",
            relationship_id, from_episode_id, to_episode_id, relationship_type
        );
        
        Ok(relationship_id)
    }
    
    /// Internal: Load relationship manager from storage
    async fn load_relationship_manager(&mut self) -> Result<()> {
        // Load all relationships from storage
        let relationships = self.storage.get_all_relationships().await?;
        
        let mut manager = RelationshipManager::new();
        manager.load_relationships(relationships);
        
        self.relationship_manager = Some(manager);
        Ok(())
    }
}
```

#### 2. Remove Relationship

```rust
impl MemoryManager {
    /// Remove a relationship by ID
    ///
    /// This method:
    /// 1. Removes from in-memory graph
    /// 2. Removes from database
    /// 3. Invalidates cache
    ///
    /// # Arguments
    /// * `relationship_id` - UUID of relationship to remove
    ///
    /// # Returns
    /// * `Ok(())` - Successfully removed
    /// * `Err(Error)` - Relationship not found or removal failed
    pub async fn remove_episode_relationship(
        &mut self,
        relationship_id: Uuid,
    ) -> Result<()> {
        // Step 1: Remove from in-memory manager
        if let Some(manager) = &mut self.relationship_manager {
            manager.remove_relationship(relationship_id)
                .map_err(|e| Error::NotFound(format!("Relationship not found: {}", e)))?;
        }
        
        // Step 2: Remove from storage
        self.storage.remove_relationship(relationship_id).await?;
        
        // Step 3: Remove from cache
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.remove_cached_relationship(relationship_id).await {
                tracing::warn!("Failed to remove from cache: {}", e);
                // Non-fatal
            }
        }
        
        tracing::info!("Removed relationship {}", relationship_id);
        Ok(())
    }
}
```

#### 3. Get Relationships (with caching)

```rust
impl MemoryManager {
    /// Get relationships for an episode
    ///
    /// This method checks cache first, then falls back to storage.
    ///
    /// # Arguments
    /// * `episode_id` - Episode UUID
    /// * `direction` - Outgoing, Incoming, or Both
    ///
    /// # Returns
    /// * `Ok(Vec<EpisodeRelationship>)` - List of relationships
    pub async fn get_episode_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        // Try cache first
        if let Some(cache) = &self.cache {
            match cache.get_cached_relationships(episode_id, direction).await {
                Ok(relationships) if !relationships.is_empty() => {
                    tracing::debug!("Cache hit for relationships of {}", episode_id);
                    return Ok(relationships);
                }
                _ => {
                    tracing::debug!("Cache miss for relationships of {}", episode_id);
                }
            }
        }
        
        // Cache miss - query storage
        let relationships = self.storage
            .get_relationships(episode_id, direction)
            .await?;
        
        // Warm cache
        if let Some(cache) = &self.cache {
            for rel in &relationships {
                if let Err(e) = cache.cache_relationship(rel).await {
                    tracing::warn!("Failed to warm cache: {}", e);
                }
            }
        }
        
        Ok(relationships)
    }
}
```

#### 4. Find Related Episodes (with filtering)

```rust
impl MemoryManager {
    /// Find episodes related to a given episode
    ///
    /// This method supports:
    /// - Type filtering (specific relationship types)
    /// - Transitive relationships (multi-hop)
    /// - Result limiting
    ///
    /// # Arguments
    /// * `episode_id` - Starting episode UUID
    /// * `filter` - Optional relationship filter
    ///
    /// # Returns
    /// * `Ok(Vec<Episode>)` - Related episodes (full objects)
    pub async fn find_related_episodes(
        &self,
        episode_id: Uuid,
        filter: RelationshipFilter,
    ) -> Result<Vec<Episode>> {
        // Get relationships based on filter
        let relationships = match filter.relationship_type {
            Some(rel_type) => {
                self.storage.get_relationships_by_type(
                    episode_id,
                    rel_type,
                    filter.direction.unwrap_or(Direction::Both),
                ).await?
            }
            None => {
                self.get_episode_relationships(
                    episode_id,
                    filter.direction.unwrap_or(Direction::Both),
                ).await?
            }
        };
        
        // Extract episode IDs
        let mut episode_ids: Vec<Uuid> = relationships.iter()
            .flat_map(|rel| {
                vec![rel.from_episode_id, rel.to_episode_id]
            })
            .filter(|&id| id != episode_id) // Exclude self
            .collect();
        
        // Remove duplicates
        episode_ids.sort();
        episode_ids.dedup();
        
        // Apply limit
        if let Some(limit) = filter.limit {
            episode_ids.truncate(limit);
        }
        
        // Fetch full episode objects
        let mut episodes = Vec::new();
        for id in episode_ids {
            if let Some(episode) = self.get_episode(id).await? {
                episodes.push(episode);
            }
        }
        
        Ok(episodes)
    }
}
```

#### 5. Get Relationship Graph

```rust
impl MemoryManager {
    /// Export the relationship graph for an episode and its neighbors
    ///
    /// Returns a graph structure suitable for visualization or analysis.
    ///
    /// # Arguments
    /// * `episode_id` - Root episode UUID
    /// * `depth` - Maximum depth to traverse (1 = direct relationships only)
    ///
    /// # Returns
    /// * `Ok(RelationshipGraph)` - Graph structure with nodes and edges
    pub async fn get_relationship_graph(
        &self,
        episode_id: Uuid,
        depth: usize,
    ) -> Result<RelationshipGraph> {
        let mut graph = RelationshipGraph::new(episode_id);
        let mut visited = std::collections::HashSet::new();
        let mut current_level = vec![episode_id];
        
        for level in 0..depth {
            let mut next_level = Vec::new();
            
            for &current_id in &current_level {
                if visited.contains(&current_id) {
                    continue;
                }
                visited.insert(current_id);
                
                // Get relationships for this episode
                let relationships = self.get_episode_relationships(
                    current_id,
                    Direction::Both,
                ).await?;
                
                for rel in relationships {
                    graph.add_edge(rel.clone());
                    
                    // Add to next level for traversal
                    if !visited.contains(&rel.to_episode_id) {
                        next_level.push(rel.to_episode_id);
                    }
                    if !visited.contains(&rel.from_episode_id) {
                        next_level.push(rel.from_episode_id);
                    }
                }
            }
            
            current_level = next_level;
        }
        
        // Fetch episode details for all nodes
        for &id in &visited {
            if let Some(episode) = self.get_episode(id).await? {
                graph.add_node(episode);
            }
        }
        
        Ok(graph)
    }
}
```

### Supporting Types

```rust
/// Filter options for finding related episodes
#[derive(Debug, Clone, Default)]
pub struct RelationshipFilter {
    /// Filter by relationship type (None = all types)
    pub relationship_type: Option<RelationshipType>,
    
    /// Filter by direction (None = Both)
    pub direction: Option<Direction>,
    
    /// Maximum number of results (None = unlimited)
    pub limit: Option<usize>,
    
    /// Minimum priority (if relationships have priority)
    pub min_priority: Option<u8>,
}

/// Graph structure for visualization and analysis
#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    /// Root episode ID
    pub root: Uuid,
    
    /// All episodes in the graph
    pub nodes: HashMap<Uuid, Episode>,
    
    /// All relationships (edges)
    pub edges: Vec<EpisodeRelationship>,
}

impl RelationshipGraph {
    pub fn new(root: Uuid) -> Self {
        Self {
            root,
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
    
    pub fn add_node(&mut self, episode: Episode) {
        self.nodes.insert(episode.episode_id, episode);
    }
    
    pub fn add_edge(&mut self, relationship: EpisodeRelationship) {
        self.edges.push(relationship);
    }
    
    /// Export to DOT format for visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph G {\n");
        
        // Add nodes
        for (id, episode) in &self.nodes {
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\"];\n",
                id,
                episode.task_description.replace('"', "\\\"")
            ));
        }
        
        // Add edges
        for edge in &self.edges {
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{:?}\"];\n",
                edge.from_episode_id,
                edge.to_episode_id,
                edge.relationship_type
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
}
```

---

## Component 2: Enhanced Query Capabilities

**File**: `memory-core/src/memory/query.rs` (new or extend existing)  
**Estimated LOC**: ~150  
**Tests**: 5+

### Relationship-Aware Episode Queries

```rust
/// Extended episode filter with relationship support
#[derive(Debug, Clone, Default)]
pub struct EpisodeFilterExtended {
    /// Existing filters (tags, date range, etc.)
    pub base_filter: EpisodeFilter,
    
    /// Filter by relationship existence
    pub has_relationship: Option<RelationshipFilterSpec>,
    
    /// Filter by related episode IDs
    pub related_to: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct RelationshipFilterSpec {
    pub relationship_type: RelationshipType,
    pub direction: Direction,
}

impl MemoryManager {
    /// Query episodes with relationship-aware filtering
    pub async fn query_episodes_with_relationships(
        &self,
        filter: EpisodeFilterExtended,
    ) -> Result<Vec<Episode>> {
        // Step 1: Apply base filters
        let mut episodes = self.query_episodes(filter.base_filter).await?;
        
        // Step 2: Apply relationship filters
        if let Some(rel_filter) = filter.has_relationship {
            episodes = self.filter_by_relationship_existence(episodes, rel_filter).await?;
        }
        
        if let Some(related_ids) = filter.related_to {
            episodes = self.filter_by_related_episodes(episodes, related_ids).await?;
        }
        
        Ok(episodes)
    }
    
    /// Filter episodes that have specific relationships
    async fn filter_by_relationship_existence(
        &self,
        episodes: Vec<Episode>,
        rel_filter: RelationshipFilterSpec,
    ) -> Result<Vec<Episode>> {
        let mut filtered = Vec::new();
        
        for episode in episodes {
            let relationships = self.storage.get_relationships_by_type(
                episode.episode_id,
                rel_filter.relationship_type,
                rel_filter.direction,
            ).await?;
            
            if !relationships.is_empty() {
                filtered.push(episode);
            }
        }
        
        Ok(filtered)
    }
    
    /// Filter episodes related to specific episodes
    async fn filter_by_related_episodes(
        &self,
        episodes: Vec<Episode>,
        related_ids: Vec<Uuid>,
    ) -> Result<Vec<Episode>> {
        let mut filtered = Vec::new();
        
        for episode in episodes {
            let relationships = self.get_episode_relationships(
                episode.episode_id,
                Direction::Both,
            ).await?;
            
            let is_related = relationships.iter().any(|rel| {
                related_ids.contains(&rel.from_episode_id) ||
                related_ids.contains(&rel.to_episode_id)
            });
            
            if is_related {
                filtered.push(episode);
            }
        }
        
        Ok(filtered)
    }
}
```

---

## Component 3: Cache Strategy

### Write-Through Caching

```rust
impl MemoryManager {
    /// Cache warming strategy for relationships
    async fn warm_relationship_cache(&self, episode_id: Uuid) -> Result<()> {
        if let Some(cache) = &self.cache {
            // Get all relationships for this episode
            let relationships = self.storage.get_relationships(
                episode_id,
                Direction::Both,
            ).await?;
            
            // Cache each relationship
            for rel in relationships {
                if let Err(e) = cache.cache_relationship(&rel).await {
                    tracing::warn!("Cache warming failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Invalidate cache for an episode's relationships
    async fn invalidate_relationship_cache(&self, episode_id: Uuid) -> Result<()> {
        if let Some(cache) = &self.cache {
            // Get cached relationships
            if let Ok(relationships) = cache.get_cached_relationships(episode_id, Direction::Both).await {
                // Remove each from cache
                for rel in relationships {
                    if let Err(e) = cache.remove_cached_relationship(rel.id).await {
                        tracing::warn!("Cache invalidation failed: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

---

## Testing Strategy

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_relationship_end_to_end() {
        let mut manager = create_test_memory_manager().await;
        
        // Create two episodes
        let ep1 = manager.create_episode(/* ... */).await.unwrap();
        let ep2 = manager.create_episode(/* ... */).await.unwrap();
        
        // Add relationship
        let rel_id = manager.add_episode_relationship(
            ep1.episode_id,
            ep2.episode_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        ).await.unwrap();
        
        // Verify it exists
        let relationships = manager.get_episode_relationships(
            ep1.episode_id,
            Direction::Outgoing,
        ).await.unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].id, rel_id);
    }
    
    #[tokio::test]
    async fn test_cycle_detection_integration() {
        let mut manager = create_test_memory_manager().await;
        
        let ep1 = manager.create_episode(/* ... */).await.unwrap();
        let ep2 = manager.create_episode(/* ... */).await.unwrap();
        let ep3 = manager.create_episode(/* ... */).await.unwrap();
        
        // Create chain: ep1 -> ep2 -> ep3
        manager.add_episode_relationship(ep1.episode_id, ep2.episode_id, RelationshipType::DependsOn, RelationshipMetadata::default()).await.unwrap();
        manager.add_episode_relationship(ep2.episode_id, ep3.episode_id, RelationshipType::DependsOn, RelationshipMetadata::default()).await.unwrap();
        
        // Try to create cycle: ep3 -> ep1
        let result = manager.add_episode_relationship(
            ep3.episode_id,
            ep1.episode_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }
    
    #[tokio::test]
    async fn test_cache_consistency() {
        let mut manager = create_test_memory_manager().await;
        
        let ep1 = manager.create_episode(/* ... */).await.unwrap();
        let ep2 = manager.create_episode(/* ... */).await.unwrap();
        
        // Add relationship
        let rel_id = manager.add_episode_relationship(
            ep1.episode_id,
            ep2.episode_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        ).await.unwrap();
        
        // Query (should hit cache)
        let relationships = manager.get_episode_relationships(
            ep1.episode_id,
            Direction::Outgoing,
        ).await.unwrap();
        assert_eq!(relationships.len(), 1);
        
        // Remove relationship
        manager.remove_episode_relationship(rel_id).await.unwrap();
        
        // Query again (should reflect removal)
        let relationships = manager.get_episode_relationships(
            ep1.episode_id,
            Direction::Outgoing,
        ).await.unwrap();
        assert_eq!(relationships.len(), 0);
    }
    
    #[tokio::test]
    async fn test_find_related_episodes() {
        let mut manager = create_test_memory_manager().await;
        
        let ep1 = manager.create_episode(/* ... */).await.unwrap();
        let ep2 = manager.create_episode(/* ... */).await.unwrap();
        let ep3 = manager.create_episode(/* ... */).await.unwrap();
        
        // Create relationships
        manager.add_episode_relationship(ep1.episode_id, ep2.episode_id, RelationshipType::DependsOn, RelationshipMetadata::default()).await.unwrap();
        manager.add_episode_relationship(ep1.episode_id, ep3.episode_id, RelationshipType::RelatedTo, RelationshipMetadata::default()).await.unwrap();
        
        // Find all related
        let related = manager.find_related_episodes(
            ep1.episode_id,
            RelationshipFilter::default(),
        ).await.unwrap();
        
        assert_eq!(related.len(), 2);
        
        // Find only DependsOn
        let related = manager.find_related_episodes(
            ep1.episode_id,
            RelationshipFilter {
                relationship_type: Some(RelationshipType::DependsOn),
                ..Default::default()
            },
        ).await.unwrap();
        
        assert_eq!(related.len(), 1);
    }
}
```

---

## Implementation Checklist

### MemoryManager Extensions
- [ ] Add `relationship_manager` field to MemoryManager
- [ ] Implement `add_episode_relationship()`
- [ ] Implement `remove_episode_relationship()`
- [ ] Implement `get_episode_relationships()`
- [ ] Implement `find_related_episodes()`
- [ ] Implement `get_relationship_graph()`
- [ ] Implement `load_relationship_manager()`

### Enhanced Queries
- [ ] Create `RelationshipFilter` struct
- [ ] Create `RelationshipGraph` struct
- [ ] Implement `query_episodes_with_relationships()`
- [ ] Implement relationship filtering logic
- [ ] Implement graph export (DOT format)

### Cache Integration
- [ ] Implement cache warming on queries
- [ ] Implement cache invalidation on changes
- [ ] Add cache hit/miss metrics
- [ ] Test cache consistency

### Testing
- [ ] Write end-to-end integration tests
- [ ] Test cycle detection integration
- [ ] Test cache consistency
- [ ] Test find_related_episodes with filters
- [ ] Test relationship graph generation

### Documentation
- [ ] Add rustdoc to all public methods
- [ ] Add usage examples
- [ ] Document caching strategy
- [ ] Document error conditions

---

## Dependencies

### Phase 1 (Complete ✅)
- Storage operations available
- Cache operations available
- Data structures defined

### Phase 2 (Required ⏳)
- `RelationshipManager` with validation
- Graph algorithms for cycle detection
- Error types defined

---

## Performance Targets

| Operation | Target (P95) | Notes |
|-----------|-------------|-------|
| add_episode_relationship() | <50ms | Includes validation + storage |
| get_episode_relationships() | <10ms | Cache hit |
| get_episode_relationships() | <50ms | Cache miss |
| find_related_episodes() | <100ms | Up to 100 related episodes |
| get_relationship_graph() | <200ms | Depth=2, up to 50 episodes |

---

## Next Steps

After Phase 3 completion:
1. Expose via MCP tools (Phase 4)
2. Add CLI commands (Phase 5)
3. Performance benchmarking (Phase 6)

---

**Status**: Ready for implementation after Phase 2 completion
