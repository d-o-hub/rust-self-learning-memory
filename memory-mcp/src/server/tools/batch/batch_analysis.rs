//! Batch pattern analysis operations
//!
//! This module provides tools for analyzing patterns across multiple episodes,
//! identifying common sequences, successful approaches, and anti-patterns.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use memory_core::{EpisodeFilter, Pattern};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

use super::batch_query::parse_task_type;

/// Group patterns by their signature for analysis
pub fn group_patterns(patterns: &[Pattern]) -> HashMap<String, Vec<Pattern>> {
    let mut groups: HashMap<String, Vec<Pattern>> = HashMap::new();

    for pattern in patterns {
        let signature = get_pattern_signature(pattern);
        groups.entry(signature).or_default().push(pattern.clone());
    }

    groups
}

/// Get a unique signature for a pattern based on its type and content
pub fn get_pattern_signature(pattern: &Pattern) -> String {
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

/// Get the type name of a pattern
pub fn get_pattern_type(pattern: &Pattern) -> &str {
    match pattern {
        Pattern::ToolSequence { .. } => "tool_sequence",
        Pattern::DecisionPoint { .. } => "decision_point",
        Pattern::ErrorRecovery { .. } => "error_recovery",
        Pattern::ContextPattern { .. } => "context_pattern",
    }
}

/// Calculate the average success rate for a collection of patterns
pub fn calculate_pattern_success_rate(patterns: &[Pattern]) -> f64 {
    let sum: f64 = patterns.iter().map(|p| p.success_rate() as f64).sum();
    sum / patterns.len().max(1) as f64
}

/// Calculate the average reward for a collection of patterns
pub fn calculate_pattern_avg_reward(_patterns: &[Pattern]) -> f64 {
    // Patterns don't store avg_reward directly, return success rate as proxy
    calculate_pattern_success_rate(_patterns)
}

/// Generate recommendations based on successful and anti-patterns
pub fn generate_recommendations(successful: &[Value], anti: &[Value]) -> Value {
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

impl MemoryMCPServer {
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
}
