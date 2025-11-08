//! Storage operations for episodes, patterns, and heuristics

use crate::TursoStorage;
use libsql::params;
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result, TaskType};
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
                reflection, patterns, metadata, domain, language
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                   reflection, patterns, metadata
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
                   reflection, patterns, metadata
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
                   reflection, patterns, metadata
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
        let metadata_json: String = row
            .get(11)
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
}
