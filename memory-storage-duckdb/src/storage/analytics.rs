use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use std::sync::Arc;

impl DuckDbStorage {
    /// Performs session windowing analysis over episodes.
    ///
    /// # Arguments
    ///
    /// * `interval_hours` - The size of the time bucket in hours.
    ///
    /// # Errors
    ///
    /// Returns an error if the session windowing query fails.
    pub async fn query_session_windowing(
        &self,
        interval_hours: u32,
    ) -> Result<Vec<serde_json::Value>> {
        // Validate interval_hours to prevent SQL injection in formatting
        if interval_hours == 0 || interval_hours > 8760 {
            // Max 1 year
            return Err(Error::Storage(
                "interval_hours must be between 1 and 8760".to_string(),
            ));
        }

        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT domain,
                       COUNT(*) AS episode_count,
                       AVG(CAST(reward->>'total' AS DOUBLE)) AS avg_reward
                FROM episodes
                GROUP BY domain, time_bucket(INTERVAL '{interval_hours} hours', start_time)"
                ))
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    let domain: String = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    let avg_reward: Option<f64> = row.get(2)?;
                    let val = serde_json::json!({
                        "domain": domain,
                        "count": count,
                        "avg_reward": avg_reward,
                    });
                    Ok(val)
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> =
                rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }

    /// Performs temporal decay scoring for episodes.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporal decay query fails.
    pub async fn query_temporal_decay(&self) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT episode_id, task_description,
                       EXP(-0.1 * date_diff('hour', start_time, now())) AS recency_score
                FROM episodes
                ORDER BY recency_score DESC
                LIMIT 10",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    let episode_id_str: String = row.get(0)?;
                    let task_desc: String = row.get(1)?;
                    let recency_score: f64 = row.get(2)?;
                    let val = serde_json::json!({
                        "id": episode_id_str,
                        "description": task_desc,
                        "recency_score": recency_score,
                    });
                    Ok(val)
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> =
                rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }

    /// Trends patterns based on frequency and rank.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern trending query fails.
    pub async fn query_pattern_trends(&self) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT pattern_type, COUNT(*) as freq,
                       RANK() OVER (ORDER BY COUNT(*) DESC) as rank
                FROM patterns
                GROUP BY pattern_type",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    let pattern_type: String = row.get(0)?;
                    let frequency: i64 = row.get(1)?;
                    let rank: i64 = row.get(2)?;
                    let val = serde_json::json!({
                        "type": pattern_type,
                        "frequency": frequency,
                        "rank": rank,
                    });
                    Ok(val)
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> =
                rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }

    /// Analyzes reward score distribution.
    ///
    /// # Errors
    ///
    /// Returns an error if the reward distribution query fails.
    pub async fn query_reward_distribution(&self) -> Result<serde_json::Value> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY CAST(reward->>'total' AS DOUBLE)) AS p50,
                       percentile_cont(0.95) WITHIN GROUP (ORDER BY CAST(reward->>'total' AS DOUBLE)) AS p95
                FROM episodes"
            )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query([])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let p50: Option<f64> = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let p95: Option<f64> = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                Ok(serde_json::json!({
                    "p50": p50,
                    "p95": p95,
                }))
            } else {
                Ok(serde_json::json!({}))
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }
}
