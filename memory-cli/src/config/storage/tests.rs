use super::*;
use memory_core::{TaskContext, TaskType};
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
