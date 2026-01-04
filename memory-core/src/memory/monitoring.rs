//! Agent monitoring and metrics collection for `SelfLearningMemory`.
//!
//! This module provides methods for tracking agent execution, performance metrics,
//! and system-wide monitoring statistics.

use crate::monitoring::{AgentMetrics, MonitoringSummary};
use crate::retrieval::CacheMetrics;
use std::time::Duration;

/// Get statistics about the memory system
///
/// # Returns
///
/// Tuple of (total episodes, completed episodes, total patterns)
pub async fn get_stats(
    episodes_fallback: &tokio::sync::RwLock<std::collections::HashMap<uuid::Uuid, crate::Episode>>,
    patterns_fallback: &tokio::sync::RwLock<
        std::collections::HashMap<crate::episode::PatternId, crate::Pattern>,
    >,
) -> (usize, usize, usize) {
    let episodes = episodes_fallback.read().await;
    let patterns = patterns_fallback.read().await;

    let total_episodes = episodes.len();
    let completed_episodes = episodes.values().filter(|e| e.is_complete()).count();
    let total_patterns = patterns.len();

    (total_episodes, completed_episodes, total_patterns)
}

/// Record an agent execution for monitoring
///
/// Tracks agent utilization, performance, and task completion rates.
/// This is the main entry point for agent monitoring.
pub async fn record_agent_execution(
    agent_monitor: &crate::monitoring::AgentMonitor,
    agent_name: &str,
    success: bool,
    duration: Duration,
) -> crate::Result<()> {
    agent_monitor
        .record_execution(agent_name, success, duration)
        .await
}

/// Record detailed agent execution information
///
/// Extended version that includes task description and error details.
pub async fn record_agent_execution_detailed(
    agent_monitor: &crate::monitoring::AgentMonitor,
    agent_name: &str,
    success: bool,
    duration: Duration,
    task_description: Option<String>,
    error_message: Option<String>,
) -> crate::Result<()> {
    agent_monitor
        .record_execution_detailed(
            agent_name,
            success,
            duration,
            task_description,
            error_message,
        )
        .await
}

/// Get performance metrics for a specific agent
///
/// Returns aggregated statistics including success rates, execution times,
/// and utilization patterns.
pub async fn get_agent_metrics(
    agent_monitor: &crate::monitoring::AgentMonitor,
    agent_name: &str,
) -> Option<AgentMetrics> {
    agent_monitor.get_agent_metrics(agent_name).await
}

/// Get metrics for all tracked agents
///
/// Returns performance data for all agents that have been monitored.
pub async fn get_all_agent_metrics(
    agent_monitor: &crate::monitoring::AgentMonitor,
) -> std::collections::HashMap<String, AgentMetrics> {
    agent_monitor.get_all_agent_metrics().await
}

/// Get monitoring system summary statistics
///
/// Returns system-wide analytics including total executions, success rates,
/// and performance metrics across all agents.
pub async fn get_monitoring_summary(
    agent_monitor: &crate::monitoring::AgentMonitor,
) -> MonitoringSummary {
    agent_monitor.get_summary_stats().await
}

/// Get query cache metrics (v0.1.12)
///
/// Returns cache performance statistics including hit rate, size,
/// and eviction counts. Useful for monitoring cache effectiveness.
pub fn get_cache_metrics(query_cache: &crate::retrieval::QueryCache) -> CacheMetrics {
    query_cache.metrics()
}

/// Clear query cache metrics (v0.1.12)
///
/// Resets all cache performance counters (hits, misses, evictions).
/// Useful for testing or when starting a new monitoring period.
pub fn clear_cache_metrics(query_cache: &crate::retrieval::QueryCache) {
    query_cache.clear_metrics();
}

/// Clear all cached query results (v0.1.12)
///
/// Invalidates all cached query results. Future retrievals will
/// perform full searches until the cache is repopulated.
pub fn clear_cache(query_cache: &crate::retrieval::QueryCache) {
    query_cache.invalidate_all();
}
