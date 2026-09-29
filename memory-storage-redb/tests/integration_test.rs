//! Integration tests for redb storage

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]

use do_memory_core::{Episode, Error, StorageBackend, TaskContext, TaskType};
use do_memory_storage_redb::{RedbQuery, RedbStorage};
use redb::ReadableDatabase;
use std::path::Path;
use tempfile::TempDir;

async fn create_test_storage() -> anyhow::Result<(RedbStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.redb");

    let storage = RedbStorage::new(&db_path).await?;
    Ok((storage, dir))
}

#[tokio::test]
async fn test_store_and_retrieve_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    // Store episode
    storage.store_episode(&episode).await.unwrap();

    // Retrieve episode
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_some());

    let retrieved_episode = retrieved.unwrap();
    assert_eq!(retrieved_episode.episode_id, episode.episode_id);
    assert_eq!(retrieved_episode.task_description, "Test task");
}

#[tokio::test]
async fn test_get_all_episodes() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store multiple episodes
    for i in 0..3 {
        let context = TaskContext::default();
        let episode = Episode::new(format!("Task {}", i), context, TaskType::Testing);
        storage.store_episode(&episode).await.unwrap();
    }

    // Get all episodes
    let query = RedbQuery { limit: None };
    let episodes = storage.get_all_episodes(&query).await.unwrap();
    assert_eq!(episodes.len(), 3);
}

#[tokio::test]
async fn test_delete_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    // Store and delete
    storage.store_episode(&episode).await.unwrap();
    storage.delete_episode(episode.episode_id).await.unwrap();

    // Verify it's deleted
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_embeddings() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let id = "test_embedding";
    let embedding = vec![0.1, 0.2, 0.3, 0.4];

    // Store embedding
    storage
        .store_embedding(id, embedding.clone())
        .await
        .unwrap();

    // Retrieve embedding
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

#[tokio::test]
async fn test_metadata() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let key = "test_key";
    let value = "test_value";

    // Store metadata
    storage.store_metadata(key, value).await.unwrap();

    // Retrieve metadata
    let retrieved = storage.get_metadata(key).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
async fn test_clear_all() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Add some data
    let context = TaskContext::default();
    let episode = Episode::new("Test".to_string(), context, TaskType::Testing);
    storage.store_episode(&episode).await.unwrap();

    // Clear all
    storage.clear_all().await.unwrap();

    // Verify cleared
    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
}

#[tokio::test]
async fn test_storage_statistics() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

// ============================================================================
// Issue #1069: opening a mismatched database must fail closed, never clear it
// ============================================================================

/// Serialize a fresh episode the way the storage layer does, for raw fixtures.
fn fixture_episode() -> (Episode, Vec<u8>) {
    let context = TaskContext::default();
    let episode = Episode::new("legacy schema task".to_string(), context, TaskType::Testing);
    let bytes = postcard::to_allocvec(&episode).unwrap();
    (episode, bytes)
}

/// Write a raw redb fixture directly, mimicking a database produced by an
/// older (or unversioned) binary: rows in `episodes` plus an optional stored
/// schema version.
fn write_raw_fixture(db_path: &Path, rows: &[(String, Vec<u8>)], version: Option<u64>) {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_write().unwrap();
    {
        let mut table = txn
            .open_table(redb::TableDefinition::<&str, &[u8]>::new("episodes"))
            .unwrap();
        for (key, value) in rows {
            table.insert(key.as_str(), value.as_slice()).unwrap();
        }
        if let Some(version) = version {
            let mut version_table = txn
                .open_table(redb::TableDefinition::<&str, u64>::new("schema_version"))
                .unwrap();
            version_table.insert("version", version).unwrap();
        }
    }
    txn.commit().unwrap();
}

fn read_raw_episode(db_path: &Path, key: &str) -> Option<Vec<u8>> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    let table = txn
        .open_table(redb::TableDefinition::<&str, &[u8]>::new("episodes"))
        .unwrap();
    table.get(key).unwrap().map(|guard| guard.value().to_vec())
}

fn read_raw_version(db_path: &Path) -> Option<u64> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    match txn.open_table(redb::TableDefinition::<&str, u64>::new("schema_version")) {
        Ok(table) => table.get("version").unwrap().map(|guard| guard.value()),
        Err(_) => None,
    }
}

#[tokio::test]
async fn test_old_schema_version_fails_closed_and_preserves_data() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("legacy.redb");

    let (episode, bytes) = fixture_episode();
    let key = episode.episode_id.to_string();
    write_raw_fixture(&db_path, &[(key.clone(), bytes.clone())], Some(1));

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("opening an old-schema database must fail closed");
    if let Error::SchemaMigrationRequired {
        path,
        stored_version,
        current_version,
        detail,
    } = &err
    {
        assert_eq!(*stored_version, Some(1));
        assert!(*current_version > 1, "current version must postdate v1");
        assert!(
            path.contains("legacy.redb"),
            "error must name the database file, got {path}"
        );
        assert!(
            !detail.is_empty(),
            "error must explain why the open was refused"
        );
    } else {
        assert!(
            err.is_schema_migration_required(),
            "expected SchemaMigrationRequired, got {err:?}"
        );
    }

    // The failed open must not have touched the data or the stored version.
    assert_eq!(
        read_raw_version(&db_path),
        Some(1),
        "stored schema version must be untouched"
    );
    let raw = read_raw_episode(&db_path, &key).expect("episode must survive the failed open");
    assert_eq!(raw, bytes, "episode bytes must be byte-for-byte intact");

    let recovered: Episode = postcard::from_bytes(&raw).unwrap();
    assert_eq!(recovered.episode_id, episode.episode_id);
    assert_eq!(recovered.task_description, "legacy schema task");
}

#[tokio::test]
async fn test_missing_schema_version_on_nonempty_db_fails_closed() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("unversioned.redb");

    let (episode, bytes) = fixture_episode();
    let key = episode.episode_id.to_string();
    write_raw_fixture(&db_path, &[(key.clone(), bytes.clone())], None);

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("a non-empty database without a version must fail closed");
    if let Error::SchemaMigrationRequired { stored_version, .. } = &err {
        assert_eq!(*stored_version, None);
    } else {
        assert!(
            err.is_schema_migration_required(),
            "expected SchemaMigrationRequired, got {err:?}"
        );
    }

    assert_eq!(read_raw_version(&db_path), None);
    assert_eq!(read_raw_episode(&db_path, &key).unwrap(), bytes);
}

#[tokio::test]
async fn test_new_empty_database_initializes_and_records_current_version() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("fresh.redb");

    let storage = RedbStorage::new(&db_path).await.unwrap();
    let recorded = storage.stored_schema_version().await.unwrap();
    assert!(
        recorded.is_some(),
        "a fresh database must record a schema version"
    );

    // With data present a matching version must not require migration.
    let context = TaskContext::default();
    let episode = Episode::new("fresh task".to_string(), context, TaskType::Testing);
    storage.store_episode(&episode).await.unwrap();
    assert!(!storage.requires_schema_migration().await.unwrap());
    drop(storage);

    assert_eq!(read_raw_version(&db_path), recorded);

    // Reopening must succeed and keep serving the data.
    let reopened = RedbStorage::new(&db_path).await.unwrap();
    assert_eq!(reopened.stored_schema_version().await.unwrap(), recorded);
    assert!(
        reopened
            .get_episode(episode.episode_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn test_open_is_not_a_clearing_path_and_explicit_reset_is() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("reset.redb");

    let storage = RedbStorage::new(&db_path).await.unwrap();
    let current = storage.stored_schema_version().await.unwrap().unwrap();

    let context = TaskContext::default();
    let episode = Episode::new("keep me".to_string(), context, TaskType::Testing);
    storage.store_episode(&episode).await.unwrap();
    assert_eq!(storage.get_statistics().await.unwrap().episode_count, 1);
    drop(storage);

    // Reopening must not clear anything.
    let storage = RedbStorage::new(&db_path).await.unwrap();
    assert!(
        storage
            .get_episode(episode.episode_id)
            .await
            .unwrap()
            .is_some(),
        "reopening must not clear data"
    );

    // The explicit reset is the sanctioned clearing path.
    storage.reset_all_tables().await.unwrap();
    assert_eq!(storage.get_statistics().await.unwrap().episode_count, 0);
    assert_eq!(
        storage.stored_schema_version().await.unwrap(),
        Some(current)
    );
}

#[tokio::test]
async fn test_reset_resolves_legacy_schema_database() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("legacy-reset.redb");

    let (episode, bytes) = fixture_episode();
    let key = episode.episode_id.to_string();
    write_raw_fixture(&db_path, &[(key.clone(), bytes)], Some(1));

    assert!(
        RedbStorage::new(&db_path).await.is_err(),
        "legacy database must not open normally"
    );

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    storage.reset_all_tables().await.unwrap();
    assert!(
        storage.stored_schema_version().await.unwrap().unwrap() > 1,
        "reset must record the current version"
    );
    drop(storage);

    // The cleared rows are gone from the file itself.
    assert!(read_raw_episode(&db_path, &key).is_none());

    // After the explicit reset the database opens normally again.
    let reopened = RedbStorage::new(&db_path).await.unwrap();
    assert!(!reopened.requires_schema_migration().await.unwrap());
}

#[tokio::test]
async fn test_migrate_schema_adopts_decodable_rows() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("migratable.redb");

    let (episode, bytes) = fixture_episode();
    let key = episode.episode_id.to_string();
    write_raw_fixture(&db_path, &[(key.clone(), bytes.clone())], Some(1));

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    storage.migrate_schema().await.unwrap();
    assert!(storage.stored_schema_version().await.unwrap().unwrap() > 1);
    drop(storage);

    // Rows preserved byte-for-byte and the database opens normally now.
    assert_eq!(read_raw_episode(&db_path, &key).unwrap(), bytes);
    let reopened = RedbStorage::new(&db_path).await.unwrap();
    assert!(
        reopened
            .get_episode(episode.episode_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn test_migrate_schema_preserves_file_when_rows_are_undecodable() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("undecodable.redb");

    let corrupt = vec![0xff_u8, 0xff, 0xff, 0xff, 0xff];
    write_raw_fixture(
        &db_path,
        &[("corrupt-key".to_string(), corrupt.clone())],
        Some(1),
    );

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    let err = storage
        .migrate_schema()
        .await
        .expect_err("undecodable rows must not be migrated");
    if let Error::SchemaMigrationRequired { stored_version, .. } = &err {
        assert_eq!(*stored_version, Some(1));
    } else {
        assert!(
            err.is_schema_migration_required(),
            "unexpected error: {err:?}"
        );
    }
    drop(storage);

    // The file must be preserved so an external tool can act on it.
    assert_eq!(read_raw_episode(&db_path, "corrupt-key").unwrap(), corrupt);
    assert_eq!(read_raw_version(&db_path), Some(1));
}
