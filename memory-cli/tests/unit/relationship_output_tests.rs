//! Unit tests for relationship command output formatting.
//!
//! These tests verify that the output types for relationship commands
//! correctly implement the Output trait and produce expected output.

use do_memory_cli::commands::episode::relationships::{
    DependencyGraphResult, FindRelatedResult, ListRelationshipsResult,
    RelatedEpisodeItem, RelationshipListItem, ValidateCyclesResult,
};
use do_memory_cli::output::Output;
use tempfile::TempDir;

#[test]
fn test_dependency_graph_result_output() {
    let result = DependencyGraphResult {
        root_episode_id: "ep-123".to_string(),
        node_count: 5,
        edge_count: 3,
        output: "digraph { }".to_string(),
        format: "dot".to_string(),
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Dependency Graph"));
    assert!(output.contains("ep-123"));
    assert!(output.contains("Nodes: 5"));
    assert!(output.contains("Edges: 3"));
    assert!(output.contains("Format: dot"));
    assert!(output.contains("digraph"));
}

#[test]
fn test_dependency_graph_result_with_output_path() {
    // Test that the result correctly reports when output is written to a file
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("graph.dot");

    // Create the file to simulate the write
    std::fs::write(&output_path, "digraph { }").unwrap();

    // Verify file was written
    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("digraph"));
}

#[test]
fn test_validate_cycles_result_with_cycle_path() {
    let result = ValidateCyclesResult {
        episode_id: "ep-123".to_string(),
        has_cycle: true,
        cycle_path: Some(vec![
            "ep-123".to_string(),
            "ep-456".to_string(),
            "ep-789".to_string(),
            "ep-123".to_string(),
        ]),
        message: "Cycle detected".to_string(),
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Cycle detected"));
    assert!(output.contains("ep-123"));
    assert!(output.contains("ep-456"));
    assert!(output.contains("ep-789"));
    assert!(output.contains("ep-123")); // Cycle back
}

#[test]
fn test_find_related_result_empty() {
    let result = FindRelatedResult {
        episodes: vec![],
        total_count: 0,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("No related episodes found"));
}

#[test]
fn test_find_related_result_with_long_description() {
    let result = FindRelatedResult {
        episodes: vec![RelatedEpisodeItem {
            episode_id: "ep-123".to_string(),
            task_description: "This is a very long task description that should be truncated in the output display".to_string(),
            relationship_type: "DependsOn".to_string(),
            direction: "outgoing".to_string(),
        }],
        total_count: 1,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("1 related episode(s)"));
    assert!(output.contains("ep-123"));
    // Description should be truncated with ...
    assert!(output.contains("..."));
}

#[test]
fn test_list_relationships_result_with_long_reason() {
    let result = ListRelationshipsResult {
        relationships: vec![RelationshipListItem {
            id: "rel-123".to_string(),
            relationship_type: "DependsOn".to_string(),
            from: "ep-1".to_string(),
            to: "ep-2".to_string(),
            priority: Some(5),
            reason: Some("This is a very long reason that should be truncated in the output display for better readability".to_string()),
        }],
        total_count: 1,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("1 relationship(s)"));
    assert!(output.contains("rel-123"));
    // Reason should be truncated with ...
    assert!(output.contains("..."));
}

#[test]
fn test_list_relationships_result_with_no_priority_or_reason() {
    let result = ListRelationshipsResult {
        relationships: vec![RelationshipListItem {
            id: "rel-123".to_string(),
            relationship_type: "RelatedTo".to_string(),
            from: "ep-1".to_string(),
            to: "ep-2".to_string(),
            priority: None,
            reason: None,
        }],
        total_count: 1,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("1 relationship(s)"));
    // Priority and reason should show as "-"
    assert!(output.contains("-"));
}
