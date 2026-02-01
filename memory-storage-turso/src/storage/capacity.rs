//! Capacity-constrained storage operations for Turso

use crate::{Result, TursoStorage};
use memory_core::Episode;
use std::collections::HashMap;
use tracing::{debug, info, warn};

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
    async fn enforce_capacity(&self, max_episodes: usize) -> Result<()> {
        let (conn, conn_id) = self.get_connection_with_id().await?;

        // Count current episodes
        const COUNT_SQL: &str = "SELECT COUNT(*) as count FROM episodes";

        let mut count_rows = conn
            .query(COUNT_SQL, ())
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Failed to count episodes: {}", e)))?;

        let current_count = if let Some(row) = count_rows
            .next()
            .await
            .map_err(|e| memory_core::Error::Storage(e.to_string()))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| memory_core::Error::Storage(e.to_string()))?;
            count as usize
        } else {
            0
        };

        if current_count <= max_episodes {
            return Ok(());
        }

        // Episodes exceed capacity - need to evict
        let to_remove = current_count - max_episodes;
        warn!(
            "Capacity exceeded: {} > {}, removing {} episodes",
            current_count, max_episodes, to_remove
        );

        // Get episodes to evict (oldest first, using LRU)
        // Order by start_time first, then by episode_id for deterministic tie-breaking
        let evict_sql = format!(
            r#"
            SELECT episode_id FROM episodes
            ORDER BY start_time ASC, episode_id ASC
            LIMIT {}
        "#,
            to_remove
        );

        let mut evict_rows = conn.query(&evict_sql, ()).await.map_err(|e| {
            memory_core::Error::Storage(format!("Failed to query episodes to evict: {}", e))
        })?;

        let mut evicted = Vec::new();
        while let Some(row) = evict_rows
            .next()
            .await
            .map_err(|e| memory_core::Error::Storage(e.to_string()))?
        {
            let episode_id: String = row
                .get(0)
                .map_err(|e| memory_core::Error::Storage(e.to_string()))?;
            evicted.push(episode_id);
        }

        // Drop the connection and query results before starting deletions
        // to avoid "database locked" errors when running tests in parallel
        drop(evict_rows);
        drop(conn);

        // Delete evicted episodes
        const DELETE_SQL: &str = "DELETE FROM episodes WHERE episode_id = ?";

        for episode_id in &evicted {
            // Delete associated embeddings first
            let _ = self._delete_embedding_internal(episode_id).await;

            // Then delete the episode
            let (conn, conn_id) = self.get_connection_with_id().await?;

            // Use prepared statement cache
            let stmt = self
                .prepared_cache
                .get_or_prepare(&conn, DELETE_SQL)
                .await
                .map_err(|e| {
                    memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
                })?;

            stmt.execute(libsql::params![episode_id.clone()])
                .await
                .map_err(|e| {
                    memory_core::Error::Storage(format!("Failed to delete episode: {}", e))
                })?;
            drop(conn);
        }

        info!(
            "Evicted {} episodes to enforce capacity limit of {}",
            evicted.len(),
            max_episodes
        );

        Ok(())
    }

    /// Get storage statistics including capacity info
    pub async fn get_capacity_statistics(&self) -> Result<CapacityStatistics> {
        let (conn, conn_id) = self.get_connection_with_id().await?;

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
            let sql = format!("SELECT COUNT(*) FROM {}", table);
            let mut rows = conn.query(&sql, ()).await.map_err(|e| {
                memory_core::Error::Storage(format!("Failed to count {}: {}", table, e))
            })?;

            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| memory_core::Error::Storage(e.to_string()))?
            {
                let count: i64 = row
                    .get(0)
                    .map_err(|e| memory_core::Error::Storage(e.to_string()))?;
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
    pub episode_count: usize,
    pub pattern_count: usize,
    pub heuristic_count: usize,
    pub embedding_count: usize,
    pub execution_record_count: usize,
    pub agent_metrics_count: usize,
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
