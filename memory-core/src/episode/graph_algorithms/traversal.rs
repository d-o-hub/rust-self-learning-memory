//! Graph traversal helper functions.

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::EpisodeRelationship;
use super::GraphError;

/// DFS helper for path existence check.
pub fn has_path_dfs_helper<S>(
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

/// DFS helper for finding a path.
pub fn find_path_dfs_helper<S>(
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

/// DFS helper for cycle detection.
pub fn has_cycle_helper<S>(
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

/// DFS helper for topological sort.
pub fn topological_sort_helper<S>(
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

/// Helper for finding cycles from a specific node.
pub fn find_cycles_helper<S>(
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
