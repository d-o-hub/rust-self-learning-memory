//! Attribution capability advertisement tests (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! `TursoStorage`, `ResilientStorage`, and `CachedTursoStorage` must all
//! advertise recommendation-attribution capability so the checked persistence
//! path (`persist_session_checked` / `persist_feedback_checked`) counts them as
//! durable. The resilient and cached wrappers must delegate to their inner
//! Turso backend rather than hardcode an independent answer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::StorageBackend;
use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
use do_memory_storage_turso::{CacheConfig, CachedTursoStorage, ResilientStorage, TursoStorage};
use libsql::Builder;
use tempfile::TempDir;

/// Build a local-file Turso backend (no network, no credentials).
async fn local_turso() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("capability.db");
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");
    let storage = TursoStorage::from_database(db).expect("turso from db");
    storage.initialize_schema().await.expect("init schema");
    (storage, dir)
}

#[tokio::test]
async fn turso_storage_advertises_recommendation_attribution() {
    let (storage, _dir) = local_turso().await;
    assert!(
        storage.supports_recommendation_attribution(),
        "TursoStorage must advertise recommendation-attribution capability"
    );
}

#[tokio::test]
async fn resilient_storage_delegates_capability_to_inner_turso() {
    let (storage, _dir) = local_turso().await;
    let resilient = ResilientStorage::new(storage, CircuitBreakerConfig::default());
    assert!(
        resilient.supports_recommendation_attribution(),
        "ResilientStorage must delegate capability to its inner Turso backend"
    );
}

#[tokio::test]
async fn cached_turso_storage_delegates_capability_to_inner_backend() {
    let (storage, _dir) = local_turso().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
    assert!(
        cached.supports_recommendation_attribution(),
        "CachedTursoStorage must delegate capability to its inner Turso backend"
    );
}
