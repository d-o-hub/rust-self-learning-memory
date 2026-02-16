//! MCP Relationship Chain Tests (Day 2-3)
//!
//! Comprehensive E2E tests covering:
//! - add_episode_relationship → get_episode_relationships → find_related_episodes → remove_episode_relationship
//! - Bidirectional navigation
//! - Cascade delete
//!
//! Target: 6+ test scenarios

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::episode::{Direction, RelationshipMetadata, RelationshipType};
use memory_core::{SelfLearningMemory, TaskContext, TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use serial_test::serial;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

/// Test helper to create a memory instance with storage
///
/// IMPORTANT: Uses zero quality threshold to avoid rejecting test episodes
/// that are intentionally simple or minimal (e.g., episodes with no steps
/// to test edge cases). This ensures test isolation and predictable behavior.
async fn setup_test_memory() -> (Arc<SelfLearningMemory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    // Use zero quality threshold for testing to avoid rejecting simple test episodes
    let mut config: memory_core::MemoryConfig = Default::default();
    config.quality_threshold = 0.0;

    let memory = Arc::new(SelfLearningMemory::with_storage(
        config,
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    (memory, dir)
}

/// Helper to create and complete an episode
async fn create_completed_episode(
    memory: &Arc<SelfLearningMemory>,
    description: &str,
    domain: &str,
    task_type: TaskType,
) -> Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        ..Default::default()
    };
    let id = memory
        .start_episode(description.to_string(), context, task_type)
        .await;

    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    id
}

// ============================================================================
// Scenario 1: Complete Relationship Chain
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_full_chain() {
    let (memory, _dir) = setup_test_memory().await;

    // Create episodes
    let ep1_id = create_completed_episode(
        &memory,
        "Design system architecture",
        "rel-chain-test",
        TaskType::Analysis,
    )
    .await;

    let ep2_id = create_completed_episode(
        &memory,
        "Implement authentication",
        "rel-chain-test",
        TaskType::CodeGeneration,
    )
    .await;

    let ep3_id = create_completed_episode(
        &memory,
        "Implement user profile",
        "rel-chain-test",
        TaskType::CodeGeneration,
    )
    .await;

    // Create relationships
    let metadata = RelationshipMetadata {
        reason: Some("Design leads to implementation".to_string()),
        priority: Some(9),
        ..Default::default()
    };

    let rel_id1 = memory
        .add_episode_relationship(
            ep1_id,
            ep2_id,
            RelationshipType::ParentChild,
            metadata.clone(),
        )
        .await
        .expect("Failed to create relationship 1");

    let _rel_id2 = memory
        .add_episode_relationship(ep1_id, ep3_id, RelationshipType::ParentChild, metadata)
        .await
        .expect("Failed to create relationship 2");

    // Get relationships
    let relationships = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");

    assert_eq!(
        relationships.len(),
        2,
        "Should have 2 outgoing relationships"
    );

    // Check relationship exists
    let exists = memory
        .relationship_exists(ep1_id, ep2_id, RelationshipType::ParentChild)
        .await
        .expect("Failed to check relationship");

    assert!(exists, "Relationship should exist");

    // Find related episodes
    let filter = memory_core::memory::relationship_query::RelationshipFilter {
        relationship_type: Some(RelationshipType::ParentChild),
        ..Default::default()
    };

    let related = memory
        .find_related_episodes(ep1_id, filter)
        .await
        .expect("Failed to find related episodes");

    assert!(!related.is_empty(), "Should find related episodes");

    // Remove one relationship
    memory
        .remove_episode_relationship(rel_id1)
        .await
        .expect("Failed to remove relationship");

    let remaining = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .unwrap();

    assert_eq!(remaining.len(), 1, "Should have 1 remaining relationship");

    println!("✓ MCP relationship full chain test passed");
}

// ============================================================================
// Scenario 2: Multiple Relationship Types
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_types() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id =
        create_completed_episode(&memory, "Task 1", "rel-types-test", TaskType::Testing).await;
    let ep2_id =
        create_completed_episode(&memory, "Task 2", "rel-types-test", TaskType::Testing).await;
    let ep3_id =
        create_completed_episode(&memory, "Task 3", "rel-types-test", TaskType::Testing).await;
    let ep4_id =
        create_completed_episode(&memory, "Task 4", "rel-types-test", TaskType::Testing).await;

    // Create different relationship types
    memory
        .add_episode_relationship(
            ep1_id,
            ep2_id,
            RelationshipType::DependsOn,
            Default::default(),
        )
        .await
        .unwrap();

    memory
        .add_episode_relationship(
            ep1_id,
            ep3_id,
            RelationshipType::Follows,
            Default::default(),
        )
        .await
        .unwrap();

    memory
        .add_episode_relationship(
            ep1_id,
            ep4_id,
            RelationshipType::RelatedTo,
            Default::default(),
        )
        .await
        .unwrap();

    // Get all outgoing relationships
    let relationships = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");

    assert_eq!(relationships.len(), 3, "Should have 3 relationships");

    // Filter by type
    let depends_on: Vec<_> = relationships
        .iter()
        .filter(|r| r.relationship_type == RelationshipType::DependsOn)
        .collect();

    assert_eq!(depends_on.len(), 1, "Should have 1 DependsOn relationship");

    // Use filter to find specific type
    let filter = memory_core::memory::relationship_query::RelationshipFilter {
        relationship_type: Some(RelationshipType::DependsOn),
        ..Default::default()
    };

    let filtered = memory
        .find_related_episodes(ep1_id, filter)
        .await
        .expect("Failed to find filtered relationships");

    assert!(!filtered.is_empty(), "Should find DependsOn relationships");

    println!("✓ MCP relationship types test passed");
}

// ============================================================================
// Scenario 3: Bidirectional Relationship Navigation
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_bidirectional_navigation() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id = create_completed_episode(&memory, "Task 1", "bidir-test", TaskType::Testing).await;
    let ep2_id = create_completed_episode(&memory, "Task 2", "bidir-test", TaskType::Testing).await;

    // Create relationship from ep1 → ep2
    memory
        .add_episode_relationship(ep1_id, ep2_id, RelationshipType::Blocks, Default::default())
        .await
        .expect("Failed to create relationship");

    // Test outgoing from ep1
    let outgoing = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(outgoing.len(), 1, "ep1 should have 1 outgoing");

    // Test incoming to ep2
    let incoming = memory
        .get_episode_relationships(ep2_id, Direction::Incoming)
        .await
        .unwrap();
    assert_eq!(incoming.len(), 1, "ep2 should have 1 incoming");

    // Test both directions from ep1
    let both = memory
        .get_episode_relationships(ep1_id, Direction::Both)
        .await
        .unwrap();
    assert!(!both.is_empty(), "ep1 should have relationships");

    println!("✓ MCP bidirectional navigation test passed");
}

// ============================================================================
// Scenario 4: Relationship Graph Building
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_graph_building() {
    let (memory, _dir) = setup_test_memory().await;

    // Create a hierarchy: ep1 → [ep2, ep3], ep2 → [ep4, ep5]
    let ep1_id = create_completed_episode(&memory, "Root", "graph-test", TaskType::Testing).await;
    let ep2_id =
        create_completed_episode(&memory, "Level 1 - A", "graph-test", TaskType::Testing).await;
    let ep3_id =
        create_completed_episode(&memory, "Level 1 - B", "graph-test", TaskType::Testing).await;
    let ep4_id =
        create_completed_episode(&memory, "Level 2 - A", "graph-test", TaskType::Testing).await;
    let ep5_id =
        create_completed_episode(&memory, "Level 2 - B", "graph-test", TaskType::Testing).await;

    // Create relationships
    memory
        .add_episode_relationship(
            ep1_id,
            ep2_id,
            RelationshipType::ParentChild,
            Default::default(),
        )
        .await
        .unwrap();

    memory
        .add_episode_relationship(
            ep1_id,
            ep3_id,
            RelationshipType::ParentChild,
            Default::default(),
        )
        .await
        .unwrap();

    memory
        .add_episode_relationship(
            ep2_id,
            ep4_id,
            RelationshipType::ParentChild,
            Default::default(),
        )
        .await
        .unwrap();

    memory
        .add_episode_relationship(
            ep2_id,
            ep5_id,
            RelationshipType::ParentChild,
            Default::default(),
        )
        .await
        .unwrap();

    // Build relationship graph
    let graph = memory
        .build_relationship_graph(ep1_id, 2)
        .await
        .expect("Failed to build graph");

    assert_eq!(graph.root, ep1_id, "Root should be ep1");
    assert!(graph.node_count() >= 3, "Graph should have multiple nodes");
    assert!(graph.edge_count() >= 2, "Graph should have multiple edges");

    println!("✓ MCP relationship graph building test passed");
}

// ============================================================================
// Scenario 5: Cascade Delete Behavior
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_cascade_delete() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id =
        create_completed_episode(&memory, "Task 1", "cascade-test", TaskType::Testing).await;
    let ep2_id =
        create_completed_episode(&memory, "Task 2", "cascade-test", TaskType::Testing).await;

    // Create relationship
    memory
        .add_episode_relationship(
            ep1_id,
            ep2_id,
            RelationshipType::DependsOn,
            Default::default(),
        )
        .await
        .unwrap();

    // Verify relationship exists
    let relationships = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(relationships.len(), 1);

    // Delete episode
    memory.delete_episode(ep1_id).await.unwrap();

    // Verify episode is deleted
    assert!(memory.get_episode(ep1_id).await.is_err());

    // Verify related episode still exists
    assert!(memory.get_episode(ep2_id).await.is_ok());

    println!("✓ MCP cascade delete test passed");
}

// ============================================================================
// Scenario 6: Relationship Metadata
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_relationship_metadata() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id =
        create_completed_episode(&memory, "Source", "metadata-test", TaskType::Testing).await;
    let ep2_id =
        create_completed_episode(&memory, "Target", "metadata-test", TaskType::Testing).await;

    // Create relationship with rich metadata
    let metadata = RelationshipMetadata {
        reason: Some("Important dependency relationship".to_string()),
        priority: Some(10),
        ..Default::default()
    };

    memory
        .add_episode_relationship(ep1_id, ep2_id, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to create relationship");

    // Get and verify metadata
    let relationships = memory
        .get_episode_relationships(ep1_id, Direction::Outgoing)
        .await
        .unwrap();

    assert_eq!(relationships.len(), 1);
    let rel = &relationships[0];
    assert!(
        rel.metadata
            .reason
            .as_ref()
            .is_some_and(|d| d.contains("Important")),
        "Should have reason"
    );
    assert!(
        rel.metadata.priority.is_some_and(|p| p >= 9),
        "Should have high priority"
    );

    println!("✓ MCP relationship metadata test passed");
}
