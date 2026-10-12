//! Durable capacity-eviction cleanup intents (issue #1070 outbox).
//!
//! `enforce_capacity` writes one intent per episode before deleting anything,
//! so a failed dependent-embedding or episode delete can never lose the id
//! list. Keeping the intent persistence in its own module keeps `capacity.rs`
//! under the 500-LOC ceiling.

use crate::{Result, TursoStorage};
use do_memory_core::{EvictionBackend, EvictionBackendFailure, EvictionOutcome};
use libsql;
use tracing::info;
use uuid::Uuid;

use super::capacity_cleanup::run_capacity_cleanup;

/// SQL to create the durable capacity-eviction cleanup intent (outbox) table.
///
/// `enforce_capacity` writes one row per episode it is about to evict *before*
/// deleting anything, so a failed dependent-embedding or episode delete can
/// never lose the id list: the row survives (or is re-created on the next
/// attempt) and `retry_pending_capacity_evictions` can replay the cleanup.
///
/// Kept next to the code that owns the table so `schema/mod.rs` stays within
/// the 500-LOC ceiling.
pub const CREATE_CAPACITY_EVICTION_INTENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS capacity_eviction_intents (
    episode_id TEXT PRIMARY KEY NOT NULL,
    backend TEXT NOT NULL DEFAULT 'durable',
    error TEXT NOT NULL DEFAULT '',
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// A durable, retryable capacity-eviction cleanup intent.
///
/// One row is written to `capacity_eviction_intents` *before* an episode is
/// deleted, so a failed dependent-embedding or episode delete can be retried
/// via [`TursoStorage::retry_pending_capacity_evictions`] without losing the id
/// list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacityEvictionIntent {
    /// Episode that still needs its embeddings (and possibly its row) deleted.
    pub episode_id: Uuid,
    /// Backend surface that failed during the last attempt.
    pub backend: EvictionBackend,
    /// Error recorded by the last attempt (empty while still pending).
    pub error: String,
    /// Number of failed attempts so far.
    pub attempts: u32,
}

/// Parse a stored episode id, falling back to the nil UUID for malformed rows.
pub(crate) fn parse_episode_id(id: &str) -> Uuid {
    Uuid::parse_str(id).unwrap_or_default()
}

/// Reverse of [`EvictionBackend`]'s `Display`, used to read persisted intents.
fn eviction_backend_from_str(value: &str) -> EvictionBackend {
    match value {
        "cache" => EvictionBackend::Cache,
        "embedding_cache" => EvictionBackend::EmbeddingCache,
        "embedding_durable" => EvictionBackend::EmbeddingDurable,
        _ => EvictionBackend::Durable,
    }
}

impl TursoStorage {
    /// Durable cleanup intents still awaiting reconciliation.
    pub async fn pending_capacity_eviction_intents(&self) -> Result<Vec<CapacityEvictionIntent>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = "SELECT episode_id, backend, error, attempts \
                           FROM capacity_eviction_intents \
                           ORDER BY created_at ASC, episode_id ASC";

        let mut rows = conn.query(SQL, ()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!(
                "Failed to list capacity eviction intents: {}",
                e
            ))
        })?;

        let mut intents = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?
        {
            let episode_id: String = row
                .get(0)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            let backend: String = row
                .get(1)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            let error: String = row
                .get(2)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            let attempts: i64 = row
                .get(3)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            intents.push(CapacityEvictionIntent {
                episode_id: parse_episode_id(&episode_id),
                backend: eviction_backend_from_str(&backend),
                error,
                attempts: u32::try_from(attempts.max(0)).unwrap_or(u32::MAX),
            });
        }
        Ok(intents)
    }

    /// Retry the cleanup for every pending capacity-eviction intent.
    ///
    /// Returns the structured outcome. On success the durably recorded intents
    /// are cleared; on failure they are kept (with an incremented attempt count
    /// and the new error) so reconciliation can be retried again.
    pub async fn retry_pending_capacity_evictions(&self) -> Result<EvictionOutcome> {
        let intents = self.pending_capacity_eviction_intents().await?;
        if intents.is_empty() {
            return Ok(EvictionOutcome::default());
        }

        let mut episode_ids: Vec<String> = Vec::new();
        for intent in &intents {
            let id = intent.episode_id.to_string();
            if !episode_ids.contains(&id) {
                episode_ids.push(id);
            }
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let cleanup = run_capacity_cleanup(&conn, &episode_ids).await;
        drop(conn);

        match cleanup {
            Ok(()) => {
                self.clear_capacity_eviction_intents(&episode_ids).await?;
                info!(
                    "Reconciled {} pending capacity evictions",
                    episode_ids.len()
                );
                Ok(EvictionOutcome {
                    evicted_ids: episode_ids.iter().map(|id| parse_episode_id(id)).collect(),
                    failures: Vec::new(),
                })
            }
            Err(failure) => {
                let backend = failure.stage.backend();
                self.mark_capacity_eviction_intents_failed(&episode_ids, backend, &failure.error)
                    .await?;
                let failures = episode_ids
                    .iter()
                    .map(|id| EvictionBackendFailure {
                        episode_id: parse_episode_id(id),
                        backend,
                        error: failure.error.clone(),
                    })
                    .collect();
                Ok(EvictionOutcome {
                    evicted_ids: Vec::new(),
                    failures,
                })
            }
        }
    }

    /// Persist a pending cleanup intent per episode before any deletion.
    pub(crate) async fn record_pending_capacity_eviction_intents(
        &self,
        episode_ids: &[String],
    ) -> Result<()> {
        if episode_ids.is_empty() {
            return Ok(());
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        const SQL: &str = "INSERT INTO capacity_eviction_intents \
             (episode_id, backend, error, attempts, created_at, updated_at) \
             VALUES (?, 'durable', '', 0, strftime('%s', 'now'), strftime('%s', 'now')) \
             ON CONFLICT(episode_id) DO UPDATE SET updated_at = excluded.updated_at";

        for id in episode_ids {
            conn.execute(SQL, libsql::params![id.clone()])
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!(
                        "Failed to record capacity eviction intent for {id}: {e}"
                    ))
                })?;
        }
        Ok(())
    }

    /// Update the persisted intents with the failing backend and error.
    pub(crate) async fn mark_capacity_eviction_intents_failed(
        &self,
        episode_ids: &[String],
        backend: EvictionBackend,
        error: &str,
    ) -> Result<()> {
        if episode_ids.is_empty() {
            return Ok(());
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        const SQL: &str = "UPDATE capacity_eviction_intents \
             SET backend = ?, error = ?, attempts = attempts + 1, \
                 updated_at = strftime('%s', 'now') \
             WHERE episode_id = ?";

        for id in episode_ids {
            conn.execute(
                SQL,
                libsql::params![backend.to_string(), error.to_string(), id.clone()],
            )
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!(
                    "Failed to mark capacity eviction intent for {id} as failed: {e}"
                ))
            })?;
        }
        Ok(())
    }

    /// Clear the persisted intents after a successful cleanup.
    pub(crate) async fn clear_capacity_eviction_intents(
        &self,
        episode_ids: &[String],
    ) -> Result<()> {
        if episode_ids.is_empty() {
            return Ok(());
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let placeholders = episode_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql =
            format!("DELETE FROM capacity_eviction_intents WHERE episode_id IN ({placeholders})");
        let params: Vec<libsql::Value> = episode_ids.iter().map(|id| id.clone().into()).collect();

        conn.execute(&sql, libsql::params_from_iter(params))
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!(
                    "Failed to clear capacity eviction intents: {e}"
                ))
            })?;
        Ok(())
    }
}
