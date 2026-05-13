use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl crate::DuckDbStorage {
    pub(crate) async fn query_episodes_since_internal(
        &self,
        since: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        let limit = do_memory_core::storage::apply_query_limit(limit);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT
                episode_id, task_type, task_description, CAST(context AS VARCHAR),
                strftime(CAST(start_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                strftime(CAST(end_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                CAST(steps AS VARCHAR), CAST(outcome AS VARCHAR), CAST(reward AS VARCHAR),
                CAST(reflection AS VARCHAR), CAST(patterns AS VARCHAR), CAST(heuristics AS VARCHAR),
                CAST(applied_patterns AS VARCHAR), CAST(salient_features AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE start_time >= ?
                ORDER BY start_time ASC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![
                    since.to_rfc3339(),
                    i64::try_from(limit).unwrap_or(1000)
                ])
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let episode_id_str: String =
                    row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let task_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let task_desc: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let start_time_str: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let end_time_str: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let steps_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
                let outcome_json: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
                let reward_json: Option<String> =
                    row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
                let reflection_json: Option<String> =
                    row.get(9).map_err(|e| Error::Storage(e.to_string()))?;
                let patterns_json: String =
                    row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
                let heuristics_json: String =
                    row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
                let applied_patterns_json: Option<String> =
                    row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
                let salient_features_json: Option<String> =
                    row.get(13).map_err(|e| Error::Storage(e.to_string()))?;
                let checkpoints_json: String =
                    row.get(14).map_err(|e| Error::Storage(e.to_string()))?;
                let metadata_json: String =
                    row.get(15).map_err(|e| Error::Storage(e.to_string()))?;

                // Load tags
                let mut tag_stmt = conn
                    .prepare("SELECT tag FROM episode_tags WHERE episode_id = ?")
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let tag_rows = tag_stmt
                    .query_map(params![episode_id_str], |r| r.get::<_, String>(0))
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let mut tags = Vec::new();
                for tag in tag_rows {
                    tags.push(tag.map_err(|e| Error::Storage(e.to_string()))?);
                }

                episodes.push(Self::map_row_to_episode(
                    &episode_id_str,
                    &task_type_str,
                    task_desc,
                    &context_json,
                    &start_time_str,
                    end_time_str,
                    &steps_json,
                    outcome_json,
                    reward_json,
                    reflection_json,
                    &patterns_json,
                    &heuristics_json,
                    applied_patterns_json,
                    salient_features_json,
                    &checkpoints_json,
                    &metadata_json,
                    tags,
                )?);
            }
            Ok::<Vec<do_memory_core::Episode>, Error>(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn query_episodes_by_metadata_internal(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        let limit = do_memory_core::storage::apply_query_limit(limit);
        let key = key.to_string();
        let value = value.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT
                episode_id, task_type, task_description, CAST(context AS VARCHAR),
                strftime(CAST(start_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                strftime(CAST(end_time AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                CAST(steps AS VARCHAR), CAST(outcome AS VARCHAR), CAST(reward AS VARCHAR),
                CAST(reflection AS VARCHAR), CAST(patterns AS VARCHAR), CAST(heuristics AS VARCHAR),
                CAST(applied_patterns AS VARCHAR), CAST(salient_features AS VARCHAR),
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE metadata->>? = ?
                ORDER BY start_time DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![key, value, i64::try_from(limit).unwrap_or(1000)])
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let episode_id_str: String =
                    row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let task_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let task_desc: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let start_time_str: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let end_time_str: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let steps_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
                let outcome_json: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
                let reward_json: Option<String> =
                    row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
                let reflection_json: Option<String> =
                    row.get(9).map_err(|e| Error::Storage(e.to_string()))?;
                let patterns_json: String =
                    row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
                let heuristics_json: String =
                    row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
                let applied_patterns_json: Option<String> =
                    row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
                let salient_features_json: Option<String> =
                    row.get(13).map_err(|e| Error::Storage(e.to_string()))?;
                let checkpoints_json: String =
                    row.get(14).map_err(|e| Error::Storage(e.to_string()))?;
                let metadata_json: String =
                    row.get(15).map_err(|e| Error::Storage(e.to_string()))?;

                // Load tags
                let mut tag_stmt = conn
                    .prepare("SELECT tag FROM episode_tags WHERE episode_id = ?")
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let tag_rows = tag_stmt
                    .query_map(params![episode_id_str], |r| r.get::<_, String>(0))
                    .map_err(|e| Error::Storage(e.to_string()))?;
                let mut tags = Vec::new();
                for tag in tag_rows {
                    tags.push(tag.map_err(|e| Error::Storage(e.to_string()))?);
                }

                episodes.push(Self::map_row_to_episode(
                    &episode_id_str,
                    &task_type_str,
                    task_desc,
                    &context_json,
                    &start_time_str,
                    end_time_str,
                    &steps_json,
                    outcome_json,
                    reward_json,
                    reflection_json,
                    &patterns_json,
                    &heuristics_json,
                    applied_patterns_json,
                    salient_features_json,
                    &checkpoints_json,
                    &metadata_json,
                    tags,
                )?);
            }
            Ok::<Vec<do_memory_core::Episode>, Error>(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}
