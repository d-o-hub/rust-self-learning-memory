use super::*;
use do_memory_core::episode::Episode;
use do_memory_core::{TaskContext, TaskType};
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
        #[cfg(feature = "adaptive-ttl")]
        episode_cache: None,
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
async fn test_episode_pattern_relationships() {
    let (storage, _dir) = create_test_storage().await;
    let ep_id = create_test_episode(&storage).await;
    let pt_id = Uuid::new_v4();

    // Note: In a real scenario, we'd store the pattern first, but for this test
    // we just need the ID to satisfy the foreign key if enforce_foreign_keys is on.
    // Actually, Turso by default might not enforce them unless configured.
    // Let's create a pattern to be safe.
    let pattern = do_memory_core::Pattern::ToolSequence {
        id: pt_id,
        tools: vec!["test_tool".to_string()],
        context: TaskContext::default(),
        success_rate: 1.0,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: Default::default(),
    };
    storage.store_pattern(&pattern).await.expect("Failed to store pattern");

    let mut metadata = RelationshipMetadata::new();
    metadata.weight = Some(0.9);

    let rel = EpisodePatternRelationship::new(
        ep_id,
        pt_id,
        RelationshipType::References,
        metadata,
    );

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
    storage.add_relationship(ep1, ep2, RelationshipType::RelatedTo, meta1).await.unwrap();

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
    storage.store_episode_pattern_relationship(&ep_pt_rel).await.unwrap();

    let neighbors = storage.get_weighted_neighbors(ep1).await.expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 2);

    let ep2_neighbor = neighbors.iter().find(|(id, _, is_pt)| *id == ep2 && !*is_pt).unwrap();
    let pt1_neighbor = neighbors.iter().find(|(id, _, is_pt)| *id == pt1 && *is_pt).unwrap();

    assert_eq!(ep2_neighbor.1, 0.6);
    assert_eq!(pt1_neighbor.1, 0.4);
}
