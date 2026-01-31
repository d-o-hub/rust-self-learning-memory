# Episode Relationships Phase 2: Core API & Business Logic

**Phase**: 2 of 6  
**Status**: ⏳ Not Started  
**Estimated Effort**: 2-3 days  
**Estimated LOC**: ~800  
**Estimated Tests**: 20+  
**Dependencies**: Phase 1 (Complete ✅)

---

## Overview

Phase 2 focuses on building the core business logic layer for episode relationships, including:
- Relationship validation and constraint enforcement
- Graph algorithms for cycle detection and traversal
- High-level relationship management API
- Integration with existing episode operations

This phase provides the foundational logic that Phases 3-5 will build upon.

---

## Architecture

### Module Structure

```
memory-core/src/episode/
├── relationships.rs          (Phase 1 - Complete ✅)
├── relationship_manager.rs   (Phase 2 - NEW)
└── graph_algorithms.rs       (Phase 2 - NEW)
```

### Component Diagram

```
┌─────────────────────────────────────────────────────┐
│          RelationshipManager (Public API)           │
│  - add_with_validation()                            │
│  - remove_with_cascade()                            │
│  - validate_relationship()                          │
└─────────────────┬───────────────────────────────────┘
                  │
                  ├─────────────────────────────────┐
                  │                                 │
    ┌─────────────▼──────────────┐   ┌─────────────▼──────────────┐
    │   Graph Algorithms         │   │   Validation Rules         │
    │  - has_cycle()             │   │  - prevent_self_loops()    │
    │  - find_cycle()            │   │  - check_acyclic()         │
    │  - topological_sort()      │   │  - validate_priority()     │
    │  - find_path()             │   │  - check_episodes_exist()  │
    └────────────────────────────┘   └────────────────────────────┘
```

---

## Component 1: RelationshipManager

**File**: `memory-core/src/episode/relationship_manager.rs`  
**Estimated LOC**: ~400  
**Tests**: 10+

### Data Structure

```rust
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::{EpisodeRelationship, RelationshipType, RelationshipMetadata};

/// Manages episode relationships with validation and graph operations
pub struct RelationshipManager {
    /// In-memory graph representation for fast traversal
    adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>>,
    
    /// Reverse index for incoming relationships
    reverse_adjacency: HashMap<Uuid, Vec<EpisodeRelationship>>,
    
    /// Cache of relationship IDs for quick existence checks
    relationship_cache: HashSet<(Uuid, Uuid, RelationshipType)>,
}
```

### Core Methods

#### 1. Constructor and Initialization

```rust
impl RelationshipManager {
    /// Create a new relationship manager
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            relationship_cache: HashSet::new(),
        }
    }
    
    /// Load relationships from storage into memory
    pub fn load_relationships(&mut self, relationships: Vec<EpisodeRelationship>) {
        // Clear existing state
        self.adjacency_list.clear();
        self.reverse_adjacency.clear();
        self.relationship_cache.clear();
        
        // Build graph
        for rel in relationships {
            // Add to forward adjacency list
            self.adjacency_list
                .entry(rel.from_episode_id)
                .or_insert_with(Vec::new)
                .push(rel.clone());
            
            // Add to reverse adjacency list
            self.reverse_adjacency
                .entry(rel.to_episode_id)
                .or_insert_with(Vec::new)
                .push(rel.clone());
            
            // Add to cache
            self.relationship_cache.insert((
                rel.from_episode_id,
                rel.to_episode_id,
                rel.relationship_type,
            ));
        }
    }
}
```

#### 2. Add Relationship with Validation

```rust
impl RelationshipManager {
    /// Add a relationship with full validation
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
            if self.would_create_cycle(from_episode_id, to_episode_id)? {
                let cycle = self.find_cycle_path(from_episode_id, to_episode_id)?;
                return Err(ValidationError::CycleDetected {
                    from: from_episode_id,
                    to: to_episode_id,
                    cycle_path: cycle,
                });
            }
        }
        
        // Validation step 4: Validate priority (if present)
        if let Some(priority) = metadata.priority {
            if priority < 1 || priority > 10 {
                return Err(ValidationError::InvalidPriority {
                    priority,
                    valid_range: (1, 10),
                });
            }
        }
        
        // Create the relationship
        let relationship = EpisodeRelationship {
            id: Uuid::new_v4(),
            from_episode_id,
            to_episode_id,
            relationship_type,
            metadata,
            created_at: chrono::Utc::now(),
        };
        
        // Update internal state
        self.add_to_graph(relationship.clone());
        
        Ok(relationship)
    }
    
    /// Internal method to add relationship to graph
    fn add_to_graph(&mut self, relationship: EpisodeRelationship) {
        // Add to forward adjacency
        self.adjacency_list
            .entry(relationship.from_episode_id)
            .or_insert_with(Vec::new)
            .push(relationship.clone());
        
        // Add to reverse adjacency
        self.reverse_adjacency
            .entry(relationship.to_episode_id)
            .or_insert_with(Vec::new)
            .push(relationship.clone());
        
        // Add to cache
        self.relationship_cache.insert((
            relationship.from_episode_id,
            relationship.to_episode_id,
            relationship.relationship_type,
        ));
    }
}
```

#### 3. Remove Relationship

```rust
impl RelationshipManager {
    /// Remove a relationship by ID
    pub fn remove_relationship(&mut self, relationship_id: Uuid) -> Result<(), RemovalError> {
        // Find the relationship
        let relationship = self.find_relationship_by_id(relationship_id)
            .ok_or(RemovalError::NotFound { id: relationship_id })?;
        
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
    
    /// Find relationship by ID (internal helper)
    fn find_relationship_by_id(&self, id: Uuid) -> Option<EpisodeRelationship> {
        for rels in self.adjacency_list.values() {
            if let Some(rel) = rels.iter().find(|r| r.id == id) {
                return Some(rel.clone());
            }
        }
        None
    }
}
```

#### 4. Query Operations

```rust
impl RelationshipManager {
    /// Check if a relationship exists
    pub fn relationship_exists(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        rel_type: RelationshipType,
    ) -> bool {
        self.relationship_cache.contains(&(from_id, to_id, rel_type))
    }
    
    /// Get all outgoing relationships for an episode
    pub fn get_outgoing(&self, episode_id: Uuid) -> Vec<EpisodeRelationship> {
        self.adjacency_list
            .get(&episode_id)
            .map(|rels| rels.clone())
            .unwrap_or_default()
    }
    
    /// Get all incoming relationships for an episode
    pub fn get_incoming(&self, episode_id: Uuid) -> Vec<EpisodeRelationship> {
        self.reverse_adjacency
            .get(&episode_id)
            .map(|rels| rels.clone())
            .unwrap_or_default()
    }
    
    /// Get relationships by type
    pub fn get_by_type(
        &self,
        episode_id: Uuid,
        rel_type: RelationshipType,
    ) -> Vec<EpisodeRelationship> {
        let mut results = Vec::new();
        
        // Get outgoing
        if let Some(rels) = self.adjacency_list.get(&episode_id) {
            results.extend(rels.iter().filter(|r| r.relationship_type == rel_type).cloned());
        }
        
        // Get incoming
        if let Some(rels) = self.reverse_adjacency.get(&episode_id) {
            results.extend(rels.iter().filter(|r| r.relationship_type == rel_type).cloned());
        }
        
        results
    }
}
```

#### 5. Cycle Detection Interface

```rust
impl RelationshipManager {
    /// Check if adding a relationship would create a cycle
    pub fn would_create_cycle(
        &self,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<bool, GraphError> {
        // Use DFS to check if path exists from to_id to from_id
        use super::graph_algorithms::has_path_dfs;
        has_path_dfs(&self.adjacency_list, to_id, from_id)
    }
    
    /// Find the cycle path if it exists
    pub fn find_cycle_path(
        &self,
        from_id: Uuid,
        to_id: Uuid,
    ) -> Result<Vec<Uuid>, GraphError> {
        use super::graph_algorithms::find_path_dfs;
        find_path_dfs(&self.adjacency_list, to_id, from_id)
    }
}
```

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Self-relationship not allowed for episode {episode_id}")]
    SelfRelationship { episode_id: Uuid },
    
    #[error("Duplicate relationship: {from} -> {to} ({rel_type:?})")]
    DuplicateRelationship {
        from: Uuid,
        to: Uuid,
        rel_type: RelationshipType,
    },
    
    #[error("Cycle detected: {from} -> {to}, cycle path: {cycle_path:?}")]
    CycleDetected {
        from: Uuid,
        to: Uuid,
        cycle_path: Vec<Uuid>,
    },
    
    #[error("Invalid priority {priority}, must be in range {valid_range:?}")]
    InvalidPriority {
        priority: u8,
        valid_range: (u8, u8),
    },
    
    #[error("Episode not found: {episode_id}")]
    EpisodeNotFound { episode_id: Uuid },
}

#[derive(Error, Debug)]
pub enum RemovalError {
    #[error("Relationship not found: {id}")]
    NotFound { id: Uuid },
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Graph traversal error: {message}")]
    TraversalError { message: String },
}
```

---

## Component 2: Graph Algorithms

**File**: `memory-core/src/episode/graph_algorithms.rs`  
**Estimated LOC**: ~400  
**Tests**: 10+

### Core Algorithms

#### 1. Depth-First Search (DFS) for Path Finding

```rust
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::EpisodeRelationship;

/// Check if a path exists from start to end using DFS
pub fn has_path_dfs(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    start: Uuid,
    end: Uuid,
) -> Result<bool, GraphError> {
    let mut visited = HashSet::new();
    has_path_dfs_helper(adjacency_list, start, end, &mut visited)
}

fn has_path_dfs_helper(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    current: Uuid,
    target: Uuid,
    visited: &mut HashSet<Uuid>,
) -> Result<bool, GraphError> {
    if current == target {
        return Ok(true);
    }
    
    if visited.contains(&current) {
        return Ok(false);
    }
    
    visited.insert(current);
    
    if let Some(neighbors) = adjacency_list.get(&current) {
        for rel in neighbors {
            if has_path_dfs_helper(adjacency_list, rel.to_episode_id, target, visited)? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}
```

#### 2. Find Path with DFS

```rust
/// Find a path from start to end, returning the episode IDs in order
pub fn find_path_dfs(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    start: Uuid,
    end: Uuid,
) -> Result<Vec<Uuid>, GraphError> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    
    if find_path_dfs_helper(adjacency_list, start, end, &mut visited, &mut path)? {
        Ok(path)
    } else {
        Err(GraphError::TraversalError {
            message: format!("No path found from {} to {}", start, end),
        })
    }
}

fn find_path_dfs_helper(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    current: Uuid,
    target: Uuid,
    visited: &mut HashSet<Uuid>,
    path: &mut Vec<Uuid>,
) -> Result<bool, GraphError> {
    path.push(current);
    
    if current == target {
        return Ok(true);
    }
    
    if visited.contains(&current) {
        path.pop();
        return Ok(false);
    }
    
    visited.insert(current);
    
    if let Some(neighbors) = adjacency_list.get(&current) {
        for rel in neighbors {
            if find_path_dfs_helper(adjacency_list, rel.to_episode_id, target, visited, path)? {
                return Ok(true);
            }
        }
    }
    
    path.pop();
    Ok(false)
}
```

#### 3. Cycle Detection

```rust
/// Detect if the graph contains any cycles
pub fn has_cycle(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
) -> Result<bool, GraphError> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    
    for &node in adjacency_list.keys() {
        if !visited.contains(&node) {
            if has_cycle_helper(adjacency_list, node, &mut visited, &mut rec_stack)? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

fn has_cycle_helper(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    current: Uuid,
    visited: &mut HashSet<Uuid>,
    rec_stack: &mut HashSet<Uuid>,
) -> Result<bool, GraphError> {
    visited.insert(current);
    rec_stack.insert(current);
    
    if let Some(neighbors) = adjacency_list.get(&current) {
        for rel in neighbors {
            let neighbor = rel.to_episode_id;
            
            if !visited.contains(&neighbor) {
                if has_cycle_helper(adjacency_list, neighbor, visited, rec_stack)? {
                    return Ok(true);
                }
            } else if rec_stack.contains(&neighbor) {
                // Back edge found - cycle detected
                return Ok(true);
            }
        }
    }
    
    rec_stack.remove(&current);
    Ok(false)
}
```

#### 4. Topological Sort

```rust
/// Perform topological sort on the graph
/// Returns Err if graph contains cycles
pub fn topological_sort(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
) -> Result<Vec<Uuid>, GraphError> {
    // Check for cycles first
    if has_cycle(adjacency_list)? {
        return Err(GraphError::TraversalError {
            message: "Cannot perform topological sort on cyclic graph".to_string(),
        });
    }
    
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    
    for &node in adjacency_list.keys() {
        if !visited.contains(&node) {
            topological_sort_helper(adjacency_list, node, &mut visited, &mut stack)?;
        }
    }
    
    stack.reverse();
    Ok(stack)
}

fn topological_sort_helper(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    current: Uuid,
    visited: &mut HashSet<Uuid>,
    stack: &mut Vec<Uuid>,
) -> Result<(), GraphError> {
    visited.insert(current);
    
    if let Some(neighbors) = adjacency_list.get(&current) {
        for rel in neighbors {
            if !visited.contains(&rel.to_episode_id) {
                topological_sort_helper(adjacency_list, rel.to_episode_id, visited, stack)?;
            }
        }
    }
    
    stack.push(current);
    Ok(())
}
```

#### 5. Transitive Closure

```rust
/// Get all episodes reachable from the starting episode
pub fn get_transitive_closure(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>>,
    start: Uuid,
) -> Result<HashSet<Uuid>, GraphError> {
    let mut reachable = HashSet::new();
    let mut to_visit = vec![start];
    
    while let Some(current) = to_visit.pop() {
        if reachable.contains(&current) {
            continue;
        }
        
        reachable.insert(current);
        
        if let Some(neighbors) = adjacency_list.get(&current) {
            for rel in neighbors {
                if !reachable.contains(&rel.to_episode_id) {
                    to_visit.push(rel.to_episode_id);
                }
            }
        }
    }
    
    // Remove the starting node from the result
    reachable.remove(&start);
    
    Ok(reachable)
}
```

---

## Testing Strategy

### Unit Tests for RelationshipManager

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
        
        assert!(matches!(result, Err(ValidationError::SelfRelationship { .. })));
    }
    
    #[test]
    fn test_prevent_duplicate_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        
        // Add first time - should succeed
        manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        ).unwrap();
        
        // Add second time - should fail
        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        
        assert!(matches!(result, Err(ValidationError::DuplicateRelationship { .. })));
    }
    
    #[test]
    fn test_detect_cycle() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        
        // Create chain: A -> B -> C
        manager.add_with_validation(a, b, RelationshipType::DependsOn, RelationshipMetadata::default()).unwrap();
        manager.add_with_validation(b, c, RelationshipType::DependsOn, RelationshipMetadata::default()).unwrap();
        
        // Try to add C -> A (would create cycle)
        let result = manager.add_with_validation(c, a, RelationshipType::DependsOn, RelationshipMetadata::default());
        
        assert!(matches!(result, Err(ValidationError::CycleDetected { .. })));
    }
    
    #[test]
    fn test_allow_non_cyclic_relationships() {
        let mut manager = RelationshipManager::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        
        // Create: A -> B, C -> B (no cycle)
        manager.add_with_validation(a, b, RelationshipType::DependsOn, RelationshipMetadata::default()).unwrap();
        let result = manager.add_with_validation(c, b, RelationshipType::DependsOn, RelationshipMetadata::default());
        
        assert!(result.is_ok());
    }
    
    // Additional tests: priority validation, removal, queries, etc.
}
```

### Unit Tests for Graph Algorithms

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_path_simple() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        
        // A -> B -> C
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);
        
        assert!(has_path_dfs(&graph, a, c).unwrap());
        assert!(!has_path_dfs(&graph, c, a).unwrap());
    }
    
    #[test]
    fn test_detect_cycle_simple() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        
        // A -> B -> A (cycle)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, a)]);
        
        assert!(has_cycle(&graph).unwrap());
    }
    
    #[test]
    fn test_topological_sort_dag() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        
        // A -> B, A -> C, B -> C
        graph.insert(a, vec![create_rel(a, b), create_rel(a, c)]);
        graph.insert(b, vec![create_rel(b, c)]);
        
        let result = topological_sort(&graph).unwrap();
        
        // A should come before B and C
        // B should come before C
        let a_pos = result.iter().position(|&x| x == a).unwrap();
        let b_pos = result.iter().position(|&x| x == b).unwrap();
        let c_pos = result.iter().position(|&x| x == c).unwrap();
        
        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < c_pos);
    }
    
    // Additional tests: transitive closure, complex graphs, edge cases
}
```

---

## Implementation Checklist

### RelationshipManager Implementation
- [ ] Create `relationship_manager.rs` file
- [ ] Implement `RelationshipManager` struct
- [ ] Implement `new()` constructor
- [ ] Implement `load_relationships()`
- [ ] Implement `add_with_validation()`
- [ ] Implement `remove_relationship()`
- [ ] Implement `relationship_exists()`
- [ ] Implement `get_outgoing()`
- [ ] Implement `get_incoming()`
- [ ] Implement `get_by_type()`
- [ ] Implement `would_create_cycle()`
- [ ] Implement `find_cycle_path()`
- [ ] Define error types (`ValidationError`, `RemovalError`)

### Graph Algorithms Implementation
- [ ] Create `graph_algorithms.rs` file
- [ ] Implement `has_path_dfs()`
- [ ] Implement `find_path_dfs()`
- [ ] Implement `has_cycle()`
- [ ] Implement `topological_sort()`
- [ ] Implement `get_transitive_closure()`
- [ ] Define `GraphError` type

### Testing
- [ ] Write tests for `add_with_validation()`
- [ ] Write tests for cycle detection
- [ ] Write tests for duplicate prevention
- [ ] Write tests for self-relationship prevention
- [ ] Write tests for priority validation
- [ ] Write tests for removal operations
- [ ] Write tests for query operations
- [ ] Write tests for all graph algorithms
- [ ] Write tests for edge cases
- [ ] Write integration tests

### Documentation
- [ ] Add rustdoc comments to all public functions
- [ ] Add usage examples
- [ ] Document error conditions
- [ ] Document performance characteristics

### Quality Gates
- [ ] All tests passing
- [ ] >90% code coverage
- [ ] Zero clippy warnings
- [ ] Formatted with rustfmt

---

## Performance Targets

| Operation | Target (P95) | Notes |
|-----------|-------------|-------|
| add_with_validation() | <5ms | Includes cycle detection |
| remove_relationship() | <1ms | In-memory operation |
| has_path_dfs() | <10ms | For graphs up to 1000 nodes |
| has_cycle() | <20ms | For graphs up to 1000 nodes |
| topological_sort() | <50ms | For graphs up to 1000 nodes |

---

## Next Steps

After Phase 2 completion:
1. Integrate with storage layer (Phase 1)
2. Add to MemoryManager API (Phase 3)
3. Expose via MCP tools (Phase 4)
4. Add CLI commands (Phase 5)

---

**Status**: Ready for implementation after Phase 1 completion ✅
