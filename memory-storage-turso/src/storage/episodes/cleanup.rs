//! Episode retention and garbage-collection operations for Turso storage (WG-075).

use crate::TursoStorage;
use crate::storage::tag_operations::get_episode_tags;
use chrono::Utc;
use do_memory_core::episode::{CleanupResult, EpisodeRetentionPolicy};
use do_memory_core::{Episode, Error, Result};
use std::time::Instant;

impl TursoStorage {
    /// Load all stored episodes ordered oldest-first for retention evaluation.
    async fn list_episodes_for_cleanup(&self) -> Result<Vec<Episode>> {
        self.with_connection_with_id(async |conn, _conn_id| {
            const SQL: &str = r#"
                SELECT episode_id, task_type, task_description, context,
                       start_time, end_time, steps, outcome, reward,
                       reflection, patterns, heuristics,
                       COALESCE(checkpoints, '[]') AS checkpoints,
                       metadata, domain, language,
                       archived_at
                FROM episodes
                ORDER BY start_time ASC, episode_id ASC
            "#;

            let mut rows = conn.query(SQL, ()).await.map_err(|e| {
                Error::Storage(format!("Failed to query episodes for cleanup: {e}"))
            })?;

            let mut episodes = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| {
                Error::Storage(format!("Failed to fetch episode row for cleanup: {e}"))
            })? {
                let mut episode = super::row::row_to_episode(&row)?;
                episode.tags = get_episode_tags(conn, &episode.episode_id).await?;
                episodes.push(episode);
            }
            Ok(episodes)
        })
        .await
    }

    /// Count episodes that `policy` would delete without mutating storage.
    pub async fn count_cleanup_candidates(&self, policy: &EpisodeRetentionPolicy) -> Result<usize> {
        let episodes = self.list_episodes_for_cleanup().await?;
        policy.count_candidates(&episodes, Utc::now())
    }

    /// Clean up expired or low-value episodes according to `policy` (WG-075).
    pub async fn cleanup_episodes(&self, policy: &EpisodeRetentionPolicy) -> Result<CleanupResult> {
        let started = Instant::now();
        let episodes = self.list_episodes_for_cleanup().await?;
        let (mut result, to_delete) = policy.plan_cleanup(&episodes, Utc::now())?;

        for id in to_delete {
            match self.delete_episode(id).await {
                Ok(()) => {
                    let _ = self.delete_embedding_backend(&id.to_string()).await;
                    result.add_deleted(id);
                }
                Err(e) => result.add_error(format!("{id}: {e}")),
            }
        }

        result.duration_ms = started.elapsed().as_millis() as u64;
        Ok(result)
    }
}
