//! Batch episode comparison operations
//!
//! This module provides tools for comparing multiple episodes to identify
//! differences in approach, performance, and outcomes.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{Episode, TaskOutcome};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Generate insights from episode comparison data
pub fn generate_comparison_insights(episodes: &[Episode], metrics: &[String]) -> Value {
    let mut insights = Vec::new();

    // Find best and worst performers
    if metrics.contains(&"reward_score".to_string()) {
        let mut with_rewards: Vec<_> = episodes
            .iter()
            .filter_map(|e| e.reward.as_ref().map(|r| (e, r.total)))
            .collect();
        with_rewards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let Some((best, score)) = with_rewards.first() {
            insights.push(json!({
                "type": "best_performer",
                "episode_id": best.episode_id,
                "metric": "reward_score",
                "value": score,
                "insight": format!("Episode achieved highest reward score of {:.2}", score)
            }));
        }
    }

    // Analyze duration patterns
    if metrics.contains(&"duration".to_string()) {
        let durations: Vec<_> = episodes
            .iter()
            .filter_map(|e| {
                e.end_time
                    .map(|end| ((end - e.start_time).num_seconds() as f64, e))
            })
            .collect();

        if !durations.is_empty() {
            let avg_duration =
                durations.iter().map(|(d, _)| d).sum::<f64>() / durations.len() as f64;
            let fastest = durations
                .iter()
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            if let Some((duration, episode)) = fastest {
                insights.push(json!({
                    "type": "efficiency",
                    "episode_id": episode.episode_id,
                    "metric": "duration",
                    "value": duration,
                    "comparison": format!("{:.1}% faster than average", ((avg_duration - duration) / avg_duration * 100.0))
                }));
            }
        }
    }

    json!(insights)
}

/// Compare approaches used across episodes
pub fn compare_episode_approaches(episodes: &[Episode]) -> Value {
    let mut tool_usage: HashMap<String, usize> = HashMap::new();
    let mut step_patterns: HashMap<String, Vec<Uuid>> = HashMap::new();

    for episode in episodes {
        // Collect tool usage
        for step in &episode.steps {
            *tool_usage.entry(step.tool.clone()).or_insert(0) += 1;
        }

        // Collect step patterns
        let pattern = episode
            .steps
            .iter()
            .map(|s| s.tool.as_str())
            .collect::<Vec<_>>()
            .join(" → ");
        step_patterns
            .entry(pattern)
            .or_default()
            .push(episode.episode_id);
    }

    let common_patterns: Vec<_> = step_patterns
        .iter()
        .filter(|(_, eps)| eps.len() > 1)
        .map(|(pattern, eps)| {
            json!({
                "pattern": pattern,
                "used_by": eps.len(),
                "episodes": eps
            })
        })
        .collect();

    json!({
        "tool_usage": tool_usage,
        "common_patterns": common_patterns,
        "unique_approaches": step_patterns.len()
    })
}

impl MemoryMCPServer {
    /// Batch compare episodes for insights
    ///
    /// This tool compares multiple episodes to identify differences in
    /// approach, performance, and outcomes. Useful for understanding why
    /// some episodes succeed while others fail.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_ids` - Array of episode UUIDs to compare (2-10 episodes)
    /// * `compare_metrics` - Metrics to compare: duration, reward_score, step_count, efficiency
    /// * `compare_approaches` - Compare execution approaches (default: true)
    /// * `generate_insights` - Generate AI insights (default: true)
    ///
    /// # Returns
    ///
    /// Returns comparison data with insights and recommendations
    pub async fn batch_compare_episodes_tool(&self, args: Value) -> Result<Value> {
        debug!("Batch comparing episodes with args: {}", args);

        let start = std::time::Instant::now();

        // Parse episode IDs
        let episode_ids = args
            .get("episode_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing required field: episode_ids"))?
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect::<Vec<_>>();

        if episode_ids.len() < 2 {
            return Err(anyhow!("At least 2 episode IDs required for comparison"));
        }

        if episode_ids.len() > 10 {
            return Err(anyhow!("Maximum 10 episodes can be compared at once"));
        }

        // Parse options
        let compare_metrics = args
            .get("compare_metrics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![
                    "duration".to_string(),
                    "reward_score".to_string(),
                    "step_count".to_string(),
                ]
            });

        let compare_approaches = args
            .get("compare_approaches")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let generate_insights = args
            .get("generate_insights")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Retrieve episodes
        let episodes = self.memory.get_episodes_by_ids(&episode_ids).await?;

        if episodes.len() < episode_ids.len() {
            return Err(anyhow!(
                "Some episodes not found. Found {} out of {}",
                episodes.len(),
                episode_ids.len()
            ));
        }

        // Build comparison data
        let mut comparisons = Vec::new();

        for episode in &episodes {
            let duration_seconds = episode
                .end_time
                .map(|end| (end - episode.start_time).num_seconds() as f64);

            let reward_score = episode.reward.as_ref().map(|r| r.total);

            let step_count = episode.steps.len();

            let efficiency =
                duration_seconds.and_then(|d| reward_score.map(|r| (r as f64) / (d + 1.0)));

            let outcome_type = match &episode.outcome {
                Some(TaskOutcome::Success { .. }) => "success",
                Some(TaskOutcome::PartialSuccess { .. }) => "partial_success",
                Some(TaskOutcome::Failure { .. }) => "failure",
                None => "incomplete",
            };

            comparisons.push(json!({
                "episode_id": episode.episode_id,
                "task_description": episode.task_description,
                "metrics": {
                    "duration_seconds": duration_seconds,
                    "reward_score": reward_score,
                    "step_count": step_count,
                    "efficiency": efficiency
                },
                "outcome": outcome_type,
                "context": {
                    "domain": episode.context.domain,
                    "language": episode.context.language,
                    "framework": episode.context.framework,
                    "complexity": format!("{:?}", episode.context.complexity)
                }
            }));
        }

        // Generate insights if requested
        let insights = if generate_insights {
            Some(generate_comparison_insights(&episodes, &compare_metrics))
        } else {
            None
        };

        // Compare approaches if requested
        let approach_comparison = if compare_approaches {
            Some(compare_episode_approaches(&episodes))
        } else {
            None
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            episodes_compared = episodes.len(),
            duration_ms = duration_ms,
            "Episode comparison completed"
        );

        Ok(json!({
            "success": true,
            "episodes_compared": episodes.len(),
            "comparisons": comparisons,
            "insights": insights,
            "approach_comparison": approach_comparison,
            "performance": {
                "duration_ms": duration_ms
            }
        }))
    }
}
