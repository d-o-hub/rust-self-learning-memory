use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    /// Load VSS extension if enabled
    pub async fn load_vss_extension(&self) -> Result<()> {
        #[cfg(feature = "vss")]
        {
            let conn_arc = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = conn_arc.lock();
                conn.execute("INSTALL vss; LOAD vss;", [])
                    .map_err(|e| Error::Storage(format!("Failed to load VSS extension: {}", e)))
            })
            .await
            .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
            Ok(())
        }
        #[cfg(not(feature = "vss"))]
        {
            Ok(())
        }
    }

    // ========== Internal CRUD Operations ==========

    pub(crate) async fn store_episode_internal(&self, episode: &do_memory_core::Episode) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let episode = episode.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT INTO episodes (
                    episode_id, task_type, task_description, context, start_time, end_time,
                    steps, outcome, reward, reflection, patterns, heuristics, checkpoints,
                    metadata, domain, language
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    episode.episode_id,
                    episode.task_type.to_string(),
                    episode.task_description,
                    serde_json::to_value(&episode.context)?,
                    episode.start_time,
                    episode.end_time,
                    serde_json::to_value(&episode.steps)?,
                    serde_json::to_value(&episode.outcome)?,
                    serde_json::to_value(&episode.reward)?,
                    serde_json::to_value(&episode.reflection)?,
                    serde_json::to_value(&episode.patterns)?,
                    serde_json::to_value(&episode.heuristics)?,
                    serde_json::to_value(&episode.checkpoints)?,
                    serde_json::to_value(&episode.metadata)?,
                    episode.context.domain,
                    episode.context.language,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store episode: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn get_episode_internal(&self, id: Uuid) -> Result<Option<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare("SELECT
                episode_id, task_type, task_description, context, start_time, end_time,
                steps, outcome, reward, reflection, patterns, heuristics, checkpoints,
                metadata
                FROM episodes WHERE episode_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt.query(params![id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let context_val: serde_json::Value = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let steps_val: serde_json::Value = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
                let outcome_val: serde_json::Value = row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
                let reward_val: serde_json::Value = row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
                let reflection_val: serde_json::Value = row.get(9).map_err(|e| Error::Storage(e.to_string()))?;
                let patterns_val: serde_json::Value = row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
                let heuristics_val: serde_json::Value = row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
                let checkpoints_val: serde_json::Value = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
                let metadata_val: serde_json::Value = row.get(13).map_err(|e| Error::Storage(e.to_string()))?;

                let episode = do_memory_core::Episode {
                    episode_id: row.get(0).map_err(|e| Error::Storage(e.to_string()))?,
                    task_type: row.get::<_, String>(1).map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e| Error::Storage(format!("Invalid task type: {}", e)))?,
                    task_description: row.get(2).map_err(|e| Error::Storage(e.to_string()))?,
                    context: serde_json::from_value(context_val).map_err(|e| Error::Storage(e.to_string()))?,
                    start_time: row.get(4).map_err(|e| Error::Storage(e.to_string()))?,
                    end_time: row.get(5).map_err(|e| Error::Storage(e.to_string()))?,
                    steps: serde_json::from_value(steps_val).map_err(|e| Error::Storage(e.to_string()))?,
                    outcome: serde_json::from_value(outcome_val).map_err(|e| Error::Storage(e.to_string()))?,
                    reward: serde_json::from_value(reward_val).map_err(|e| Error::Storage(e.to_string()))?,
                    reflection: serde_json::from_value(reflection_val).map_err(|e| Error::Storage(e.to_string()))?,
                    patterns: serde_json::from_value(patterns_val).map_err(|e| Error::Storage(e.to_string()))?,
                    heuristics: serde_json::from_value(heuristics_val).map_err(|e| Error::Storage(e.to_string()))?,
                    applied_patterns: Vec::new(),
                    salient_features: None,
                    checkpoints: serde_json::from_value(checkpoints_val).map_err(|e| Error::Storage(e.to_string()))?,
                    metadata: serde_json::from_value(metadata_val).map_err(|e| Error::Storage(e.to_string()))?,
                    tags: Vec::new(),
                };
                Ok(Some(episode))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn delete_episode_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute("DELETE FROM episodes WHERE episode_id = ?", params![id])
                .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn store_pattern_internal(&self, pattern: &do_memory_core::Pattern) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, context_tags, occurrence_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    pattern.id.to_string(),
                    pattern.pattern_type.to_string(),
                    serde_json::to_value(&pattern.data)?,
                    pattern.effectiveness.success_rate(),
                    pattern.context.domain,
                    pattern.context.language,
                    pattern.context.tags,
                    pattern.effectiveness.usage_count(),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store pattern: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn get_pattern_internal(&self, id: Uuid) -> Result<Option<do_memory_core::Pattern>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare("SELECT * FROM patterns WHERE pattern_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt.query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let data_val: serde_json::Value = row.get("pattern_data").map_err(|e| Error::Storage(e.to_string()))?;
                let pattern = do_memory_core::Pattern {
                    id: row.get::<_, String>("pattern_id").map_err(|e| Error::Storage(e.to_string()))?.parse().unwrap_or(id),
                    pattern_type: row.get::<_, String>("pattern_type").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|_| Error::Pattern("Invalid type".into()))?,
                    data: serde_json::from_value(data_val).map_err(|e| Error::Storage(e.to_string()))?,
                    effectiveness: Default::default(),
                    context: Default::default(),
                    created_at: row.get("created_at").map_err(|e| Error::Storage(e.to_string()))?,
                    updated_at: row.get("updated_at").map_err(|e| Error::Storage(e.to_string()))?,
                };
                Ok(Some(pattern))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn store_heuristic_internal(&self, heuristic: &do_memory_core::Heuristic) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let heuristic = heuristic.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT INTO heuristics (
                    heuristic_id, condition_text, action_text, confidence, evidence
                ) VALUES (?, ?, ?, ?, ?)",
                params![
                    heuristic.id,
                    heuristic.condition,
                    heuristic.action,
                    heuristic.confidence,
                    serde_json::to_value(&heuristic.evidence)?,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store heuristic: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    pub(crate) async fn get_heuristic_internal(&self, id: Uuid) -> Result<Option<do_memory_core::Heuristic>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare("SELECT * FROM heuristics WHERE heuristic_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt.query(params![id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let evidence_val: serde_json::Value = row.get("evidence").map_err(|e| Error::Storage(e.to_string()))?;
                let heuristic = do_memory_core::Heuristic {
                    id: row.get(0).map_err(|e| Error::Storage(e.to_string()))?,
                    condition: row.get(1).map_err(|e| Error::Storage(e.to_string()))?,
                    action: row.get(2).map_err(|e| Error::Storage(e.to_string()))?,
                    confidence: row.get(3).map_err(|e| Error::Storage(e.to_string()))?,
                    evidence: serde_json::from_value(evidence_val).map_err(|e| Error::Storage(e.to_string()))?,
                };
                Ok(Some(heuristic))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??
    }

    // ========== Analytical Queries ==========

    pub async fn query_session_windowing(&self, interval_hours: u32) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare(&format!(
                "SELECT domain,
                       COUNT(*) AS episode_count,
                       AVG(CAST(reward->>'score' AS DOUBLE)) AS avg_reward
                FROM episodes
                GROUP BY domain, time_bucket(INTERVAL '{} hours', start_time)",
                interval_hours
            ))
            .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt.query_map([], |row| {
                let val = serde_json::json!({
                    "domain": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?,
                    "avg_reward": row.get::<_, Option<f64>>(2)?,
                });
                Ok(val)
            })
            .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<_>, _> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    pub async fn query_temporal_decay(&self) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare(
                "SELECT episode_id, task_description,
                       EXP(-0.1 * date_diff('hour', created_at, now())) AS recency_score
                FROM episodes
                ORDER BY recency_score DESC
                LIMIT 10"
            )
            .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt.query_map([], |row| {
                let val = serde_json::json!({
                    "id": row.get::<_, Uuid>(0)?,
                    "description": row.get::<_, String>(1)?,
                    "recency_score": row.get::<_, f64>(2)?,
                });
                Ok(val)
            })
            .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<_>, _> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    pub async fn query_pattern_trends(&self) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare(
                "SELECT pattern_type, COUNT(*) as freq,
                       RANK() OVER (ORDER BY COUNT(*) DESC) as rank
                FROM patterns
                GROUP BY pattern_type"
            )
            .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt.query_map([], |row| {
                let val = serde_json::json!({
                    "type": row.get::<_, String>(0)?,
                    "frequency": row.get::<_, i64>(1)?,
                    "rank": row.get::<_, i64>(2)?,
                });
                Ok(val)
            })
            .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<_>, _> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    pub async fn query_reward_distribution(&self) -> Result<serde_json::Value> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare(
                "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY CAST(reward->>'score' AS DOUBLE)) AS p50,
                       percentile_cont(0.95) WITHIN GROUP (ORDER BY CAST(reward->>'score' AS DOUBLE)) AS p95
                FROM episodes"
            )
            .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt.query([])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                Ok(serde_json::json!({
                    "p50": row.get::<_, Option<f64>>(0)?,
                    "p95": row.get::<_, Option<f64>>(1)?,
                }))
            } else {
                Ok(serde_json::json!({}))
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    // ========== Vector Search ==========

    pub async fn search_embeddings_vss(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn.prepare(
                "SELECT item_id, array_cosine_similarity(embedding_vector, ?::FLOAT[]) AS score
                FROM embeddings
                ORDER BY score DESC
                LIMIT ?"
            )
            .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt.query_map(params![vector, limit as i64], |row| {
                let val = serde_json::json!({
                    "item_id": row.get::<_, String>(0)?,
                    "score": row.get::<_, f64>(1)?,
                });
                Ok(val)
            })
            .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<_>, _> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
