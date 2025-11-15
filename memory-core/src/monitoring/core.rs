use super::types::{AgentMetrics, AgentType, ExecutionRecord, MonitoringConfig, TaskMetrics};
use crate::storage::StorageBackend;
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Lightweight agent monitoring system for tracking utilization and performance
///
/// The AgentMonitor provides real-time tracking of agent executions with minimal
/// performance overhead. It maintains in-memory metrics and optionally persists
/// data to durable storage for analysis.
#[derive(Clone)]
pub struct AgentMonitor {
    /// Configuration
    config: MonitoringConfig,
    /// In-memory agent metrics storage
    agent_metrics: Arc<RwLock<HashMap<String, AgentMetrics>>>,
    /// In-memory task metrics storage
    task_metrics: Arc<RwLock<HashMap<String, TaskMetrics>>>,
    /// Recent execution records (limited by config.max_records)
    execution_records: Arc<RwLock<Vec<ExecutionRecord>>>,
    /// Durable storage backend (optional)
    storage: Option<Arc<dyn StorageBackend>>,
}

impl AgentMonitor {
    /// Create a new agent monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Create a new agent monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            config,
            agent_metrics: Arc::new(RwLock::new(HashMap::new())),
            task_metrics: Arc::new(RwLock::new(HashMap::new())),
            execution_records: Arc::new(RwLock::new(Vec::new())),
            storage: None,
        }
    }

    /// Create a monitor with storage backend for persistence
    pub fn with_storage(config: MonitoringConfig, storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            storage: Some(storage),
            ..Self::with_config(config)
        }
    }

    /// Record an agent execution
    ///
    /// This is the main entry point for tracking agent performance. Call this
    /// after each agent execution to update metrics.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name/identifier of the agent
    /// * `success` - Whether the execution was successful
    /// * `duration` - How long the execution took
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_core::monitoring::AgentMonitor;
    /// use std::time::Instant;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let monitor = AgentMonitor::new();
    ///
    /// let start = Instant::now();
    /// // ... agent work ...
    /// let duration = start.elapsed();
    ///
    /// monitor.record_execution("feature-implementer", true, duration).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_execution(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
    ) -> Result<()> {
        self.record_execution_detailed(agent_name, success, duration, None, None)
            .await
    }

    /// Record an agent execution with detailed information
    ///
    /// Extended version of `record_execution` that includes task description
    /// and error details for richer analytics.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name/identifier of the agent
    /// * `success` - Whether the execution was successful
    /// * `duration` - How long the execution took
    /// * `task_description` - Optional description of the task performed
    /// * `error_message` - Optional error message if execution failed
    pub async fn record_execution_detailed(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let agent_type = AgentType::from(agent_name);
        let record = ExecutionRecord::new(
            agent_name.to_string(),
            agent_type,
            success,
            duration,
            task_description.clone(),
            error_message,
        );

        // Update agent metrics
        {
            let mut metrics = self.agent_metrics.write().await;
            let agent_metric =
                metrics
                    .entry(agent_name.to_string())
                    .or_insert_with(|| AgentMetrics {
                        agent_name: agent_name.to_string(),
                        agent_type,
                        ..Default::default()
                    });
            agent_metric.update(&record);
        }

        // Update task metrics if task description provided
        if let Some(task_desc) = &task_description {
            let mut task_metrics = self.task_metrics.write().await;
            let task_metric =
                task_metrics
                    .entry(task_desc.clone())
                    .or_insert_with(|| TaskMetrics {
                        task_type: task_desc.clone(),
                        ..Default::default()
                    });
            task_metric.update(&record);
        }

        // Store execution record
        {
            let mut records = self.execution_records.write().await;
            records.push(record);

            // Maintain size limit
            if records.len() > self.config.max_records {
                records.remove(0); // Remove oldest
            }
        }

        // Persist to storage if configured
        if self.config.enable_persistence {
            if let Some(_storage) = &self.storage {
                // TODO: Implement storage persistence
                // For now, we'll store in memory only
            }
        }

        Ok(())
    }

    /// Get metrics for a specific agent
    ///
    /// Returns aggregated performance metrics for the specified agent.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name of the agent to get metrics for
    ///
    /// # Returns
    ///
    /// AgentMetrics containing execution statistics, or None if agent not found
    pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics> {
        let metrics = self.agent_metrics.read().await;
        metrics.get(agent_name).cloned()
    }

    /// Get all agent metrics
    ///
    /// Returns metrics for all agents that have been tracked.
    pub async fn get_all_agent_metrics(&self) -> HashMap<String, AgentMetrics> {
        let metrics = self.agent_metrics.read().await;
        metrics.clone()
    }

    /// Get task metrics for a specific task type
    ///
    /// Returns aggregated metrics for tasks matching the given description.
    pub async fn get_task_metrics(&self, task_type: &str) -> Option<TaskMetrics> {
        let metrics = self.task_metrics.read().await;
        metrics.get(task_type).cloned()
    }

    /// Get recent execution records
    ///
    /// Returns the most recent execution records, limited by configuration.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of records to return (0 for all)
    pub async fn get_recent_executions(&self, limit: usize) -> Vec<ExecutionRecord> {
        let records = self.execution_records.read().await;
        if limit == 0 || limit >= records.len() {
            records.clone()
        } else {
            records.iter().rev().take(limit).rev().cloned().collect()
        }
    }

    /// Get system-wide summary statistics
    ///
    /// Returns aggregated statistics across all agents and tasks.
    pub async fn get_summary_stats(&self) -> MonitoringSummary {
        let agent_metrics = self.agent_metrics.read().await;
        let task_metrics = self.task_metrics.read().await;
        let execution_records = self.execution_records.read().await;

        let total_executions = execution_records.len() as u64;
        let successful_executions = execution_records.iter().filter(|r| r.success).count() as u64;

        let total_duration: Duration = execution_records.iter().map(|r| r.duration).sum();

        let avg_duration = if total_executions > 0 {
            total_duration / total_executions as u32
        } else {
            Duration::ZERO
        };

        MonitoringSummary {
            total_agents: agent_metrics.len() as u64,
            total_executions,
            successful_executions,
            success_rate: if total_executions > 0 {
                successful_executions as f64 / total_executions as f64
            } else {
                0.0
            },
            total_duration,
            avg_duration,
            total_tasks: task_metrics.len() as u64,
        }
    }

    /// Clear all metrics and execution records
    ///
    /// Resets the monitor to its initial state. Use with caution.
    pub async fn clear(&self) {
        let mut agent_metrics = self.agent_metrics.write().await;
        agent_metrics.clear();

        let mut task_metrics = self.task_metrics.write().await;
        task_metrics.clear();

        let mut records = self.execution_records.write().await;
        records.clear();
    }
}

/// Summary statistics for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringSummary {
    /// Total number of unique agents tracked
    pub total_agents: u64,
    /// Total number of executions recorded
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Overall success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Total execution time across all agents
    pub total_duration: Duration,
    /// Average execution duration
    pub avg_duration: Duration,
    /// Total number of unique task types
    pub total_tasks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_record_execution() {
        let monitor = AgentMonitor::new();

        // Record a successful execution
        monitor
            .record_execution("test-agent", true, Duration::from_secs(1))
            .await
            .unwrap();

        // Check metrics
        let metrics = monitor.get_agent_metrics("test-agent").await.unwrap();
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.success_rate(), 1.0);
        assert_eq!(metrics.avg_duration, Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_record_failed_execution() {
        let monitor = AgentMonitor::new();

        // Record a failed execution
        monitor
            .record_execution("test-agent", false, Duration::from_secs(2))
            .await
            .unwrap();

        // Check metrics
        let metrics = monitor.get_agent_metrics("test-agent").await.unwrap();
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_multiple_executions() {
        let monitor = AgentMonitor::new();

        // Record multiple executions
        monitor
            .record_execution("test-agent", true, Duration::from_secs(1))
            .await
            .unwrap();
        monitor
            .record_execution("test-agent", false, Duration::from_secs(2))
            .await
            .unwrap();
        monitor
            .record_execution("test-agent", true, Duration::from_secs(3))
            .await
            .unwrap();

        // Check metrics
        let metrics = monitor.get_agent_metrics("test-agent").await.unwrap();
        assert_eq!(metrics.total_executions, 3);
        assert_eq!(metrics.successful_executions, 2);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
        assert_eq!(metrics.avg_duration, Duration::from_secs(2)); // (1+2+3)/3 = 2
    }

    #[tokio::test]
    async fn test_detailed_recording() {
        let monitor = AgentMonitor::new();

        monitor
            .record_execution_detailed(
                "test-agent",
                true,
                Duration::from_secs(1),
                Some("Test task".to_string()),
                None,
            )
            .await
            .unwrap();

        // Check task metrics
        let task_metrics = monitor.get_task_metrics("Test task").await.unwrap();
        assert_eq!(task_metrics.total_tasks, 1);
        assert_eq!(task_metrics.completed_tasks, 1);
    }

    #[tokio::test]
    async fn test_summary_stats() {
        let monitor = AgentMonitor::new();

        monitor
            .record_execution("agent1", true, Duration::from_secs(1))
            .await
            .unwrap();
        monitor
            .record_execution("agent2", false, Duration::from_secs(2))
            .await
            .unwrap();

        let summary = monitor.get_summary_stats().await;
        assert_eq!(summary.total_agents, 2);
        assert_eq!(summary.total_executions, 2);
        assert_eq!(summary.successful_executions, 1);
        assert_eq!(summary.success_rate, 0.5);
    }

    #[tokio::test]
    async fn test_disabled_monitoring() {
        let config = MonitoringConfig {
            enabled: false,
            ..Default::default()
        };
        let monitor = AgentMonitor::with_config(config);

        monitor
            .record_execution("test-agent", true, Duration::from_secs(1))
            .await
            .unwrap();

        // Should have no metrics when disabled
        let metrics = monitor.get_agent_metrics("test-agent").await;
        assert!(metrics.is_none());
    }
}
