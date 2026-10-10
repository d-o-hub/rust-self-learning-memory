//! Capacity-constrained storage operations for Turso

use crate::{Result, TursoStorage};
use do_memory_core::{Episode, EvictionBackendFailure, EvictionOutcome};
use libsql;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::capacity_cleanup::run_capacity_cleanup;
use super::capacity_intents::parse_episode_id;

impl TursoStorage {
    /// Store an episode with capacity management
    ///
    /// When the episode limit is reached, the least relevant episodes are evicted
    /// based on the configured eviction policy.
    pub async fn store_episode_with_capacity(
        &self,
        episode: &Episode,
        max_episodes: usize,
    ) -> Result<()> {
        debug!(
            "Storing episode with capacity management: {}, max_episodes={}",
            episode.episode_id, max_episodes
        );

        // First, store the episode
        self.store_episode(episode).await?;

        // Then, check if we need to evict episodes
        self.enforce_capacity(max_episodes).await?;

        Ok(())
    }

    /// Enforce the maximum episode capacity
    ///
    /// Uses the configured eviction policy to determine which episodes to remove
    /// when the capacity is exceeded.
    ///
    /// # Errors
    /// Returns an error when the eviction partially failed: the dependent
    /// embedding delete and the episode delete are atomic, so a failure rolls
    /// both back, records a retryable intent, and is reported here instead of
    /// being silently swallowed. Call
    /// [`Self::retry_pending_capacity_evictions`] to reconcile.
    pub async fn enforce_capacity(&self, max_episodes: usize) -> Result<()> {
        let outcome = self.enforce_capacity_with_outcome(max_episodes).await?;

        if outcome.needs_reconciliation() {
            return Err(do_memory_core::Error::Storage(format!(
                "Capacity eviction incomplete: {} dependent delete(s) failed and were recorded as \
                 retryable intents; call retry_pending_capacity_evictions to reconcile",
                outcome.failures.len()
            )));
        }

        Ok(())
    }

    /// Enforce capacity and return the structured eviction outcome (issue #1070).
    ///
    /// The dependent embedding rows (every legacy/dimension table that exists)
    /// and the episode rows are deleted in a single transaction. On failure the
    /// transaction rolls back - leaving episode and embeddings mutually
    /// consistent - and the episode ids are persisted as retryable intents
    /// before this returns an [`EvictionOutcome`] with the failures.
    pub async fn enforce_capacity_with_outcome(
        &self,
        max_episodes: usize,
    ) -> Result<EvictionOutcome> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Count current episodes
        const COUNT_SQL: &str = "SELECT COUNT(*) as count FROM episodes";

        let mut count_rows = conn.query(COUNT_SQL, ()).await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to count episodes: {}", e))
        })?;

        let current_count = if let Some(row) = count_rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            count as usize
        } else {
            0
        };
        drop(count_rows);

        if current_count <= max_episodes {
            return Ok(EvictionOutcome::default());
        }

        // Episodes exceed capacity - need to evict
        let to_remove = current_count - max_episodes;
        warn!(
            "Capacity exceeded: {} > {}, removing {} episodes",
            current_count, max_episodes, to_remove
        );

        // Get episodes to evict (oldest first, using LRU)
        // Order by start_time first, then by episode_id for deterministic tie-breaking
        let evict_sql =
            "SELECT episode_id FROM episodes ORDER BY start_time ASC, episode_id ASC LIMIT ?";

        let mut evict_rows = conn
            .query(evict_sql, libsql::params![to_remove as i64])
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to query episodes to evict: {}", e))
            })?;

        let mut evicted = Vec::new();
        while let Some(row) = evict_rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?
        {
            let episode_id: String = row
                .get(0)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
            evicted.push(episode_id);
        }

        // Drop the selection connection/rows before running the eviction
        // transaction to avoid "database locked" errors in parallel tests.
        drop(evict_rows);
        drop(conn);

        if evicted.is_empty() {
            return Ok(EvictionOutcome::default());
        }

        // Durable outbox BEFORE deletion: even if the process dies mid-cleanup
        // (the transaction auto-rolls back), the id list survives for retry.
        self.record_pending_capacity_eviction_intents(&evicted)
            .await?;

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let cleanup = run_capacity_cleanup(&conn, &evicted).await;
        drop(conn);

        match cleanup {
            Ok(()) => {
                if let Err(e) = self.clear_capacity_eviction_intents(&evicted).await {
                    // The deletes are committed and idempotent, so a leftover
                    // intent is replayed harmlessly on the next retry.
                    warn!(
                        "Capacity eviction succeeded but clearing its intents failed: {}",
                        e
                    );
                }
                info!(
                    "Evicted {} episodes to enforce capacity limit of {}",
                    evicted.len(),
                    max_episodes
                );
                Ok(EvictionOutcome {
                    evicted_ids: evicted.iter().map(|id| parse_episode_id(id)).collect(),
                    failures: Vec::new(),
                })
            }
            Err(failure) => {
                let backend = failure.stage.backend();
                if let Err(e) = self
                    .mark_capacity_eviction_intents_failed(&evicted, backend, &failure.error)
                    .await
                {
                    warn!(
                        "Failed to record capacity-eviction failure detail for {} episodes: {}",
                        evicted.len(),
                        e
                    );
                }
                warn!(
                    "Capacity eviction rolled back for {} episodes ({}): {}",
                    evicted.len(),
                    backend,
                    failure.error
                );
                let failures = evicted
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

    /// Get storage statistics including capacity info
    pub async fn get_capacity_statistics(&self) -> Result<CapacityStatistics> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Count records in each table
        let tables = [
            "episodes",
            "patterns",
            "heuristics",
            "embeddings",
            "execution_records",
            "agent_metrics",
            "task_metrics",
        ];

        let mut table_counts = HashMap::new();
        for table in tables {
            // SAFETY: Table names are hardcoded in the local `tables` array above.
            // These are not user-controlled values, preventing SQL injection.
            // CodeQL may flag this as a potential SQL injection, but it is a false positive.
            let sql = format!("SELECT COUNT(*) FROM {}", table);
            let mut rows = conn.query(&sql, ()).await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to count {}: {}", table, e))
            })?;

            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?
            {
                let count: i64 = row
                    .get(0)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                table_counts.insert(table.to_string(), count as usize);
            }
        }

        Ok(CapacityStatistics {
            episode_count: table_counts.get("episodes").copied().unwrap_or(0),
            pattern_count: table_counts.get("patterns").copied().unwrap_or(0),
            heuristic_count: table_counts.get("heuristics").copied().unwrap_or(0),
            embedding_count: table_counts.get("embeddings").copied().unwrap_or(0),
            execution_record_count: table_counts.get("execution_records").copied().unwrap_or(0),
            agent_metrics_count: table_counts.get("agent_metrics").copied().unwrap_or(0),
            task_metrics_count: table_counts.get("task_metrics").copied().unwrap_or(0),
        })
    }
}

/// Storage statistics for capacity monitoring
#[derive(Debug, Clone)]
pub struct CapacityStatistics {
    /// Number of stored episodes.
    pub episode_count: usize,
    /// Number of stored patterns.
    pub pattern_count: usize,
    /// Number of stored heuristics.
    pub heuristic_count: usize,
    /// Number of stored embeddings.
    pub embedding_count: usize,
    /// Number of stored execution records.
    pub execution_record_count: usize,
    /// Number of stored agent metrics rows.
    pub agent_metrics_count: usize,
    /// Number of stored task metrics rows.
    pub task_metrics_count: usize,
}

impl std::fmt::Display for CapacityStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CapacityStatistics(episodes={}, patterns={}, heuristics={}, embeddings={})",
            self.episode_count, self.pattern_count, self.heuristic_count, self.embedding_count
        )
    }
}
