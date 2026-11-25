//! Episode retrieval and deletion tools for MCP server
//!
//! This module provides tools for getting episode details and deleting
//! episodes permanently.

use crate::server::MemoryMCPServer;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use tracing::debug;
use tracing::warn;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Get episode details by ID
    ///
    /// This tool retrieves complete details of an episode including
    /// all steps, outcome, reflection, and extracted patterns.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to retrieve
    /// * `fields` - Optional array of field paths to return (e.g., ["episode.id", "episode.task_description"])
    ///
    /// # Field Selection
    ///
    /// Clients can request specific fields to reduce token usage:
    /// ```json
    /// {
    ///   "episode_id": "123e4567-e89b-12d3-a456-426614174000",
    ///   "fields": ["episode.id", "episode.task_description", "episode.outcome"]
    /// }
    /// ```
    pub async fn get_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting episode with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        let episode = self
            .memory
            .get_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to get episode: {}", e))?;

        // Convert episode to JSON
        let episode_json = serde_json::to_value(&episode)
            .map_err(|e| anyhow!("Failed to serialize episode: {}", e))?;

        let summary_json = self
            .memory
            .get_episode_summary(episode_id)
            .await
            .ok()
            .flatten()
            .and_then(|s| serde_json::to_value(s).ok());

        let result = json!({
            "success": true,
            "episode": episode_json,
            "summary": summary_json
        });

        // Apply field projection if requested
        if let Some(fields_value) = args.get("fields") {
            if let Some(field_array) = fields_value.as_array() {
                let field_list: Vec<String> = field_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if !field_list.is_empty() {
                    use crate::server::tools::field_projection::FieldSelector;
                    let selector = FieldSelector::new(field_list.into_iter().collect());
                    return selector.apply(&result);
                }
            }
        }

        Ok(result)
    }

    /// Delete an episode permanently
    ///
    /// This tool removes an episode from all storage backends.
    /// **Warning**: This operation cannot be undone.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to delete
    /// * `confirm` - Must be set to true to confirm deletion
    pub async fn delete_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Deleting episode with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        // Require explicit confirmation
        let confirm = args
            .get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(anyhow!(
                "Deletion requires explicit confirmation. Set 'confirm' to true."
            ));
        }

        self.memory
            .delete_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to delete episode: {}", e))?;

        warn!(
            episode_id = %episode_id,
            "Deleted episode via MCP (permanent)"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "message": "Episode deleted permanently"
        }))
    }

    /// Clean up episodes according to a retention policy (`WG-075`).
    ///
    /// Defaults to `dry_run: true` for safety; live deletion (`dry_run: false`)
    /// requires `confirm: true`.
    pub async fn cleanup_episodes_tool(&self, args: Value) -> Result<Value> {
        use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};

        debug!("Cleaning up episodes with args: {}", args);

        let mut policy = match args.get("preset").and_then(|v| v.as_str()) {
            None | Some("default") => EpisodeRetentionPolicy::default(),
            Some("storage_limited" | "storage-limited") => {
                EpisodeRetentionPolicy::storage_limited()
            }
            Some("aggressive") => EpisodeRetentionPolicy::aggressive(),
            Some("archival") => EpisodeRetentionPolicy::archival(),
            Some(other) => {
                return Err(anyhow!(
                    "Invalid preset '{other}'. Expected one of: default, storage_limited, aggressive, archival"
                ));
            }
        };

        if let Some(crit_str) = args.get("criterion").and_then(|v| v.as_str()) {
            let criterion = match crit_str {
                "combined" => RetentionCriterion::Combined,
                "age_based" | "age-based" => RetentionCriterion::AgeBased,
                "reward_based" | "reward-based" => RetentionCriterion::RewardBased,
                "unreferenced" => RetentionCriterion::Unreferenced,
                "failed_only" | "failed-only" => RetentionCriterion::FailedOnly,
                "keep_all" | "keep-all" => RetentionCriterion::KeepAll,
                other => {
                    return Err(anyhow!(
                        "Invalid criterion '{other}'. Expected one of: combined, age_based, reward_based, unreferenced, failed_only, keep_all"
                    ));
                }
            };
            policy = policy.with_criterion(criterion);
        }

        if let Some(days) = args.get("max_age_days").and_then(|v| v.as_i64()) {
            policy = policy.with_max_age_days(days);
        }
        if let Some(threshold) = args.get("min_reward_threshold").and_then(|v| v.as_f64()) {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(anyhow!(
                    "min_reward_threshold must be between 0.0 and 1.0 (got {threshold})"
                ));
            }
            policy = policy.with_min_reward_threshold(threshold);
        }
        if let Some(max_ep) = args.get("max_episodes").and_then(|v| v.as_u64()) {
            policy = policy.with_max_episodes(max_ep as usize);
        }
        if let Some(batch) = args.get("cleanup_batch_size").and_then(|v| v.as_u64()) {
            if batch == 0 {
                return Err(anyhow!("cleanup_batch_size must be greater than 0"));
            }
            policy = policy.with_cleanup_batch_size(batch as usize);
        }
        if let Some(keep_patterns) = args.get("keep_pattern_sources").and_then(|v| v.as_bool()) {
            policy.keep_pattern_sources = keep_patterns;
        }
        if let Some(keep_heuristics) = args.get("keep_heuristic_sources").and_then(|v| v.as_bool())
        {
            policy.keep_heuristic_sources = keep_heuristics;
        }

        let count_only = args
            .get("count_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let dry_run = args
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let confirm = args
            .get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !dry_run && !count_only && !confirm {
            return Err(anyhow!(
                "Live episode cleanup requires explicit confirmation. Set 'confirm' to true or use 'dry_run': true."
            ));
        }

        policy = policy.with_dry_run(dry_run);
        policy
            .validate()
            .map_err(|e| anyhow!("Invalid retention policy: {e}"))?;

        if count_only {
            let count = self
                .memory
                .count_cleanup_candidates(&policy)
                .await
                .map_err(|e| anyhow!("Failed to count cleanup candidates: {e}"))?;
            return Ok(json!({
                "success": true,
                "count_only": true,
                "dry_run": true,
                "candidate_count": count
            }));
        }

        let result = self
            .memory
            .cleanup_episodes(&policy)
            .await
            .map_err(|e| anyhow!("Failed to clean up episodes: {e}"))?;

        Ok(json!({
            "success": true,
            "count_only": false,
            "dry_run": dry_run,
            "evaluated": result.evaluated,
            "deleted": result.deleted,
            "kept": result.kept,
            "deleted_ids": result.deleted_ids,
            "kept_ids": result.kept_ids,
            "duration_ms": result.duration_ms,
            "errors": result.errors
        }))
    }
}
