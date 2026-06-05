use super::*;
use do_memory_core::episode::RelationshipType;
use tempfile::TempDir;

async fn create_test_storage() -> (RedbStorage, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.redb");
    let storage = RedbStorage::new(&db_path)
        .await
        .expect("Failed to create storage");
    (storage, dir)
}

fn create_test_relationship(from_id: Uuid, to_id: Uuid) -> EpisodeRelationship {
    EpisodeRelationship::with_reason(
        from_id,
        to_id,
        RelationshipType::ParentChild,
        "Test relationship".to_string(),
    )
}

#[tokio::test]
async fn test_cache_and_get_relationship() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();
    let relationship = create_test_relationship(from_id, to_id);
    let rel_id = relationship.id;

    // Cache the relationship
    storage
        .cache_relationship(&relationship)
        .expect("Failed to cache relationship");

    // Retrieve it
    let cached = storage
        .get_cached_relationship(rel_id)
        .expect("Failed to get relationship");
    assert!(cached.is_some());
    let cached_rel = cached.unwrap();
    assert_eq!(cached_rel.id, rel_id);
    assert_eq!(cached_rel.from_episode_id, from_id);
    assert_eq!(cached_rel.to_episode_id, to_id);
}

#[tokio::test]
async fn test_remove_cached_relationship() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();
    let relationship = create_test_relationship(from_id, to_id);
    let rel_id = relationship.id;

    storage
        .cache_relationship(&relationship)
        .expect("Failed to cache relationship");

    // Remove it
    storage
        .remove_cached_relationship(rel_id)
        .expect("Failed to remove relationship");

    // Verify it's gone
    let cached = storage
        .get_cached_relationship(rel_id)
        .expect("Failed to get relationship");
    assert!(cached.is_none());
}

#[tokio::test]
async fn test_get_cached_relationships_outgoing() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = Uuid::new_v4();
    let to_id1 = Uuid::new_v4();
    let to_id2 = Uuid::new_v4();

    let rel1 = create_test_relationship(from_id, to_id1);
    let rel2 = create_test_relationship(from_id, to_id2);

    storage.cache_relationship(&rel1).expect("Failed to cache");
    storage.cache_relationship(&rel2).expect("Failed to cache");

    let relationships = storage
        .get_cached_relationships(from_id, Direction::Outgoing)
        .expect("Failed to get relationships");

    assert_eq!(relationships.len(), 2);
    assert!(relationships.iter().any(|r| r.to_episode_id == to_id1));
    assert!(relationships.iter().any(|r| r.to_episode_id == to_id2));
}

#[tokio::test]
async fn test_get_cached_relationships_incoming() {
    let (storage, _dir) = create_test_storage().await;
    let to_id = Uuid::new_v4();
    let from_id1 = Uuid::new_v4();
    let from_id2 = Uuid::new_v4();

    let rel1 = create_test_relationship(from_id1, to_id);
    let rel2 = create_test_relationship(from_id2, to_id);

    storage.cache_relationship(&rel1).expect("Failed to cache");
    storage.cache_relationship(&rel2).expect("Failed to cache");

    let relationships = storage
        .get_cached_relationships(to_id, Direction::Incoming)
        .expect("Failed to get relationships");

    assert_eq!(relationships.len(), 2);
    assert!(relationships.iter().any(|r| r.from_episode_id == from_id1));
    assert!(relationships.iter().any(|r| r.from_episode_id == from_id2));
}

#[tokio::test]
async fn test_clear_relationships_cache() {
    let (storage, _dir) = create_test_storage().await;
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();

    for _ in 0..5 {
        let rel = create_test_relationship(from_id, to_id);
        storage.cache_relationship(&rel).expect("Failed to cache");
    }

    let count_before = storage
        .count_cached_relationships()
        .expect("Failed to count");
    assert_eq!(count_before, 5);

    storage
        .clear_relationships_cache()
        .expect("Failed to clear cache");

    let count_after = storage
        .count_cached_relationships()
        .expect("Failed to count");
    assert_eq!(count_after, 0);
}

#[tokio::test]
async fn test_count_cached_relationships() {
    let (storage, _dir) = create_test_storage().await;

    // Add at least one relationship to ensure table exists
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();
    let rel = create_test_relationship(from_id, to_id);
    storage.cache_relationship(&rel).expect("Failed to cache");

    let count_initial = storage
        .count_cached_relationships()
        .expect("Failed to count");
    assert_eq!(count_initial, 1);

    // Add 2 more relationships (total should be 3)
    for i in 0..2 {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let rel = create_test_relationship(from_id, to_id);
        storage.cache_relationship(&rel).expect("Failed to cache");

        let count = storage
            .count_cached_relationships()
            .expect("Failed to count");
        assert_eq!(count, i + 2); // +2 because we start at 1
    }
}
