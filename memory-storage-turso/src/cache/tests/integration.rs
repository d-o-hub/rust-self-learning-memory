//! Cache wrapper tests - Integration tests
//!
//! Integration tests for the CachedTursoStorage wrapper.
//! Tests cover StorageBackend trait implementation and error handling.

use super::{CacheConfig, CachedTursoStorage};
use crate::TursoStorage;
use libsql::Builder;
use memory_core::{Episode, Evidence, Heuristic, Pattern, StorageBackend, TaskContext, TaskType};
use tempfile::TempDir;
use uuid::Uuid;

use super::create_test_episode;
use super::create_test_heuristic;
use super::create_test_pattern;
use super::create_test_turso_storage;

// ========== Integration Tests ==========

#[tokio::test]
async fn test_storage_backend_trait_episode() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode = create_test_episode(Uuid::new_v4());

    // Use StorageBackend trait methods
    cached.store_episode(&episode).await.unwrap();
    let result = cached.get_episode(episode.episode_id).await.unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().episode_id, episode.episode_id);
}

#[tokio::test]
async fn test_storage_backend_trait_pattern() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let pattern_id = Uuid::new_v4();
    let pattern = create_test_pattern(pattern_id);

    // Use StorageBackend trait methods
    cached.store_pattern(&pattern).await.unwrap();
    let result = cached.get_pattern(pattern_id).await.unwrap();

    assert!(result.is_some());
}

#[tokio::test]
async fn test_storage_backend_trait_heuristic() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let heuristic_id = Uuid::new_v4();
    let heuristic = create_test_heuristic(heuristic_id);

    // Use StorageBackend trait methods
    cached.store_heuristic(&heuristic).await.unwrap();
    let result = cached.get_heuristic(heuristic_id).await.unwrap();

    assert!(result.is_some());
}

// ========== Error Handling Tests ==========

#[tokio::test]
async fn test_get_nonexistent_episode() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let result = cached.get_episode_cached(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_pattern() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let result = cached.get_pattern_cached(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_heuristic() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let result = cached.get_heuristic_cached(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_episode() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Should not error when deleting non-existent episode
    let result = cached.delete_episode_cached(Uuid::new_v4()).await;
    assert!(result.is_ok());
}
