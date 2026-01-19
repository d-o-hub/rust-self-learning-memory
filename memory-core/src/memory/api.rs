//! Public API methods for `SelfLearningMemory`
//!
//! This module contains all public methods that users interact with,
//! organized by functionality area.

use crate::error::Result;
use crate::monitoring::AgentMetrics;
use std::time::Duration;

use super::SelfLearningMemory;

// ============================================================================
// Monitoring and Statistics
// ============================================================================

impl SelfLearningMemory {
    /// Get statistics about the memory system
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        super::monitoring::get_stats(&self.episodes_fallback, &self.patterns_fallback).await
    }

    /// Record an agent execution for monitoring
    pub async fn record_agent_execution(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
    ) -> Result<()> {
        super::monitoring::record_agent_execution(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
        )
        .await
    }

    /// Record detailed agent execution information
    pub async fn record_agent_execution_detailed(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        super::monitoring::record_agent_execution_detailed(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
            task_description,
            error_message,
        )
        .await
    }

    /// Get performance metrics for a specific agent
    pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics> {
        super::monitoring::get_agent_metrics(&self.agent_monitor, agent_name).await
    }

    /// Get metrics for all tracked agents
    pub async fn get_all_agent_metrics(&self) -> std::collections::HashMap<String, AgentMetrics> {
        super::monitoring::get_all_agent_metrics(&self.agent_monitor).await
    }

    /// Get monitoring system summary statistics
    pub async fn get_monitoring_summary(&self) -> crate::monitoring::MonitoringSummary {
        super::monitoring::get_monitoring_summary(&self.agent_monitor).await
    }

    /// Get query cache metrics (v0.1.12)
    #[must_use]
    pub fn get_cache_metrics(&self) -> crate::retrieval::CacheMetrics {
        super::monitoring::get_cache_metrics(&self.query_cache)
    }

    /// Clear query cache metrics (v0.1.12)
    pub fn clear_cache_metrics(&self) {
        super::monitoring::clear_cache_metrics(&self.query_cache);
    }

    /// Clear all cached query results (v0.1.12)
    pub fn clear_cache(&self) {
        super::monitoring::clear_cache(&self.query_cache);
    }
}
