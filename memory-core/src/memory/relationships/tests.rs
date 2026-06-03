use super::*;
use crate::types::{TaskContext, TaskType};

#[tokio::test]
async fn test_add_relationship_validates_episodes() {
    let memory = SelfLearningMemory::new();
    let fake_from = Uuid::new_v4();
    let fake_to = Uuid::new_v4();

    let result = memory
        .add_episode_relationship(
            fake_from,
            fake_to,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_relationships_empty() {
    let memory = SelfLearningMemory::new();
    let episode_id = Uuid::new_v4();

    let result = memory
        .get_episode_relationships(episode_id, Direction::Both)
        .await
        .unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_find_related_empty() {
    let memory = SelfLearningMemory::new();
    let episode_id = Uuid::new_v4();

    let result = memory
        .find_related_episodes(episode_id, RelationshipFilter::default())
        .await
        .unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_relationship_exists_no_storage() {
    let memory = SelfLearningMemory::new();

    let result = memory
        .relationship_exists(Uuid::new_v4(), Uuid::new_v4(), RelationshipType::DependsOn)
        .await
        .unwrap();

    assert!(!result);
}

#[tokio::test]
async fn test_get_dependencies_empty() {
    let memory = SelfLearningMemory::new();
    let episode_id = Uuid::new_v4();

    let deps = memory.get_episode_dependencies(episode_id).await.unwrap();
    assert!(deps.is_empty());
}

#[tokio::test]
async fn test_get_dependents_empty() {
    let memory = SelfLearningMemory::new();
    let episode_id = Uuid::new_v4();

    let deps = memory.get_episode_dependents(episode_id).await.unwrap();
    assert!(deps.is_empty());
}

async fn setup_test_memory() -> (SelfLearningMemory, Uuid, Uuid) {
    let memory = SelfLearningMemory::new();
    let ep1 = memory
        .start_episode(
            "Task 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let ep2 = memory
        .start_episode(
            "Task 2".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    (memory, ep1, ep2)
}

#[tokio::test]
async fn test_build_relationship_graph_single_node() {
    let (memory, ep1, _) = setup_test_memory().await;

    let graph = memory.build_relationship_graph(ep1, 3).await.unwrap();

    assert_eq!(graph.root, ep1);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
}

#[tokio::test]
async fn test_get_episode_with_relationships() {
    let (memory, ep1, _) = setup_test_memory().await;

    let result = memory.get_episode_with_relationships(ep1).await.unwrap();

    assert_eq!(result.episode.episode_id, ep1);
    assert!(result.outgoing.is_empty());
    assert!(result.incoming.is_empty());
    assert_eq!(result.total_relationships(), 0);
}

#[tokio::test]
async fn test_add_remove_episode_relationship_success() {
    let (memory, ep1, ep2) = setup_test_memory().await;

    let rel_id = memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::DependsOn,
            RelationshipMetadata::with_reason("Test".to_string()),
        )
        .await
        .unwrap();

    let exists = memory
        .relationship_exists(ep1, ep2, RelationshipType::DependsOn)
        .await
        .unwrap();
    assert!(exists);

    memory.remove_episode_relationship(rel_id).await.unwrap();

    let exists_after = memory
        .relationship_exists(ep1, ep2, RelationshipType::DependsOn)
        .await
        .unwrap();
    assert!(!exists_after);
}

#[tokio::test]
async fn test_find_related_episodes_with_filters() {
    let (memory, ep1, ep2) = setup_test_memory().await;
    let ep3 = memory
        .start_episode(
            "Task 3".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // ep1 -> ep2 (DependsOn)
    memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    // ep1 -> ep3 (ParentChild)
    memory
        .add_episode_relationship(
            ep1,
            ep3,
            RelationshipType::ParentChild,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let filter = RelationshipFilter::new().with_type(RelationshipType::DependsOn);
    let related = memory.find_related_episodes(ep1, filter).await.unwrap();

    assert_eq!(related.len(), 1);
    assert_eq!(related[0], ep2);
}

#[tokio::test]
async fn test_build_relationship_graph_complex() {
    let (memory, ep1, ep2) = setup_test_memory().await;
    let ep3 = memory
        .start_episode(
            "Task 3".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // ep1 -> ep2 -> ep3
    memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();
    memory
        .add_episode_relationship(
            ep2,
            ep3,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let graph = memory.build_relationship_graph(ep1, 2).await.unwrap();

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
}

#[tokio::test]
async fn test_cycle_detection() {
    let (memory, ep1, ep2) = setup_test_memory().await;

    // ep1 depends on ep2
    memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    // ep2 depends on ep1 -> Cycle!
    let result = memory
        .add_episode_relationship(
            ep2,
            ep1,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[tokio::test]
async fn test_get_all_relationships() {
    let (memory, ep1, ep2) = setup_test_memory().await;

    memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let all = memory.get_all_relationships().await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn test_get_relationship_by_id() {
    let (memory, ep1, ep2) = setup_test_memory().await;

    let rel_id = memory
        .add_episode_relationship(
            ep1,
            ep2,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let rel = memory.get_relationship_by_id(rel_id).await.unwrap();
    assert!(rel.is_some());
    assert_eq!(rel.unwrap().id, rel_id);
}
