//! Reporting for monitoring storage
//!
//! Provides the MonitoringStorage struct and analytics reporting
//! functionality for monitoring data.

use super::super::types::{AgentMetrics, ExecutionRecord, TaskMetrics};
use super::collector::{MonitoringStorageBackend, SimpleMonitoringStorage};
use crate::Result;

use std::collections::HashMap;
use std::sync::Arc;

/// Aggregated analytics data for monitoring dashboards
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonitoringAnalytics {
    /// Total executions across all agents
    pub total_executions: u64,
    /// Overall success rate
    pub success_rate: f64,
    /// Average execution duration in seconds
    pub avg_duration_secs: f64,
    /// Top performing agents by success rate
    pub top_performing_agents: Vec<(String, f64)>,
    /// Recent execution failures for debugging
    pub recent_failures: Vec<ExecutionRecord>,
}

impl Default for MonitoringAnalytics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            success_rate: 0.0,
            avg_duration_secs: 0.0,
            top_performing_agents: Vec::new(),
            recent_failures: Vec::new(),
        }
    }
}

/// Storage layer for agent monitoring data
///
/// Handles persistence of monitoring metrics to durable storage (Turso)
/// and fast cache (redb) for analysis and retrieval.
#[allow(dead_code)]
pub struct MonitoringStorage {
    /// Durable storage backend with monitoring capabilities
    durable_storage: Option<Arc<dyn MonitoringStorageBackend>>,
    /// Cache storage backend (redb) - optional for fast access
    cache_storage: Option<Arc<dyn MonitoringStorageBackend>>,
}

impl Default for MonitoringStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl MonitoringStorage {
    /// Create a new monitoring storage instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            durable_storage: None,
            cache_storage: None,
        }
    }

    /// Create storage with durable backend
    pub fn with_durable(durable: Arc<dyn MonitoringStorageBackend>) -> Self {
        Self {
            durable_storage: Some(durable),
            cache_storage: None,
        }
    }

    /// Create storage with both durable and cache backends
    pub fn with_backends(
        durable: Arc<dyn MonitoringStorageBackend>,
        cache: Arc<dyn MonitoringStorageBackend>,
    ) -> Self {
        Self {
            durable_storage: Some(durable),
            cache_storage: Some(cache),
        }
    }

    /// Check if durable storage backend is configured
    #[must_use]
    pub fn has_durable_storage(&self) -> bool {
        self.durable_storage.is_some()
    }

    /// Check if cache storage backend is configured
    #[must_use]
    pub fn has_cache_storage(&self) -> bool {
        self.cache_storage.is_some()
    }

    /// Store an execution record
    ///
    /// Persists the execution record to durable storage and updates cache.
    pub async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()> {
        // Store in durable storage first
        if let Some(storage) = &self.durable_storage {
            storage.store_execution_record(record).await?;
        }

        // Update cache for fast access
        if let Some(storage) = &self.cache_storage {
            storage.store_execution_record(record).await?;
        }

        Ok(())
    }

    /// Store agent metrics
    ///
    /// Persists aggregated agent metrics for analysis.
    pub async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()> {
        // Store in durable storage first
        if let Some(storage) = &self.durable_storage {
            storage.store_agent_metrics(metrics).await?;
        }

        // Update cache for fast access
        if let Some(storage) = &self.cache_storage {
            storage.store_agent_metrics(metrics).await?;
        }

        Ok(())
    }

    /// Store task metrics
    ///
    /// Persists aggregated task metrics for analysis.
    pub async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()> {
        // Store in durable storage first
        if let Some(storage) = &self.durable_storage {
            storage.store_task_metrics(metrics).await?;
        }

        // Update cache for fast access
        if let Some(storage) = &self.cache_storage {
            storage.store_task_metrics(metrics).await?;
        }

        Ok(())
    }

    /// Retrieve agent metrics from storage
    ///
    /// Loads metrics from cache first, falling back to durable storage.
    pub async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>> {
        // Try cache first for faster access
        if let Some(storage) = &self.cache_storage {
            if let Some(metrics) = storage.load_agent_metrics(agent_name).await? {
                return Ok(Some(metrics));
            }
        }

        // Fall back to durable storage
        if let Some(storage) = &self.durable_storage {
            return storage.load_agent_metrics(agent_name).await;
        }

        Ok(None)
    }

    /// Retrieve execution records within a time range
    ///
    /// Loads historical execution data for analysis.
    pub async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        // Try cache first for faster access
        if let Some(storage) = &self.cache_storage {
            let records = storage.load_execution_records(agent_name, limit).await?;
            if !records.is_empty() {
                return Ok(records);
            }
        }

        // Fall back to durable storage
        if let Some(storage) = &self.durable_storage {
            return storage.load_execution_records(agent_name, limit).await;
        }

        Ok(Vec::new())
    }

    /// Load task metrics by task type
    pub async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>> {
        // Try cache first for faster access
        if let Some(storage) = &self.cache_storage {
            if let Some(metrics) = storage.load_task_metrics(task_type).await? {
                return Ok(Some(metrics));
            }
        }

        // Fall back to durable storage
        if let Some(storage) = &self.durable_storage {
            return storage.load_task_metrics(task_type).await;
        }

        Ok(None)
    }

    /// Get aggregated analytics data
    ///
    /// Returns system-wide analytics for monitoring dashboards.
    pub async fn get_analytics(&self) -> Result<MonitoringAnalytics> {
        let records = self
            .load_execution_records(None, 1000)
            .await
            .unwrap_or_default();

        if records.is_empty() {
            return Ok(MonitoringAnalytics::default());
        }

        let total_executions = records.len() as u64;
        let successful = records.iter().filter(|r| r.success).count() as u64;
        let success_rate = if total_executions > 0 {
            successful as f64 / total_executions as f64
        } else {
            0.0
        };

        let total_duration: std::time::Duration = records.iter().map(|r| r.duration).sum();
        let avg_duration_secs = total_duration.as_secs_f64() / total_executions as f64;

        // Top performing agents
        let mut agent_stats: HashMap<String, (u64, u64)> = HashMap::new();
        for r in &records {
            let entry = agent_stats.entry(r.agent_name.clone()).or_default();
            entry.1 += 1;
            if r.success {
                entry.0 += 1;
            }
        }

        let top_agents: Vec<(String, f64)> = agent_stats
            .into_iter()
            .filter_map(|(name, (succ, total))| {
                if total > 0 {
                    Some((name, succ as f64 / total as f64))
                } else {
                    None
                }
            })
            .filter(|&(_, rate)| rate > 0.5)
            .collect();

        let mut top_performing_agents = top_agents;
        top_performing_agents
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_performing_agents = top_performing_agents.into_iter().take(5).collect();

        // Recent failures
        let recent_failures: Vec<ExecutionRecord> = records
            .into_iter()
            .filter(|r| !r.success)
            .take(10)
            .collect();

        Ok(MonitoringAnalytics {
            total_executions,
            success_rate,
            avg_duration_secs,
            top_performing_agents,
            recent_failures,
        })
    }
}
