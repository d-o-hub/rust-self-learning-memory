// Core tool handlers for the MCP server
//!
//! This module contains core tool execution methods: list_tools, get_tool, query_memory, and analyze_patterns.

use crate::types::Tool;
use anyhow::Result;
use memory_core::{Episode, Pattern, TaskOutcome};
use serde_json::json;
use std::collections::HashMap;
use tracing::debug;

/// Calculate a success score for an episode (higher = more successful)
fn outcome_score(episode: &Episode) -> u8 {
    match &episode.outcome {
        Some(TaskOutcome::Success { .. }) => 3,
        Some(TaskOutcome::PartialSuccess { .. }) => 2,
        Some(TaskOutcome::Failure { .. }) => 1,
        None => 0,
    }
}

impl crate::server::MemoryMCPServer {
    /// List all available tools
    ///
    /// Returns tools based on progressive disclosure - commonly used tools
    /// are returned first, advanced tools are shown after usage patterns indicate need.
    pub async fn list_tools(&self) -> Vec<Tool> {
        let tools = self.tools.read();
        let usage = self.tool_usage.read();

        // Sort tools by usage frequency
        let mut sorted_tools: Vec<_> = tools.iter().cloned().collect();
        sorted_tools.sort_by(|a, b| {
            let usage_a = usage.get(&a.name).unwrap_or(&0);
            let usage_b = usage.get(&b.name).unwrap_or(&0);
            usage_b.cmp(usage_a)
        });

        debug!("Listed {} tools (sorted by usage)", sorted_tools.len());
        sorted_tools
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let tools = self.tools.read();
        tools.iter().find(|t| t.name == name).cloned()
    }

    /// Execute the query_memory tool
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `domain` - Task domain
    /// * `task_type` - Optional task type filter
    /// * `limit` - Maximum results to return
    /// * `sort` - Sort order (relevance, newest, oldest, duration, success)
    ///
    /// # Returns
    ///
    /// Returns a JSON array of relevant episodes
    pub async fn query_memory(
        &self,
        query: String,
        domain: String,
        task_type: Option<String>,
        limit: usize,
        sort: String,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("query_memory").await;

        // Start monitoring request
        let request_id = format!(
            "query_memory_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "query_memory".to_string())
            .await;

        debug!(
            "Querying memory: query='{}', domain='{}', limit={}",
            query, domain, limit
        );

        let start = std::time::Instant::now();

        // Build task context from parameters
        let context = memory_core::TaskContext {
            domain,
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: task_type
                .as_ref()
                .map(|t| vec![t.clone()])
                .unwrap_or_default(),
        };

        // Query actual memory for relevant episodes
        let episodes = self
            .memory
            .retrieve_relevant_context(query.clone(), context.clone(), limit)
            .await;

        // Strict filtering: only return episodes that actually contain the query.
        let query_lc = query.to_lowercase();
        let mut episodes: Vec<_> = episodes
            .into_iter()
            .filter(|ep| {
                if ep.task_description.to_lowercase().contains(&query_lc) {
                    return true;
                }
                for step in &ep.steps {
                    if step.action.to_lowercase().contains(&query_lc) {
                        return true;
                    }
                    if step
                        .parameters
                        .to_string()
                        .to_lowercase()
                        .contains(&query_lc)
                    {
                        return true;
                    }
                    if let Some(result) = &step.result {
                        if serde_json::to_string(result)
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&query_lc)
                        {
                            return true;
                        }
                    }
                }
                false
            })
            .collect();

        // Apply sorting
        match sort.as_str() {
            "newest" => {
                episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));
            }
            "oldest" => {
                episodes.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            }
            "duration" => {
                episodes.sort_by(|a, b| {
                    let dur_a = a.end_time.map(|e| e - a.start_time);
                    let dur_b = b.end_time.map(|e| e - b.start_time);
                    dur_b.cmp(&dur_a)
                });
            }
            "success" => {
                episodes.sort_by(|a, b| {
                    let score_a = outcome_score(a);
                    let score_b = outcome_score(b);
                    score_b.cmp(&score_a)
                });
            }
            _ => {} // "relevance" - keep default order
        }

        // Also get relevant patterns
        let patterns = self
            .memory
            .retrieve_relevant_patterns(&context, limit)
            .await;

        // Calculate insights from retrieved data
        let success_count = episodes
            .iter()
            .filter(|e| e.reward.as_ref().is_some_and(|r| r.total > 0.7))
            .count();

        let avg_success_rate = if !episodes.is_empty() {
            success_count as f32 / episodes.len() as f32
        } else {
            0.0
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // End monitoring request
        self.monitoring.end_request(&request_id, true, None).await;

        debug!("Memory query completed in {}ms", duration_ms);

        Ok(json!({
            "episodes": episodes,
            "patterns": patterns,
            "insights": {
                "total_episodes": episodes.len(),
                "relevant_patterns": patterns.len(),
                "success_rate": avg_success_rate
            }
        }))
    }

    /// Execute the analyze_patterns tool
    ///
    /// # Arguments
    ///
    /// * `task_type` - Type of task to analyze
    /// * `min_success_rate` - Minimum success rate filter
    /// * `limit` - Maximum patterns to return
    ///
    /// # Returns
    ///
    /// Returns a JSON array of patterns with statistics
    pub async fn analyze_patterns(
        &self,
        task_type: String,
        min_success_rate: f32,
        limit: usize,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("analyze_patterns").await;

        debug!(
            "Analyzing patterns: task_type='{}', min_success_rate={}, limit={}",
            task_type, min_success_rate, limit
        );

        // Build context for pattern retrieval
        let context = memory_core::TaskContext {
            domain: task_type.clone(),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: vec![task_type],
        };

        // Retrieve patterns from memory
        let all_patterns = self
            .memory
            .retrieve_relevant_patterns(&context, limit * 2)
            .await;

        // Filter by success rate and limit
        let filtered_patterns: Vec<_> = all_patterns
            .into_iter()
            .filter(|p| p.success_rate() >= min_success_rate)
            .take(limit)
            .collect();

        // Calculate statistics
        let total_patterns = filtered_patterns.len();
        let avg_success_rate = min_success_rate;

        // Extract most common tools from patterns
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        for pattern in &filtered_patterns {
            match pattern {
                Pattern::ToolSequence { tools, .. } => {
                    for tool in tools {
                        *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                    }
                }
                Pattern::DecisionPoint { action, .. } => {
                    *tool_counts.entry(action.clone()).or_insert(0) += 1;
                }
                Pattern::ErrorRecovery { recovery_steps, .. } => {
                    for step in recovery_steps {
                        *tool_counts.entry(step.clone()).or_insert(0) += 1;
                    }
                }
                Pattern::ContextPattern {
                    recommended_approach,
                    ..
                } => {
                    *tool_counts.entry(recommended_approach.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut most_common_tools: Vec<_> = tool_counts.into_iter().collect();
        most_common_tools.sort_by(|a, b| b.1.cmp(&a.1));
        let most_common_tools: Vec<String> = most_common_tools
            .into_iter()
            .take(5)
            .map(|(tool, _)| tool)
            .collect();

        Ok(json!({
            "patterns": filtered_patterns,
            "statistics": {
                "total_patterns": total_patterns,
                "avg_success_rate": avg_success_rate,
                "most_common_tools": most_common_tools
            }
        }))
    }

    /// Execute the bulk_episodes tool
    ///
    /// # Arguments
    ///
    /// * `episode_ids` - List of episode UUIDs to retrieve
    ///
    /// # Returns
    ///
    /// Returns a result with requested count, found count, and episodes
    pub async fn get_episodes_by_ids(
        &self,
        episode_ids: &[uuid::Uuid],
    ) -> Result<Vec<memory_core::Episode>> {
        self.track_tool_usage("bulk_episodes").await;

        debug!("Bulk retrieving {} episodes", episode_ids.len());

        let episodes = self.memory.get_episodes_by_ids(episode_ids).await?;

        debug!(
            "Found {} of {} requested episodes",
            episodes.len(),
            episode_ids.len()
        );

        Ok(episodes)
    }
}
