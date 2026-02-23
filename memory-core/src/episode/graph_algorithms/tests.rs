//! Tests for graph algorithms.

#[cfg(test)]
mod tests {
    use crate::episode::graph_algorithms::{
        find_all_cycles_from_node, find_path_dfs, get_ancestors, get_transitive_closure, has_cycle,
        has_path_dfs, topological_sort,
    };
    use crate::episode::{EpisodeRelationship, RelationshipMetadata, RelationshipType};
    use std::collections::HashMap;
    use uuid::Uuid;

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
