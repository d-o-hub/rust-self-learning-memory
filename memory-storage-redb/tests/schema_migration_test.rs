//! Issue #1069: fail-closed schema inspection, non-destructive migration and
//! derived-index rebuild.
//!
//! These tests exercise the new `schema`/`migration` code paths directly,
//! including the accessors, `open_unchecked`, unknown-table handling, the
//! decodability check for every primary table type and index rebuilding.

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]

use do_memory_core::memory::attribution::RecommendationSession;
use do_memory_core::{
    Episode, Error, Heuristic, OutcomeStats, Pattern, PatternEffectiveness, TaskContext, TaskType,
};
use do_memory_storage_redb::RedbStorage;
use redb::{ReadableDatabase, TableDefinition};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

fn temp_db(name: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(name);
    (dir, path)
}

fn sample_episode(description: &str) -> (Episode, Vec<u8>) {
    let episode = Episode::new(
        description.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let bytes = postcard::to_allocvec(&episode).unwrap();
    (episode, bytes)
}

fn sample_pattern() -> Pattern {
    Pattern::DecisionPoint {
        id: Uuid::new_v4(),
        condition: "if cache miss then query".to_string(),
        action: "warm the cache".to_string(),
        context: TaskContext::default(),
        outcome_stats: OutcomeStats {
            success_count: 1,
            failure_count: 0,
            total_count: 1,
            avg_duration_secs: 0.5,
        },
        effectiveness: PatternEffectiveness::new(),
    }
}

fn sample_session(episode_id: Uuid) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: Vec::new(),
        recommended_playbook_ids: Vec::new(),
    }
}

fn write_bytes(db_path: &Path, table: &str, rows: &[(&str, Vec<u8>)]) {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_write().unwrap();
    {
        let mut handle = txn
            .open_table(TableDefinition::<&str, &[u8]>::new(table))
            .unwrap();
        for (key, value) in rows {
            handle.insert(*key, value.as_slice()).unwrap();
        }
    }
    txn.commit().unwrap();
}

fn write_str_rows(db_path: &Path, table: &str, rows: &[(&str, &str)]) {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_write().unwrap();
    {
        let mut handle = txn
            .open_table(TableDefinition::<&str, &str>::new(table))
            .unwrap();
        for (key, value) in rows {
            handle.insert(*key, *value).unwrap();
        }
    }
    txn.commit().unwrap();
}

fn write_version(db_path: &Path, version: u64) {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_write().unwrap();
    {
        let mut handle = txn
            .open_table(TableDefinition::<&str, u64>::new("schema_version"))
            .unwrap();
        handle.insert("version", version).unwrap();
    }
    txn.commit().unwrap();
}

/// Create `table` with the wrong `(&str, u64)` value type so opening it as a
/// byte table fails.
fn write_u64_rows(db_path: &Path, table: &str, rows: &[(&str, u64)]) {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_write().unwrap();
    {
        let mut handle = txn
            .open_table(TableDefinition::<&str, u64>::new(table))
            .unwrap();
        for (key, value) in rows {
            handle.insert(*key, *value).unwrap();
        }
    }
    txn.commit().unwrap();
}

fn read_bytes(db_path: &Path, table: &str, key: &str) -> Option<Vec<u8>> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    match txn.open_table(TableDefinition::<&str, &[u8]>::new(table)) {
        Ok(handle) => handle.get(key).unwrap().map(|guard| guard.value().to_vec()),
        Err(_) => None,
    }
}

fn read_str(db_path: &Path, table: &str, key: &str) -> Option<String> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    match txn.open_table(TableDefinition::<&str, &str>::new(table)) {
        Ok(handle) => handle
            .get(key)
            .unwrap()
            .map(|guard| guard.value().to_string()),
        Err(_) => None,
    }
}

fn read_version(db_path: &Path) -> Option<u64> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    match txn.open_table(TableDefinition::<&str, u64>::new("schema_version")) {
        Ok(handle) => handle.get("version").unwrap().map(|guard| guard.value()),
        Err(_) => None,
    }
}

fn read_u64(db_path: &Path, table: &str, key: &str) -> Option<u64> {
    let db = redb::Database::create(db_path).unwrap();
    let txn = db.begin_read().unwrap();
    match txn.open_table(TableDefinition::<&str, u64>::new(table)) {
        Ok(handle) => handle.get(key).unwrap().map(|guard| guard.value()),
        Err(_) => None,
    }
}

fn assert_migration_error(err: &Error, expected_stored: Option<u64>) {
    if let Error::SchemaMigrationRequired {
        path,
        stored_version,
        current_version,
        detail,
    } = err
    {
        assert_eq!(*stored_version, expected_stored);
        assert!(*current_version > 1);
        assert!(!path.is_empty(), "error must name the database file");
        assert!(!detail.is_empty(), "error must explain the refusal");
    } else {
        assert!(
            err.is_schema_migration_required(),
            "expected SchemaMigrationRequired, got {err:?}"
        );
    }
}

#[tokio::test]
async fn test_accessors_track_database_state() {
    let (_dir, db_path) = temp_db("state.redb");

    let storage = RedbStorage::new(&db_path).await.unwrap();
    assert_eq!(storage.path(), db_path.as_path());
    let recorded = storage.stored_schema_version().await.unwrap();
    assert!(recorded.is_some(), "fresh database records a version");
    assert!(!storage.requires_schema_migration().await.unwrap());

    // Still compatible once data present: the version matches.
    let (episode, _) = sample_episode("stateful task");
    storage.store_episode(&episode).await.unwrap();
    assert!(!storage.requires_schema_migration().await.unwrap());
    drop(storage);

    // ... until an older version is stamped by "another binary".
    write_version(&db_path, 1);
    let unchecked = RedbStorage::open_unchecked(&db_path).await.unwrap();
    assert_eq!(unchecked.stored_schema_version().await.unwrap(), Some(1));
    assert!(unchecked.requires_schema_migration().await.unwrap());
}

#[tokio::test]
async fn test_open_unchecked_does_not_modify_the_database() {
    let (_dir, db_path) = temp_db("untouched.redb");
    let (episode, bytes) = sample_episode("untouched task");
    let key = episode.episode_id.to_string();
    write_bytes(&db_path, "episodes", &[(key.as_str(), bytes.clone())]);
    write_version(&db_path, 1);

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    assert!(storage.requires_schema_migration().await.unwrap());
    drop(storage);

    assert_eq!(read_version(&db_path), Some(1));
    assert_eq!(read_bytes(&db_path, "episodes", key.as_str()), Some(bytes));
}

#[tokio::test]
async fn test_unknown_table_is_treated_as_data() {
    let (_dir, db_path) = temp_db("unknown.redb");
    write_bytes(&db_path, "future_table", &[("k", vec![1, 2, 3])]);

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("unknown table without a version must fail closed");
    assert_migration_error(&err, None);
    assert_eq!(
        read_bytes(&db_path, "future_table", "k"),
        Some(vec![1, 2, 3])
    );
}

#[tokio::test]
async fn test_recommendation_index_rows_are_treated_as_data() {
    let (_dir, db_path) = temp_db("index-only.redb");
    write_str_rows(
        &db_path,
        "recommendation_episode_index",
        &[("episode", "session")],
    );

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("index rows without a version must fail closed");
    assert_migration_error(&err, None);
    assert_eq!(
        read_str(&db_path, "recommendation_episode_index", "episode"),
        Some("session".to_string())
    );
}

#[tokio::test]
async fn test_migrate_schema_records_version_on_empty_database() {
    let (_dir, db_path) = temp_db("empty-migrate.redb");
    {
        // A valid, existing but completely empty database file.
        let db = redb::Database::create(&db_path).unwrap();
        let txn = db.begin_write().unwrap();
        txn.commit().unwrap();
    }

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    assert_eq!(storage.stored_schema_version().await.unwrap(), None);
    assert!(!storage.requires_schema_migration().await.unwrap());
    storage.migrate_schema().await.unwrap();
    let recorded = storage.stored_schema_version().await.unwrap();
    assert!(recorded.is_some());
    drop(storage);

    assert_eq!(read_version(&db_path), recorded);
    // A normal open now succeeds against the stamped version.
    assert!(RedbStorage::new(&db_path).await.is_ok());
}

#[tokio::test]
async fn test_migrate_schema_is_noop_when_version_is_current() {
    let (_dir, db_path) = temp_db("current.redb");
    let storage = RedbStorage::new(&db_path).await.unwrap();
    let (episode, _) = sample_episode("already current");
    storage.store_episode(&episode).await.unwrap();
    let before = storage.stored_schema_version().await.unwrap();

    storage.migrate_schema().await.unwrap();

    assert_eq!(storage.stored_schema_version().await.unwrap(), before);
    assert!(
        storage
            .get_episode(episode.episode_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn test_migrate_schema_adopts_patterns_and_heuristics() {
    let (_dir, db_path) = temp_db("adopt.redb");
    let pattern = sample_pattern();
    let heuristic = Heuristic::new("cache miss".to_string(), "warm cache".to_string(), 0.75);

    {
        let storage = RedbStorage::new(&db_path).await.unwrap();
        let (episode, _) = sample_episode("adopt me");
        storage.store_episode(&episode).await.unwrap();
        storage.store_pattern(&pattern).await.unwrap();
        storage.store_heuristic(&heuristic).await.unwrap();
    }
    write_version(&db_path, 1);

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    storage.migrate_schema().await.unwrap();
    assert!(storage.stored_schema_version().await.unwrap().unwrap() > 1);
    drop(storage);

    // Every row survived and is readable through a normal open.
    let reopened = RedbStorage::new(&db_path).await.unwrap();
    assert!(reopened.get_pattern(pattern.id()).await.unwrap().is_some());
    assert!(
        reopened
            .get_heuristic(heuristic.heuristic_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn test_migrate_schema_rejects_undecodable_pattern_row() {
    let corrupt = vec![0xfe_u8; 6];
    migrate_rejects_corrupt_row("patterns", corrupt).await;
}

#[tokio::test]
async fn test_migrate_schema_rejects_undecodable_heuristic_row() {
    let corrupt = vec![0xfd_u8; 6];
    migrate_rejects_corrupt_row("heuristics", corrupt).await;
}

/// A database holding a valid episode plus one corrupt row in `table` must not
/// be migrated: the typed error is returned and the file is left untouched.
async fn migrate_rejects_corrupt_row(table: &str, corrupt: Vec<u8>) {
    let (_dir, db_path) = temp_db("corrupt.redb");
    let (episode, bytes) = sample_episode("valid episode");
    let key = episode.episode_id.to_string();
    write_bytes(&db_path, "episodes", &[(key.as_str(), bytes.clone())]);
    write_bytes(&db_path, table, &[("corrupt-key", corrupt.clone())]);
    write_version(&db_path, 1);

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    let err = storage
        .migrate_schema()
        .await
        .expect_err("corrupt rows must not migrate");
    assert_migration_error(&err, Some(1));
    drop(storage);

    assert_eq!(
        read_bytes(&db_path, table, "corrupt-key"),
        Some(corrupt),
        "corrupt row must be preserved for external tooling"
    );
    assert_eq!(read_bytes(&db_path, "episodes", key.as_str()), Some(bytes));
    assert_eq!(read_version(&db_path), Some(1));
}

#[tokio::test]
async fn test_rebuild_indexes_reconstructs_from_sessions() {
    let (_dir, db_path) = temp_db("index-rebuild.redb");
    let first = sample_session(Uuid::new_v4());
    let second = sample_session(Uuid::new_v4());

    {
        let storage = RedbStorage::new(&db_path).await.unwrap();
        storage.store_recommendation_session(&first).await.unwrap();
        storage.store_recommendation_session(&second).await.unwrap();
    }

    // Corrupt the derived index; the rebuild must recreate it from sessions.
    write_str_rows(
        &db_path,
        "recommendation_episode_index",
        &[("stale", "stale")],
    );

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    storage.rebuild_indexes().await.unwrap();
    // Idempotent: a second rebuild produces the same mapping.
    storage.rebuild_indexes().await.unwrap();
    drop(storage);

    assert_eq!(
        read_str(
            &db_path,
            "recommendation_episode_index",
            &first.episode_id.to_string()
        ),
        Some(first.session_id.to_string())
    );
    assert_eq!(
        read_str(
            &db_path,
            "recommendation_episode_index",
            &second.episode_id.to_string()
        ),
        Some(second.session_id.to_string())
    );
    assert_eq!(
        read_str(&db_path, "recommendation_episode_index", "stale"),
        None,
        "stale index entries must be dropped"
    );

    // Sessions themselves are untouched by the index rebuild.
    let reopened = RedbStorage::new(&db_path).await.unwrap();
    assert!(
        reopened
            .get_recommendation_session(first.session_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn test_wrong_type_data_table_is_reported() {
    let (_dir, db_path) = temp_db("wrong-type-data.redb");
    write_u64_rows(&db_path, "episodes", &[("k", 1)]);

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("a mistyped table must surface as an error");
    assert!(
        matches!(&err, Error::Storage(message) if message.contains("episodes")),
        "expected a Storage error naming episodes, got {err:?}"
    );
    assert_eq!(read_u64(&db_path, "episodes", "k"), Some(1));
}

#[tokio::test]
async fn test_wrong_type_schema_version_table_is_reported() {
    let (_dir, db_path) = temp_db("wrong-type-version.redb");
    write_str_rows(&db_path, "schema_version", &[("version", "1")]);

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("a mistyped schema_version table must surface as an error");
    assert!(
        matches!(&err, Error::Storage(message) if message.contains("schema version")),
        "expected a Storage error naming the schema version table, got {err:?}"
    );
}

#[tokio::test]
async fn test_wrong_type_recommendation_index_is_reported() {
    let (_dir, db_path) = temp_db("wrong-type-index.redb");
    write_bytes(
        &db_path,
        "recommendation_episode_index",
        &[("k", vec![1, 2])],
    );

    let err = RedbStorage::new(&db_path)
        .await
        .err()
        .expect("a mistyped recommendation index must surface as an error");
    assert!(
        matches!(&err, Error::Storage(message) if message.contains("recommendation_episode_index")),
        "expected a Storage error naming the recommendation index, got {err:?}"
    );
}

#[tokio::test]
async fn test_rebuild_indexes_surfaces_undecodable_session() {
    let (_dir, db_path) = temp_db("bad-session.redb");
    let corrupt = vec![0xfa_u8; 5];
    write_bytes(
        &db_path,
        "recommendation_sessions",
        &[("session-key", corrupt.clone())],
    );

    let storage = RedbStorage::open_unchecked(&db_path).await.unwrap();
    let err = storage
        .rebuild_indexes()
        .await
        .expect_err("a corrupt session row must fail the rebuild");
    assert!(
        matches!(&err, Error::Storage(message) if message.contains("recommendation session")),
        "expected a Storage error naming the session row, got {err:?}"
    );
    drop(storage);

    // The source row is untouched: the rebuild never clears primary data.
    assert_eq!(
        read_bytes(&db_path, "recommendation_sessions", "session-key"),
        Some(corrupt)
    );
}
