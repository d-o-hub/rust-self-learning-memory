use super::*;
use serde_json::json;

#[test]
fn test_add_relationship_input_parsing() {
    let json = json!({
        "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
        "relationship_type": "depends_on",
        "reason": "Test reason",
        "priority": 5,
        "created_by": "test_user"
    });

    let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
    assert_eq!(
        input.from_episode_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
    assert_eq!(input.relationship_type, "depends_on");
    assert_eq!(input.reason, Some("Test reason".to_string()));
    assert_eq!(input.priority, Some(5));
    assert_eq!(input.created_by, Some("test_user".to_string()));
}

#[test]
fn test_add_relationship_input_minimal() {
    let json = json!({
        "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
        "relationship_type": "parent_child"
    });

    let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
    assert_eq!(
        input.from_episode_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
    assert_eq!(input.relationship_type, "parent_child");
    assert_eq!(input.reason, None);
    assert_eq!(input.priority, None);
    assert_eq!(input.created_by, None);
}

#[test]
fn test_remove_relationship_input_parsing() {
    let json = json!({
        "relationship_id": "550e8400-e29b-41d4-a716-446655440000"
    });

    let input: RemoveEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
    assert_eq!(
        input.relationship_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn test_all_relationship_types() {
    let types = vec![
        "parent_child",
        "depends_on",
        "follows",
        "related_to",
        "blocks",
        "duplicates",
        "references",
    ];

    for rel_type in types {
        let json = json!({
            "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
            "relationship_type": rel_type
        });

        let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.relationship_type, rel_type);
    }
}

#[test]
fn test_get_relationships_input_parsing() {
    let json = json!({
        "episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "direction": "outgoing",
        "relationship_type": "depends_on"
    });

    let input: GetEpisodeRelationshipsInput = serde_json::from_value(json).unwrap();
    assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(input.direction, Some("outgoing".to_string()));
    assert_eq!(input.relationship_type, Some("depends_on".to_string()));
}

#[test]
fn test_find_related_episodes_input_parsing() {
    let json = json!({
        "episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "relationship_type": "depends_on",
        "limit": 5,
        "include_metadata": true
    });

    let input: FindRelatedEpisodesInput = serde_json::from_value(json).unwrap();
    assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(input.relationship_type, Some("depends_on".to_string()));
    assert_eq!(input.limit, Some(5));
    assert_eq!(input.include_metadata, Some(true));
}

#[test]
fn test_check_relationship_exists_input_parsing() {
    let json = json!({
        "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
        "relationship_type": "depends_on"
    });

    let input: CheckRelationshipExistsInput = serde_json::from_value(json).unwrap();
    assert_eq!(
        input.from_episode_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
    assert_eq!(input.relationship_type, "depends_on");
}

#[test]
fn test_dependency_graph_input_parsing() {
    let json = json!({
        "episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "depth": 3,
        "format": "dot"
    });

    let input: DependencyGraphInput = serde_json::from_value(json).unwrap();
    assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(input.depth, Some(3));
    assert_eq!(input.format, Some("dot".to_string()));
}

#[test]
fn test_validate_no_cycles_input_parsing() {
    let json = json!({
        "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
        "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
        "relationship_type": "depends_on"
    });

    let input: ValidateNoCyclesInput = serde_json::from_value(json).unwrap();
    assert_eq!(
        input.from_episode_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
    assert_eq!(input.relationship_type, "depends_on");
}

#[test]
fn test_get_topological_order_input_parsing() {
    let json = json!({
        "episode_ids": [
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001",
            "550e8400-e29b-41d4-a716-446655440002"
        ]
    });

    let input: GetTopologicalOrderInput = serde_json::from_value(json).unwrap();
    assert_eq!(input.episode_ids.len(), 3);
    assert_eq!(input.episode_ids[0], "550e8400-e29b-41d4-a716-446655440000");
}
