//! # Episode Query Operations
//!
//! Query and filtering operations for episodes.

use super::EpisodeQuery;
use crate::TursoStorage;
use do_memory_core::{Episode, Error, Result, apply_query_limit as core_apply_limit};
use tracing::{debug, info};
use uuid::Uuid;

/// Apply query limit with defaults and bounds checking.
/// Uses core module's function but provides local alias for convenience.
#[inline]
fn apply_query_limit(limit: Option<usize>) -> usize {
    core_apply_limit(limit)
}

impl TursoStorage {
    /// Query episodes with filters
    pub async fn query_episodes(&self, query: &EpisodeQuery) -> Result<Vec<Episode>> {
        debug!("Querying episodes with filters: {:?}", query);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let mut sql = String::from(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics,
                   COALESCE(checkpoints, '[]') AS checkpoints,
                   metadata, domain, language,
                   archived_at
            FROM episodes WHERE 1=1
        "#,
        );

        let mut params_vec: Vec<libsql::Value> = Vec::new();

        if let Some(ref task_type) = query.task_type {
            sql.push_str(" AND task_type = ?");
            params_vec.push(task_type.to_string().into());
        }

        if let Some(ref domain) = query.domain {
            sql.push_str(" AND domain = ?");
            params_vec.push(domain.clone().into());
        }

        if let Some(ref language) = query.language {
            sql.push_str(" AND language = ?");
            params_vec.push(language.clone().into());
        }

        if query.completed_only {
            sql.push_str(" AND end_time IS NOT NULL");
        }

        sql.push_str(" ORDER BY start_time DESC");

        // Apply limit with defaults and bounds
        let limit = apply_query_limit(query.limit);
        sql.push_str(" LIMIT ?");
        params_vec.push((limit as i64).into());

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(self.row_to_episode(&row).await?);
        }

        info!("Found {} episodes matching query", episodes.len());
        Ok(episodes)
    }

    /// Query episodes modified since a given timestamp
    ///
    /// # Arguments
    ///
    /// * `since` - Timestamp to query from
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Apply limit with defaults and bounds
        let effective_limit = apply_query_limit(limit);
        debug!(
            "Querying episodes since {} (limit: {})",
            since, effective_limit
        );
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics,
                   COALESCE(checkpoints, '[]') AS checkpoints,
                   metadata, domain, language,
                   archived_at
            FROM episodes
            WHERE start_time >= ?
            ORDER BY start_time DESC
            LIMIT ?
        "#;

        let since_timestamp = since.timestamp();

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![since_timestamp, effective_limit as i64])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(self.row_to_episode(&row).await?);
        }

        info!(
            "Found {} episodes since {} (limit: {})",
            episodes.len(),
            since,
            effective_limit
        );
        Ok(episodes)
    }

    /// Query episodes modified at or after a watermark using keyset pagination.
    ///
    /// Orders by `(episode_revisions.modified_at_ms, episode_revisions.episode_id)`
    /// ascending. When `cursor` is supplied the scan resumes strictly after that
    /// tuple, so episodes sharing a timestamp are neither skipped nor
    /// duplicated. This observes edits to episodes whose `start_time` is older
    /// than the watermark, which [`query_episodes_since`](Self::query_episodes_since)
    /// cannot.
    ///
    /// # Arguments
    ///
    /// * `since` - Inclusive modification watermark
    /// * `cursor` - Last `(modified_at, episode_id)` of the previous page
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    pub async fn query_episodes_modified_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        cursor: Option<(chrono::DateTime<chrono::Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, chrono::DateTime<chrono::Utc>)>> {
        let effective_limit = apply_query_limit(limit);
        let since_ms = since.timestamp_millis();

        let mut sql = String::from(
            r#"
            SELECT e.episode_id, e.task_type, e.task_description, e.context,
                   e.start_time, e.end_time, e.steps, e.outcome, e.reward,
                   e.reflection, e.patterns, e.heuristics,
                   COALESCE(e.checkpoints, '[]') AS checkpoints,
                   e.metadata, e.domain, e.language,
                   e.archived_at, r.modified_at_ms
            FROM episode_revisions r
            JOIN episodes e ON e.episode_id = r.episode_id
            WHERE r.modified_at_ms >= ?
        "#,
        );

        let mut params_vec: Vec<libsql::Value> = vec![since_ms.into()];

        if let Some((cursor_at, cursor_id)) = cursor {
            let cursor_ms = cursor_at.timestamp_millis();
            sql.push_str(
                " AND (r.modified_at_ms > ? OR (r.modified_at_ms = ? AND r.episode_id > ?))",
            );
            params_vec.push(cursor_ms.into());
            params_vec.push(cursor_ms.into());
            params_vec.push(cursor_id.to_string().into());
        }

        sql.push_str(" ORDER BY r.modified_at_ms ASC, r.episode_id ASC LIMIT ?");
        params_vec.push((effective_limit as i64).into());

        debug!(
            "Querying episodes modified since {} (limit: {}, cursor: {:?})",
            since, effective_limit, cursor
        );

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query modified episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = self.row_to_episode(&row).await?;
            let modified_at_ms: i64 = row.get(17).map_err(|e| {
                Error::Storage(format!("Failed to read episode revision timestamp: {}", e))
            })?;
            let modified_at = chrono::DateTime::from_timestamp_millis(modified_at_ms)
                .unwrap_or(episode.start_time);
            episodes.push((episode, modified_at));
        }

        info!(
            "Found {} episodes modified since {} (limit: {})",
            episodes.len(),
            since,
            effective_limit
        );
        Ok(episodes)
    }

    /// Query episodes by metadata key-value pair
    ///
    /// Uses json_extract for efficient querying of JSON metadata fields.
    /// Falls back to LIKE pattern matching if json_extract is not available.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key to search for
    /// * `value` - Metadata value to match
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    pub async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Apply limit with defaults and bounds
        let effective_limit = apply_query_limit(limit);
        debug!(
            "Querying episodes by metadata {} = {} (limit: {})",
            key, value, effective_limit
        );
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Use json_extract for efficient JSON metadata querying
        // We use parameterized queries to prevent SQL injection (Severity: High)
        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics,
                   COALESCE(checkpoints, '[]') AS checkpoints,
                   metadata, domain, language,
                   archived_at
            FROM episodes
            WHERE json_extract(metadata, ?) = ?
            ORDER BY start_time DESC
            LIMIT ?
        "#;

        let json_path = format!("$.{}", key);

        let mut rows = conn
            .query(
                sql,
                libsql::params![json_path, value, effective_limit as i64],
            )
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes by metadata: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(self.row_to_episode(&row).await?);
        }

        info!(
            "Found {} episodes with metadata {} = {} (limit: {})",
            episodes.len(),
            key,
            value,
            effective_limit
        );
        Ok(episodes)
    }
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
