//! Batch query operations for episodes
//!
//! This module provides efficient bulk retrieval of episodes with optional
//! filtering, inclusion of related data (steps, patterns, reflections),
//! and automatic aggregation of statistics.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{Episode, EpisodeFilter, Pattern, TaskOutcome};
use serde_json::{json, Value};
use tracing::{debug, info};
use uuid::Uuid;

/// Helper function to parse episode filter from JSON value
pub fn parse_episode_filter(filter_obj: &Value) -> Result<EpisodeFilter> {
    let mut builder = EpisodeFilter::builder();

    if let Some(domain) = filter_obj.get("domain").and_then(|v| v.as_str()) {
        builder = builder.domains(vec![domain.to_string()]);
    }

    if let Some(task_type) = filter_obj.get("task_type").and_then(|v| v.as_str()) {
        if let Ok(tt) = parse_task_type(task_type) {
            builder = builder.task_types(vec![tt]);
        }
    }

    if let Some(tags) = filter_obj.get("tags").and_then(|v| v.as_array()) {
        let tag_strs: Vec<String> = tags
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        builder = builder.with_any_tags(tag_strs);
    }

    if let Some(success_only) = filter_obj.get("success_only").and_then(|v| v.as_bool()) {
        builder = builder.success_only(success_only);
    }

    Ok(builder.build())
}

/// Helper function to parse task type string to enum
pub fn parse_task_type(task_type: &str) -> Result<memory_core::TaskType> {
    match task_type {
        "code_generation" => Ok(memory_core::TaskType::CodeGeneration),
        "debugging" => Ok(memory_core::TaskType::Debugging),
        "refactoring" => Ok(memory_core::TaskType::Refactoring),
        "testing" => Ok(memory_core::TaskType::Testing),
        "analysis" => Ok(memory_core::TaskType::Analysis),
        "documentation" => Ok(memory_core::TaskType::Documentation),
        _ => Err(anyhow!("Invalid task_type: {}", task_type)),
    }
}

/// Compute aggregate statistics from episodes and patterns
pub fn compute_aggregate_stats(episodes: &[Episode], patterns: &[Pattern]) -> Value {
    let total = episodes.len();
    let completed = episodes.iter().filter(|e| e.outcome.is_some()).count();

    let successful = episodes
        .iter()
        .filter(|e| matches!(e.outcome, Some(TaskOutcome::Success { .. })))
        .count();

    let failed = episodes
        .iter()
        .filter(|e| matches!(e.outcome, Some(TaskOutcome::Failure { .. })))
        .count();

    let avg_duration = episodes
        .iter()
        .filter_map(|e| {
            e.end_time
                .map(|end| (end - e.start_time).num_seconds() as f64)
        })
        .sum::<f64>()
        / completed.max(1) as f64;

    let avg_reward: f64 = episodes
        .iter()
        .filter_map(|e| e.reward.as_ref().map(|r| r.total as f64))
        .sum::<f64>()
        / completed.max(1) as f64;

    let avg_steps =
        episodes.iter().map(|e| e.steps.len()).sum::<usize>() as f64 / total.max(1) as f64;

    json!({
        "total_episodes": total,
        "completed": completed,
        "successful": successful,
        "failed": failed,
        "success_rate": successful as f64 / completed.max(1) as f64,
        "avg_duration_seconds": avg_duration,
        "avg_reward_score": avg_reward,
        "avg_steps": avg_steps,
        "total_patterns": patterns.len()
    })
}

impl MemoryMCPServer {
    /// Batch query episodes with rich filtering and aggregation
    ///
    /// This tool provides efficient bulk retrieval of episodes with optional
    /// filtering, inclusion of related data (steps, patterns, reflections),
    /// and automatic aggregation of statistics.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_ids` - Optional array of specific episode UUIDs to retrieve
    /// * `filter` - Optional filter criteria (domain, task_type, tags, date_range, etc.)
    /// * `include_steps` - Include execution steps (default: true)
    /// * `include_patterns` - Include extracted patterns (default: false)
    /// * `include_reflections` - Include reflections (default: false)
    /// * `limit` - Maximum episodes to return (default: 100, max: 1000)
    /// * `offset` - Pagination offset (default: 0)
    /// * `aggregate_stats` - Compute aggregate statistics (default: true)
    ///
    /// # Returns
    ///
    /// Returns episodes with optional related data and statistics
    pub async fn batch_query_episodes_tool(&self, args: Value) -> Result<Value> {
        debug!("Batch querying episodes with args: {}", args);

        let start = std::time::Instant::now();

        // Parse episode IDs if provided
        let episode_ids: Option<Vec<Uuid>> = args
            .get("episode_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect::<Vec<_>>()
            });

        // Parse options
        let include_steps = args
            .get("include_steps")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let include_patterns = args
            .get("include_patterns")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let include_reflections = args
            .get("include_reflections")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100)
            .min(1000) as usize;

        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        let aggregate_stats = args
            .get("aggregate_stats")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Retrieve episodes
        let episodes = if let Some(ids) = episode_ids {
            // Specific episode IDs provided
            self.memory.get_episodes_by_ids(&ids).await?
        } else if let Some(filter_obj) = args.get("filter") {
            // Filter-based query
            let filter = parse_episode_filter(filter_obj)?;
            self.memory
                .list_episodes_filtered(filter, Some(limit), Some(offset))
                .await?
        } else {
            // No filter - get latest episodes
            self.memory
                .list_episodes(Some(limit), Some(offset), None)
                .await?
        };

        let episode_count = episodes.len();

        // Build episode data with requested inclusions
        let mut episode_data = Vec::new();
        let mut all_patterns = Vec::new();

        for episode in &episodes {
            let mut ep_json = serde_json::to_value(episode)?;

            // Optionally exclude steps
            if !include_steps {
                if let Some(obj) = ep_json.as_object_mut() {
                    obj.remove("steps");
                }
            }

            // Optionally exclude reflections
            if !include_reflections {
                if let Some(obj) = ep_json.as_object_mut() {
                    obj.remove("reflection");
                }
            }

            // Optionally include patterns
            if include_patterns {
                let patterns = self.memory.get_episode_patterns(episode.episode_id).await?;
                if let Some(obj) = ep_json.as_object_mut() {
                    obj.insert("patterns".to_string(), json!(patterns));
                }
                all_patterns.extend(patterns);
            }

            episode_data.push(ep_json);
        }

        // Compute aggregate statistics if requested
        let statistics = if aggregate_stats {
            Some(compute_aggregate_stats(&episodes, &all_patterns))
        } else {
            None
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            episodes_retrieved = episode_count,
            duration_ms = duration_ms,
            "Batch query completed"
        );

        Ok(json!({
            "success": true,
            "requested_count": if args.get("episode_ids").is_some() {
                args["episode_ids"].as_array().unwrap().len()
            } else {
                limit
            },
            "found_count": episode_count,
            "episodes": episode_data,
            "statistics": statistics,
            "performance": {
                "duration_ms": duration_ms,
                "episodes_per_second": (episode_count as f64 / (duration_ms as f64 / 1000.0)) as u64
            }
        }))
    }
}
