use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use uuid::Uuid;

impl crate::DuckDbStorage {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn map_row_to_episode(
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
        applied_patterns_json: Option<String>,
        salient_features_json: Option<String>,
        checkpoints_json: &str,
        metadata_json: &str,
        tags: Vec<String>,
    ) -> Result<do_memory_core::Episode> {
        let episode = do_memory_core::Episode {
            episode_id: Uuid::parse_str(episode_id_str)
                .map_err(|e| Error::Storage(format!("episode_id parse: {e}")))?,
            task_type: task_type_str
                .parse()
                .map_err(|e| Error::Storage(format!("task_type parse: {e}")))?,
            task_description: task_desc,
            context: serde_json::from_str(context_json)
                .map_err(|e| Error::Storage(format!("context parse: {e}")))?,
            start_time: DateTime::parse_from_rfc3339(start_time_str)
                .or_else(|_| DateTime::parse_from_str(start_time_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                .map_err(|e| Error::Storage(format!("start_time parse: {e}")))?
                .with_timezone(&Utc),
            end_time: end_time_str
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map(|t| t.with_timezone(&Utc))
                        .map_err(|e| Error::Storage(format!("end_time parse: {e}")))
                })
                .transpose()?,
            steps: serde_json::from_str(steps_json)
                .map_err(|e| Error::Storage(format!("steps parse: {e}")))?,
            outcome: outcome_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("outcome parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            reward: reward_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("reward parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            reflection: reflection_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("reflection parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            patterns: serde_json::from_str(patterns_json)
                .map_err(|e| Error::Storage(format!("patterns parse: {e}")))?,
            heuristics: serde_json::from_str(heuristics_json)
                .map_err(|e| Error::Storage(format!("heuristics parse: {e}")))?,
            applied_patterns: applied_patterns_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(Vec::new())
                    } else {
                        serde_json::from_str(&s)
                            .map_err(|e| Error::Storage(format!("applied_patterns parse: {e}")))
                    }
                })
                .transpose()?
                .unwrap_or_default(),
            salient_features: salient_features_json
                .map(|s| {
                    if s == "null" || s.is_empty() {
                        Ok(None)
                    } else {
                        serde_json::from_str(&s)
                            .map(Some)
                            .map_err(|e| Error::Storage(format!("salient_features parse: {e}")))
                    }
                })
                .transpose()?
                .flatten(),
            checkpoints: serde_json::from_str(checkpoints_json)
                .map_err(|e| Error::Storage(format!("checkpoints parse: {e}")))?,
            metadata: serde_json::from_str(metadata_json)
                .map_err(|e| Error::Storage(format!("metadata parse: {e}")))?,
            tags,
        };
        Ok(episode)
    }
}
