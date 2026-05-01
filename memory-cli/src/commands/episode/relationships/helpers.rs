//! Helper functions for relationship graph operations

use std::fmt::Write;

use colored::Colorize;
use do_memory_core::episode::RelationshipType;
use uuid::Uuid;

pub(super) fn topological_sort_kahn(nodes: &[Uuid], edges: &[(Uuid, Uuid)]) -> Vec<Uuid> {
    let mut in_degree: std::collections::HashMap<Uuid, usize> =
        nodes.iter().map(|&id| (id, 0)).collect();
    let mut adj_list: std::collections::HashMap<Uuid, Vec<Uuid>> =
        nodes.iter().map(|&id| (id, Vec::new())).collect();

    // Build adjacency list and calculate in-degrees
    // Validate that all edge nodes exist in the nodes list
    for (from, to) in edges {
        let Some(adj_entry) = adj_list.get_mut(from) else {
            tracing::warn!(
                "Edge references unknown node {} (not in node list), skipping",
                from
            );
            continue;
        };
        adj_entry.push(*to);

        let Some(deg_entry) = in_degree.get_mut(to) else {
            tracing::warn!(
                "Edge references unknown node {} (not in node list), skipping",
                to
            );
            continue;
        };
        *deg_entry += 1;
    }

    // Find all nodes with in-degree 0
    let mut queue: std::collections::VecDeque<Uuid> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node);

        // Safe indexing: we know node exists in adj_list
        let neighbors = match adj_list.get(&node) {
            Some(n) => n,
            None => continue,
        };

        for &neighbor in neighbors {
            let Some(deg_entry) = in_degree.get_mut(&neighbor) else {
                continue;
            };
            *deg_entry -= 1;
            if *deg_entry == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    sorted
}

pub(super) fn render_ascii_tree(
    graph: &do_memory_core::memory::relationship_query::RelationshipGraph,
    root_id: Uuid,
) -> String {
    let mut output = String::new();
    let mut visited = std::collections::HashSet::new();

    fn render_node(
        graph: &do_memory_core::memory::relationship_query::RelationshipGraph,
        node_id: Uuid,
        prefix: &str,
        is_last: bool,
        output: &mut String,
        visited: &mut std::collections::HashSet<Uuid>,
    ) {
        if visited.contains(&node_id) {
            writeln!(
                output,
                "{}[{}] (cycle)",
                prefix,
                node_id.to_string().dimmed()
            )
            .unwrap();
            return;
        }
        visited.insert(node_id);

        // Get episode info
        let label = if let Some(ep) = graph.nodes.get(&node_id) {
            let desc = if ep.task_description.len() > 30 {
                format!("{}...", &ep.task_description[..27])
            } else {
                ep.task_description.clone()
            };
            format!("{} ({})", desc, node_id.to_string().dimmed())
        } else {
            node_id.to_string()
        };

        let branch = if is_last { "└── " } else { "├── " };
        writeln!(output, "{}{}{}", prefix, branch, label).unwrap();

        // Find outgoing relationships
        let outgoing: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.from_episode_id == node_id)
            .collect();

        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        for (i, edge) in outgoing.iter().enumerate() {
            let child_is_last = i == outgoing.len() - 1;
            let rel_label = format!("{:?}", edge.relationship_type).cyan();
            writeln!(
                output,
                "{}{}── {} → ",
                child_prefix,
                if child_is_last { "└" } else { "├" },
                rel_label
            )
            .unwrap();
            render_node(
                graph,
                edge.to_episode_id,
                &child_prefix,
                child_is_last,
                output,
                visited,
            );
        }
    }

    render_node(graph, root_id, "", true, &mut output, &mut visited);
    output
}

/// Validate that no cycles exist in relationships
pub(super) fn detect_cycle_in_graph(
    graph: &do_memory_core::memory::relationship_query::RelationshipGraph,
    relationship_type: Option<RelationshipType>,
) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();

    // Check from root
    has_cycle_util(
        graph,
        graph.root,
        &mut visited,
        &mut rec_stack,
        relationship_type,
    )
}

fn has_cycle_util(
    graph: &do_memory_core::memory::relationship_query::RelationshipGraph,
    node_id: Uuid,
    visited: &mut std::collections::HashSet<Uuid>,
    rec_stack: &mut std::collections::HashSet<Uuid>,
    relationship_type: Option<RelationshipType>,
) -> bool {
    visited.insert(node_id);
    rec_stack.insert(node_id);

    // Find outgoing edges of specified type
    for edge in &graph.edges {
        let matches = edge.from_episode_id == node_id;
        if !matches {
            continue;
        }

        // Check relationship type filter
        if let Some(ref rel_type) = relationship_type {
            if edge.relationship_type != *rel_type {
                continue;
            }
        }

        let neighbor = edge.to_episode_id;
        if !visited.contains(&neighbor) {
            if has_cycle_util(graph, neighbor, visited, rec_stack, relationship_type) {
                return true;
            }
        } else if rec_stack.contains(&neighbor) {
            return true;
        }
    }

    rec_stack.remove(&node_id);
    false
}
