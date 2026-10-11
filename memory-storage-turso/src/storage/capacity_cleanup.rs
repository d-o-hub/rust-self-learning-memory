//! Atomic-or-repairable capacity-eviction cleanup (issue #1070).
//!
//! `enforce_capacity` used to delete embeddings and then episodes as two
//! independent statements, discarding the embedding result. A failed embedding
//! delete could therefore be hidden while the episode was removed, orphaning
//! vectors. This module runs both deletes inside one transaction and reports
//! *which* stage failed, so the caller can either observe an atomic rollback
//! (episode + embeddings stay consistent) or record a durable retry intent.
//!
//! The cleanup is expressed against [`CleanupConnection`] so the failure paths
//! can be unit tested with a deterministic fake - the native libSQL path is not
//! needed to prove that a failed embedding delete rolls back before the episode
//! delete runs, or that a failed episode delete never commits.

use crate::{Error, Result};
use async_trait::async_trait;
use do_memory_core::EvictionBackend;
use libsql;
use tracing::warn;

/// Stage of the eviction cleanup that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CleanupStage {
    /// The surrounding transaction could not be started or committed.
    Transaction,
    /// Deleting dependent embedding rows (legacy or dimension tables).
    Embeddings,
    /// Deleting the evicted episode rows.
    Episodes,
}

impl CleanupStage {
    /// Map the failed stage onto the shared eviction failure vocabulary.
    #[must_use]
    pub(crate) fn backend(self) -> EvictionBackend {
        match self {
            Self::Embeddings => EvictionBackend::EmbeddingDurable,
            Self::Transaction | Self::Episodes => EvictionBackend::Durable,
        }
    }
}

/// A failure in one stage of the eviction cleanup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanupFailure {
    /// Which stage failed.
    pub(crate) stage: CleanupStage,
    /// Human-readable error (no secrets expected).
    pub(crate) error: String,
}

/// Minimal connection capability the atomic eviction cleanup needs.
///
/// Deliberately narrow so the transaction/rollback behaviour is unit testable
/// against a fake without the native libSQL path.
#[async_trait]
pub(crate) trait CleanupConnection: Send + Sync {
    /// Run a parameter-less control statement (`BEGIN TRANSACTION`, `COMMIT`, `ROLLBACK`).
    async fn execute_control(&self, sql: &str) -> Result<u64>;
    /// Run a parameterized statement.
    async fn execute_sql(&self, sql: &str, params: Vec<libsql::Value>) -> Result<u64>;
    /// Existing embedding tables (legacy `embeddings` plus every dimension table).
    async fn embedding_tables(&self) -> Result<Vec<String>>;
}

#[async_trait]
impl CleanupConnection for libsql::Connection {
    async fn execute_control(&self, sql: &str) -> Result<u64> {
        self.execute(sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to run '{sql}': {e}")))
    }

    async fn execute_sql(&self, sql: &str, params: Vec<libsql::Value>) -> Result<u64> {
        self.execute(sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| Error::Storage(e.to_string()))
    }

    async fn embedding_tables(&self) -> Result<Vec<String>> {
        // Covers both table families in one query: the legacy `embeddings`
        // table and any `embeddings_<dimension>` table (turso_multi_dimension),
        // regardless of which feature set created the database.
        const SQL: &str = "SELECT name FROM sqlite_master \
             WHERE type = 'table' \
               AND (name = 'embeddings' OR name LIKE 'embeddings\\_%' ESCAPE '\\') \
             ORDER BY name";

        let mut rows = self.query(SQL, ()).await.map_err(|e| {
            Error::Storage(format!("Failed to list embedding tables to evict: {e}"))
        })?;

        let mut tables = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read embedding table row: {e}")))?
        {
            let name: String = row.get(0).map_err(|e| {
                Error::Storage(format!("Failed to parse embedding table name: {e}"))
            })?;
            tables.push(name);
        }
        Ok(tables)
    }
}

/// Delete the dependent embedding rows and the episode rows for `episode_ids`
/// inside a single transaction.
///
/// * On success both deletes have committed.
/// * On any failure the transaction is rolled back, so the episode rows and
///   their embeddings stay mutually consistent and nothing is half-deleted.
/// * The returned [`CleanupFailure`] names the stage that failed so the caller
///   can record a durable retry intent with the right backend.
pub(crate) async fn run_capacity_cleanup(
    conn: &dyn CleanupConnection,
    episode_ids: &[String],
) -> std::result::Result<(), CleanupFailure> {
    if episode_ids.is_empty() {
        return Ok(());
    }

    // Resolve the embedding tables before opening the transaction so a lookup
    // failure cannot leave a transaction dangling.
    let tables = conn.embedding_tables().await.map_err(|e| CleanupFailure {
        stage: CleanupStage::Embeddings,
        error: format!("Failed to resolve embedding tables: {e}"),
    })?;

    conn.execute_control("BEGIN TRANSACTION")
        .await
        .map_err(|e| CleanupFailure {
            stage: CleanupStage::Transaction,
            error: format!("Failed to begin eviction transaction: {e}"),
        })?;

    match cleanup_body(conn, &tables, episode_ids).await {
        Ok(()) => match conn.execute_control("COMMIT").await {
            Ok(_) => Ok(()),
            Err(e) => {
                rollback(conn).await;
                Err(CleanupFailure {
                    stage: CleanupStage::Transaction,
                    error: format!("Failed to commit eviction transaction: {e}"),
                })
            }
        },
        Err(failure) => {
            rollback(conn).await;
            Err(failure)
        }
    }
}

/// The transactional body: embedding deletes first, episode delete last.
async fn cleanup_body(
    conn: &dyn CleanupConnection,
    tables: &[String],
    episode_ids: &[String],
) -> std::result::Result<(), CleanupFailure> {
    let placeholders = episode_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    for table in tables {
        // SAFETY: `table` comes from `sqlite_master` (schema-owned, never user
        // input), so it cannot inject SQL; identifiers cannot be parameterized.
        let sql = format!("DELETE FROM {table} WHERE item_id IN ({placeholders})");
        let params: Vec<libsql::Value> = episode_ids.iter().map(|id| id.clone().into()).collect();
        conn.execute_sql(&sql, params)
            .await
            .map_err(|e| CleanupFailure {
                stage: CleanupStage::Embeddings,
                error: format!("Failed to delete embeddings from {table}: {e}"),
            })?;
    }

    let sql = format!("DELETE FROM episodes WHERE episode_id IN ({placeholders})");
    let params: Vec<libsql::Value> = episode_ids.iter().map(|id| id.clone().into()).collect();
    conn.execute_sql(&sql, params)
        .await
        .map_err(|e| CleanupFailure {
            stage: CleanupStage::Episodes,
            error: format!("Failed to delete evicted episodes: {e}"),
        })?;

    Ok(())
}

/// Roll back the still-open transaction, logging (but never propagating) a
/// failure: the caller's own error is the actionable one.
async fn rollback(conn: &dyn CleanupConnection) {
    if let Err(e) = conn.execute_control("ROLLBACK").await {
        warn!("Failed to roll back eviction transaction: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, PoisonError};

    /// Deterministic fake: records every statement and can fail on a marker, so
    /// a mid-transaction failure is injected without any database.
    struct FakeConn {
        executed: Mutex<Vec<String>>,
        tables: Vec<String>,
        /// Substring that, when present in a statement, makes it fail.
        fail_on: Option<String>,
    }

    impl FakeConn {
        fn new(tables: &[&str]) -> Self {
            Self {
                executed: Mutex::new(Vec::new()),
                tables: tables.iter().map(|t| (*t).to_string()).collect(),
                fail_on: None,
            }
        }

        fn failing_on(tables: &[&str], marker: &str) -> Self {
            Self {
                fail_on: Some(marker.to_string()),
                ..Self::new(tables)
            }
        }

        fn recorded(&self) -> Vec<String> {
            self.executed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        }
    }

    #[async_trait]
    impl CleanupConnection for FakeConn {
        async fn execute_control(&self, sql: &str) -> Result<u64> {
            self.executed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(sql.to_string());
            if let Some(marker) = &self.fail_on {
                if sql.contains(marker) {
                    return Err(Error::Storage(format!("injected failure for '{sql}'")));
                }
            }
            Ok(1)
        }

        async fn execute_sql(&self, sql: &str, _params: Vec<libsql::Value>) -> Result<u64> {
            self.executed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(sql.to_string());
            if let Some(marker) = &self.fail_on {
                if sql.contains(marker) {
                    return Err(Error::Storage(format!("injected failure for '{sql}'")));
                }
            }
            Ok(1)
        }

        async fn embedding_tables(&self) -> Result<Vec<String>> {
            Ok(self.tables.clone())
        }
    }

    const IDS: &[&str] = &["ep-1", "ep-2"];

    fn ids() -> Vec<String> {
        IDS.iter().map(|s| (*s).to_string()).collect()
    }

    #[tokio::test]
    async fn cleanup_commits_embeddings_then_episodes() {
        let conn = FakeConn::new(&["embeddings", "embeddings_384"]);

        run_capacity_cleanup(&conn, &ids()).await.unwrap();

        assert_eq!(
            conn.recorded(),
            vec![
                "BEGIN TRANSACTION".to_string(),
                "DELETE FROM embeddings WHERE item_id IN (?,?)".to_string(),
                "DELETE FROM embeddings_384 WHERE item_id IN (?,?)".to_string(),
                "DELETE FROM episodes WHERE episode_id IN (?,?)".to_string(),
                "COMMIT".to_string(),
            ],
            "both table families purged before the episode delete, then committed"
        );
    }

    #[tokio::test]
    async fn cleanup_skips_everything_for_no_ids() {
        let conn = FakeConn::new(&["embeddings"]);

        run_capacity_cleanup(&conn, &[]).await.unwrap();

        assert!(
            conn.recorded().is_empty(),
            "no ids means no transaction is opened at all"
        );
    }

    #[tokio::test]
    async fn embedding_delete_failure_rolls_back_before_the_episode_delete() {
        // The second embedding table fails: the first delete must be rolled
        // back and the episode delete must never run, keeping episode and
        // embeddings consistent.
        let conn = FakeConn::failing_on(&["embeddings", "embeddings_384"], "embeddings_384");

        let failure = run_capacity_cleanup(&conn, &ids())
            .await
            .expect_err("injected embedding failure must surface");

        assert_eq!(failure.stage, CleanupStage::Embeddings);
        assert!(
            failure.error.contains("embeddings_384"),
            "{}",
            failure.error
        );
        assert_eq!(failure.stage.backend(), EvictionBackend::EmbeddingDurable);

        let recorded = conn.recorded();
        assert_eq!(
            recorded,
            vec![
                "BEGIN TRANSACTION".to_string(),
                "DELETE FROM embeddings WHERE item_id IN (?,?)".to_string(),
                "DELETE FROM embeddings_384 WHERE item_id IN (?,?)".to_string(),
                "ROLLBACK".to_string(),
            ]
        );
        assert!(
            !recorded.iter().any(|s| s.contains("DELETE FROM episodes")),
            "the episode delete must not run after an embedding failure"
        );
        assert!(
            !recorded.iter().any(|s| s == "COMMIT"),
            "a failed cleanup must never commit"
        );
    }

    #[tokio::test]
    async fn episode_delete_failure_rolls_back_the_embedding_deletes() {
        let conn = FakeConn::failing_on(&["embeddings"], "DELETE FROM episodes");

        let failure = run_capacity_cleanup(&conn, &ids())
            .await
            .expect_err("injected episode failure must surface");

        assert_eq!(failure.stage, CleanupStage::Episodes);
        assert_eq!(failure.stage.backend(), EvictionBackend::Durable);

        let recorded = conn.recorded();
        assert_eq!(
            recorded,
            vec![
                "BEGIN TRANSACTION".to_string(),
                "DELETE FROM embeddings WHERE item_id IN (?,?)".to_string(),
                "DELETE FROM episodes WHERE episode_id IN (?,?)".to_string(),
                "ROLLBACK".to_string(),
            ],
            "the embedding delete is undone together with the failed episode delete"
        );
        assert!(!recorded.iter().any(|s| s == "COMMIT"));
    }

    #[tokio::test]
    async fn begin_failure_is_attributed_to_the_transaction_stage() {
        let conn = FakeConn::failing_on(&["embeddings"], "BEGIN TRANSACTION");

        let failure = run_capacity_cleanup(&conn, &ids())
            .await
            .expect_err("begin failure must surface");

        assert_eq!(failure.stage, CleanupStage::Transaction);
        assert_eq!(failure.stage.backend(), EvictionBackend::Durable);
        assert_eq!(
            conn.recorded(),
            vec!["BEGIN TRANSACTION".to_string()],
            "nothing runs when BEGIN fails"
        );
    }
}
