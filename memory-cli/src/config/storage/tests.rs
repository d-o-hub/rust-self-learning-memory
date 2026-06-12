#![allow(unsafe_code)]

use super::*;
use do_memory_core::{TaskContext, TaskType};
use serial_test::serial;
use tempfile::tempdir;

struct LocalDatabaseUrlGuard {
    original: Option<String>,
}

impl LocalDatabaseUrlGuard {
    fn set_invalid() -> Self {
        let original = std::env::var("LOCAL_DATABASE_URL").ok();
        unsafe {
            std::env::set_var("LOCAL_DATABASE_URL", "invalid://redb-only-test");
        }
        Self { original }
    }
}

impl Drop for LocalDatabaseUrlGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = &self.original {
                std::env::set_var("LOCAL_DATABASE_URL", value);
            } else {
                std::env::remove_var("LOCAL_DATABASE_URL");
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn redb_only_storage_reports_redb_backends() {
    let _guard = LocalDatabaseUrlGuard::set_invalid();
    let tmp = tempdir().expect("failed to create tempdir");
    let redb_path = tmp.path().join("memory.redb");

    let mut config = Config::default();
    config.database.turso_url = None;
    config.database.turso_token = None;
    config.database.redb_path = Some(redb_path.display().to_string());

    let storage = initialize_storage(&config)
        .await
        .expect("failed to initialize redb-only storage");

    assert!(matches!(
        storage.storage_info.primary_storage,
        StorageType::Redb
    ));
    assert!(matches!(
        storage.storage_info.cache_storage,
        StorageType::Redb
    ));
    assert!(storage.memory.has_turso_storage());
    assert!(storage.memory.has_cache_storage());
}

#[tokio::test]
#[serial]
async fn redb_only_storage_persists_across_reinitialization() {
    let _guard = LocalDatabaseUrlGuard::set_invalid();
    let tmp = tempdir().expect("failed to create tempdir");
    let redb_path = tmp.path().join("persist.redb");

    let mut config = Config::default();
    config.database.turso_url = None;
    config.database.turso_token = None;
    config.database.redb_path = Some(redb_path.display().to_string());

    let episode_id = {
        let storage = initialize_storage(&config)
            .await
            .expect("failed to initialize first memory instance");
        let id = storage
            .memory
            .start_episode(
                "persist me".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        drop(storage);
        id
    };

    let storage = initialize_storage(&config)
        .await
        .expect("failed to initialize second memory instance");
    let episode = storage
        .memory
        .get_episode(episode_id)
        .await
        .expect("expected episode to persist in redb storage");

    assert_eq!(episode.task_description, "persist me");
}

#[test]
fn test_extract_db_path_from_file_url() {
    assert_eq!(super::extract_db_path("file:/tmp/test.db"), "/tmp/test.db");
}

#[test]
fn test_extract_db_path_plain_path() {
    assert_eq!(super::extract_db_path("/tmp/test.db"), "/tmp/test.db");
}

#[test]
fn test_create_memory_config_defaults() {
    let config = Config::default();
    let mc = super::create_memory_config(&config);
    // Should not panic and produce a valid MemoryConfig
    assert!(mc.enable_summarization);
    assert!(mc.enable_spatiotemporal_indexing);
}

#[test]
fn test_storage_type_debug() {
    // Cover Debug impl for StorageType
    assert_eq!(format!("{:?}", StorageType::Turso), "Turso");
    assert_eq!(format!("{:?}", StorageType::Memory), "Memory");
    assert_eq!(format!("{:?}", StorageType::Redb), "Redb");
    assert_eq!(format!("{:?}", StorageType::None), "None");
}

#[test]
fn test_storage_type_clone() {
    // Cover Clone impl for StorageType
    let s = StorageType::LocalSqlite;
    let s2 = s.clone();
    assert!(matches!(s2, StorageType::LocalSqlite));
}

#[tokio::test]
#[serial]
async fn memory_only_fallback_when_no_storage_configured() {
    let _guard = LocalDatabaseUrlGuard::set_invalid();
    let mut config = Config::default();
    config.database.turso_url = None;
    config.database.turso_token = None;
    config.database.redb_path = None;

    let storage = initialize_storage(&config)
        .await
        .expect("should fall back to memory storage");

    assert!(matches!(
        storage.storage_info.primary_storage,
        StorageType::Memory
    ));
}
