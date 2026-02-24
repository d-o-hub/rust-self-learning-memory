use super::*;
use memory_core::episode::Episode;
use memory_core::{TaskContext, TaskType};
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to create test database");

    let storage = TursoStorage {
        db: Arc::new(db),
        pool: None,
        #[cfg(feature = "keepalive-pool")]
        keepalive_pool: None,
        adaptive_pool: None,
        caching_pool: None,
        prepared_cache: Arc::new(crate::PreparedStatementCache::with_config(
            crate::PreparedCacheConfig::default(),
        )),
        config: crate::TursoConfig::default(),
        #[cfg(feature = "compression")]
        compression_stats: Arc::new(std::sync::Mutex::new(
            crate::CompressionStatistics::default(),
        )),
    };

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

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
