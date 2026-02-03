//! Helper functions for relationship graph operations

use colored::Colorize;
use memory_core::episode::RelationshipType;
use uuid::Uuid;

pub(super) fn topological_sort_kahn(nodes: &[Uuid], edges: &[(Uuid, Uuid)]) -> Vec<Uuid> {
    let mut in_degree: std::collections::HashMap<Uuid, usize> =
        nodes.iter().map(|&id| (id, 0)).collect();
    let mut adj_list: std::collections::HashMap<Uuid, Vec<Uuid>> =
        nodes.iter().map(|&id| (id, Vec::new())).collect();

    // Build adjacency list and calculate in-degrees
    for (from, to) in edges {
        adj_list.get_mut(from).unwrap().push(*to);
        *in_degree.get_mut(to).unwrap() += 1;
    }

    // Find all nodes with in-degree 0
    let mut queue: std::collections::VecDeque<Uuid> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node);

        for &neighbor in &adj_list[&node] {
            let deg = in_degree.get_mut(&neighbor).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_arg_conversion() {
        assert_eq!(
            RelationshipTypeArg::ParentChild.to_core_type(),
            RelationshipType::ParentChild
        );
        assert_eq!(
            RelationshipTypeArg::DependsOn.to_core_type(),
            RelationshipType::DependsOn
        );
        assert_eq!(
            RelationshipTypeArg::Follows.to_core_type(),
            RelationshipType::Follows
        );
        assert_eq!(
            RelationshipTypeArg::RelatedTo.to_core_type(),
            RelationshipType::RelatedTo
        );
        assert_eq!(
            RelationshipTypeArg::Blocks.to_core_type(),
            RelationshipType::Blocks
        );
        assert_eq!(
            RelationshipTypeArg::Duplicates.to_core_type(),
            RelationshipType::Duplicates
        );
        assert_eq!(
            RelationshipTypeArg::References.to_core_type(),
            RelationshipType::References
        );
    }

    #[test]
    fn test_direction_arg_conversion() {
        assert_eq!(
            DirectionArg::Outgoing.to_core_direction(),
            Direction::Outgoing
        );
        assert_eq!(
            DirectionArg::Incoming.to_core_direction(),
            Direction::Incoming
        );
        assert_eq!(DirectionArg::Both.to_core_direction(), Direction::Both);
    }

    #[test]
    fn test_add_relationship_result_output() {
        let result = AddRelationshipResult {
            relationship_id: "abc-123".to_string(),
            from_episode_id: "def-456".to_string(),
            to_episode_id: "ghi-789".to_string(),
            relationship_type: "DependsOn".to_string(),
            success: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Relationship Created"));
        assert!(output.contains("abc-123"));
        assert!(output.contains("def-456"));
        assert!(output.contains("ghi-789"));
        assert!(output.contains("DependsOn"));
    }

    #[test]
    fn test_remove_relationship_result_output() {
        let result = RemoveRelationshipResult {
            relationship_id: "abc-123".to_string(),
            success: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✓"));
        assert!(output.contains("abc-123"));
    }

    #[test]
    fn test_list_relationships_result_output() {
        let result = ListRelationshipsResult {
            relationships: vec![RelationshipListItem {
                id: "rel-1".to_string(),
                relationship_type: "DependsOn".to_string(),
                from: "ep-1".to_string(),
                to: "ep-2".to_string(),
                priority: Some(8),
                reason: Some("Test reason".to_string()),
            }],
            total_count: 1,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("1 relationship(s)"));
        assert!(output.contains("rel-1"));
        assert!(output.contains("ep-1"));
        assert!(output.contains("ep-2"));
    }

    #[test]
    fn test_list_relationships_empty_output() {
        let result = ListRelationshipsResult {
            relationships: vec![],
            total_count: 0,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("No relationships found"));
    }

    #[test]
    fn test_find_related_result_output() {
        let result = FindRelatedResult {
            episodes: vec![RelatedEpisodeItem {
                episode_id: "ep-2".to_string(),
                task_description: "Related task".to_string(),
                relationship_type: "DependsOn".to_string(),
                direction: "outgoing".to_string(),
            }],
            total_count: 1,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("1 related episode(s)"));
        assert!(output.contains("ep-2"));
        assert!(output.contains("Related task"));
    }

    #[test]
    fn test_validate_cycles_result_no_cycle() {
        let result = ValidateCyclesResult {
            episode_id: "ep-1".to_string(),
            has_cycle: false,
            cycle_path: None,
            message: "No cycles detected".to_string(),
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✓"));
        assert!(output.contains("No cycles detected"));
    }

    #[test]
    fn test_validate_cycles_result_with_cycle() {
        let result = ValidateCyclesResult {
            episode_id: "ep-1".to_string(),
            has_cycle: true,
            cycle_path: Some(vec![
                "ep-1".to_string(),
                "ep-2".to_string(),
                "ep-1".to_string(),
            ]),
            message: "Cycle detected".to_string(),
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✗"));
        assert!(output.contains("Cycle detected"));
    }

    #[test]
    fn test_topological_sort_result() {
        let result = TopologicalSortResult {
            ordered_episodes: vec!["ep-1".to_string(), "ep-2".to_string(), "ep-3".to_string()],
            has_cycle: false,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Topological Order"));
        assert!(output.contains("1. ep-1"));
        assert!(output.contains("2. ep-2"));
        assert!(output.contains("3. ep-3"));
    }

    #[test]
    fn test_topological_sort_result_with_cycle() {
        let result = TopologicalSortResult {
            ordered_episodes: vec![],
            has_cycle: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✗"));
        assert!(output.contains("cycle detected"));
    }

    #[test]
    fn test_topological_sort_kahn() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let nodes = vec![id1, id2, id3];
        let edges = vec![(id1, id2), (id2, id3)];

        let sorted = topological_sort_kahn(&nodes, &edges);

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], id1);
        assert_eq!(sorted[1], id2);
        assert_eq!(sorted[2], id3);
    }

    #[test]
    fn test_topological_sort_kahn_with_cycle() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let nodes = vec![id1, id2, id3];
        // Create cycle: 1 -> 2 -> 3 -> 1
        let edges = vec![(id1, id2), (id2, id3), (id3, id1)];

        let sorted = topological_sort_kahn(&nodes, &edges);

        // Should not include all nodes due to cycle
        assert!(sorted.len() < 3);
    }
}
pub(super) fn render_ascii_tree(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    root_id: Uuid,
) -> String {
    let mut output = String::new();
    let mut visited = std::collections::HashSet::new();

    fn render_node(
        graph: &memory_core::memory::relationship_query::RelationshipGraph,
        node_id: Uuid,
        prefix: &str,
        is_last: bool,
        output: &mut String,
        visited: &mut std::collections::HashSet<Uuid>,
    ) {
        if visited.contains(&node_id) {
            output.push_str(&format!(
                "{}[{}] (cycle)\n",
                prefix,
                node_id.to_string().dimmed()
            ));
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
        output.push_str(&format!("{}{}{}\n", prefix, branch, label));

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
            output.push_str(&format!(
                "{}{}── {} → ",
                child_prefix,
                if child_is_last { "└" } else { "├" },
                rel_label
            ));
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
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
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
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
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
