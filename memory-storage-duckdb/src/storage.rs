use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    /// Load VSS extension if enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the extension cannot be loaded or the database task fails.
    pub async fn load_vss_extension(&self) -> Result<()> {
        #[cfg(feature = "vss")]
        {
            let conn_arc = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = conn_arc.lock();
                conn.execute("INSTALL vss; LOAD vss;", [])
                    .map_err(|e| Error::Storage(format!("Failed to load VSS extension: {e}")))?;
                Ok::<(), Error>(())
            })
            .await
            .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
            Ok(())
        }
        #[cfg(not(feature = "vss"))]
        {
            Ok(())
        }
    }

    // ========== Internal CRUD Operations ==========

    /// Stores an episode in the `DuckDB` database.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or database execution fails.
    pub(crate) async fn store_episode_internal(
        &self,
        episode: &do_memory_core::Episode,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let episode = episode.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let context_json = serde_json::to_string(&episode.context)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let steps_json = serde_json::to_string(&episode.steps)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let outcome_json = serde_json::to_string(&episode.outcome)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let reward_json = serde_json::to_string(&episode.reward)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let reflection_json = serde_json::to_string(&episode.reflection)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let patterns_json = serde_json::to_string(&episode.patterns)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let heuristics_json = serde_json::to_string(&episode.heuristics)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let checkpoints_json = serde_json::to_string(&episode.checkpoints)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
            let metadata_json = serde_json::to_string(&episode.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

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
                    episode.end_time.as_ref().map(DateTime::to_rfc3339),
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
            .map_err(|e| Error::Storage(format!("Failed to store episode: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;

        // Emit standardized event
        self.emit_event(do_memory_core::types::event::MemoryEvent::EpisodeStored {
            episode_id: episode.episode_id.to_string(),
            backend: "duckdb".to_string(),
            timestamp: do_memory_core::types::event::unix_now_secs(),
        })
        .await;

        Ok(())
    }

    /// Retrieves an episode by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
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
                let episode_id_str: String =
                    row.get(0).map_err(|e| Error::Storage(format!("col 0: {e}")))?;
                let task_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(format!("col 1: {e}")))?;
                let task_desc: String =
                    row.get(2).map_err(|e| Error::Storage(format!("col 2: {e}")))?;
                let context_json: String =
                    row.get(3).map_err(|e| Error::Storage(format!("col 3: {e}")))?;
                let start_time_str: String =
                    row.get(4).map_err(|e| Error::Storage(format!("col 4: {e}")))?;
                let end_time_str: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(format!("col 5: {e}")))?;
                let steps_json: String =
                    row.get(6).map_err(|e| Error::Storage(format!("col 6: {e}")))?;
                let outcome_json: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(format!("col 7: {e}")))?;
                let reward_json: Option<String> =
                    row.get(8).map_err(|e| Error::Storage(format!("col 8: {e}")))?;
                let reflection_json: Option<String> =
                    row.get(9).map_err(|e| Error::Storage(format!("col 9: {e}")))?;
                let patterns_json: String =
                    row.get(10).map_err(|e| Error::Storage(format!("col 10: {e}")))?;
                let heuristics_json: String =
                    row.get(11).map_err(|e| Error::Storage(format!("col 11: {e}")))?;
                let checkpoints_json: String =
                    row.get(12).map_err(|e| Error::Storage(format!("col 12: {e}")))?;
                let metadata_json: String =
                    row.get(13).map_err(|e| Error::Storage(format!("col 13: {e}")))?;

                let episode = Self::map_row_to_episode(
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
                    &checkpoints_json,
                    &metadata_json,
                )?;
                Ok::<Option<do_memory_core::Episode>, Error>(Some(episode))
            } else {
                Ok::<Option<do_memory_core::Episode>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    #[allow(clippy::too_many_arguments)]
    fn map_row_to_episode(
        episode_id_str: &str,
        task_type_str: &str,
        task_desc: String,
        context_json: &str,
        start_time_str: &str,
        end_time_str: Option<String>,
        steps_json: &str,
        outcome_json: Option<String>,
        reward_json: Option<String>,
        reflection_json: Option<String>,
        patterns_json: &str,
        heuristics_json: &str,
        checkpoints_json: &str,
        metadata_json: &str,
    ) -> Result<do_memory_core::Episode> {
        let episode = do_memory_core::Episode {
            episode_id: Uuid::parse_str(episode_id_str)
                .map_err(|e| Error::Storage(format!("episode_id parse: {e}")))?,
            task_type: task_type_str
                .parse()
                .map_err(|e| Error::Storage(format!("task_type parse: {e}")))?,
            task_description: task_desc,
            context: serde_json::from_str(context_json).map_err(|e| {
                Error::Storage(format!("context parse: {e} | json='{context_json}'"))
            })?,
            start_time: DateTime::parse_from_rfc3339(start_time_str)
                .or_else(|_| DateTime::parse_from_str(start_time_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                .map_err(|e| {
                    Error::Storage(format!("start_time parse: {e} | val='{start_time_str}'"))
                })?
                .with_timezone(&Utc),
            end_time: end_time_str
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map(|t| t.with_timezone(&Utc))
                        .map_err(|e| Error::Storage(format!("end_time parse: {e} | val='{s}'")))
                })
                .transpose()?,
            steps: serde_json::from_str(steps_json)
                .map_err(|e| Error::Storage(format!("steps parse: {e} | json='{steps_json}'")))?,
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
            patterns: serde_json::from_str(patterns_json).map_err(|e| {
                Error::Storage(format!("patterns parse: {e} | json='{patterns_json}'"))
            })?,
            heuristics: serde_json::from_str(heuristics_json).map_err(|e| {
                Error::Storage(format!("heuristics parse: {e} | json='{heuristics_json}'"))
            })?,
            applied_patterns: Vec::new(),
            salient_features: None,
            checkpoints: serde_json::from_str(checkpoints_json).map_err(|e| {
                Error::Storage(format!(
                    "checkpoints parse: {e} | json='{checkpoints_json}'"
                ))
            })?,
            metadata: serde_json::from_str(metadata_json).map_err(|e| {
                Error::Storage(format!("metadata parse: {e} | json='{metadata_json}'"))
            })?,
            tags: Vec::new(),
        };
        Ok(episode)
    }

    /// Deletes an episode by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the delete operation fails.
    pub(crate) async fn delete_episode_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "DELETE FROM episodes WHERE episode_id = ?",
                params![id.to_string()],
            )
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;

        // Emit standardized event
        self.emit_event(
            do_memory_core::types::event::MemoryEvent::EpisodeGarbageCollected {
                id: id.to_string(),
                reason: "manual".to_string(),
                timestamp: do_memory_core::types::event::unix_now_secs(),
            },
        )
        .await;

        Ok(())
    }

    /// Stores a pattern in the `DuckDB` database.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or database execution fails.
    pub(crate) async fn store_pattern_internal(
        &self,
        pattern: &do_memory_core::Pattern,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let data_json = serde_json::to_string(&pattern)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

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
                    i64::try_from(pattern.sample_size()).unwrap_or(0),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store pattern: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    /// Retrieves a pattern by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub(crate) async fn get_pattern_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Pattern>> {
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
                let pattern = serde_json::from_str(&data_json).map_err(|e| {
                    Error::Storage(format!("pattern parse: {e} | json='{data_json}'"))
                })?;
                Ok::<Option<do_memory_core::Pattern>, Error>(Some(pattern))
            } else {
                Ok::<Option<do_memory_core::Pattern>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    /// Stores a heuristic in the `DuckDB` database.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or database execution fails.
    pub(crate) async fn store_heuristic_internal(
        &self,
        heuristic: &do_memory_core::Heuristic,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let heuristic = heuristic.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let evidence_json = serde_json::to_string(&heuristic.evidence)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

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
            .map_err(|e| Error::Storage(format!("Failed to store heuristic: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    /// Retrieves a heuristic by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub(crate) async fn get_heuristic_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Heuristic>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT heuristic_id, condition_text, action_text, confidence, CAST(evidence AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'), strftime(CAST(updated_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ') FROM heuristics WHERE heuristic_id = ?")
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
                    evidence: serde_json::from_str(&evidence_json).map_err(|e| {
                        Error::Storage(format!("heuristic evidence parse: {e} | json='{evidence_json}'"))
                    })?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic created_at parse: {e} | val='{created_at_str}'")))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .or_else(|_| DateTime::parse_from_str(&updated_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic updated_at parse: {e} | val='{updated_at_str}'")))?
                        .with_timezone(&Utc),
                };
                Ok::<Option<do_memory_core::Heuristic>, Error>(Some(heuristic))
            } else {
                Ok::<Option<do_memory_core::Heuristic>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Analytical Queries ==========

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
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE start_time >= ?
                ORDER BY start_time ASC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![since.to_rfc3339(), i64::try_from(limit).unwrap_or(1000)], |row| {
                    Ok(Self::map_row_to_episode(
                        &row.get::<_, String>(0)?,
                        &row.get::<_, String>(1)?,
                        row.get(2)?,
                        &row.get::<_, String>(3)?,
                        &row.get::<_, String>(4)?,
                        row.get(5)?,
                        &row.get::<_, String>(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        &row.get::<_, String>(10)?,
                        &row.get::<_, String>(11)?,
                        &row.get::<_, String>(12)?,
                        &row.get::<_, String>(13)?,
                    ))
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            for row in rows {
                episodes.push(row.map_err(|e| Error::Storage(e.to_string()))??);
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
                CAST(checkpoints AS VARCHAR),
                CAST(metadata AS VARCHAR)
                FROM episodes WHERE metadata->>? = ?
                ORDER BY start_time DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let rows = stmt
                .query_map(params![key, value, i64::try_from(limit).unwrap_or(1000)], |row| {
                    Ok(Self::map_row_to_episode(
                        &row.get::<_, String>(0)?,
                        &row.get::<_, String>(1)?,
                        row.get(2)?,
                        &row.get::<_, String>(3)?,
                        &row.get::<_, String>(4)?,
                        row.get(5)?,
                        &row.get::<_, String>(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        &row.get::<_, String>(10)?,
                        &row.get::<_, String>(11)?,
                        &row.get::<_, String>(12)?,
                        &row.get::<_, String>(13)?,
                    ))
                })
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut episodes = Vec::new();
            for row in rows {
                episodes.push(row.map_err(|e| Error::Storage(e.to_string()))??);
            }
            Ok::<Vec<do_memory_core::Episode>, Error>(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Embedding Storage Methods ==========

    pub(crate) async fn store_embedding_internal(
        &self,
        id: &str,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let dimension = i32::try_from(embedding.len()).unwrap_or(0);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let vector_json = serde_json::to_string(&embedding)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO embeddings (
                    embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                ) VALUES (?, ?, ?, ?, ?::FLOAT[], ?, ?)",
                params![
                    id,
                    id, // Using id as item_id for now as StorageBackend only provides id
                    "generic",
                    "{}", // embedding_data placeholder
                    vector_json,
                    dimension,
                    "default"
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_embedding_internal(&self, id: &str) -> Result<Option<Vec<f32>>> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(embedding_vector AS VARCHAR) FROM embeddings WHERE embedding_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let vector_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let vector: Vec<f32> = serde_json::from_str(&vector_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<Vec<f32>>, Error>(Some(vector))
            } else {
                Ok::<Option<Vec<f32>>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn delete_embedding_internal(&self, id: &str) -> Result<bool> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let affected = conn
                .execute("DELETE FROM embeddings WHERE embedding_id = ?", params![id])
                .map_err(|e| Error::Storage(format!("Failed to delete embedding: {e}")))?;
            Ok::<bool, Error>(affected > 0)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut conn = conn_arc.lock();
            let tx = conn
                .transaction()
                .map_err(|e| Error::Storage(format!("Failed to begin transaction: {e}")))?;
            {
                let mut stmt = tx.prepare(
                    "INSERT OR REPLACE INTO embeddings (
                        embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                    ) VALUES (?, ?, ?, ?, ?::FLOAT[], ?, ?)",
                ).map_err(|e| Error::Storage(e.to_string()))?;

                for (id, embedding) in embeddings {
                    let dimension = i32::try_from(embedding.len()).unwrap_or(0);
                    let vector_json = serde_json::to_string(&embedding)
                        .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
                    stmt.execute(params![
                        id,
                        id,
                        "generic",
                        "{}",
                        vector_json,
                        dimension,
                        "default"
                    ])
                    .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
                }
            }
            tx.commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_embeddings_batch_internal(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn_arc = Arc::clone(&self.conn);
        let ids = ids.to_vec();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let placeholders = vec!["?"; ids.len()].join(",");
            let query = format!(
                "SELECT embedding_id, CAST(embedding_vector AS VARCHAR) FROM embeddings WHERE embedding_id IN ({placeholders})"
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(duckdb::params_from_iter(ids.iter()))
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut results_map = std::collections::HashMap::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let vector_json: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let vector: Vec<f32> = serde_json::from_str(&vector_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                results_map.insert(id, vector);
            }

            let ordered_results: Vec<Option<Vec<f32>>> = ids
                .into_iter()
                .map(|id| results_map.remove(&id))
                .collect();
            Ok::<Vec<Option<Vec<f32>>>, Error>(ordered_results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Relationship Storage Methods ==========

    pub(crate) async fn store_relationship_internal(
        &self,
        rel: &do_memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let rel = rel.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let metadata_json = serde_json::to_string(&rel.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO episode_relationships (
                    relationship_id, from_episode_id, to_episode_id, relationship_type,
                    reason, created_by, priority, metadata, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    rel.id.to_string(),
                    rel.from_episode_id.to_string(),
                    rel.to_episode_id.to_string(),
                    rel.relationship_type.as_str(),
                    rel.metadata.reason,
                    rel.metadata.created_by,
                    rel.metadata.priority.map(i32::from),
                    metadata_json,
                    rel.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store relationship: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn remove_relationship_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "DELETE FROM episode_relationships WHERE relationship_id = ?",
                params![id.to_string()],
            )
            .map_err(|e| Error::Storage(format!("Failed to remove relationship: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_relationships_internal(
        &self,
        episode_id: Uuid,
        direction: do_memory_core::episode::Direction,
    ) -> Result<Vec<do_memory_core::episode::EpisodeRelationship>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let query = match direction {
                do_memory_core::episode::Direction::Outgoing => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE from_episode_id = ?"
                }
                do_memory_core::episode::Direction::Incoming => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE to_episode_id = ?"
                }
                do_memory_core::episode::Direction::Both => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE from_episode_id = ? OR to_episode_id = ?"
                }
            };

            let mut stmt = conn
                .prepare(query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let id_str = episode_id.to_string();
            let mut rows = if direction == do_memory_core::episode::Direction::Both {
                stmt.query(params![id_str, id_str])
            } else {
                stmt.query(params![id_str])
            }
            .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rels = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let from_id: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let to_id: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let rel_type_str: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let metadata_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let created_at_str: String = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;

                rels.push(do_memory_core::episode::EpisodeRelationship {
                    id: Uuid::parse_str(&id).map_err(|e| Error::Storage(e.to_string()))?,
                    from_episode_id: Uuid::parse_str(&from_id)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    to_episode_id: Uuid::parse_str(&to_id)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    relationship_type: do_memory_core::episode::RelationshipType::parse(&rel_type_str)
                        .map_err(Error::Storage)?,
                    metadata: serde_json::from_str(&metadata_json)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(e.to_string()))?
                        .with_timezone(&Utc),
                });
            }
            Ok::<Vec<do_memory_core::episode::EpisodeRelationship>, Error>(rels)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn relationship_exists_internal(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        rel_type: do_memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT 1 FROM episode_relationships
                     WHERE from_episode_id = ? AND to_episode_id = ? AND relationship_type = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![
                    from_episode_id.to_string(),
                    to_episode_id.to_string(),
                    rel_type.as_str()
                ])
                .map_err(|e| Error::Storage(e.to_string()))?;

            Ok::<bool, Error>(
                rows.next()
                    .map_err(|e| Error::Storage(e.to_string()))?
                    .is_some(),
            )
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Recommendation Attribution Methods ==========

    pub(crate) async fn store_recommendation_session_internal(
        &self,
        session: &do_memory_core::memory::attribution::RecommendationSession,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let session = session.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let payload_json = serde_json::to_string(&session)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO recommendation_sessions (
                    session_id, episode_id, timestamp, payload
                ) VALUES (?, ?, ?, ?)",
                params![
                    session.session_id.to_string(),
                    session.episode_id.to_string(),
                    session.timestamp.to_rfc3339(),
                    payload_json,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store recommendation session: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_recommendation_session_internal(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_sessions WHERE session_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![session_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let session = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(Some(session))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn get_recommendation_session_for_episode_internal(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_sessions WHERE episode_id = ? ORDER BY timestamp DESC LIMIT 1")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![episode_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let session = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(Some(session))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_recommendation_feedback_internal(
        &self,
        feedback: &do_memory_core::memory::attribution::RecommendationFeedback,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let feedback = feedback.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let payload_json = serde_json::to_string(&feedback)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO recommendation_feedback (session_id, payload) VALUES (?, ?)",
                params![feedback.session_id.to_string(), payload_json],
            )
            .map_err(|e| Error::Storage(format!("Failed to store recommendation feedback: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_recommendation_feedback_internal(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationFeedback>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_feedback WHERE session_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![session_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let feedback = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationFeedback>, Error>(Some(feedback))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationFeedback>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Monitoring Storage Methods ==========

    pub(crate) async fn store_execution_record_internal(
        &self,
        record: &do_memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let record = record.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT INTO execution_records (
                    agent_name, agent_type, success, duration_ms, started_at, task_description, error_message
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    record.agent_name,
                    record.agent_type.to_string(),
                    record.success,
                    i64::try_from(record.duration.as_millis()).unwrap_or(0),
                    record.started_at.to_rfc3339(),
                    record.task_description,
                    record.error_message,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store execution record: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn store_agent_metrics_internal(
        &self,
        metrics: &do_memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let metrics = metrics.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "INSERT OR REPLACE INTO agent_metrics (
                    agent_name, agent_type, total_executions, successful_executions,
                    total_duration_ms, avg_duration_ms, min_duration_ms, max_duration_ms,
                    last_execution, current_streak, longest_streak, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    metrics.agent_name,
                    metrics.agent_type.to_string(),
                    i64::try_from(metrics.total_executions).unwrap_or(0),
                    i64::try_from(metrics.successful_executions).unwrap_or(0),
                    i64::try_from(metrics.total_duration.as_millis()).unwrap_or(0),
                    i64::try_from(metrics.avg_duration.as_millis()).unwrap_or(0),
                    i64::try_from(metrics.min_duration.as_millis()).unwrap_or(0),
                    i64::try_from(metrics.max_duration.as_millis()).unwrap_or(0),
                    metrics.last_execution.map(|t| t.to_rfc3339()),
                    i32::try_from(metrics.current_streak).unwrap_or(0),
                    i32::try_from(metrics.longest_streak).unwrap_or(0),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store agent metrics: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn store_task_metrics_internal(
        &self,
        metrics: &do_memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let metrics = metrics.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let success_rates_json = serde_json::to_string(&metrics.agent_success_rates)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO task_metrics (
                    task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                    agent_success_rates, updated_at
                ) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    metrics.task_type,
                    i64::try_from(metrics.total_tasks).unwrap_or(0),
                    i64::try_from(metrics.completed_tasks).unwrap_or(0),
                    i64::try_from(metrics.avg_completion_time.as_millis()).unwrap_or(0),
                    success_rates_json,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store task metrics: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn load_agent_metrics_internal(
        &self,
        agent_name: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::AgentMetrics>> {
        let conn_arc = Arc::clone(&self.conn);
        let agent_name = agent_name.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT agent_name, agent_type, total_executions, successful_executions,
                     total_duration_ms, min_duration_ms, max_duration_ms,
                     strftime(CAST(last_execution AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM agent_metrics WHERE agent_name = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![agent_name])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let total_executions: i64 =
                    row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let successful_executions: i64 =
                    row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let total_duration_ms: i64 =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let min_duration_ms: i64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let max_duration_ms: i64 = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
                let last_execution_str: Option<String> =
                    row.get(7).map_err(|e| Error::Storage(e.to_string()))?;

                let agent_type =
                    do_memory_core::monitoring::types::AgentType::from(agent_type_str.as_str());

                let metrics = do_memory_core::monitoring::types::AgentMetrics {
                    agent_name,
                    agent_type,
                    total_executions: u64::try_from(total_executions).unwrap_or(0),
                    successful_executions: u64::try_from(successful_executions).unwrap_or(0),
                    total_duration: std::time::Duration::from_millis(
                        u64::try_from(total_duration_ms).unwrap_or(0),
                    ),
                    avg_duration: if total_executions > 0 {
                        std::time::Duration::from_millis(
                            u64::try_from(total_duration_ms / total_executions).unwrap_or(0),
                        )
                    } else {
                        std::time::Duration::ZERO
                    },
                    min_duration: std::time::Duration::from_millis(
                        u64::try_from(min_duration_ms).unwrap_or(0),
                    ),
                    max_duration: std::time::Duration::from_millis(
                        u64::try_from(max_duration_ms).unwrap_or(0),
                    ),
                    last_execution: last_execution_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    current_streak: 0, // Not stored in retrieved rows in current query
                    longest_streak: 0, // Not stored in retrieved rows in current query
                };
                Ok::<Option<do_memory_core::monitoring::types::AgentMetrics>, Error>(Some(metrics))
            } else {
                Ok::<Option<do_memory_core::monitoring::types::AgentMetrics>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn load_execution_records_internal(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<do_memory_core::monitoring::types::ExecutionRecord>> {
        let conn_arc = Arc::clone(&self.conn);
        let agent_name = agent_name.map(std::string::ToString::to_string);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut query = "SELECT agent_name, agent_type, success, duration_ms,
                             strftime(CAST(started_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'),
                             task_description, error_message
                             FROM execution_records"
                .to_string();
            if agent_name.is_some() {
                query.push_str(" WHERE agent_name = ?");
            }
            query.push_str(" ORDER BY started_at DESC LIMIT ?");

            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = if let Some(name) = agent_name {
                stmt.query(params![name, i64::try_from(limit).unwrap_or(1000)])
            } else {
                stmt.query(params![i64::try_from(limit).unwrap_or(1000)])
            }
            .map_err(|e| Error::Storage(e.to_string()))?;

            let mut records = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let agent_name: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_type_str: String =
                    row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let success: bool = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let duration_ms: i64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let started_at_str: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let task_description: Option<String> =
                    row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let error_message: Option<String> =
                    row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

                let agent_type =
                    do_memory_core::monitoring::types::AgentType::from(agent_type_str.as_str());
                let started_at = DateTime::parse_from_rfc3339(&started_at_str)
                    .or_else(|_| DateTime::parse_from_str(&started_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                    .map_err(|e| Error::Storage(e.to_string()))?
                    .with_timezone(&Utc);

                records.push(do_memory_core::monitoring::types::ExecutionRecord {
                    agent_name,
                    agent_type,
                    success,
                    duration: std::time::Duration::from_millis(
                        u64::try_from(duration_ms).unwrap_or(0),
                    ),
                    started_at,
                    task_description,
                    error_message,
                });
            }
            Ok::<Vec<do_memory_core::monitoring::types::ExecutionRecord>, Error>(records)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn load_task_metrics_internal(
        &self,
        task_type: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::TaskMetrics>> {
        let conn_arc = Arc::clone(&self.conn);
        let task_type = task_type.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                     CAST(agent_success_rates AS VARCHAR)
                     FROM task_metrics WHERE task_type = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![task_type])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let task_type: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let total_tasks: i64 = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let completed_tasks: i64 = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let avg_completion_time_ms: i64 =
                    row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let agent_success_rates_json: String =
                    row.get(4).map_err(|e| Error::Storage(e.to_string()))?;

                let metrics = do_memory_core::monitoring::types::TaskMetrics {
                    task_type,
                    total_tasks: u64::try_from(total_tasks).unwrap_or(0),
                    completed_tasks: u64::try_from(completed_tasks).unwrap_or(0),
                    avg_completion_time: std::time::Duration::from_millis(
                        u64::try_from(avg_completion_time_ms).unwrap_or(0),
                    ),
                    agent_success_rates: serde_json::from_str(&agent_success_rates_json)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                };
                Ok::<Option<do_memory_core::monitoring::types::TaskMetrics>, Error>(Some(metrics))
            } else {
                Ok::<Option<do_memory_core::monitoring::types::TaskMetrics>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Vector Search ==========

    /// Searches for similar embeddings using VSS.
    ///
    /// # Arguments
    ///
    /// * `vector` - The query embedding vector.
    /// * `limit` - The maximum number of results to return.
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or the database query fails.
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
                    "SELECT item_id, list_cosine_similarity(embedding_vector, ?::FLOAT[]) AS score
                FROM embeddings
                ORDER BY score DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let vector_json = serde_json::to_string(&vector).unwrap_or_default();
            let rows = stmt
                .query_map(
                    params![vector_json, i64::try_from(limit).unwrap_or(10)],
                    |row| {
                        let item_id: String = row.get(0)?;
                        let score: f64 = row.get(1)?;
                        let val = serde_json::json!({
                            "item_id": item_id,
                            "score": score,
                        });
                        Ok(val)
                    },
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> =
                rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }
}
