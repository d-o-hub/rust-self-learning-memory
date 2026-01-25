//! Cache wrapper tests - Concurrent access tests
//!
//! Concurrent access tests for the CachedTursoStorage wrapper.
//! Tests cover thread safety and concurrent read/write operations.

use super::{CacheConfig, CachedTursoStorage};
use crate::TursoStorage;
use libsql::Builder;
use memory_core::{Episode, TaskContext, TaskType};
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use super::create_test_episode;
use super::create_test_turso_storage;

// ========== Concurrent Access Tests ==========

#[tokio::test]
async fn test_concurrent_episode_access() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = Arc::new(CachedTursoStorage::new(storage, CacheConfig::default()));

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);

    // Store episode
    cached.store_episode_cached(&episode).await.unwrap();

    // Spawn multiple concurrent reads
    let mut handles = Vec::new();
    for _ in 0..10 {
        let cached_clone = Arc::clone(&cached);
        let handle = tokio::spawn(async move { cached_clone.get_episode_cached(episode_id).await });
        handles.push(handle);
    }

    // Wait for all reads to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .collect();

    // All should return the episode
    for result in results {
        assert!(result.is_ok());
        let episode_opt = result.unwrap();
        assert!(episode_opt.is_some());
        assert_eq!(episode_opt.unwrap().episode_id, episode_id);
    }
}

#[tokio::test]
async fn test_concurrent_episode_updates() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = Arc::new(CachedTursoStorage::new(storage, CacheConfig::default()));

    let episode_id = Uuid::new_v4();

    // Spawn multiple concurrent writes
    let mut handles = Vec::new();
    for i in 0..5 {
        let cached_clone = Arc::clone(&cached);
        let handle = tokio::spawn(async move {
            let episode = create_test_episode(episode_id);
            let mut updated = episode.clone();
            updated.task_description = format!("Updated episode {}", i);
            let _ = cached_clone.store_episode_cached(&updated).await;
        });
        handles.push(handle);
    }

    // Wait for all writes to complete
    let _results: Vec<()> = futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .collect();

    // Verify final state exists
    let result = cached.get_episode_cached(episode_id).await.unwrap();
    assert!(result.is_some());
}

#[tokio::test]
async fn test_mixed_read_write_operations() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = Arc::new(CachedTursoStorage::new(storage, CacheConfig::default()));

    let mut handles = Vec::new();

    // Mix of read and write operations
    for i in 0..20 {
        let cached_clone = Arc::clone(&cached);
        let id = Uuid::new_v4();

        let handle = tokio::spawn(async move {
            if i % 3 == 0 {
                // Write
                let episode = create_test_episode(id);
                let _ = cached_clone.store_episode_cached(&episode).await;
            } else {
                // Read (might hit or miss)
                let _ = cached_clone.get_episode_cached(id).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    let _results: Vec<()> = futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .collect();

    // All should complete without panicking
}
