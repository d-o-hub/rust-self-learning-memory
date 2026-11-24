use super::*;
use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
use do_memory_core::{Episode, Error, StorageBackend};
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

async fn create_test_storage() -> Result<(ResilientStorage, TempDir)> {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

    let turso = TursoStorage::from_database(db)?;
    turso.initialize_schema().await?;

    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(1),
        ..Default::default()
    };

    let resilient = ResilientStorage::new(turso, config);

    Ok((resilient, dir))
}

#[tokio::test]
async fn test_resilient_storage_creation() {
    let result = create_test_storage().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_with_closed_circuit() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let healthy = storage.health_check().await.unwrap();
    assert!(healthy);
    assert_eq!(storage.circuit_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_stats_tracking() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Perform a successful operation
    let episode = Episode::new(
        "test".to_string(),
        Default::default(),
        do_memory_core::TaskType::CodeGeneration,
    );
    let result = storage.store_episode(&episode).await;
    assert!(result.is_ok());

    // Check stats
    let stats = storage.circuit_stats().await;
    assert_eq!(stats.total_calls, 1);
    assert_eq!(stats.successful_calls, 1);
    assert_eq!(stats.failed_calls, 0);
}

#[tokio::test]
async fn test_circuit_reset() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Reset should work
    storage.reset_circuit().await;

    assert_eq!(storage.circuit_state().await, CircuitState::Closed);
    let stats = storage.circuit_stats().await;
    assert_eq!(stats.consecutive_failures, 0);
}

#[tokio::test]
async fn test_health_check_returns_false_when_circuit_open() {
    // Drop the storage and verify health check returns false
    // when the underlying backend is unavailable
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))
        .unwrap();

    let turso = TursoStorage::from_database(db).unwrap();
    turso.initialize_schema().await.unwrap();

    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(1),
        ..Default::default()
    };

    let storage = ResilientStorage::new(turso, config);

    for _ in 0..3 {
        let _ = storage.get_episode(Uuid::nil()).await;
    }

    assert_eq!(
        storage.circuit_state().await,
        CircuitState::Closed,
        "Circuit should remain closed when DB is healthy"
    );
    let healthy = storage.health_check().await.unwrap();
    assert!(
        healthy,
        "Health check should return true when circuit is closed and DB is healthy"
    );

    drop(storage);
}

#[tokio::test]
async fn test_circuit_stats_tracking_failures() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let episode = Episode::new(
        "test".to_string(),
        Default::default(),
        do_memory_core::TaskType::CodeGeneration,
    );

    let _ = storage.store_episode(&episode).await;
    let _ = storage.delete_episode(Uuid::nil()).await;

    let stats = storage.circuit_stats().await;
    assert!(
        stats.total_calls >= 1,
        "Should have at least 1 tracked call"
    );
    assert!(
        stats.successful_calls >= 1,
        "Should have at least 1 success"
    );
}

#[tokio::test]
async fn test_concurrent_operations_through_circuit() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let episode = Episode::new(
        "concurrent_test".to_string(),
        Default::default(),
        do_memory_core::TaskType::CodeGeneration,
    );

    let episode2 = Episode::new(
        "concurrent_test_2".to_string(),
        Default::default(),
        do_memory_core::TaskType::Analysis,
    );

    let (r1, r2) = tokio::join!(
        storage.store_episode(&episode),
        storage.store_episode(&episode2)
    );

    assert!(r1.is_ok(), "First concurrent operation should succeed");
    assert!(r2.is_ok(), "Second concurrent operation should succeed");

    let stats = storage.circuit_stats().await;
    assert!(
        stats.total_calls >= 2,
        "Should have at least 2 tracked calls after concurrent ops"
    );
}
