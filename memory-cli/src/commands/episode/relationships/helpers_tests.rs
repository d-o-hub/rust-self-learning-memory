//! Tests for relationship graph helper functions
//!
//! Tests for topological_sort_kahn, detect_cycle_in_graph, render_ascii_tree,
//! and output formatting for relationship result types.

use super::helpers::*;
use super::types::*;
use crate::output::Output;
use do_memory_core::episode::{
    Direction, EpisodeRelationship, RelationshipMetadata, RelationshipType,
};
use do_memory_core::memory::relationship_query::RelationshipGraph;
use std::collections::HashMap;
use uuid::Uuid;

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
fn test_remove_relationship_result_output_failure() {
    let result = RemoveRelationshipResult {
        relationship_id: "abc-123".to_string(),
        success: false,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("✗"));
    assert!(output.contains("Failed to remove"));
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

#[test]
fn test_topological_sort_kahn_with_unknown_from_node() {
    // Arrange: edges referencing nodes not in the node list
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let unknown_id = Uuid::new_v4(); // Not in nodes list

    let nodes = vec![id1, id2];
    // Edge from unknown node to id1
    let edges = vec![(unknown_id, id1), (id1, id2)];

    // Act: Should skip the unknown edge and still sort correctly
    let sorted = topological_sort_kahn(&nodes, &edges);

    // Assert: id1 should come before id2, unknown_id is skipped
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0], id1);
    assert_eq!(sorted[1], id2);
}

#[test]
fn test_topological_sort_kahn_with_unknown_to_node() {
    // Arrange: edges referencing nodes not in the node list
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let unknown_id = Uuid::new_v4(); // Not in nodes list

    let nodes = vec![id1, id2];
    // Edge from id1 to unknown node
    let edges = vec![(id1, unknown_id)];

    // Act: Should skip the unknown edge
    let sorted = topological_sort_kahn(&nodes, &edges);

    // Assert: Both nodes should appear (no dependency to unknown)
    assert_eq!(sorted.len(), 2);
    // Order depends on which has in-degree 0 first
    assert!(sorted.contains(&id1));
    assert!(sorted.contains(&id2));
}

#[test]
fn test_topological_sort_kahn_empty_nodes() {
    // Arrange: empty nodes list
    let nodes: Vec<Uuid> = vec![];
    let edges: Vec<(Uuid, Uuid)> = vec![];

    // Act
    let sorted = topological_sort_kahn(&nodes, &edges);

    // Assert: empty result
    assert!(sorted.is_empty());
}

#[test]
fn test_topological_sort_kahn_single_node() {
    // Arrange: single node with no edges
    let id1 = Uuid::new_v4();
    let nodes = vec![id1];
    let edges: Vec<(Uuid, Uuid)> = vec![];

    // Act
    let sorted = topological_sort_kahn(&nodes, &edges);

    // Assert: single node in result
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], id1);
}

#[test]
fn test_detect_cycle_in_graph_no_cycle() {
    // Arrange: graph with no cycle
    let id1 = Uuid::new_v4();
    let graph = RelationshipGraph {
        root: id1,
        nodes: HashMap::new(),
        edges: vec![],
    };

    // Act
    let has_cycle = detect_cycle_in_graph(&graph, None);

    // Assert: no cycle detected
    assert!(!has_cycle);
}

#[test]
fn test_detect_cycle_in_graph_with_cycle() {
    // Arrange: graph with a cycle
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let graph = RelationshipGraph {
        root: id1,
        nodes: HashMap::new(),
        edges: vec![
            EpisodeRelationship::new(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
            EpisodeRelationship::new(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
            EpisodeRelationship::new(
                id3,
                id1,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
        ],
    };

    // Act
    let has_cycle = detect_cycle_in_graph(&graph, None);

    // Assert: cycle detected
    assert!(has_cycle);
}

#[test]
fn test_detect_cycle_in_graph_with_type_filter() {
    // Arrange: graph with edges of different types
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let graph = RelationshipGraph {
        root: id1,
        nodes: HashMap::new(),
        edges: vec![
            EpisodeRelationship::new(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
            EpisodeRelationship::new(
                id2,
                id3,
                RelationshipType::References,
                RelationshipMetadata::new(),
            ),
            EpisodeRelationship::new(
                id3,
                id1,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
        ],
    };

    // Act: filter by DependsOn type
    let has_cycle = detect_cycle_in_graph(&graph, Some(RelationshipType::DependsOn));

    // Assert: cycle exists only through DependsOn edges (id1 -> id2, but id2->id3 is References)
    // So no complete DependsOn cycle
    assert!(!has_cycle);
}

#[test]
fn test_render_ascii_tree_empty_graph() {
    // Arrange: empty graph
    let id1 = Uuid::new_v4();
    let graph = RelationshipGraph {
        root: id1,
        nodes: HashMap::new(),
        edges: vec![],
    };

    // Act
    let output = render_ascii_tree(&graph, id1);

    // Assert: should produce output for root node
    assert!(output.contains(&id1.to_string()));
}

#[test]
fn test_render_ascii_tree_with_cycle() {
    // Arrange: graph with a cycle to test the cycle detection in render
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    let graph = RelationshipGraph {
        root: id1,
        nodes: HashMap::new(),
        edges: vec![
            EpisodeRelationship::new(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
            EpisodeRelationship::new(
                id2,
                id1,
                RelationshipType::DependsOn,
                RelationshipMetadata::new(),
            ),
        ],
    };

    // Act
    let output = render_ascii_tree(&graph, id1);

    // Assert: should show cycle marker
    assert!(output.contains("cycle"));
}
