use super::*;
use do_memory_core::episode::Episode;
use do_memory_core::{TaskContext, TaskType};
use tempfile::TempDir;

async fn create_test_storage() -> (TursoStorage, TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test_memory.db");
    let storage = TursoStorage::new_local(&path)
        .await
        .expect("Failed to create local storage");
    storage.initialize_schema().await.unwrap();
    (storage, dir)
}

async fn create_test_episode(storage: &TursoStorage) -> Uuid {
    let episode = Episode::new(
        "Test episode".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let episode_id = episode.episode_id;
    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");
    episode_id
}

#[tokio::test]
async fn test_add_relationship() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let metadata = RelationshipMetadata::with_reason("Test relationship".to_string());
    let rel_id = storage
        .add_relationship(from_id, to_id, RelationshipType::ParentChild, metadata)
        .await
        .expect("Failed to add relationship");

    assert_ne!(rel_id, Uuid::nil());
}

#[tokio::test]
async fn test_store_relationship() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let metadata = RelationshipMetadata::with_reason("Test store_relationship".to_string());
    let relationship =
        EpisodeRelationship::new(from_id, to_id, RelationshipType::RelatedTo, metadata);

    storage
        .store_relationship(&relationship)
        .await
        .expect("Failed to store relationship");

    let rels = storage
        .get_relationships(from_id, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].id, relationship.id);
}

#[tokio::test]
async fn test_get_relationship_by_id() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let metadata = RelationshipMetadata::with_reason("ID lookup test".to_string());
    let rel_id = storage
        .add_relationship(from_id, to_id, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    let retrieved = storage
        .get_relationship_by_id(rel_id)
        .await
        .expect("Failed to get relationship by ID");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, rel_id);
}

#[tokio::test]
async fn test_get_all_relationships() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;

    storage
        .add_relationship(
            ep1,
            ep2,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let all = storage.get_all_relationships().await.unwrap();
    assert!(all.len() >= 1);
}

#[tokio::test]
async fn test_get_relationships() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let metadata = RelationshipMetadata::with_reason("Test".to_string());
    storage
        .add_relationship(from_id, to_id, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    let outgoing = storage
        .get_relationships(from_id, Direction::Outgoing)
        .await
        .expect("Failed to get outgoing relationships");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].from_episode_id, from_id);
    assert_eq!(outgoing[0].to_episode_id, to_id);

    let incoming = storage
        .get_relationships(to_id, Direction::Incoming)
        .await
        .expect("Failed to get incoming relationships");
    assert_eq!(incoming.len(), 1);

    // Test Direction::Both
    let both = storage
        .get_relationships(from_id, Direction::Both)
        .await
        .unwrap();
    assert_eq!(both.len(), 1);
}

#[tokio::test]
async fn test_remove_relationship() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let metadata = RelationshipMetadata::with_reason("Test".to_string());
    let rel_id = storage
        .add_relationship(from_id, to_id, RelationshipType::ParentChild, metadata)
        .await
        .expect("Failed to add relationship");

    storage
        .remove_relationship(rel_id)
        .await
        .expect("Failed to remove relationship");

    let relationships = storage
        .get_relationships(from_id, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");
    assert_eq!(relationships.len(), 0);
}

#[tokio::test]
async fn test_get_relationships_by_type() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    storage
        .add_relationship(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let rels = storage
        .get_relationships_by_type(from_id, RelationshipType::DependsOn, Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(rels.len(), 1);

    let incoming = storage
        .get_relationships_by_type(to_id, RelationshipType::DependsOn, Direction::Incoming)
        .await
        .unwrap();
    assert_eq!(incoming.len(), 1);

    let both = storage
        .get_relationships_by_type(from_id, RelationshipType::DependsOn, Direction::Both)
        .await
        .unwrap();
    assert_eq!(both.len(), 1);

    let other = storage
        .get_relationships_by_type(from_id, RelationshipType::ParentChild, Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(other.len(), 0);
}

#[tokio::test]
async fn test_relationship_exists() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = create_test_episode(&storage).await;
    let to_id = create_test_episode(&storage).await;

    let exists_before = storage
        .relationship_exists(from_id, to_id, RelationshipType::DependsOn)
        .await
        .expect("Failed to check relationship existence");
    assert!(!exists_before);

    let metadata = RelationshipMetadata::with_reason("Test".to_string());
    storage
        .add_relationship(from_id, to_id, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    let exists_after = storage
        .relationship_exists(from_id, to_id, RelationshipType::DependsOn)
        .await
        .expect("Failed to check relationship existence");
    assert!(exists_after);
}

#[tokio::test]
async fn test_get_dependent_episodes() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;

    // ep1 depends on ep2 -> ep2 is depended on by ep1
    storage
        .add_relationship(
            ep1,
            ep2,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        )
        .await
        .unwrap();

    let dependents = storage.get_dependent_episodes(ep2).await.unwrap();
    assert_eq!(dependents.len(), 1);
    assert_eq!(dependents[0], ep1);
}

#[tokio::test]
async fn test_get_dependencies() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;
    let ep3 = create_test_episode(&storage).await;

    // ep1 depends on ep2 and ep3
    let metadata = RelationshipMetadata::with_reason("Dependency".to_string());
    storage
        .add_relationship(ep1, ep2, RelationshipType::DependsOn, metadata.clone())
        .await
        .expect("Failed to add relationship");
    storage
        .add_relationship(ep1, ep3, RelationshipType::DependsOn, metadata)
        .await
        .expect("Failed to add relationship");

    let deps = storage
        .get_dependencies(ep1)
        .await
        .expect("Failed to get dependencies");
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&ep2));
    assert!(deps.contains(&ep3));
}

#[tokio::test]
async fn test_weighted_relationships() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;

    let mut metadata = RelationshipMetadata::with_reason("Weighted edge".to_string());
    metadata.weight = Some(0.75);

    storage
        .add_relationship(ep1, ep2, RelationshipType::RelatedTo, metadata)
        .await
        .expect("Failed to add weighted relationship");

    let rels = storage
        .get_relationships(ep1, Direction::Outgoing)
        .await
        .expect("Failed to get relationships");
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].metadata.weight, Some(0.75));
}

#[tokio::test]
async fn test_relationship_minimal_metadata() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;

    // Test with minimal metadata (exercising NULL paths in parameterized queries)
    let metadata = RelationshipMetadata::default();
    let rel_id = storage
        .add_relationship(ep1, ep2, RelationshipType::RelatedTo, metadata)
        .await
        .unwrap();

    let retrieved = storage
        .get_relationship_by_id(rel_id)
        .await
        .unwrap()
        .unwrap();
    assert!(retrieved.metadata.reason.is_none());
    assert!(retrieved.metadata.created_by.is_none());
    assert!(retrieved.metadata.priority.is_none());
    assert!(retrieved.metadata.weight.is_none());
}

#[tokio::test]
async fn test_episode_pattern_relationships() {
    let (storage, _dir) = create_test_storage().await;
    let ep_id = create_test_episode(&storage).await;
    let pt_id = Uuid::new_v4();

    let pattern = do_memory_core::Pattern::ToolSequence {
        id: pt_id,
        tools: vec!["test_tool".to_string()],
        context: TaskContext::default(),
        success_rate: 1.0,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: Default::default(),
    };
    storage
        .store_pattern(&pattern)
        .await
        .expect("Failed to store pattern");

    let mut metadata = RelationshipMetadata::new();
    metadata.weight = Some(0.9);

    let rel = EpisodePatternRelationship::new(ep_id, pt_id, RelationshipType::References, metadata);

    storage
        .store_episode_pattern_relationship(&rel)
        .await
        .expect("Failed to store episode-pattern relationship");

    let rels = storage
        .get_episode_pattern_relationships(ep_id)
        .await
        .expect("Failed to get episode-pattern relationships");
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].pattern_id, pt_id);
    assert_eq!(rels[0].metadata.weight, Some(0.9));
}

#[tokio::test]
async fn test_get_weighted_neighbors() {
    let (storage, _dir) = create_test_storage().await;
    let ep1 = create_test_episode(&storage).await;
    let ep2 = create_test_episode(&storage).await;
    let pt1 = Uuid::new_v4();

    // 1. Add episode-episode relationship
    let mut meta1 = RelationshipMetadata::new();
    meta1.weight = Some(0.6);
    storage
        .add_relationship(ep1, ep2, RelationshipType::RelatedTo, meta1)
        .await
        .unwrap();

    // 2. Add episode-pattern relationship
    let pattern = do_memory_core::Pattern::ToolSequence {
        id: pt1,
        tools: vec!["tool".to_string()],
        context: TaskContext::default(),
        success_rate: 1.0,
        avg_latency: chrono::Duration::milliseconds(50),
        occurrence_count: 1,
        effectiveness: Default::default(),
    };
    storage.store_pattern(&pattern).await.unwrap();

    let mut meta2 = RelationshipMetadata::new();
    meta2.weight = Some(0.4);
    let ep_pt_rel = EpisodePatternRelationship::new(ep1, pt1, RelationshipType::References, meta2);
    storage
        .store_episode_pattern_relationship(&ep_pt_rel)
        .await
        .unwrap();

    let neighbors = storage
        .get_weighted_neighbors(ep1)
        .await
        .expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 2);

    let ep2_neighbor = neighbors
        .iter()
        .find(|(id, _, is_pt)| *id == ep2 && !*is_pt)
        .unwrap();
    let pt1_neighbor = neighbors
        .iter()
        .find(|(id, _, is_pt)| *id == pt1 && *is_pt)
        .unwrap();

    assert_eq!(ep2_neighbor.1, 0.6);
    assert_eq!(pt1_neighbor.1, 0.4);
}
