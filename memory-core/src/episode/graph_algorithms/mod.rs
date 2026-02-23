//! Graph algorithms for episode relationship analysis.
//!
//! This module provides graph traversal and analysis algorithms for detecting
//! cycles, finding paths, and computing transitive closures in episode relationship graphs.

#[cfg(test)]
mod tests;
mod traversal;

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use super::EpisodeRelationship;
use super::relationship_errors::GraphError;

pub use self::traversal::{
    find_cycles_helper, find_path_dfs_helper, has_cycle_helper, has_path_dfs_helper,
    topological_sort_helper,
};

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
