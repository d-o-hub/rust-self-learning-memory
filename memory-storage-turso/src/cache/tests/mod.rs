//! Cache wrapper tests
//!
//! Unit and integration tests for the CachedTursoStorage wrapper.
//! Tests cover cache hit/miss behavior, invalidation, eviction, and concurrent access.
//!
//! ## Module Structure
//!
//! - `unit.rs`: Unit tests for basic cache operations
//! - `integration.rs`: Integration tests for StorageBackend trait
//! - `concurrent.rs`: Tests for concurrent access patterns

pub mod concurrent;
pub mod integration;
pub mod unit;

// Re-export test helpers for use in submodules
pub use super::{CacheConfig, CachedTursoStorage};
use crate::TursoStorage;
use libsql::Builder;
use memory_core::{Episode, Evidence, Heuristic, Pattern, TaskContext, TaskType};
use tempfile::TempDir;
use uuid::Uuid;

/// Create a test Turso storage instance
pub async fn create_test_turso_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_cache.db");

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to create test database");

    let storage = TursoStorage::from_database(db).expect("Failed to create storage");
    storage
        .initialize_schema()
        .await
        .expect("Failed to init schema");

    (storage, dir)
}

/// Create a test episode
pub fn create_test_episode(id: Uuid) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Test episode {}", id),
        context: TaskContext {
            domain: "test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        start_time: chrono::Utc::now(),
        end_time: None,
        metadata: std::collections::HashMap::new(),
    }
}

/// Create a test pattern
pub fn create_test_pattern(id: Uuid) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: TaskContext {
            domain: "test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.8,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: Default::default(),
    }
}

/// Create a test heuristic
pub fn create_test_heuristic(id: Uuid) -> Heuristic {
    Heuristic {
        heuristic_id: id,
        condition: "condition".to_string(),
        action: "action".to_string(),
        confidence: 0.75,
        evidence: Evidence {
            episode_ids: vec![],
            success_rate: 0.75,
            sample_size: 10,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}
