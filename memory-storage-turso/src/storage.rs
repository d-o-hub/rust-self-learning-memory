//! Storage operations for episodes, patterns, and heuristics

use crate::TursoStorage;
use async_trait::async_trait;
use libsql::params;
use memory_core::embeddings::{
    cosine_similarity, EmbeddingStorageBackend, SimilarityMetadata, SimilaritySearchResult,
};
use memory_core::{
    episode::PatternId,
    monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics},
    Episode, Error, Heuristic, Pattern, Result, TaskType,
};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Query builder for episodes
#[derive(Debug, Clone, Default)]
pub struct EpisodeQuery {
    pub task_type: Option<TaskType>,
    pub domain: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
    pub completed_only: bool,
}

/// Query builder for patterns
#[derive(Debug, Clone, Default)]
pub struct PatternQuery {
    pub domain: Option<String>,
    pub language: Option<String>,
    pub min_success_rate: Option<f32>,
    pub limit: Option<usize>,
}

/// Pattern metadata including timestamps
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    pub pattern_id: PatternId,
    pub pattern_type: String,
    pub success_rate: f32,
    pub occurrence_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TursoStorage {
    /// Store an episode
    ///
    /// Uses INSERT OR REPLACE for upsert semantics.
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        debug!("Storing episode: {}", episode.episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, heuristics, metadata, domain, language
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let context_json = serde_json::to_string(&episode.context).map_err(Error::Serialization)?;
        let steps_json = serde_json::to_string(&episode.steps).map_err(Error::Serialization)?;
        let outcome_json = episode
            .outcome
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(Error::Serialization)?;
        let reward_json = episode
            .reward
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(Error::Serialization)?;
        let reflection_json = episode
            .reflection
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(Error::Serialization)?;
        let patterns_json =
            serde_json::to_string(&episode.patterns).map_err(Error::Serialization)?;
        let heuristics_json =
            serde_json::to_string(&episode.heuristics).map_err(Error::Serialization)?;
        let metadata_json =
            serde_json::to_string(&episode.metadata).map_err(Error::Serialization)?;

        conn.execute(
            sql,
            params![
                episode.episode_id.to_string(),
                episode.task_type.to_string(),
                episode.task_description.clone(),
                context_json,
                episode.start_time.timestamp(),
                episode.end_time.map(|t| t.timestamp()),
                steps_json,
                outcome_json,
                reward_json,
                reflection_json,
                patterns_json,
                heuristics_json,
                metadata_json,
                episode.context.domain.clone(),
                episode.context.language.clone(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store episode: {}", e)))?;

        info!("Successfully stored episode: {}", episode.episode_id);
        Ok(())
    }

    /// Retrieve an episode by ID
    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Option<Episode>> {
        debug!("Retrieving episode: {}", episode_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata
            FROM episodes WHERE episode_id = ?
        "#;

        let mut rows = conn
            .query(sql, params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episode: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = self.row_to_episode(&row)?;
            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }

    /// Query episodes with filters
    pub async fn query_episodes(&self, query: &EpisodeQuery) -> Result<Vec<Episode>> {
        debug!("Querying episodes with filters: {:?}", query);
        let conn = self.get_connection().await?;

        let mut sql = String::from(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata
            FROM episodes WHERE 1=1
        "#,
        );

        let mut params_vec = Vec::new();

        if let Some(ref task_type) = query.task_type {
            sql.push_str(" AND task_type = ?");
            params_vec.push(task_type.to_string());
        }

        if let Some(ref domain) = query.domain {
            sql.push_str(" AND domain = ?");
            params_vec.push(domain.clone());
        }

        if let Some(ref language) = query.language {
            sql.push_str(" AND language = ?");
            params_vec.push(language.clone());
        }

        if query.completed_only {
            sql.push_str(" AND end_time IS NOT NULL");
        }

        sql.push_str(" ORDER BY start_time DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

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
            episodes.push(self.row_to_episode(&row)?);
        }

        info!("Found {} episodes matching query", episodes.len());
        Ok(episodes)
    }

    /// Query episodes modified since a given timestamp
    ///
    /// Returns all episodes where start_time is >= the given timestamp.
    /// This is used for incremental synchronization.
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>> {
        debug!("Querying episodes since {}", since);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata
            FROM episodes
            WHERE start_time >= ?
            ORDER BY start_time DESC
        "#;

        let since_timestamp = since.timestamp();

        let mut rows = conn
            .query(sql, params![since_timestamp])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(self.row_to_episode(&row)?);
        }

        info!("Found {} episodes modified since {}", episodes.len(), since);
        Ok(episodes)
    }

    /// Query episodes by metadata key-value pair
    ///
    /// Returns all episodes whose metadata contains the specified key-value pair.
    /// This enables efficient querying of specialized data like monitoring records.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key to search for
    /// * `value` - Metadata value to match
    ///
    /// # Returns
    ///
    /// Vector of episodes matching the metadata criteria
    pub async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
        debug!("Querying episodes by metadata: {} = {}", key, value);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata
            FROM episodes
            WHERE metadata LIKE '%' || ? || '%'
            ORDER BY start_time DESC
        "#;

        let search_pattern = format!("\"{}\":\"{}\"", key, value);

        let mut rows = conn
            .query(sql, params![search_pattern])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes by metadata: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = self.row_to_episode(&row)?;

            // Double-check the metadata match (since LIKE might be imprecise)
            if let Some(metadata_value) = episode.metadata.get(key) {
                if metadata_value == value {
                    episodes.push(episode);
                }
            }
        }

        info!(
            "Found {} episodes with metadata {} = {}",
            episodes.len(),
            key,
            value
        );
        Ok(episodes)
    }

    /// Delete an episode
    pub async fn delete_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Deleting episode: {}", episode_id);
        let conn = self.get_connection().await?;

        conn.execute(
            "DELETE FROM episodes WHERE episode_id = ?",
            params![episode_id.to_string()],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;

        info!("Deleted episode: {}", episode_id);
        Ok(())
    }

    /// Store a pattern
    pub async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        debug!("Storing pattern: {}", pattern.id());
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO patterns (
                pattern_id, pattern_type, pattern_data, success_rate,
                context_domain, context_language, context_tags, occurrence_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let pattern_data = serde_json::to_string(pattern).map_err(Error::Serialization)?;

        let (domain, language, tags) = if let Some(ctx) = pattern.context() {
            (
                Some(ctx.domain.clone()),
                ctx.language.clone(),
                Some(serde_json::to_string(&ctx.tags).map_err(Error::Serialization)?),
            )
        } else {
            (None, None, None)
        };

        let pattern_type = match pattern {
            Pattern::ToolSequence { .. } => "tool_sequence",
            Pattern::DecisionPoint { .. } => "decision_point",
            Pattern::ErrorRecovery { .. } => "error_recovery",
            Pattern::ContextPattern { .. } => "context_pattern",
        };

        let occurrence_count = match pattern {
            Pattern::ToolSequence {
                occurrence_count, ..
            } => *occurrence_count,
            _ => 1,
        };

        conn.execute(
            sql,
            params![
                pattern.id().to_string(),
                pattern_type,
                pattern_data,
                pattern.success_rate(),
                domain,
                language,
                tags,
                occurrence_count as i64,
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store pattern: {}", e)))?;

        info!("Successfully stored pattern: {}", pattern.id());
        Ok(())
    }

    /// Retrieve a pattern by ID
    pub async fn get_pattern(&self, pattern_id: PatternId) -> Result<Option<Pattern>> {
        debug!("Retrieving pattern: {}", pattern_id);
        let conn = self.get_connection().await?;

        let sql = "SELECT pattern_data FROM patterns WHERE pattern_id = ?";

        let mut rows = conn
            .query(sql, params![pattern_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query pattern: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            let pattern_data: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get pattern_data: {}", e)))?;
            let pattern: Pattern =
                serde_json::from_str(&pattern_data).map_err(Error::Serialization)?;
            Ok(Some(pattern))
        } else {
            Ok(None)
        }
    }

    /// Get pattern metadata including timestamps
    pub async fn get_pattern_metadata(
        &self,
        pattern_id: PatternId,
    ) -> Result<Option<PatternMetadata>> {
        debug!("Retrieving pattern metadata: {}", pattern_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT pattern_id, pattern_type, success_rate, occurrence_count,
                   created_at, updated_at
            FROM patterns WHERE pattern_id = ?
        "#;

        let mut rows = conn
            .query(sql, params![pattern_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query pattern metadata: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern metadata row: {}", e)))?
        {
            let metadata = self.row_to_pattern_metadata(&row)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Query patterns with filters
    pub async fn query_patterns(&self, query: &PatternQuery) -> Result<Vec<Pattern>> {
        debug!("Querying patterns with filters: {:?}", query);
        let conn = self.get_connection().await?;

        let mut sql = String::from("SELECT pattern_data FROM patterns WHERE 1=1");
        let mut params_vec = Vec::new();

        if let Some(ref domain) = query.domain {
            sql.push_str(" AND context_domain = ?");
            params_vec.push(domain.clone());
        }

        if let Some(ref language) = query.language {
            sql.push_str(" AND context_language = ?");
            params_vec.push(language.clone());
        }

        if let Some(min_success_rate) = query.min_success_rate {
            sql.push_str(&format!(" AND success_rate >= {}", min_success_rate));
        }

        sql.push_str(" ORDER BY success_rate DESC, occurrence_count DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query patterns: {}", e)))?;

        let mut patterns = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch pattern row: {}", e)))?
        {
            let pattern_data: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get pattern_data: {}", e)))?;
            let pattern: Pattern =
                serde_json::from_str(&pattern_data).map_err(Error::Serialization)?;
            patterns.push(pattern);
        }

        info!("Found {} patterns matching query", patterns.len());
        Ok(patterns)
    }

    /// Store a heuristic
    pub async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        debug!("Storing heuristic: {}", heuristic.heuristic_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO heuristics (
                heuristic_id, condition_text, action_text, confidence, evidence
            ) VALUES (?, ?, ?, ?, ?)
        "#;

        let evidence_json =
            serde_json::to_string(&heuristic.evidence).map_err(Error::Serialization)?;

        conn.execute(
            sql,
            params![
                heuristic.heuristic_id.to_string(),
                heuristic.condition.clone(),
                heuristic.action.clone(),
                heuristic.confidence,
                evidence_json,
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store heuristic: {}", e)))?;

        info!("Successfully stored heuristic: {}", heuristic.heuristic_id);
        Ok(())
    }

    /// Retrieve a heuristic by ID
    pub async fn get_heuristic(&self, heuristic_id: Uuid) -> Result<Option<Heuristic>> {
        debug!("Retrieving heuristic: {}", heuristic_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT heuristic_id, condition_text, action_text, confidence,
                   evidence, created_at, updated_at
            FROM heuristics WHERE heuristic_id = ?
        "#;

        let mut rows = conn
            .query(sql, params![heuristic_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query heuristic: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch heuristic row: {}", e)))?
        {
            let heuristic = self.row_to_heuristic(&row)?;
            Ok(Some(heuristic))
        } else {
            Ok(None)
        }
    }

    /// Get all heuristics with minimum confidence
    pub async fn get_heuristics(&self, min_confidence: f32) -> Result<Vec<Heuristic>> {
        debug!(
            "Retrieving heuristics with min confidence: {}",
            min_confidence
        );
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT heuristic_id, condition_text, action_text, confidence,
                   evidence, created_at, updated_at
            FROM heuristics
            WHERE confidence >= ?
            ORDER BY confidence DESC
        "#;

        let mut rows = conn
            .query(sql, params![min_confidence])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query heuristics: {}", e)))?;

        let mut heuristics = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch heuristic row: {}", e)))?
        {
            heuristics.push(self.row_to_heuristic(&row)?);
        }

        info!(
            "Found {} heuristics with min confidence {}",
            heuristics.len(),
            min_confidence
        );
        Ok(heuristics)
    }

    /// Helper: Convert a row to an Episode
    fn row_to_episode(&self, row: &libsql::Row) -> Result<Episode> {
        let episode_id_str: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get episode_id: {}", e)))?;
        let task_type_str: String = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get task_type: {}", e)))?;
        let task_description: String = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get task_description: {}", e)))?;
        let context_json: String = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get context: {}", e)))?;
        let start_time: i64 = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get start_time: {}", e)))?;
        let end_time: Option<i64> = row
            .get(5)
            .map_err(|e| Error::Storage(format!("Failed to get end_time: {}", e)))?;
        let steps_json: String = row
            .get(6)
            .map_err(|e| Error::Storage(format!("Failed to get steps: {}", e)))?;
        let outcome_json: Option<String> = row
            .get(7)
            .map_err(|e| Error::Storage(format!("Failed to get outcome: {}", e)))?;
        let reward_json: Option<String> = row
            .get(8)
            .map_err(|e| Error::Storage(format!("Failed to get reward: {}", e)))?;
        let reflection_json: Option<String> = row
            .get(9)
            .map_err(|e| Error::Storage(format!("Failed to get reflection: {}", e)))?;
        let patterns_json: String = row
            .get(10)
            .map_err(|e| Error::Storage(format!("Failed to get patterns: {}", e)))?;
        let heuristics_json: String = row
            .get(11)
            .map_err(|e| Error::Storage(format!("Failed to get heuristics: {}", e)))?;
        let metadata_json: String = row
            .get(12)
            .map_err(|e| Error::Storage(format!("Failed to get metadata: {}", e)))?;

        Ok(Episode {
            episode_id: Uuid::parse_str(&episode_id_str)
                .map_err(|e| Error::Storage(format!("Invalid episode UUID: {}", e)))?,
            task_type: self.parse_task_type(&task_type_str)?,
            task_description,
            context: serde_json::from_str(&context_json).map_err(Error::Serialization)?,
            start_time: chrono::DateTime::from_timestamp(start_time, 0)
                .ok_or_else(|| Error::Storage("Invalid start_time timestamp".to_string()))?,
            end_time: match end_time {
                Some(t) => Some(
                    chrono::DateTime::from_timestamp(t, 0)
                        .ok_or_else(|| Error::Storage("Invalid end_time timestamp".to_string()))?,
                ),
                None => None,
            },
            steps: serde_json::from_str(&steps_json).map_err(Error::Serialization)?,
            outcome: outcome_json
                .map(|json| serde_json::from_str(&json))
                .transpose()
                .map_err(Error::Serialization)?,
            reward: reward_json
                .map(|json| serde_json::from_str(&json))
                .transpose()
                .map_err(Error::Serialization)?,
            reflection: reflection_json
                .map(|json| serde_json::from_str(&json))
                .transpose()
                .map_err(Error::Serialization)?,
            patterns: serde_json::from_str(&patterns_json).map_err(Error::Serialization)?,
            heuristics: serde_json::from_str(&heuristics_json).map_err(Error::Serialization)?,
            metadata: serde_json::from_str(&metadata_json).map_err(Error::Serialization)?,
        })
    }

    /// Helper: Convert a row to a Heuristic
    fn row_to_heuristic(&self, row: &libsql::Row) -> Result<Heuristic> {
        let heuristic_id_str: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get heuristic_id: {}", e)))?;
        let condition: String = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get condition: {}", e)))?;
        let action: String = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get action: {}", e)))?;
        let confidence: f64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get confidence: {}", e)))?;
        let evidence_json: String = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get evidence: {}", e)))?;
        let created_at: i64 = row
            .get(5)
            .map_err(|e| Error::Storage(format!("Failed to get created_at: {}", e)))?;
        let updated_at: i64 = row
            .get(6)
            .map_err(|e| Error::Storage(format!("Failed to get updated_at: {}", e)))?;

        Ok(Heuristic {
            heuristic_id: Uuid::parse_str(&heuristic_id_str)
                .map_err(|e| Error::Storage(format!("Invalid heuristic UUID: {}", e)))?,
            condition,
            action,
            confidence: confidence as f32,
            evidence: serde_json::from_str(&evidence_json).map_err(Error::Serialization)?,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .ok_or_else(|| Error::Storage("Invalid created_at timestamp".to_string()))?,
            updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                .ok_or_else(|| Error::Storage("Invalid updated_at timestamp".to_string()))?,
        })
    }

    /// Helper: Parse task type from string
    fn parse_task_type(&self, s: &str) -> Result<TaskType> {
        match s {
            "code_generation" => Ok(TaskType::CodeGeneration),
            "debugging" => Ok(TaskType::Debugging),
            "refactoring" => Ok(TaskType::Refactoring),
            "testing" => Ok(TaskType::Testing),
            "analysis" => Ok(TaskType::Analysis),
            "documentation" => Ok(TaskType::Documentation),
            "other" => Ok(TaskType::Other),
            _ => Err(Error::Storage(format!("Unknown task type: {}", s))),
        }
    }

    // ======= Monitoring Storage Methods =======

    /// Store an execution record for monitoring
    pub async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()> {
        debug!("Storing execution record for agent: {}", record.agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT INTO execution_records (
                agent_name, agent_type, success, duration_ms,
                started_at, task_description, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                record.agent_name.clone(),
                record.agent_type.to_string(),
                record.success,
                record.duration.as_millis() as i64,
                record.started_at.timestamp(),
                record.task_description.clone(),
                record.error_message.clone(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store execution record: {}", e)))?;

        info!(
            "Successfully stored execution record for agent: {}",
            record.agent_name
        );
        Ok(())
    }

    /// Store aggregated agent metrics
    pub async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()> {
        debug!("Storing agent metrics for: {}", metrics.agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO agent_metrics (
                agent_name, agent_type, total_executions, successful_executions,
                total_duration_ms, avg_duration_ms, min_duration_ms, max_duration_ms,
                last_execution, current_streak, longest_streak, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                metrics.agent_name.clone(),
                metrics.agent_type.to_string(),
                metrics.total_executions as i64,
                metrics.successful_executions as i64,
                metrics.total_duration.as_millis() as i64,
                metrics.avg_duration.as_millis() as i64,
                metrics.min_duration.as_millis() as i64,
                metrics.max_duration.as_millis() as i64,
                metrics.last_execution.map(|t| t.timestamp()),
                metrics.current_streak as i64,
                metrics.longest_streak as i64,
                chrono::Utc::now().timestamp(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store agent metrics: {}", e)))?;

        info!(
            "Successfully stored agent metrics for: {}",
            metrics.agent_name
        );
        Ok(())
    }

    /// Store task metrics
    pub async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()> {
        debug!("Storing task metrics for: {}", metrics.task_type);
        let conn = self.get_connection().await?;

        let agent_success_rates_json =
            serde_json::to_string(&metrics.agent_success_rates).map_err(Error::Serialization)?;

        let sql = r#"
            INSERT OR REPLACE INTO task_metrics (
                task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                agent_success_rates, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                metrics.task_type.clone(),
                metrics.total_tasks as i64,
                metrics.completed_tasks as i64,
                metrics.avg_completion_time.as_millis() as i64,
                agent_success_rates_json,
                chrono::Utc::now().timestamp(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store task metrics: {}", e)))?;

        info!(
            "Successfully stored task metrics for: {}",
            metrics.task_type
        );
        Ok(())
    }

    /// Load agent metrics by name
    pub async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>> {
        debug!("Loading agent metrics for: {}", agent_name);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT agent_name, agent_type, total_executions, successful_executions,
                   total_duration_ms, avg_duration_ms, min_duration_ms, max_duration_ms,
                   last_execution, current_streak, longest_streak
            FROM agent_metrics WHERE agent_name = ?
        "#;

        let mut rows = conn
            .query(sql, params![agent_name])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query agent metrics: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch agent metrics row: {}", e)))?
        {
            let metrics = self.row_to_agent_metrics(&row)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }

    /// Load execution records with optional filtering
    pub async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        debug!("Loading execution records (limit: {})", limit);
        let conn = self.get_connection().await?;

        let mut sql = String::from(
            r#"
            SELECT agent_name, agent_type, success, duration_ms, started_at,
                   task_description, error_message
            FROM execution_records WHERE 1=1
        "#,
        );

        let mut params_vec = Vec::new();

        if let Some(agent) = agent_name {
            sql.push_str(" AND agent_name = ?");
            params_vec.push(agent.to_string());
        }

        sql.push_str(" ORDER BY started_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        let mut rows = conn
            .query(&sql, libsql::params_from_iter(params_vec))
            .await
            .map_err(|e| Error::Storage(format!("Failed to query execution records: {}", e)))?;

        let mut records = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch execution record row: {}", e)))?
        {
            records.push(self.row_to_execution_record(&row)?);
        }

        info!("Found {} execution records", records.len());
        Ok(records)
    }

    /// Load task metrics by task type
    pub async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>> {
        debug!("Loading task metrics for: {}", task_type);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT task_type, total_tasks, completed_tasks, avg_completion_time_ms,
                   agent_success_rates
            FROM task_metrics WHERE task_type = ?
        "#;

        let mut rows = conn
            .query(sql, params![task_type])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query task metrics: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch task metrics row: {}", e)))?
        {
            let metrics = self.row_to_task_metrics(&row)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }

    /// Helper: Convert row to AgentMetrics
    fn row_to_agent_metrics(&self, row: &libsql::Row) -> Result<AgentMetrics> {
        let agent_name: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get agent_name: {}", e)))?;
        let agent_type_str: String = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get agent_type: {}", e)))?;
        let total_executions: i64 = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get total_executions: {}", e)))?;
        let successful_executions: i64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get successful_executions: {}", e)))?;
        let total_duration_ms: i64 = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get total_duration_ms: {}", e)))?;
        let avg_duration_ms: i64 = row
            .get(5)
            .map_err(|e| Error::Storage(format!("Failed to get avg_duration_ms: {}", e)))?;
        let min_duration_ms: i64 = row
            .get(6)
            .map_err(|e| Error::Storage(format!("Failed to get min_duration_ms: {}", e)))?;
        let max_duration_ms: i64 = row
            .get(7)
            .map_err(|e| Error::Storage(format!("Failed to get max_duration_ms: {}", e)))?;
        let last_execution: Option<i64> = row
            .get(8)
            .map_err(|e| Error::Storage(format!("Failed to get last_execution: {}", e)))?;
        let current_streak: i64 = row
            .get(9)
            .map_err(|e| Error::Storage(format!("Failed to get current_streak: {}", e)))?;
        let longest_streak: i64 = row
            .get(10)
            .map_err(|e| Error::Storage(format!("Failed to get longest_streak: {}", e)))?;

        Ok(AgentMetrics {
            agent_name,
            agent_type: AgentType::from(agent_type_str.as_str()),
            total_executions: total_executions as u64,
            successful_executions: successful_executions as u64,
            total_duration: std::time::Duration::from_millis(total_duration_ms as u64),
            avg_duration: std::time::Duration::from_millis(avg_duration_ms as u64),
            min_duration: std::time::Duration::from_millis(min_duration_ms as u64),
            max_duration: std::time::Duration::from_millis(max_duration_ms as u64),
            last_execution: last_execution.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            current_streak: current_streak as u32,
            longest_streak: longest_streak as u32,
        })
    }

    /// Helper: Convert row to ExecutionRecord
    fn row_to_execution_record(&self, row: &libsql::Row) -> Result<ExecutionRecord> {
        let agent_name: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get agent_name: {}", e)))?;
        let agent_type_str: String = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get agent_type: {}", e)))?;
        let success: bool = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get success: {}", e)))?;
        let duration_ms: i64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get duration_ms: {}", e)))?;
        let started_at: i64 = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get started_at: {}", e)))?;
        let task_description: Option<String> = row
            .get(5)
            .map_err(|e| Error::Storage(format!("Failed to get task_description: {}", e)))?;
        let error_message: Option<String> = row
            .get(6)
            .map_err(|e| Error::Storage(format!("Failed to get error_message: {}", e)))?;

        Ok(ExecutionRecord {
            agent_name,
            agent_type: AgentType::from(agent_type_str.as_str()),
            success,
            duration: std::time::Duration::from_millis(duration_ms as u64),
            started_at: chrono::DateTime::from_timestamp(started_at, 0)
                .ok_or_else(|| Error::Storage("Invalid started_at timestamp".to_string()))?,
            task_description,
            error_message,
        })
    }

    /// Helper: Convert row to TaskMetrics
    fn row_to_task_metrics(&self, row: &libsql::Row) -> Result<TaskMetrics> {
        let task_type: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get task_type: {}", e)))?;
        let total_tasks: i64 = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get total_tasks: {}", e)))?;
        let completed_tasks: i64 = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get completed_tasks: {}", e)))?;
        let avg_completion_time_ms: i64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get avg_completion_time_ms: {}", e)))?;
        let agent_success_rates_json: String = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get agent_success_rates: {}", e)))?;

        let agent_success_rates: HashMap<AgentType, f64> =
            serde_json::from_str(&agent_success_rates_json).map_err(Error::Serialization)?;

        Ok(TaskMetrics {
            task_type,
            total_tasks: total_tasks as u64,
            completed_tasks: completed_tasks as u64,
            avg_completion_time: std::time::Duration::from_millis(avg_completion_time_ms as u64),
            agent_success_rates,
        })
    }

    /// Helper: Convert row to PatternMetadata
    fn row_to_pattern_metadata(&self, row: &libsql::Row) -> Result<PatternMetadata> {
        let pattern_id_str: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get pattern_id: {}", e)))?;
        let pattern_type: String = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get pattern_type: {}", e)))?;
        let success_rate: f64 = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get success_rate: {}", e)))?;
        let occurrence_count: i64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get occurrence_count: {}", e)))?;
        let created_at: i64 = row
            .get(4)
            .map_err(|e| Error::Storage(format!("Failed to get created_at: {}", e)))?;
        let updated_at: i64 = row
            .get(5)
            .map_err(|e| Error::Storage(format!("Failed to get updated_at: {}", e)))?;

        let pattern_id = PatternId::parse_str(&pattern_id_str)
            .map_err(|e| Error::Storage(format!("Invalid pattern UUID: {}", e)))?;

        Ok(PatternMetadata {
            pattern_id,
            pattern_type,
            success_rate: success_rate as f32,
            occurrence_count: occurrence_count as usize,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .ok_or_else(|| Error::Storage("Invalid created_at timestamp".to_string()))?,
            updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                .ok_or_else(|| Error::Storage("Invalid updated_at timestamp".to_string()))?,
        })
    }

    /// Store an embedding in the database
    async fn store_embedding(&self, item_id: &str, item_type: &str, embedding: &[f32]) -> Result<()> {
        debug!("Storing embedding for {}: {}", item_type, item_id);

        let conn = self.get_connection().await?;
        let embedding_json = serde_json::to_string(embedding).map_err(Error::Serialization)?;
        let embedding_id = format!("{}_{}", item_type, item_id);

        let sql = r#"
            INSERT OR REPLACE INTO embeddings (
                embedding_id, item_id, item_type, embedding_data, dimension, model
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                embedding_id,
                item_id,
                item_type,
                embedding_json,
                embedding.len() as i64,
                "unknown",
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store embedding: {}", e)))?;

        Ok(())
    }

    /// Retrieve an embedding from the database
    async fn get_embedding(&self, item_id: &str, item_type: &str) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving embedding for {}: {}", item_type, item_id);

        let conn = self.get_connection().await?;
        let embedding_id = format!("{}_{}", item_type, item_id);

        let sql = "SELECT embedding_data FROM embeddings WHERE embedding_id = ?";

        let mut rows = conn
            .query(sql, params![embedding_id])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query embedding: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
        {
            let embedding_json: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get embedding_data: {}", e)))?;
            let embedding: Vec<f32> =
                serde_json::from_str(&embedding_json).map_err(Error::Serialization)?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl EmbeddingStorageBackend for TursoStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing episode embedding: {}", episode_id);
        self.store_embedding(&episode_id.to_string(), "episode", &embedding)
            .await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        debug!("Storing pattern embedding: {}", pattern_id);
        self.store_embedding(&pattern_id.to_string(), "pattern", &embedding)
            .await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving episode embedding: {}", episode_id);
        self.get_embedding(&episode_id.to_string(), "episode")
            .await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving pattern embedding: {}", pattern_id);
        self.get_embedding(&pattern_id.to_string(), "pattern")
            .await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes (limit: {}, threshold: {})",
            limit, threshold
        );

        let conn = self.get_connection().await?;

        // First, get all episode embeddings
        let sql = r#"
            SELECT item_id, embedding_data
            FROM embeddings
            WHERE item_type = 'episode'
        "#;

        let mut rows = conn
            .query(sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to query embeddings: {}", e)))?;

        let mut candidates: Vec<(String, Vec<f32>)> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
        {
            let item_id: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get item_id: {}", e)))?;
            let embedding_json: String = row
                .get(1)
                .map_err(|e| Error::Storage(format!("Failed to get embedding_data: {}", e)))?;
            let embedding: Vec<f32> =
                serde_json::from_str(&embedding_json).map_err(Error::Serialization)?;

            candidates.push((item_id, embedding));
        }

        // Calculate similarities and filter
        let mut results = Vec::new();
        for (episode_id_str, embedding) in candidates {
            let similarity = cosine_similarity(&query_embedding, &embedding);

            if similarity >= threshold {
                // Try to get the episode
                if let Ok(episode_id) = Uuid::parse_str(&episode_id_str) {
                    if let Ok(Some(episode)) = self.get_episode(episode_id).await {
                        results.push(SimilaritySearchResult {
                            item: episode,
                            similarity,
                            metadata: SimilarityMetadata {
                                embedding_model: "unknown".to_string(),
                                embedding_timestamp: None,
                                context: serde_json::json!({}),
                            },
                        });
                    }
                }
            }
        }

        // Sort by similarity (highest first)
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        info!("Found {} similar episodes", results.len());
        Ok(results)
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns (limit: {}, threshold: {})",
            limit, threshold
        );

        let conn = self.get_connection().await?;

        // First, get all pattern embeddings
        let sql = r#"
            SELECT item_id, embedding_data
            FROM embeddings
            WHERE item_type = 'pattern'
        "#;

        let mut rows = conn
            .query(sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to query embeddings: {}", e)))?;

        let mut candidates: Vec<(String, Vec<f32>)> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
        {
            let item_id: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to get item_id: {}", e)))?;
            let embedding_json: String = row
                .get(1)
                .map_err(|e| Error::Storage(format!("Failed to get embedding_data: {}", e)))?;
            let embedding: Vec<f32> =
                serde_json::from_str(&embedding_json).map_err(Error::Serialization)?;

            candidates.push((item_id, embedding));
        }

        // Calculate similarities and filter
        let mut results = Vec::new();
        for (pattern_id_str, embedding) in candidates {
            let similarity = cosine_similarity(&query_embedding, &embedding);

            if similarity >= threshold {
                // Try to get the pattern
                if let Ok(pattern_id) = PatternId::parse_str(&pattern_id_str) {
                    if let Ok(Some(pattern)) = self.get_pattern(pattern_id).await {
                        results.push(SimilaritySearchResult {
                            item: pattern,
                            similarity,
                            metadata: SimilarityMetadata {
                                embedding_model: "unknown".to_string(),
                                embedding_timestamp: None,
                                context: serde_json::json!({}),
                            },
                        });
                    }
                }
            }
        }

        // Sort by similarity (highest first)
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        info!("Found {} similar patterns", results.len());
        Ok(results)
    }
}
