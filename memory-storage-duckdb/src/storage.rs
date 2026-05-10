use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
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
                    .map_err(|e| Error::Storage(format!("Failed to load VSS extension: {}", e)))?;
                Ok::<(), Error>(())
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

    pub(crate) async fn store_episode_internal(
        &self,
        episode: &do_memory_core::Episode,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let episode = episode.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let context_json = serde_json::to_string(&episode.context)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let steps_json = serde_json::to_string(&episode.steps)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let outcome_json = serde_json::to_string(&episode.outcome)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let reward_json = serde_json::to_string(&episode.reward)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let reflection_json = serde_json::to_string(&episode.reflection)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let patterns_json = serde_json::to_string(&episode.patterns)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let heuristics_json = serde_json::to_string(&episode.heuristics)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let checkpoints_json = serde_json::to_string(&episode.checkpoints)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;
            let metadata_json = serde_json::to_string(&episode.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;

            conn.execute(
                "INSERT INTO episodes (
                    episode_id, task_type, task_description, context, start_time, end_time,
                    steps, outcome, reward, reflection, patterns, heuristics, checkpoints,
                    metadata, domain, language
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    episode.episode_id.to_string(),
                    episode.task_type.to_string(),
                    episode.task_description,
                    context_json,
                    episode.start_time.to_rfc3339(),
                    episode.end_time.as_ref().map(|t| t.to_rfc3339()),
                    steps_json,
                    outcome_json,
                    reward_json,
                    reflection_json,
                    patterns_json,
                    heuristics_json,
                    checkpoints_json,
                    metadata_json,
                    episode.context.domain,
                    episode.context.language,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store episode: {}", e)))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(())
    }

    pub(crate) async fn get_episode_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Episode>> {
        let conn_arc = Arc::clone(&self.conn);
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
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE episode_id = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let episode_id_str: String = row.get(0).map_err(|e| Error::Storage(format!("col 0: {}", e)))?;
                let task_type_str: String = row.get(1).map_err(|e| Error::Storage(format!("col 1: {}", e)))?;
                let task_desc: String = row.get(2).map_err(|e| Error::Storage(format!("col 2: {}", e)))?;
                let context_json: String = row.get(3).map_err(|e| Error::Storage(format!("col 3: {}", e)))?;
                let start_time_str: String = row.get(4).map_err(|e| Error::Storage(format!("col 4: {}", e)))?;
                let end_time_str: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(format!("col 5: {}", e)))?;
                let steps_json: String = row.get(6).map_err(|e| Error::Storage(format!("col 6: {}", e)))?;
                let outcome_json: Option<String> = row.get(7).map_err(|e| Error::Storage(format!("col 7: {}", e)))?;
                let reward_json: Option<String> = row.get(8).map_err(|e| Error::Storage(format!("col 8: {}", e)))?;
                let reflection_json: Option<String> = row.get(9).map_err(|e| Error::Storage(format!("col 9: {}", e)))?;
                let patterns_json: String = row.get(10).map_err(|e| Error::Storage(format!("col 10: {}", e)))?;
                let heuristics_json: String =
                    row.get(11).map_err(|e| Error::Storage(format!("col 11: {}", e)))?;
                let checkpoints_json: String =
                    row.get(12).map_err(|e| Error::Storage(format!("col 12: {}", e)))?;
                let metadata_json: String = row.get(13).map_err(|e| Error::Storage(format!("col 13: {}", e)))?;

                let episode = do_memory_core::Episode {
                    episode_id: Uuid::parse_str(&episode_id_str)
                        .map_err(|e| Error::Storage(format!("episode_id parse: {}", e)))?,
                    task_type: task_type_str
                        .parse()
                        .map_err(|e| Error::Storage(format!("task_type parse: {}", e)))?,
                    task_description: task_desc,
                    context: serde_json::from_str(&context_json)
                        .map_err(|e| Error::Storage(format!("context parse: {} | json='{}'", e, context_json)))?,
                    start_time: DateTime::parse_from_rfc3339(&start_time_str)
                        .or_else(|_| DateTime::parse_from_str(&start_time_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("start_time parse: {} | val='{}'", e, start_time_str)))?
                        .with_timezone(&Utc),
                    end_time: end_time_str
                        .map(|s| {
                            DateTime::parse_from_rfc3339(&s)
                                .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                                .map(|t| t.with_timezone(&Utc))
                                .map_err(|e| Error::Storage(format!("end_time parse: {} | val='{}'", e, s)))
                        })
                        .transpose()?,
                    steps: serde_json::from_str(&steps_json)
                        .map_err(|e| Error::Storage(format!("steps parse: {} | json='{}'", e, steps_json)))?,
                    outcome: outcome_json.and_then(|s| {
                        if s == "null" || s.is_empty() {
                            None
                        } else {
                            serde_json::from_str(&s).ok()
                        }
                    }),
                    reward: reward_json.and_then(|s| {
                        if s == "null" || s.is_empty() {
                            None
                        } else {
                            serde_json::from_str(&s).ok()
                        }
                    }),
                    reflection: reflection_json.and_then(|s| {
                        if s == "null" || s.is_empty() {
                            None
                        } else {
                            serde_json::from_str(&s).ok()
                        }
                    }),
                    patterns: serde_json::from_str(&patterns_json)
                        .map_err(|e| Error::Storage(format!("patterns parse: {} | json='{}'", e, patterns_json)))?,
                    heuristics: serde_json::from_str(&heuristics_json)
                        .map_err(|e| Error::Storage(format!("heuristics parse: {} | json='{}'", e, heuristics_json)))?,
                    applied_patterns: Vec::new(),
                    salient_features: None,
                    checkpoints: serde_json::from_str(&checkpoints_json)
                        .map_err(|e| Error::Storage(format!("checkpoints parse: {} | json='{}'", e, checkpoints_json)))?,
                    metadata: serde_json::from_str(&metadata_json)
                        .map_err(|e| Error::Storage(format!("metadata parse: {} | json='{}'", e, metadata_json)))?,
                    tags: Vec::new(),
                };
                Ok::<Option<do_memory_core::Episode>, Error>(Some(episode))
            } else {
                Ok::<Option<do_memory_core::Episode>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(res)
    }

    pub(crate) async fn delete_episode_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "DELETE FROM episodes WHERE episode_id = ?",
                params![id.to_string()],
            )
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(())
    }

    pub(crate) async fn store_pattern_internal(
        &self,
        pattern: &do_memory_core::Pattern,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let data_json = serde_json::to_string(&pattern)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;

            conn.execute(
                "INSERT INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, occurrence_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    pattern.id().to_string(),
                    "placeholder",
                    data_json,
                    pattern.success_rate(),
                    pattern
                        .context()
                        .map(|c| c.domain.clone())
                        .unwrap_or_default(),
                    pattern.context().and_then(|c| c.language.clone()),
                    pattern.sample_size() as i64,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store pattern: {}", e)))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(())
    }

    pub(crate) async fn get_pattern_internal(
        &self,
        id: Uuid) -> Result<Option<do_memory_core::Pattern>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(pattern_data AS VARCHAR) FROM patterns WHERE pattern_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let data_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let pattern = serde_json::from_str(&data_json)
                    .map_err(|e| Error::Storage(format!("pattern parse: {} | json='{}'", e, data_json)))?;
                Ok::<Option<do_memory_core::Pattern>, Error>(Some(pattern))
            } else {
                Ok::<Option<do_memory_core::Pattern>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(res)
    }

    pub(crate) async fn store_heuristic_internal(
        &self,
        heuristic: &do_memory_core::Heuristic,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let heuristic = heuristic.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let evidence_json = serde_json::to_string(&heuristic.evidence)
                .map_err(|e| Error::Storage(format!("Serialization error: {}", e)))?;

            conn.execute(
                "INSERT INTO heuristics (
                    heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    heuristic.heuristic_id.to_string(),
                    heuristic.condition,
                    heuristic.action,
                    heuristic.confidence,
                    evidence_json,
                    heuristic.created_at.to_rfc3339(),
                    heuristic.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store heuristic: {}", e)))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(())
    }

    pub(crate) async fn get_heuristic_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Heuristic>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT heuristic_id, condition_text, action_text, confidence, CAST(evidence AS VARCHAR),
                         strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                         strftime(CAST(updated_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                         FROM heuristics WHERE heuristic_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id_str: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let condition: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let action: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let confidence: f32 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let evidence_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let created_at_str: String = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let updated_at_str: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

                let heuristic = do_memory_core::Heuristic {
                    heuristic_id: Uuid::parse_str(&id_str)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    condition,
                    action,
                    confidence,
                    evidence: serde_json::from_str(&evidence_json)
                        .map_err(|e| Error::Storage(format!("heuristic evidence parse: {} | json='{}'", e, evidence_json)))?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic created_at parse: {} | val='{}'", e, created_at_str)))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .or_else(|_| DateTime::parse_from_str(&updated_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic updated_at parse: {} | val='{}'", e, updated_at_str)))?
                        .with_timezone(&Utc),
                };
                Ok::<Option<do_memory_core::Heuristic>, Error>(Some(heuristic))
            } else {
                Ok::<Option<do_memory_core::Heuristic>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;
        Ok(res)
    }

    // ========== Analytical Queries ==========

    pub async fn query_session_windowing(
        &self,
        interval_hours: u32,
    ) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT domain,
                       COUNT(*) AS episode_count,
                       AVG(CAST(reward->>'score' AS DOUBLE)) AS avg_reward
                FROM episodes
                GROUP BY domain, time_bucket(INTERVAL '{} hours', start_time)",
                    interval_hours
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

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

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

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

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

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    pub async fn query_reward_distribution(&self) -> Result<serde_json::Value> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY CAST(reward->>'score' AS DOUBLE)) AS p50,
                       percentile_cont(0.95) WITHIN GROUP (ORDER BY CAST(reward->>'score' AS DOUBLE)) AS p95
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
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    // ========== Vector Search ==========

    pub async fn search_embeddings_vss(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT item_id, array_cosine_similarity(embedding_vector, ?::FLOAT[]) AS score
                FROM embeddings
                ORDER BY score DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let vector_json = serde_json::to_string(&vector).unwrap_or_default();
            let rows = stmt
                .query_map(params![vector_json, limit as i64], |row| {
                    let item_id: String = row.get(0)?;
                    let score: f64 = row.get(1)?;
                    let val = serde_json::json!({
                        "item_id": item_id,
                        "score": score,
                    });
                    Ok(val)
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> = rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
