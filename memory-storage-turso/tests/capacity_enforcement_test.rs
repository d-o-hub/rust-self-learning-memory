//! Integration tests for capacity-constrained storage
//!
//! NOTE: the legacy tests that build storage through `TursoStorage::new` are
//! ignored due to a memory corruption bug in the libsql/turso native library
//! that causes `malloc_consolidate(): unaligned fastbin chunk detected` in CI
//! environments. See ADR-027 for details. The issue-#1070 eviction-cleanup
//! tests below use the pool-free `TursoStorage::new_local` path (the same
//! pattern as `capability_attribution_test.rs`) so their failure-injection
//! assertions actually run.

#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unreadable_literal)]

use do_memory_core::semantic::{EpisodeSummary, SemanticSummarizer};
use do_memory_core::{
    Episode, EvictionBackend, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};
use do_memory_storage_turso::TursoStorage;
use tempfile::TempDir;

/// Helper to create a test storage instance
async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await?;

    storage.initialize_schema().await?;

    Ok((storage, dir))
}

/// Helper to create a test episode with specific quality
fn create_test_episode(task_desc: &str, _quality_score: f32) -> Episode {
    let mut episode = Episode::new(
        task_desc.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    // Add a step to make it more realistic
    let mut step = ExecutionStep::new(1, "tester".to_string(), "Run tests".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Tests passed".to_string(),
    });
    episode.add_step(step);

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "Task completed".to_string(),
        artifacts: vec![format!("{}.rs", task_desc)],
    });

    episode
}

/// Helper to create a test episode with a specific start time offset (for deterministic ordering)
fn create_test_episode_with_offset(task_desc: &str, seconds_offset: i64) -> Episode {
    use chrono::{Duration, Utc};

    let mut episode = Episode::new(
        task_desc.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    // Set start_time with explicit offset for deterministic ordering
    episode.start_time = Utc::now() - Duration::seconds(seconds_offset);

    // Add a step to make it more realistic
    let mut step = ExecutionStep::new(1, "tester".to_string(), "Run tests".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Tests passed".to_string(),
    });
    episode.add_step(step);

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "Task completed".to_string(),
        artifacts: vec![format!("{}.rs", task_desc)],
    });

    episode
}

/// Helper to create episode summary
async fn create_test_summary(episode: &Episode) -> anyhow::Result<EpisodeSummary> {
    let summarizer = SemanticSummarizer::new();
    summarizer.summarize_episode(episode).await
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_store_and_retrieve_episode_summary() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("test_task", 0.8);
    let summary = create_test_summary(&episode).await?;

    // Store the episode first
    storage.store_episode(&episode).await?;

    // Store the summary
    storage.store_summary(summary.episode_id, &summary).await?;

    // Retrieve the summary
    let retrieved = storage
        .get_summary(episode.episode_id)
        .await?
        .ok_or("Summary not found")?;

    assert_eq!(retrieved.episode_id, summary.episode_id);
    assert_eq!(retrieved.summary_text, summary.summary_text);
    assert_eq!(retrieved.key_concepts, summary.key_concepts);
    assert_eq!(retrieved.key_steps, summary.key_steps);
    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_capacity_enforcement_lru() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let max_episodes = 3;

    // Store 3 episodes at capacity with explicit time ordering
    // task_0 is oldest (30 seconds ago), task_1 is next (20 seconds ago), etc.
    let episode0 = create_test_episode_with_offset("task_0", 30);
    let task_0_id = episode0.episode_id;
    let episode1 = create_test_episode_with_offset("task_1", 20);
    let episode2 = create_test_episode_with_offset("task_2", 10);

    storage.store_episode(&episode0).await?;
    storage.store_episode(&episode1).await?;
    storage.store_episode(&episode2).await?;

    // Verify we have 3 episodes
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 3);

    // Store 4th episode - should evict the oldest (task_0)
    let episode4 = create_test_episode("task_3", 0.5);
    storage
        .store_episode_with_capacity(&episode4, max_episodes)
        .await?;

    // Verify still at capacity (oldest evicted)
    let final_count = storage.get_statistics().await?.episode_count;
    assert_eq!(final_count, max_episodes);

    // Verify task_0 was evicted
    let result = storage.get_episode(task_0_id).await?;
    assert!(result.is_none(), "Oldest episode should be evicted");

    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_capacity_enforcement_basic() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Store 5 episodes with capacity of 3
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        storage.store_episode_with_capacity(&episode, 3).await?;
    }

    // Verify only 3 episodes remain
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 3);

    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_summary_cascade_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("test_cascade", 0.7);
    let summary = create_test_summary(&episode).await?;
    let episode_id = episode.episode_id;

    // Store episode and summary
    storage.store_episode(&episode).await?;
    storage.store_summary(summary.episode_id, &summary).await?;

    // Verify summary exists
    let retrieved = storage.get_summary(episode_id).await?;
    assert!(retrieved.is_some());

    // Delete episode using delete_episode
    storage.delete_episode(episode_id).await?;

    // Verify summary was cascade deleted (manually check - depends on FK constraint)
    let after_delete = storage.get_summary(episode_id).await?;
    assert!(after_delete.is_none(), "Summary should be deleted");
    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_capacity_count_accuracy() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Perform multiple insert/evict cycles
    for i in 0..15 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);

        storage.store_episode_with_capacity(&episode, 5).await?;

        // Check count after each operation
        let count = storage.get_statistics().await?.episode_count;

        // Should never exceed capacity
        assert!(
            count <= 5,
            "Episode count {} exceeds capacity 5 at iteration {}",
            count,
            i
        );

        // After capacity is reached, should stay at capacity
        if i >= 5 {
            assert_eq!(
                count, 5,
                "Episode count should be exactly at capacity after iteration {}",
                i
            );
        }
    }
    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_batch_eviction() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Fill to capacity
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);
        episode_ids.push(episode.episode_id);
        storage.store_episode_with_capacity(&episode, 10).await?;
    }

    // Manually evict first 3 episodes
    for id in &episode_ids[0..3] {
        storage.delete_episode(*id).await?;
    }

    // Verify count
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 2);

    // Verify evicted episodes are gone
    for id in &episode_ids[0..3] {
        let result = storage.get_episode(*id).await?;
        assert!(result.is_none(), "Episode should be deleted");
    }

    // Verify remaining episodes still exist
    for id in &episode_ids[3..5] {
        let result = storage.get_episode(*id).await?;
        assert!(result.is_some(), "Episode should still exist");
    }
    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_no_eviction_under_capacity() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    // Store only 5 episodes (under capacity of 10)
    for i in 0..5 {
        let episode = create_test_episode(&format!("task_{}", i), 0.5);

        storage.store_episode_with_capacity(&episode, 10).await?;
    }

    // Verify all 5 episodes stored
    let count = storage.get_statistics().await?.episode_count;
    assert_eq!(count, 5);
    Ok(())
}

#[tokio::test]
#[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
async fn test_summary_without_embedding() -> Result<(), Box<dyn std::error::Error>> {
    let (storage, _dir) = create_test_storage().await?;

    let episode = create_test_episode("no_embedding", 0.6);
    let mut summary = create_test_summary(&episode).await?;

    // Explicitly remove embedding
    summary.summary_embedding = None;

    // Store episode and summary
    storage.store_episode(&episode).await?;
    storage.store_summary(summary.episode_id, &summary).await?;

    // Retrieve and verify
    let retrieved = storage
        .get_summary(episode.episode_id)
        .await?
        .ok_or("Summary not found")?;

    assert_eq!(retrieved.summary_embedding, None);
    assert_eq!(retrieved.summary_text, summary.summary_text);
    Ok(())
}

// ============================================================================
// Issue #1070: error-aware, repairable capacity eviction cleanup
//
// These tests use the pool-free `TursoStorage::new_local` path so they run in
// CI (see `capability_attribution_test.rs`); the libsql memory-corruption bug
// that forces the `#[ignore]`s above is tied to the pooled `TursoStorage::new`
// path.
// ============================================================================

/// Build a pool-free local-file storage (safe to run in CI).
async fn create_local_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("capacity_local.db");
    let storage = TursoStorage::new_local(&db_path).await?;
    storage.initialize_schema().await?;
    Ok((storage, dir))
}

/// Run a parameter-less statement on a fresh connection.
async fn exec_sql(storage: &TursoStorage, sql: &str) -> anyhow::Result<()> {
    let conn = storage.get_connection().await?;
    conn.execute(sql, ()).await?;
    Ok(())
}

/// `SELECT COUNT(*)` helper for the embedding tables.
async fn count_rows(storage: &TursoStorage, sql: &str) -> anyhow::Result<i64> {
    let conn = storage.get_connection().await?;
    let mut rows = conn.query(sql, ()).await?;
    if let Some(row) = rows.next().await? {
        Ok(row.get(0)?)
    } else {
        Ok(0)
    }
}

/// Every embedding table that exists for this feature set (legacy + dimension).
async fn embedding_table_names(storage: &TursoStorage) -> anyhow::Result<Vec<String>> {
    let conn = storage.get_connection().await?;
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' \
               AND (name = 'embeddings' OR name LIKE 'embeddings\\_%' ESCAPE '\\')",
            (),
        )
        .await?;
    let mut names = Vec::new();
    while let Some(row) = rows.next().await? {
        let name: String = row.get(0)?;
        names.push(name);
    }
    Ok(names)
}

const INJECTED_TRIGGER_PREFIX: &str = "w1070_inject_";

/// Count embedding rows for `item_id` across every embedding table family.
///
/// Used instead of `get_embedding_backend`, which under `turso_multi_dimension`
/// still queries the legacy `embeddings` table (a pre-existing gap outside this
/// issue's scope).
async fn embedding_row_count(storage: &TursoStorage, item_id: &str) -> anyhow::Result<i64> {
    let mut total = 0;
    for table in embedding_table_names(storage).await? {
        total += count_rows(
            storage,
            &format!("SELECT COUNT(*) FROM {table} WHERE item_id = '{item_id}'"),
        )
        .await?;
    }
    Ok(total)
}

/// Make every embedding delete fail (deterministic real-DB injection).
async fn install_embedding_delete_triggers(storage: &TursoStorage) -> anyhow::Result<()> {
    for table in embedding_table_names(storage).await? {
        let sql = format!(
            "CREATE TRIGGER IF NOT EXISTS {INJECTED_TRIGGER_PREFIX}emb_{table} \
             BEFORE DELETE ON {table} \
             BEGIN SELECT RAISE(ABORT, 'injected embedding delete failure'); END"
        );
        exec_sql(storage, &sql).await?;
    }
    Ok(())
}

/// Make the episode delete fail (deterministic real-DB injection).
async fn install_episode_delete_trigger(storage: &TursoStorage) -> anyhow::Result<()> {
    exec_sql(
        storage,
        "CREATE TRIGGER IF NOT EXISTS w1070_inject_episode \
         BEFORE DELETE ON episodes \
         BEGIN SELECT RAISE(ABORT, 'injected episode delete failure'); END",
    )
    .await
}

/// Remove every injected trigger so a retry can succeed.
async fn drop_injected_triggers(storage: &TursoStorage) -> anyhow::Result<()> {
    let names = {
        let conn = storage.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT name FROM sqlite_master \
                 WHERE type = 'trigger' AND name LIKE 'w1070_inject_%'",
                (),
            )
            .await?;
        let mut names = Vec::new();
        while let Some(row) = rows.next().await? {
            let name: String = row.get(0)?;
            names.push(name);
        }
        drop(rows);
        drop(conn);
        names
    };
    for name in names {
        exec_sql(storage, &format!("DROP TRIGGER IF EXISTS {name}")).await?;
    }
    Ok(())
}

/// Store `count` episodes (oldest first) each with an embedding.
async fn seed_episodes_with_embeddings(
    storage: &TursoStorage,
    count: i64,
) -> anyhow::Result<Vec<uuid::Uuid>> {
    let mut ids = Vec::new();
    for i in 0..count {
        let episode = create_test_episode_with_offset(&format!("task_{i}"), 10 * (count - i));
        storage.store_episode(&episode).await?;
        storage
            .store_embedding_backend(&episode.episode_id.to_string(), vec![0.1f32; 384])
            .await?;
        ids.push(episode.episode_id);
    }
    Ok(ids)
}

#[tokio::test]
async fn test_capacity_eviction_atomic_cleanup_purges_embeddings()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let (storage, _dir) = create_local_storage().await?;
    let ids = seed_episodes_with_embeddings(&storage, 3).await?;

    // Act
    let outcome = storage.enforce_capacity_with_outcome(2).await?;

    // Assert: one episode evicted, its embedding gone, no failures recorded.
    assert!(
        outcome.is_complete(),
        "atomic cleanup must report no failures"
    );
    assert_eq!(outcome.evicted_ids.len(), 1);
    assert_eq!(storage.get_statistics().await?.episode_count, 2);
    assert_eq!(
        embedding_row_count(&storage, &ids[0].to_string()).await?,
        0,
        "evicted episode embedding must be deleted"
    );
    assert_eq!(
        storage.pending_capacity_eviction_intents().await?.len(),
        0,
        "no pending eviction intents may remain"
    );
    Ok(())
}

#[tokio::test]
async fn test_embedding_delete_failure_records_retryable_intent()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let (storage, _dir) = create_local_storage().await?;
    let ids = seed_episodes_with_embeddings(&storage, 3).await?;
    install_embedding_delete_triggers(&storage).await?;

    // Act: the dependent embedding delete fails.
    let outcome = storage.enforce_capacity_with_outcome(2).await?;

    // Assert: explicit partial result attributed to the embedding delete.
    assert!(outcome.needs_reconciliation());
    assert!(
        outcome
            .failures
            .iter()
            .all(|f| f.backend == EvictionBackend::EmbeddingDurable),
        "embedding failure must be attributed to EmbeddingDurable: {:?}",
        outcome.failures
    );

    // Rollback keeps episode and embedding mutually consistent.
    assert_eq!(storage.get_statistics().await?.episode_count, 3);
    assert_eq!(
        embedding_row_count(&storage, &ids[0].to_string()).await?,
        1,
        "rolled-back eviction must leave the embedding in place"
    );

    // A retryable intent is durably recorded without losing the id list.
    let intents = storage.pending_capacity_eviction_intents().await?;
    assert_eq!(intents.len(), 1, "one episode needs reconciliation");
    assert_eq!(intents[0].episode_id, ids[0]);
    assert_eq!(intents[0].backend, EvictionBackend::EmbeddingDurable);
    assert_ne!(intents[0].error, "");

    // `enforce_capacity` never reports success after a swallowed dependent delete.
    assert!(storage.enforce_capacity(2).await.is_err());

    // Repair: once the injection is gone, retry reconciles and clears intents.
    drop_injected_triggers(&storage).await?;
    let retry = storage.retry_pending_capacity_evictions().await?;
    assert!(retry.is_complete());
    assert_eq!(storage.get_statistics().await?.episode_count, 2);
    assert_eq!(embedding_row_count(&storage, &ids[0].to_string()).await?, 0);
    assert_eq!(
        storage.pending_capacity_eviction_intents().await?.len(),
        0,
        "no pending eviction intents may remain"
    );
    Ok(())
}

#[tokio::test]
async fn test_episode_delete_failure_is_observable_and_repairable()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let (storage, _dir) = create_local_storage().await?;
    let ids = seed_episodes_with_embeddings(&storage, 3).await?;
    install_episode_delete_trigger(&storage).await?;

    // Act: the episode delete fails after the embedding deletes succeeded.
    let outcome = storage.enforce_capacity_with_outcome(2).await?;

    // Assert: attributed to the durable episode surface.
    assert!(outcome.needs_reconciliation());
    assert!(
        outcome
            .failures
            .iter()
            .all(|f| f.backend == EvictionBackend::Durable),
        "episode failure must be attributed to Durable: {:?}",
        outcome.failures
    );

    // The whole transaction rolled back: episode still present and its
    // embedding delete was undone with it.
    assert_eq!(storage.get_statistics().await?.episode_count, 3);
    assert_eq!(
        embedding_row_count(&storage, &ids[0].to_string()).await?,
        1,
        "embedding delete must be rolled back together with the failed episode delete"
    );

    // The id list survives in the outbox, so it is repairable.
    let intents = storage.pending_capacity_eviction_intents().await?;
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].episode_id, ids[0]);

    drop_injected_triggers(&storage).await?;
    let retry = storage.retry_pending_capacity_evictions().await?;
    assert!(retry.is_complete());
    assert_eq!(storage.get_statistics().await?.episode_count, 2);
    assert_eq!(embedding_row_count(&storage, &ids[0].to_string()).await?, 0);
    assert_eq!(
        storage.pending_capacity_eviction_intents().await?.len(),
        0,
        "no pending eviction intents may remain"
    );
    Ok(())
}

#[tokio::test]
async fn test_cleanup_covers_legacy_and_dimension_embedding_tables()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange: make sure both embedding table families exist, whichever feature
    // set this build uses.
    let (storage, _dir) = create_local_storage().await?;
    exec_sql(
        &storage,
        "CREATE TABLE IF NOT EXISTS embeddings (\
             embedding_id TEXT PRIMARY KEY, item_id TEXT NOT NULL, item_type TEXT NOT NULL, \
             embedding_data TEXT NOT NULL, embedding_vector BLOB, dimension INTEGER NOT NULL, \
             model TEXT NOT NULL, created_at INTEGER)",
    )
    .await?;
    exec_sql(
        &storage,
        "CREATE TABLE IF NOT EXISTS embeddings_384 (\
             embedding_id TEXT PRIMARY KEY, item_id TEXT NOT NULL, item_type TEXT NOT NULL, \
             embedding_data TEXT NOT NULL, embedding_vector BLOB, dimension INTEGER NOT NULL DEFAULT 384, \
             model TEXT NOT NULL, created_at INTEGER)",
    )
    .await?;

    let ids = seed_episodes_with_embeddings(&storage, 3).await?;
    // Drop the API-stored embeddings so each episode has exactly one row per
    // family, then seed both families directly.
    exec_sql(&storage, "DELETE FROM embeddings").await?;
    exec_sql(&storage, "DELETE FROM embeddings_384").await?;
    for id in &ids {
        exec_sql(
            &storage,
            &format!(
                "INSERT INTO embeddings \
                 (embedding_id, item_id, item_type, embedding_data, dimension, model) \
                 VALUES ('legacy_{id}', '{id}', 'embedding', '[]', 384, 'test')"
            ),
        )
        .await?;
        exec_sql(
            &storage,
            &format!(
                "INSERT INTO embeddings_384 \
                 (embedding_id, item_id, item_type, embedding_data, dimension, model) \
                 VALUES ('dim_{id}', '{id}', 'embedding', '[]', 384, 'test')"
            ),
        )
        .await?;
    }

    // Act
    let outcome = storage.enforce_capacity_with_outcome(2).await?;

    // Assert: the oldest episode's rows are gone from both families.
    assert!(outcome.is_complete());
    let evicted = ids[0];
    assert_eq!(
        count_rows(
            &storage,
            &format!("SELECT COUNT(*) FROM embeddings WHERE item_id = '{evicted}'")
        )
        .await?,
        0,
        "legacy embeddings table must be purged"
    );
    assert_eq!(
        count_rows(
            &storage,
            &format!("SELECT COUNT(*) FROM embeddings_384 WHERE item_id = '{evicted}'")
        )
        .await?,
        0,
        "turso_multi_dimension embeddings_384 table must be purged"
    );

    // Rows for the two kept episodes survive in both families.
    assert_eq!(
        count_rows(&storage, "SELECT COUNT(*) FROM embeddings").await?,
        2
    );
    assert_eq!(
        count_rows(&storage, "SELECT COUNT(*) FROM embeddings_384").await?,
        2
    );
    Ok(())
}
