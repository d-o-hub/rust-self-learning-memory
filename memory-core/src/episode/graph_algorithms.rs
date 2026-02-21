//! Graph algorithms for episode relationship analysis.
//!
//! This module provides graph traversal and analysis algorithms for detecting
//! cycles, finding paths, and computing transitive closures in episode relationship graphs.

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use super::EpisodeRelationship;
use super::relationship_errors::GraphError;

/// Check if a path exists from start to end using DFS.
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
/// * `start` - The starting node (episode ID)
/// * `end` - The target node (episode ID)
///
/// # Returns
///
/// `Ok(true)` if a path exists, `Ok(false)` otherwise, or `Err(GraphError)` on error.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use memory_core::episode::graph_algorithms::has_path_dfs;
/// use memory_core::episode::{EpisodeRelationship, RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// let mut graph = HashMap::new();
/// let a = Uuid::new_v4();
/// let b = Uuid::new_v4();
///
/// graph.insert(a, vec![EpisodeRelationship::new(
///     a, b, RelationshipType::DependsOn, RelationshipMetadata::default()
/// )]);
///
/// assert!(has_path_dfs(&graph, a, b).unwrap());
/// assert!(!has_path_dfs(&graph, b, a).unwrap());
/// ```
pub fn has_path_dfs<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    start: Uuid,
    end: Uuid,
) -> Result<bool, GraphError>
where
    S: std::hash::BuildHasher,
{
    let mut visited = HashSet::new();
    has_path_dfs_helper(adjacency_list, start, end, &mut visited)
}

fn has_path_dfs_helper<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    current: Uuid,
    target: Uuid,
    visited: &mut HashSet<Uuid>,
) -> Result<bool, GraphError>
where
    S: std::hash::BuildHasher,
{
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

/// Find a path from start to end, returning the episode IDs in order.
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
/// * `start` - The starting node (episode ID)
/// * `end` - The target node (episode ID)
///
/// # Returns
///
/// `Ok(Vec<Uuid>)` containing the path from start to end (inclusive),
/// or `Err(GraphError)` if no path exists.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use memory_core::episode::graph_algorithms::find_path_dfs;
/// use memory_core::episode::{EpisodeRelationship, RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// let mut graph = HashMap::new();
/// let a = Uuid::new_v4();
/// let b = Uuid::new_v4();
/// let c = Uuid::new_v4();
///
/// graph.insert(a, vec![EpisodeRelationship::new(
///     a, b, RelationshipType::DependsOn, RelationshipMetadata::default()
/// )]);
/// graph.insert(b, vec![EpisodeRelationship::new(
///     b, c, RelationshipType::DependsOn, RelationshipMetadata::default()
/// )]);
///
/// let path = find_path_dfs(&graph, a, c).unwrap();
/// assert_eq!(path, vec![a, b, c]);
/// ```
pub fn find_path_dfs<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    start: Uuid,
    end: Uuid,
) -> Result<Vec<Uuid>, GraphError>
where
    S: std::hash::BuildHasher,
{
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    if find_path_dfs_helper(adjacency_list, start, end, &mut visited, &mut path)? {
        Ok(path)
    } else {
        Err(GraphError::TraversalError {
            message: format!("No path found from {start} to {end}"),
        })
    }
}

fn find_path_dfs_helper<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    current: Uuid,
    target: Uuid,
    visited: &mut HashSet<Uuid>,
    path: &mut Vec<Uuid>,
) -> Result<bool, GraphError>
where
    S: std::hash::BuildHasher,
{
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

/// Detect if the graph contains any cycles using DFS.
///
/// Uses a three-color marking approach (white, gray, black) where:
/// - White: Node not yet visited
/// - Gray: Node is currently being processed (in recursion stack)
/// - Black: Node and all descendants fully processed
///
/// A back edge to a gray node indicates a cycle.
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
///
/// # Returns
///
/// `Ok(true)` if a cycle exists, `Ok(false)` otherwise, or `Err(GraphError)` on error.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use memory_core::episode::graph_algorithms::has_cycle;
/// use memory_core::episode::{EpisodeRelationship, RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// // Create a cyclic graph: A -> B -> A
/// let mut graph = HashMap::new();
/// let a = Uuid::new_v4();
/// let b = Uuid::new_v4();
///
/// graph.insert(a, vec![EpisodeRelationship::new(
///     a, b, RelationshipType::DependsOn, RelationshipMetadata::default()
/// )]);
/// graph.insert(b, vec![EpisodeRelationship::new(
///     b, a, RelationshipType::DependsOn, RelationshipMetadata::default()
/// )]);
///
/// assert!(has_cycle(&graph).unwrap());
/// ```
pub fn has_cycle<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
) -> Result<bool, GraphError>
where
    S: std::hash::BuildHasher,
{
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for &node in adjacency_list.keys() {
        if !visited.contains(&node)
            && has_cycle_helper(adjacency_list, node, &mut visited, &mut rec_stack)?
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn has_cycle_helper<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    current: Uuid,
    visited: &mut HashSet<Uuid>,
    rec_stack: &mut HashSet<Uuid>,
) -> Result<bool, GraphError>
where
    S: std::hash::BuildHasher,
{
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

/// Perform topological sort on the graph.
///
/// Returns nodes in an order where all dependencies come before dependent nodes.
/// Only works on directed acyclic graphs (DAGs).
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
///
/// # Returns
///
/// `Ok(Vec<Uuid>)` containing topologically sorted nodes,
/// or `Err(GraphError)` if the graph contains cycles.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use memory_core::episode::graph_algorithms::topological_sort;
/// use memory_core::episode::{EpisodeRelationship, RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// // Create a DAG: A -> B, A -> C, B -> C
/// let mut graph = HashMap::new();
/// let a = Uuid::new_v4();
/// let b = Uuid::new_v4();
/// let c = Uuid::new_v4();
///
/// graph.insert(a, vec![
///     EpisodeRelationship::new(a, b, RelationshipType::DependsOn, RelationshipMetadata::default()),
///     EpisodeRelationship::new(a, c, RelationshipType::DependsOn, RelationshipMetadata::default()),
/// ]);
/// graph.insert(b, vec![
///     EpisodeRelationship::new(b, c, RelationshipType::DependsOn, RelationshipMetadata::default()),
/// ]);
///
/// let sorted = topological_sort(&graph).unwrap();
/// // A should come before B and C, B should come before C
/// let a_pos = sorted.iter().position(|x| *x == a).unwrap();
/// let b_pos = sorted.iter().position(|x| *x == b).unwrap();
/// let c_pos = sorted.iter().position(|x| *x == c).unwrap();
/// assert!(a_pos < b_pos);
/// assert!(a_pos < c_pos);
/// assert!(b_pos < c_pos);
/// ```
pub fn topological_sort<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
) -> Result<Vec<Uuid>, GraphError>
where
    S: std::hash::BuildHasher,
{
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

fn topological_sort_helper<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    current: Uuid,
    visited: &mut HashSet<Uuid>,
    stack: &mut Vec<Uuid>,
) -> Result<(), GraphError>
where
    S: std::hash::BuildHasher,
{
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

/// Get all episodes reachable from the starting episode using BFS.
///
/// Computes the transitive closure (all descendants) of the starting node.
/// The starting node itself is NOT included in the result.
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
/// * `start` - The starting node (episode ID)
///
/// # Returns
///
/// `Ok(HashSet<Uuid>)` containing all reachable episode IDs (excluding start),
/// or `Err(GraphError)` on error.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use memory_core::episode::graph_algorithms::get_transitive_closure;
/// use memory_core::episode::{EpisodeRelationship, RelationshipType, RelationshipMetadata};
/// use uuid::Uuid;
///
/// // Create graph: A -> B -> C, A -> D
/// let mut graph = HashMap::new();
/// let a = Uuid::new_v4();
/// let b = Uuid::new_v4();
/// let c = Uuid::new_v4();
/// let d = Uuid::new_v4();
///
/// graph.insert(a, vec![
///     EpisodeRelationship::new(a, b, RelationshipType::DependsOn, RelationshipMetadata::default()),
///     EpisodeRelationship::new(a, d, RelationshipType::DependsOn, RelationshipMetadata::default()),
/// ]);
/// graph.insert(b, vec![
///     EpisodeRelationship::new(b, c, RelationshipType::DependsOn, RelationshipMetadata::default()),
/// ]);
///
/// let reachable = get_transitive_closure(&graph, a).unwrap();
/// assert!(reachable.contains(&b));
/// assert!(reachable.contains(&c));
/// assert!(reachable.contains(&d));
/// assert!(!reachable.contains(&a)); // Start node not included
/// ```
pub fn get_transitive_closure<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    start: Uuid,
) -> Result<HashSet<Uuid>, GraphError>
where
    S: std::hash::BuildHasher,
{
    let mut reachable = HashSet::new();
    let mut to_visit = VecDeque::new();

    // Start with neighbors of the start node
    if let Some(neighbors) = adjacency_list.get(&start) {
        for rel in neighbors {
            if !reachable.contains(&rel.to_episode_id) {
                to_visit.push_back(rel.to_episode_id);
            }
        }
    }

    while let Some(current) = to_visit.pop_front() {
        if reachable.contains(&current) {
            continue;
        }

        reachable.insert(current);

        if let Some(neighbors) = adjacency_list.get(&current) {
            for rel in neighbors {
                if !reachable.contains(&rel.to_episode_id) {
                    to_visit.push_back(rel.to_episode_id);
                }
            }
        }
    }

    Ok(reachable)
}

/// Get all ancestors of an episode (nodes that can reach this episode).
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
/// * `target` - The target episode ID
///
/// # Returns
///
/// `Ok(HashSet<Uuid>)` containing all ancestor episode IDs (excluding target),
/// or `Err(GraphError)` on error.
pub fn get_ancestors<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    target: Uuid,
) -> Result<HashSet<Uuid>, GraphError>
where
    S: std::hash::BuildHasher,
{
    // Build reverse adjacency list
    let mut reverse_adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for (from_id, rels) in adjacency_list {
        for rel in rels {
            reverse_adj
                .entry(rel.to_episode_id)
                .or_default()
                .push(*from_id);
        }
    }

    let mut ancestors = HashSet::new();
    let mut to_visit = VecDeque::new();

    if let Some(parents) = reverse_adj.get(&target) {
        for &parent in parents {
            to_visit.push_back(parent);
        }
    }

    while let Some(current) = to_visit.pop_front() {
        if ancestors.contains(&current) || current == target {
            continue;
        }

        ancestors.insert(current);

        if let Some(grandparents) = reverse_adj.get(&current) {
            for &gp in grandparents {
                if !ancestors.contains(&gp) {
                    to_visit.push_back(gp);
                }
            }
        }
    }

    Ok(ancestors)
}

/// Find all simple cycles in the graph starting from a given node.
///
/// # Arguments
///
/// * `adjacency_list` - The graph represented as an adjacency list
/// * `start` - The starting node to search for cycles
///
/// # Returns
///
/// `Ok(Vec<Vec<Uuid>>)` containing all cycles (each cycle is a path from start back to start),
/// or `Err(GraphError)` on error.
pub fn find_all_cycles_from_node<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    start: Uuid,
) -> Result<Vec<Vec<Uuid>>, GraphError>
where
    S: std::hash::BuildHasher,
{
    let mut cycles = Vec::new();
    let mut path = vec![start];
    let mut visited = HashSet::new();

    find_cycles_helper(
        adjacency_list,
        start,
        start,
        &mut path,
        &mut visited,
        &mut cycles,
    )?;

    Ok(cycles)
}

fn find_cycles_helper<S>(
    adjacency_list: &HashMap<Uuid, Vec<EpisodeRelationship>, S>,
    start: Uuid,
    current: Uuid,
    path: &mut Vec<Uuid>,
    visited: &mut HashSet<Uuid>,
    cycles: &mut Vec<Vec<Uuid>>,
) -> Result<(), GraphError>
where
    S: std::hash::BuildHasher,
{
    if let Some(neighbors) = adjacency_list.get(&current) {
        for rel in neighbors {
            let neighbor = rel.to_episode_id;

            if neighbor == start && path.len() > 1 {
                // Found a cycle back to start
                cycles.push(path.clone());
                continue;
            }

            if visited.contains(&neighbor) || path.contains(&neighbor) {
                continue;
            }

            path.push(neighbor);
            visited.insert(neighbor);

            find_cycles_helper(adjacency_list, start, neighbor, path, visited, cycles)?;

            path.pop();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::{EpisodeRelationship, RelationshipMetadata, RelationshipType};

    fn create_rel(from: Uuid, to: Uuid) -> EpisodeRelationship {
        EpisodeRelationship::new(
            from,
            to,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
    }

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
        assert!(has_path_dfs(&graph, a, b).unwrap());
        assert!(has_path_dfs(&graph, b, c).unwrap());
    }

    #[test]
    fn test_has_path_same_node() {
        let graph: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();
        let a = Uuid::new_v4();

        // Path from node to itself should return true
        assert!(has_path_dfs(&graph, a, a).unwrap());
    }

    #[test]
    fn test_has_path_no_edges() {
        let graph: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        assert!(!has_path_dfs(&graph, a, b).unwrap());
    }

    #[test]
    fn test_has_path_branching() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // A -> B, A -> C, B -> D, C -> D
        graph.insert(a, vec![create_rel(a, b), create_rel(a, c)]);
        graph.insert(b, vec![create_rel(b, d)]);
        graph.insert(c, vec![create_rel(c, d)]);

        assert!(has_path_dfs(&graph, a, d).unwrap());
        assert!(has_path_dfs(&graph, b, d).unwrap());
        assert!(has_path_dfs(&graph, c, d).unwrap());
        assert!(!has_path_dfs(&graph, d, a).unwrap());
    }

    #[test]
    fn test_find_path_simple() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);

        let path = find_path_dfs(&graph, a, c).unwrap();
        assert_eq!(path, vec![a, b, c]);
    }

    #[test]
    fn test_find_path_same_node() {
        let graph: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();
        let a = Uuid::new_v4();

        let path = find_path_dfs(&graph, a, a).unwrap();
        assert_eq!(path, vec![a]);
    }

    #[test]
    fn test_find_path_not_found() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B (no path to C)
        graph.insert(a, vec![create_rel(a, b)]);

        let result = find_path_dfs(&graph, a, c);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No path found"));
    }

    #[test]
    fn test_find_path_branching() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B, A -> C (both paths exist, DFS will find one)
        graph.insert(a, vec![create_rel(a, b), create_rel(a, c)]);

        let path = find_path_dfs(&graph, a, c).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], a);
        assert_eq!(path[1], c);
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
    fn test_detect_cycle_self_loop() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();

        // A -> A (self-loop cycle)
        graph.insert(a, vec![create_rel(a, a)]);

        assert!(has_cycle(&graph).unwrap());
    }

    #[test]
    fn test_detect_no_cycle_dag() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B, B -> C (no cycle)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);

        assert!(!has_cycle(&graph).unwrap());
    }

    #[test]
    fn test_detect_cycle_complex() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // A -> B -> C -> D -> B (cycle: B -> C -> D -> B)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);
        graph.insert(c, vec![create_rel(c, d)]);
        graph.insert(d, vec![create_rel(d, b)]);

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

    #[test]
    fn test_topological_sort_cyclic_fails() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        // A -> B -> A (cycle)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, a)]);

        let result = topological_sort(&graph);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot perform topological sort on cyclic graph")
        );
    }

    #[test]
    fn test_topological_sort_empty() {
        let graph: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();

        let result = topological_sort(&graph).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_transitive_closure_simple() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);

        let reachable = get_transitive_closure(&graph, a).unwrap();
        assert!(reachable.contains(&b));
        assert!(reachable.contains(&c));
        assert!(!reachable.contains(&a)); // Start node not included
        assert_eq!(reachable.len(), 2);
    }

    #[test]
    fn test_transitive_closure_branching() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // A -> B, A -> C, B -> D
        graph.insert(a, vec![create_rel(a, b), create_rel(a, c)]);
        graph.insert(b, vec![create_rel(b, d)]);

        let reachable = get_transitive_closure(&graph, a).unwrap();
        assert!(reachable.contains(&b));
        assert!(reachable.contains(&c));
        assert!(reachable.contains(&d));
        assert!(!reachable.contains(&a));
        assert_eq!(reachable.len(), 3);
    }

    #[test]
    fn test_transitive_closure_no_outgoing() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        // A has no outgoing edges, B is isolated
        graph.insert(b, vec![]);

        let reachable = get_transitive_closure(&graph, a).unwrap();
        assert!(reachable.is_empty());
    }

    #[test]
    fn test_get_ancestors_simple() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C (ancestors of C are B and A)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);

        let ancestors = get_ancestors(&graph, c).unwrap();
        assert!(ancestors.contains(&a));
        assert!(ancestors.contains(&b));
        assert!(!ancestors.contains(&c));
        assert_eq!(ancestors.len(), 2);
    }

    #[test]
    fn test_get_ancestors_multiple_parents() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> C, B -> C (C has two parents)
        graph.insert(a, vec![create_rel(a, c)]);
        graph.insert(b, vec![create_rel(b, c)]);

        let ancestors = get_ancestors(&graph, c).unwrap();
        assert!(ancestors.contains(&a));
        assert!(ancestors.contains(&b));
        assert_eq!(ancestors.len(), 2);
    }

    #[test]
    fn test_find_all_cycles_from_node() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> A (cycle)
        // A -> C (no cycle back)
        graph.insert(a, vec![create_rel(a, b), create_rel(a, c)]);
        graph.insert(b, vec![create_rel(b, a)]);

        let cycles = find_all_cycles_from_node(&graph, a).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec![a, b]);
    }

    #[test]
    fn test_find_all_cycles_no_cycles() {
        let mut graph = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // A -> B -> C (no cycles)
        graph.insert(a, vec![create_rel(a, b)]);
        graph.insert(b, vec![create_rel(b, c)]);

        let cycles = find_all_cycles_from_node(&graph, a).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_large_graph_performance() {
        // Test with 1000 nodes in a chain
        let mut graph = HashMap::new();
        let nodes: Vec<Uuid> = (0..1000).map(|_| Uuid::new_v4()).collect();

        for i in 0..nodes.len() - 1 {
            graph.insert(nodes[i], vec![create_rel(nodes[i], nodes[i + 1])]);
        }

        // Should complete quickly for 1000 nodes
        let start = std::time::Instant::now();
        let has_cycle_result = has_cycle(&graph).unwrap();
        let duration = start.elapsed();

        assert!(!has_cycle_result);
        assert!(duration.as_millis() < 100); // Should be fast

        // Test path finding
        let start = std::time::Instant::now();
        let path = find_path_dfs(&graph, nodes[0], nodes[999]).unwrap();
        let duration = start.elapsed();

        assert_eq!(path.len(), 1000);
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_cycle_detection_performance() {
        // Test cycle detection with a complex graph
        let mut graph = HashMap::new();
        let nodes: Vec<Uuid> = (0..500).map(|_| Uuid::new_v4()).collect();

        // Create a DAG structure
        for i in 0..nodes.len() - 1 {
            let mut edges = Vec::new();
            // Each node connects to next 2 nodes (if they exist)
            if i + 1 < nodes.len() {
                edges.push(create_rel(nodes[i], nodes[i + 1]));
            }
            if i + 2 < nodes.len() {
                edges.push(create_rel(nodes[i], nodes[i + 2]));
            }
            if !edges.is_empty() {
                graph.insert(nodes[i], edges);
            }
        }

        let start = std::time::Instant::now();
        let result = has_cycle(&graph).unwrap();
        let duration = start.elapsed();

        assert!(!result);
        assert!(duration.as_millis() < 100);
    }
}
