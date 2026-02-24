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

#[tokio::test]
async fn test_build_relationship_graph_single_node() {
    let memory = SelfLearningMemory::new();

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let graph = memory
        .build_relationship_graph(episode_id, 3)
        .await
        .unwrap();

    assert_eq!(graph.root, episode_id);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
}

#[tokio::test]
async fn test_get_episode_with_relationships() {
    let memory = SelfLearningMemory::new();

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let result = memory
        .get_episode_with_relationships(episode_id)
        .await
        .unwrap();

    assert_eq!(result.episode.episode_id, episode_id);
    assert!(result.outgoing.is_empty());
    assert!(result.incoming.is_empty());
    assert_eq!(result.total_relationships(), 0);
}
