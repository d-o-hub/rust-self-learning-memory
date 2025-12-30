//! Episode CRUD operations for Turso storage

use crate::TursoStorage;
use memory_core::{Episode, Error, Result, TaskType};
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
            libsql::params![
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
                   reflection, patterns, heuristics, metadata, domain, language
            FROM episodes WHERE episode_id = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episode: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            let episode = Self::row_to_episode(&row)?;
            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }

    /// Delete an episode by ID
    pub async fn delete_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Deleting episode: {}", episode_id);
        let conn = self.get_connection().await?;

        let sql = "DELETE FROM episodes WHERE episode_id = ?";

        conn.execute(sql, libsql::params![episode_id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;

        info!("Successfully deleted episode: {}", episode_id);
        Ok(())
    }

    /// Query episodes with filters
    pub async fn query_episodes(&self, query: &super::EpisodeQuery) -> Result<Vec<Episode>> {
        debug!("Querying episodes with filters: {:?}", query);
        let conn = self.get_connection().await?;

        let mut sql = String::from(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language
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
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!("Found {} episodes matching query", episodes.len());
        Ok(episodes)
    }

    /// Query episodes modified since a given timestamp
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>> {
        debug!("Querying episodes since {}", since);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language
            FROM episodes
            WHERE start_time >= ?
            ORDER BY start_time DESC
        "#;

        let since_timestamp = since.timestamp();

        let mut rows = conn
            .query(sql, libsql::params![since_timestamp])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!("Found {} episodes modified since {}", episodes.len(), since);
        Ok(episodes)
    }

    /// Query episodes by metadata key-value pair
    pub async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
        debug!("Querying episodes by metadata {} = {}", key, value);
        let conn = self.get_connection().await?;

        let sql = format!(
            r#"
            SELECT episode_id, task_type, task_description, context,
                   start_time, end_time, steps, outcome, reward,
                   reflection, patterns, heuristics, metadata, domain, language
            FROM episodes
            WHERE metadata LIKE '%"{}": "{}%'
            ORDER BY start_time DESC
        "#,
            key, value
        );

        let mut rows = conn
            .query(&sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

        let mut episodes = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch episode row: {}", e)))?
        {
            episodes.push(Self::row_to_episode(&row)?);
        }

        info!(
            "Found {} episodes with metadata {} = {}",
            episodes.len(),
            key,
            value
        );
        Ok(episodes)
    }
}

/// Convert a database row to an Episode
impl TursoStorage {
    pub(crate) fn row_to_episode(row: &libsql::Row) -> Result<Episode> {
        let episode_id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
        let task_type: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
        let task_description: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
        let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
        let start_time_timestamp: i64 = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
        let end_time_timestamp: Option<i64> = row.get(5).ok();
        let steps_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
        let outcome_json: Option<String> = row.get(7).ok();
        let reward_json: Option<String> = row.get(8).ok();
        let reflection_json: Option<String> = row.get(9).ok();
        let patterns_json: String = row.get(10).map_err(|e| Error::Storage(e.to_string()))?;
        let heuristics_json: String = row.get(11).map_err(|e| Error::Storage(e.to_string()))?;
        let metadata_json: String = row.get(12).map_err(|e| Error::Storage(e.to_string()))?;
        let _domain: String = row.get(13).map_err(|e| Error::Storage(e.to_string()))?;
        let _language: Option<String> = row.get(14).ok();

        let context: memory_core::TaskContext = serde_json::from_str(&context_json)
            .map_err(|e| Error::Storage(format!("Failed to parse context: {}", e)))?;
        let steps: Vec<memory_core::episode::ExecutionStep> = serde_json::from_str(&steps_json)
            .map_err(|e| Error::Storage(format!("Failed to parse steps: {}", e)))?;
        let outcome = outcome_json
            .map(|s| serde_json::from_str::<memory_core::TaskOutcome>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse outcome: {}", e)))?;
        let reward = reward_json
            .map(|s| serde_json::from_str::<memory_core::types::RewardScore>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse reward: {}", e)))?;
        let reflection = reflection_json
            .map(|s| serde_json::from_str::<memory_core::Reflection>(&s))
            .transpose()
            .map_err(|e| Error::Storage(format!("Failed to parse reflection: {}", e)))?;
        let patterns: Vec<memory_core::episode::PatternId> =
            serde_json::from_str(&patterns_json)
                .map_err(|e| Error::Storage(format!("Failed to parse patterns: {}", e)))?;
        let heuristics: Vec<Uuid> = serde_json::from_str(&heuristics_json)
            .map_err(|e| Error::Storage(format!("Failed to parse heuristics: {}", e)))?;
        let metadata: std::collections::HashMap<String, String> =
            serde_json::from_str(&metadata_json)
                .map_err(|e| Error::Storage(format!("Failed to parse metadata: {}", e)))?;

        Ok(Episode {
            episode_id: uuid::Uuid::parse_str(&episode_id)
                .map_err(|e| Error::Storage(format!("Invalid episode ID: {}", e)))?,
            task_type: task_type
                .parse::<TaskType>()
                .map_err(|e| Error::Storage(e.to_string()))?,
            task_description,
            context,
            steps,
            outcome,
            reward,
            reflection,
            patterns,
            heuristics,
            applied_patterns: Vec::new(),
            salient_features: None,
            start_time: chrono::DateTime::from_timestamp(start_time_timestamp, 0)
                .unwrap_or_default(),
            end_time: end_time_timestamp.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
            metadata,
        })
    }
}
