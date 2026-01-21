//! Advanced batch operation tools for MCP server
//!
//! This module provides high-performance batch tools for querying,
//! analyzing, and comparing multiple episodes efficiently.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{Episode, EpisodeFilter, Pattern, TaskOutcome, TaskType};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

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

    /// Batch pattern analysis across multiple episodes
    ///
    /// This tool analyzes patterns across a collection of episodes,
    /// identifying common sequences, successful approaches, and
    /// anti-patterns to avoid.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `domain` - Task domain to analyze
    /// * `task_type` - Optional specific task type
    /// * `time_range` - Optional time range {start, end} (ISO8601)
    /// * `min_episodes` - Minimum episodes a pattern must appear in (default: 3)
    /// * `min_success_rate` - Minimum success rate (0.0-1.0, default: 0.6)
    /// * `include_anti_patterns` - Include patterns with low success (default: true)
    /// * `limit` - Maximum patterns to return (default: 50)
    ///
    /// # Returns
    ///
    /// Returns discovered patterns with statistics and recommendations
    pub async fn batch_pattern_analysis_tool(&self, args: Value) -> Result<Value> {
        debug!("Batch analyzing patterns with args: {}", args);

        let start = std::time::Instant::now();

        // Parse required fields
        let domain = args
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: domain"))?
            .to_string();

        let task_type = args
            .get("task_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let min_episodes = args
            .get("min_episodes")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        let min_success_rate = args
            .get("min_success_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.6) as f32;

        let include_anti_patterns = args
            .get("include_anti_patterns")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

        // Build filter for episodes
        let mut filter_builder = EpisodeFilter::builder()
            .domains(vec![domain.clone()])
            .completed_only(true);

        if let Some(tt) = &task_type {
            if let Ok(task_type_enum) = parse_task_type(tt) {
                filter_builder = filter_builder.task_types(vec![task_type_enum]);
            }
        }

        // Parse time range if provided
        if let Some(time_range) = args.get("time_range") {
            if let Some(start_str) = time_range.get("start").and_then(|v| v.as_str()) {
                if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_str) {
                    filter_builder =
                        filter_builder.date_from(start_time.with_timezone(&chrono::Utc));
                }
            }
            if let Some(end_str) = time_range.get("end").and_then(|v| v.as_str()) {
                if let Ok(end_time) = chrono::DateTime::parse_from_rfc3339(end_str) {
                    filter_builder = filter_builder.date_to(end_time.with_timezone(&chrono::Utc));
                }
            }
        }

        let filter = filter_builder.build();

        // Retrieve episodes matching criteria
        let episodes = self
            .memory
            .list_episodes_filtered(filter, Some(1000), None)
            .await?;

        info!(
            "Analyzing patterns from {} episodes in domain '{}'",
            episodes.len(),
            domain
        );

        // Collect all patterns from these episodes
        let mut all_patterns = Vec::new();
        for episode in &episodes {
            let patterns = self.memory.get_episode_patterns(episode.episode_id).await?;
            all_patterns.extend(patterns);
        }

        // Group patterns by type and signature
        let pattern_groups = group_patterns(&all_patterns);

        // Analyze each pattern group
        let mut successful_patterns = Vec::new();
        let mut anti_patterns = Vec::new();

        for (signature, patterns) in pattern_groups {
            if patterns.len() < min_episodes {
                continue; // Not frequent enough
            }

            let success_rate = calculate_pattern_success_rate(&patterns);
            let avg_reward = calculate_pattern_avg_reward(&patterns);

            let pattern_analysis = json!({
                "signature": signature,
                "type": get_pattern_type(&patterns[0]),
                "occurrences": patterns.len(),
                "success_rate": success_rate,
                "avg_reward": avg_reward,
                "example": patterns.first()
            });

            if success_rate >= min_success_rate as f64 {
                successful_patterns.push(pattern_analysis);
            } else if include_anti_patterns {
                anti_patterns.push(pattern_analysis);
            }
        }

        // Sort by success rate and occurrences
        successful_patterns.sort_by(|a, b| {
            let score_a = a["success_rate"].as_f64().unwrap_or(0.0)
                * a["occurrences"].as_u64().unwrap_or(0) as f64;
            let score_b = b["success_rate"].as_f64().unwrap_or(0.0)
                * b["occurrences"].as_u64().unwrap_or(0) as f64;
            score_b.partial_cmp(&score_a).unwrap()
        });

        anti_patterns.sort_by(|a, b| {
            let score_a = a["success_rate"].as_f64().unwrap_or(1.0);
            let score_b = b["success_rate"].as_f64().unwrap_or(1.0);
            score_a.partial_cmp(&score_b).unwrap()
        });

        // Limit results
        successful_patterns.truncate(limit);
        anti_patterns.truncate(limit / 2);

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            successful_patterns = successful_patterns.len(),
            anti_patterns = anti_patterns.len(),
            duration_ms = duration_ms,
            "Pattern analysis completed"
        );

        Ok(json!({
            "success": true,
            "domain": domain,
            "task_type": task_type,
            "analysis": {
                "episodes_analyzed": episodes.len(),
                "total_patterns_found": all_patterns.len(),
                "successful_patterns": successful_patterns,
                "anti_patterns": anti_patterns
            },
            "recommendations": generate_recommendations(&successful_patterns, &anti_patterns),
            "performance": {
                "duration_ms": duration_ms
            }
        }))
    }

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

// Helper functions

fn parse_episode_filter(filter_obj: &Value) -> Result<EpisodeFilter> {
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

fn parse_task_type(task_type: &str) -> Result<TaskType> {
    match task_type {
        "code_generation" => Ok(TaskType::CodeGeneration),
        "debugging" => Ok(TaskType::Debugging),
        "refactoring" => Ok(TaskType::Refactoring),
        "testing" => Ok(TaskType::Testing),
        "analysis" => Ok(TaskType::Analysis),
        "documentation" => Ok(TaskType::Documentation),
        _ => Err(anyhow!("Invalid task_type: {}", task_type)),
    }
}

fn compute_aggregate_stats(episodes: &[Episode], patterns: &[Pattern]) -> Value {
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

fn group_patterns(patterns: &[Pattern]) -> HashMap<String, Vec<Pattern>> {
    let mut groups: HashMap<String, Vec<Pattern>> = HashMap::new();

    for pattern in patterns {
        let signature = get_pattern_signature(pattern);
        groups.entry(signature).or_default().push(pattern.clone());
    }

    groups
}

fn get_pattern_signature(pattern: &Pattern) -> String {
    match pattern {
        Pattern::ToolSequence { tools, .. } => {
            format!("tool_seq:{}", tools.join("→"))
        }
        Pattern::DecisionPoint { condition, .. } => {
            format!("decision:{}", condition)
        }
        Pattern::ErrorRecovery { error_type, .. } => {
            format!("error_recovery:{}", error_type)
        }
        Pattern::ContextPattern {
            context_features, ..
        } => {
            format!("context:{}", context_features.join(","))
        }
    }
}

fn get_pattern_type(pattern: &Pattern) -> &str {
    match pattern {
        Pattern::ToolSequence { .. } => "tool_sequence",
        Pattern::DecisionPoint { .. } => "decision_point",
        Pattern::ErrorRecovery { .. } => "error_recovery",
        Pattern::ContextPattern { .. } => "context_pattern",
    }
}

fn calculate_pattern_success_rate(patterns: &[Pattern]) -> f64 {
    let sum: f64 = patterns.iter().map(|p| p.success_rate() as f64).sum();
    sum / patterns.len().max(1) as f64
}

fn calculate_pattern_avg_reward(_patterns: &[Pattern]) -> f64 {
    // Patterns don't store avg_reward directly, return success rate as proxy
    calculate_pattern_success_rate(_patterns)
}

fn generate_recommendations(successful: &[Value], anti: &[Value]) -> Value {
    let mut recommendations = Vec::new();

    if !successful.is_empty() {
        recommendations.push(json!({
            "type": "best_practice",
            "message": format!(
                "Follow these {} high-success patterns for better outcomes",
                successful.len().min(5)
            ),
            "patterns": successful.iter().take(5).collect::<Vec<_>>()
        }));
    }

    if !anti.is_empty() {
        recommendations.push(json!({
            "type": "anti_pattern",
            "message": format!(
                "Avoid these {} approaches that commonly lead to failures",
                anti.len().min(3)
            ),
            "patterns": anti.iter().take(3).collect::<Vec<_>>()
        }));
    }

    json!(recommendations)
}

fn generate_comparison_insights(episodes: &[Episode], metrics: &[String]) -> Value {
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

fn compare_episode_approaches(episodes: &[Episode]) -> Value {
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
