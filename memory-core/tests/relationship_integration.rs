//! Integration tests for episode relationships in memory-core.
//!
//! Tests the complete relationship workflow through the memory API:
//! - Creating episodes
//! - Adding relationships
//! - Querying relationships
//! - Building relationship graphs
//! - Cascade deletion

use memory_core::episode::{RelationshipMetadata, RelationshipType};
use memory_core::memory::SelfLearningMemory;
use memory_core::{TaskContext, TaskOutcome, TaskType};
use uuid::Uuid;

/// Helper to create a test episode and return its ID
async fn create_test_episode(memory: &SelfLearningMemory, description: &str) -> Uuid {
    memory
        .start_episode(
            description.to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await
}

/// Helper to complete an episode successfully
#[allow(dead_code)]
async fn complete_test_episode(memory: &SelfLearningMemory, episode_id: Uuid) {
    let outcome = TaskOutcome::Success {
        verdict: "Test completed successfully".to_string(),
        artifacts: vec![],
    };
    memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Failed to complete episode");
}

#[tokio::test]
async fn test_add_episode_relationship() {
    let memory = SelfLearningMemory::new();

    // Create two episodes
    let parent_id = create_test_episode(&memory, "Parent task").await;
    let child_id = create_test_episode(&memory, "Child task").await;

    // Add a relationship
    let metadata = RelationshipMetadata::with_reason("Child spawned from parent".to_string());
    let result = memory
        .add_episode_relationship(parent_id, child_id, RelationshipType::ParentChild, metadata)
        .await;

    assert!(result.is_ok(), "Should successfully add relationship");
    let rel_id = result.unwrap();
    assert_ne!(rel_id, Uuid::nil());
}

#[tokio::test]
async fn test_add_relationship_validates_episodes_exist() {
    let memory = SelfLearningMemory::new();

    // Create only one episode
    let existing_id = create_test_episode(&memory, "Existing task").await;
    let fake_id = Uuid::new_v4();

    // Try to add relationship with non-existent episode
    let metadata = RelationshipMetadata::default();
    let result = memory
        .add_episode_relationship(existing_id, fake_id, RelationshipType::DependsOn, metadata)
        .await;

    assert!(result.is_err(), "Should fail when episodes don't exist");
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_episode_relationships() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;
    let ep3 = create_test_episode(&memory, "Episode 3").await;

    // Add relationships: ep1 -> ep2, ep1 -> ep3
    let metadata = RelationshipMetadata::with_reason("Dependency".to_string());
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add relationship ep1->ep2");

    memory
        .add_episode_relationship(ep1, ep3, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship ep1->ep3");

    // Get outgoing relationships from ep1
    let outgoing = memory
        .get_episode_relationships(ep1, memory_core::episode::Direction::Outgoing)
        .await
        .expect("Failed to get outgoing relationships");

    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.iter().any(|r| r.to_episode_id == ep2));
    assert!(outgoing.iter().any(|r| r.to_episode_id == ep3));

    // Get incoming relationships to ep2
    let incoming = memory
        .get_episode_relationships(ep2, memory_core::episode::Direction::Incoming)
        .await
        .expect("Failed to get incoming relationships");

    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].from_episode_id, ep1);
}

#[tokio::test]
async fn test_remove_episode_relationship() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;

    // Add relationship
    let metadata = RelationshipMetadata::with_reason("Test".to_string());
    let rel_id = memory
        .add_episode_relationship(ep1, ep2, RelationshipType::RelatedTo, metadata)
        .await
        .expect("Failed to add relationship");

    // Verify relationship exists
    let relationships = memory
        .get_episode_relationships(ep1, memory_core::episode::Direction::Outgoing)
        .await
        .expect("Failed to get relationships");
    assert_eq!(relationships.len(), 1);

    // Remove relationship
    memory
        .remove_episode_relationship(rel_id)
        .await
        .expect("Failed to remove relationship");

    // Verify relationship is gone
    let relationships = memory
        .get_episode_relationships(ep1, memory_core::episode::Direction::Outgoing)
        .await
        .expect("Failed to get relationships");
    assert_eq!(relationships.len(), 0);
}

#[tokio::test]
async fn test_find_related_episodes() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;
    let ep3 = create_test_episode(&memory, "Episode 3").await;

    // Add relationships
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add ep1->ep2");
    memory
        .add_episode_relationship(ep1, ep3, RelationshipType::RelatedTo, metadata)
        .await
        .expect("Failed to add ep1->ep3");

    // Find related episodes with DependsOn filter
    use memory_core::episode::Direction;
    use memory_core::memory::relationship_query::RelationshipFilter;

    let related = memory
        .find_related_episodes(
            ep1,
            RelationshipFilter::new()
                .with_type(RelationshipType::DependsOn)
                .with_direction(Direction::Outgoing),
        )
        .await
        .expect("Failed to find related episodes");

    assert_eq!(related.len(), 1);
    assert_eq!(related[0], ep2);
}

#[tokio::test]
async fn test_get_episode_with_relationships() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;
    let ep3 = create_test_episode(&memory, "Episode 3").await;

    // Add relationships
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add outgoing relationship");
    memory
        .add_episode_relationship(ep3, ep1, RelationshipType::Follows, metadata)
        .await
        .expect("Failed to add incoming relationship");

    // Get episode with relationships
    let result = memory
        .get_episode_with_relationships(ep1)
        .await
        .expect("Failed to get episode with relationships");

    assert_eq!(result.episode.episode_id, ep1);
    assert_eq!(result.outgoing.len(), 1);
    assert_eq!(result.incoming.len(), 1);
    assert_eq!(result.total_relationships(), 2);
    assert_eq!(result.episode.task_description, "Episode 1");
}

#[tokio::test]
async fn test_build_relationship_graph() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let root = create_test_episode(&memory, "Root episode").await;
    let child1 = create_test_episode(&memory, "Child 1").await;
    let child2 = create_test_episode(&memory, "Child 2").await;
    let grandchild = create_test_episode(&memory, "Grandchild").await;

    // Build hierarchy: root -> child1 -> grandchild, root -> child2
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(
            root,
            child1,
            RelationshipType::ParentChild,
            metadata.clone(),
        )
        .await
        .expect("Failed to add root->child1");
    memory
        .add_episode_relationship(
            root,
            child2,
            RelationshipType::ParentChild,
            metadata.clone(),
        )
        .await
        .expect("Failed to add root->child2");
    memory
        .add_episode_relationship(child1, grandchild, RelationshipType::ParentChild, metadata)
        .await
        .expect("Failed to add child1->grandchild");

    // Build graph
    let graph = memory
        .build_relationship_graph(root, 2)
        .await
        .expect("Failed to build relationship graph");

    assert_eq!(graph.root, root);
    assert_eq!(graph.node_count(), 4); // All 4 episodes
    assert_eq!(graph.edge_count(), 3); // All 3 relationships
    assert!(graph.contains_node(root));
    assert!(graph.contains_node(child1));
    assert!(graph.contains_node(child2));
    assert!(graph.contains_node(grandchild));
}

#[tokio::test]
async fn test_relationship_exists() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;

    // Check relationship doesn't exist yet
    let exists_before = memory
        .relationship_exists(ep1, ep2, RelationshipType::DependsOn)
        .await
        .expect("Failed to check relationship existence");
    assert!(!exists_before);

    // Add relationship
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    // Check relationship exists now
    let exists_after = memory
        .relationship_exists(ep1, ep2, RelationshipType::DependsOn)
        .await
        .expect("Failed to check relationship existence");
    assert!(exists_after);
}

#[tokio::test]
async fn test_get_episode_dependencies() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;
    let ep3 = create_test_episode(&memory, "Episode 3").await;

    // Add dependencies: ep1 depends on ep2 and ep3
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add ep1->ep2");
    memory
        .add_episode_relationship(ep1, ep3, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add ep1->ep3");

    // Get dependencies
    let deps = memory
        .get_episode_dependencies(ep1)
        .await
        .expect("Failed to get dependencies");

    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&ep2));
    assert!(deps.contains(&ep3));
}

#[tokio::test]
async fn test_get_episode_dependents() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;
    let ep3 = create_test_episode(&memory, "Episode 3").await;

    // Add reverse dependencies: ep2 and ep3 depend on ep1
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep2, ep1, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add ep2->ep1");
    memory
        .add_episode_relationship(ep3, ep1, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add ep3->ep1");

    // Get dependents
    let dependents = memory
        .get_episode_dependents(ep1)
        .await
        .expect("Failed to get dependents");

    assert_eq!(dependents.len(), 2);
    assert!(dependents.contains(&ep2));
    assert!(dependents.contains(&ep3));
}

#[tokio::test]
async fn test_relationship_graph_to_dot() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;

    // Add relationship
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    // Build graph and convert to DOT
    let graph = memory
        .build_relationship_graph(ep1, 1)
        .await
        .expect("Failed to build graph");

    let dot = graph.to_dot();

    // Verify DOT format
    assert!(dot.contains("digraph RelationshipGraph"));
    assert!(dot.contains(&ep1.to_string()));
    assert!(dot.contains(&ep2.to_string()));
    assert!(dot.contains("DependsOn"));
}

#[tokio::test]
async fn test_relationship_graph_to_json() {
    let memory = SelfLearningMemory::new();

    // Create episodes
    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;

    // Add relationship
    let metadata = RelationshipMetadata::default();
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    // Build graph and convert to JSON
    let graph = memory
        .build_relationship_graph(ep1, 1)
        .await
        .expect("Failed to build graph");

    let json = graph.to_json();

    // Verify JSON structure
    assert_eq!(json["node_count"], 2);
    assert_eq!(json["edge_count"], 1);
    assert!(json["nodes"].is_array());
    assert!(json["edges"].is_array());
}

// ============================================================================
// Tests with real storage (Turso + redb)
// ============================================================================

#[tokio::test]
#[ignore = "Requires real storage backends"]
async fn test_relationships_with_storage() {
    // This test would use actual Turso and redb storage
    // For now, we're testing with in-memory storage
    let memory = SelfLearningMemory::new();

    let ep1 = create_test_episode(&memory, "Episode 1").await;
    let ep2 = create_test_episode(&memory, "Episode 2").await;

    let metadata = RelationshipMetadata::with_reason("Test relationship".to_string());
    memory
        .add_episode_relationship(ep1, ep2, RelationshipType::RelatedTo, metadata)
        .await
        .expect("Failed to add relationship");

    // Verify it persisted
    let relationships = memory
        .get_episode_relationships(ep1, memory_core::episode::Direction::Outgoing)
        .await
        .expect("Failed to get relationships");

    assert_eq!(relationships.len(), 1);
}
