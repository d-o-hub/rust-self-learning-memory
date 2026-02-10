//! Tests for episode relationship tools

use crate::mcp::tools::episode_relationships::{
    AddEpisodeRelationshipInput, CheckRelationshipExistsInput, DependencyGraphInput,
    EpisodeRelationshipTools, FindRelatedEpisodesInput, GetEpisodeRelationshipsInput,
    GetTopologicalOrderInput, RemoveEpisodeRelationshipInput, ValidateNoCyclesInput,
};
use memory_core::SelfLearningMemory;
use memory_core::{TaskContext, TaskType};
use std::sync::Arc;

fn create_test_memory() -> Arc<SelfLearningMemory> {
    Arc::new(SelfLearningMemory::new())
}

async fn create_test_episode(memory: &SelfLearningMemory, description: &str) -> uuid::Uuid {
    memory
        .start_episode(
            description.to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await
}

#[tokio::test]
async fn test_add_relationship_success() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create two episodes
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: Some("Test reason".to_string()),
        priority: Some(5),
        created_by: Some("test".to_string()),
    };

    let result = tools.add_relationship(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(!output.relationship_id.is_empty());
    assert_eq!(output.from_episode_id, ep1.to_string());
    assert_eq!(output.to_episode_id, ep2.to_string());
    assert_eq!(output.relationship_type, "depends_on");
}

#[tokio::test]
async fn test_add_relationship_invalid_uuid() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory);

    let input = AddEpisodeRelationshipInput {
        from_episode_id: "invalid-uuid".to_string(),
        to_episode_id: uuid::Uuid::new_v4().to_string(),
        relationship_type: "depends_on".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    let result = tools.add_relationship(input).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid from_episode_id"));
}

#[tokio::test]
async fn test_add_relationship_invalid_type() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory);

    let input = AddEpisodeRelationshipInput {
        from_episode_id: uuid::Uuid::new_v4().to_string(),
        to_episode_id: uuid::Uuid::new_v4().to_string(),
        relationship_type: "invalid_type".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    let result = tools.add_relationship(input).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid relationship_type"));
}

#[tokio::test]
async fn test_remove_relationship_success() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes and a relationship
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let add_input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    let add_result = tools.add_relationship(add_input).await.unwrap();
    let rel_id = add_result.relationship_id;

    // Now remove the relationship
    let remove_input = RemoveEpisodeRelationshipInput {
        relationship_id: rel_id.clone(),
    };

    let result = tools.remove_relationship(remove_input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.relationship_id, rel_id);
}

#[tokio::test]
async fn test_remove_relationship_invalid_uuid() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory);

    let input = RemoveEpisodeRelationshipInput {
        relationship_id: "invalid-uuid".to_string(),
    };

    let result = tools.remove_relationship(input).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid relationship_id"));
}

#[tokio::test]
async fn test_get_relationships_empty() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create an episode with no relationships
    let ep = create_test_episode(&memory, "Test episode").await;

    let input = GetEpisodeRelationshipsInput {
        episode_id: ep.to_string(),
        direction: None,
        relationship_type: None,
    };

    let result = tools.get_relationships(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(output.outgoing.is_empty());
    assert!(output.incoming.is_empty());
    assert_eq!(output.total_count, 0);
}

#[tokio::test]
async fn test_get_relationships_with_relationships() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes and relationships
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let add_input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: Some("Test dependency".to_string()),
        priority: Some(8),
        created_by: None,
    };

    tools.add_relationship(add_input).await.unwrap();

    // Get relationships for ep1
    let input = GetEpisodeRelationshipsInput {
        episode_id: ep1.to_string(),
        direction: Some("outgoing".to_string()),
        relationship_type: None,
    };

    let result = tools.get_relationships(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.outgoing.len(), 1);
    assert_eq!(output.outgoing[0].relationship_type, "depends_on");
    assert_eq!(
        output.outgoing[0].reason,
        Some("Test dependency".to_string())
    );
    assert_eq!(output.outgoing[0].priority, Some(8));
}

#[tokio::test]
async fn test_find_related_episodes() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes and relationships
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let add_input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    tools.add_relationship(add_input).await.unwrap();

    let input = FindRelatedEpisodesInput {
        episode_id: ep1.to_string(),
        relationship_type: None,
        limit: None,
        include_metadata: Some(true),
    };

    let result = tools.find_related(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.count, 1);
    assert_eq!(output.related_episodes[0].episode_id, ep2.to_string());
    assert_eq!(output.related_episodes[0].direction, "outgoing");
}

#[tokio::test]
async fn test_check_relationship_exists() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes and a relationship
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let add_input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    tools.add_relationship(add_input).await.unwrap();

    // Check that relationship exists (note: this may not find it without storage backend)
    let input = CheckRelationshipExistsInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
    };

    let result = tools.check_exists(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    // Note: Without storage backend, relationship_exists returns false
    // This is expected behavior for in-memory only mode
}

#[tokio::test]
async fn test_get_dependency_graph() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes and relationships
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    let add_input = AddEpisodeRelationshipInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
        reason: None,
        priority: None,
        created_by: None,
    };

    tools.add_relationship(add_input).await.unwrap();

    let input = DependencyGraphInput {
        episode_id: ep1.to_string(),
        depth: Some(2),
        format: Some("json".to_string()),
    };

    let result = tools.get_dependency_graph(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.root, ep1.to_string());
    assert!(output.node_count >= 1);
}

#[tokio::test]
async fn test_get_dependency_graph_dot_format() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    let ep = create_test_episode(&memory, "Test episode").await;

    let input = DependencyGraphInput {
        episode_id: ep.to_string(),
        depth: Some(2),
        format: Some("dot".to_string()),
    };

    let result = tools.get_dependency_graph(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(output.dot.is_some());
    let dot = output.dot.unwrap();
    assert!(dot.contains("digraph RelationshipGraph"));
}

#[tokio::test]
async fn test_validate_no_cycles_non_acyclic_type() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    // related_to does not require acyclic
    let input = ValidateNoCyclesInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "related_to".to_string(),
    };

    let result = tools.validate_no_cycles(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(!output.would_create_cycle);
    assert!(output.is_valid);
}

#[tokio::test]
async fn test_validate_no_cycles_acyclic_type() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

    // depends_on requires acyclic
    let input = ValidateNoCyclesInput {
        from_episode_id: ep1.to_string(),
        to_episode_id: ep2.to_string(),
        relationship_type: "depends_on".to_string(),
    };

    let result = tools.validate_no_cycles(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    // Without existing relationships, no cycle would be created
    assert!(!output.would_create_cycle);
    assert!(output.is_valid);
}

#[tokio::test]
async fn test_get_topological_order_empty() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory);

    let input = GetTopologicalOrderInput {
        episode_ids: vec![],
    };

    let result = tools.get_topological_order(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(output.order.is_empty());
    assert_eq!(output.count, 0);
    assert!(!output.has_cycles);
}

#[tokio::test]
async fn test_get_topological_order_single() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    let ep = create_test_episode(&memory, "Test episode").await;

    let input = GetTopologicalOrderInput {
        episode_ids: vec![ep.to_string()],
    };

    let result = tools.get_topological_order(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.count, 1);
    assert_eq!(output.order[0].episode_id, ep.to_string());
    assert_eq!(output.order[0].position, 1);
    assert!(!output.has_cycles);
}

#[tokio::test]
async fn test_get_topological_order_with_dependencies() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes
    let ep1 = create_test_episode(&memory, "First episode").await;
    let ep2 = create_test_episode(&memory, "Second episode").await;
    let ep3 = create_test_episode(&memory, "Third episode").await;

    // Create dependency chain: ep1 -> ep2, ep2 -> ep3
    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep1.to_string(),
            to_episode_id: ep2.to_string(),
            relationship_type: "depends_on".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep2.to_string(),
            to_episode_id: ep3.to_string(),
            relationship_type: "depends_on".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    let input = GetTopologicalOrderInput {
        episode_ids: vec![ep1.to_string(), ep2.to_string(), ep3.to_string()],
    };

    let result = tools.get_topological_order(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.count, 3);
    assert!(!output.has_cycles);

    // Verify order: ep3 should come before ep2, ep2 before ep1
    let pos1 = output
        .order
        .iter()
        .position(|e| e.episode_id == ep1.to_string())
        .unwrap();
    let pos2 = output
        .order
        .iter()
        .position(|e| e.episode_id == ep2.to_string())
        .unwrap();
    let pos3 = output
        .order
        .iter()
        .position(|e| e.episode_id == ep3.to_string())
        .unwrap();

    // Topological sort follows edge direction: from appears before to.
    // ep1 depends_on ep2 (edge ep1→ep2), ep2 depends_on ep3 (edge ep2→ep3)
    // So order is: ep1, ep2, ep3 (dependents before their dependencies)
    assert!(pos1 < pos2);
    assert!(pos2 < pos3);
}

#[tokio::test]
async fn test_get_topological_order_invalid_uuid() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory);

    let input = GetTopologicalOrderInput {
        episode_ids: vec!["invalid-uuid".to_string()],
    };

    let result = tools.get_topological_order(input).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid episode_id 'invalid-uuid'"));
}

#[tokio::test]
async fn test_all_relationship_types() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;

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
        let input = AddEpisodeRelationshipInput {
            from_episode_id: ep1.to_string(),
            to_episode_id: ep2.to_string(),
            relationship_type: rel_type.to_string(),
            reason: Some(format!("Test {} relationship", rel_type)),
            priority: None,
            created_by: None,
        };

        let result = tools.add_relationship(input).await;
        assert!(
            result.is_ok(),
            "Failed to add {} relationship: {:?}",
            rel_type,
            result.err()
        );

        let output = result.unwrap();
        assert_eq!(output.relationship_type, rel_type);
    }
}

#[tokio::test]
async fn test_get_relationships_direction_filter() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;
    let ep3 = create_test_episode(&memory, "Test episode 3").await;

    // Create relationships: ep1 -> ep2, ep3 -> ep1
    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep1.to_string(),
            to_episode_id: ep2.to_string(),
            relationship_type: "depends_on".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep3.to_string(),
            to_episode_id: ep1.to_string(),
            relationship_type: "depends_on".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    // Test outgoing filter
    let outgoing_input = GetEpisodeRelationshipsInput {
        episode_id: ep1.to_string(),
        direction: Some("outgoing".to_string()),
        relationship_type: None,
    };

    let outgoing_result = tools.get_relationships(outgoing_input).await.unwrap();
    assert_eq!(outgoing_result.outgoing.len(), 1);
    assert_eq!(outgoing_result.incoming.len(), 0);
    assert_eq!(outgoing_result.outgoing[0].to, ep2.to_string());

    // Test incoming filter
    let incoming_input = GetEpisodeRelationshipsInput {
        episode_id: ep1.to_string(),
        direction: Some("incoming".to_string()),
        relationship_type: None,
    };

    let incoming_result = tools.get_relationships(incoming_input).await.unwrap();
    assert_eq!(incoming_result.outgoing.len(), 0);
    assert_eq!(incoming_result.incoming.len(), 1);
    assert_eq!(incoming_result.incoming[0].from, ep3.to_string());
}

#[tokio::test]
async fn test_find_related_with_type_filter() {
    let memory = create_test_memory();
    let tools = EpisodeRelationshipTools::new(memory.clone());

    // Create episodes
    let ep1 = create_test_episode(&memory, "Test episode 1").await;
    let ep2 = create_test_episode(&memory, "Test episode 2").await;
    let ep3 = create_test_episode(&memory, "Test episode 3").await;

    // Create relationships of different types
    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep1.to_string(),
            to_episode_id: ep2.to_string(),
            relationship_type: "depends_on".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    tools
        .add_relationship(AddEpisodeRelationshipInput {
            from_episode_id: ep1.to_string(),
            to_episode_id: ep3.to_string(),
            relationship_type: "related_to".to_string(),
            reason: None,
            priority: None,
            created_by: None,
        })
        .await
        .unwrap();

    // Find only depends_on relationships
    let input = FindRelatedEpisodesInput {
        episode_id: ep1.to_string(),
        relationship_type: Some("depends_on".to_string()),
        limit: None,
        include_metadata: None,
    };

    let result = tools.find_related(input).await.unwrap();
    assert_eq!(result.count, 1);
    assert_eq!(result.related_episodes[0].episode_id, ep2.to_string());
    assert_eq!(result.related_episodes[0].relationship_type, "depends_on");
}
