use super::types::{AgentMetrics, ExecutionRecord, TaskMetrics};
use crate::storage::StorageBackend;
use crate::{Episode, Pattern, Heuristic, Result, Error};
use crate::episode::PatternId;
use crate::types::{TaskOutcome, TaskType, TaskContext, RewardScore};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;

/// Extended storage backend trait for monitoring data
///
/// This trait extends the basic StorageBackend with monitoring-specific operations.
/// Implementations should provide persistence for execution records, agent metrics, and task metrics.
#[async_trait::async_trait]
pub trait MonitoringStorageBackend: StorageBackend + Send + Sync {
    /// Store an execution record for monitoring
    async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()>;

    /// Store aggregated agent metrics
    async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()>;

    /// Store task metrics
    async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()>;

    /// Load agent metrics by name
    async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>>;

    /// Load execution records with optional filtering
    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>>;

    /// Load task metrics by task type
    async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>>;
}

/// Simple monitoring storage that wraps basic StorageBackend implementations
/// 
/// This provides basic monitoring capabilities by storing execution records
/// as episodes in the underlying storage systems.
pub struct SimpleMonitoringStorage {
    /// Primary storage backend (Turso)
    primary: Arc<dyn StorageBackend>,
    /// Cache storage backend (redb)
    cache: Arc<dyn StorageBackend>,
}

impl SimpleMonitoringStorage {
    /// Create a new simple monitoring storage with the given backends
    pub fn new(primary: Arc<dyn StorageBackend>, cache: Arc<dyn StorageBackend>) -> Self {
        Self {
            primary,
            cache,
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for SimpleMonitoringStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        // Store in both primary and cache
        if let Err(e) = self.primary.store_episode(episode).await {
            return Err(e);
        }
        if let Err(e) = self.cache.store_episode(episode).await {
            return Err(e);
        }
        Ok(())
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        // Try cache first, then primary
        if let Ok(Some(episode)) = self.cache.get_episode(id).await {
            return Ok(Some(episode));
        }
        self.primary.get_episode(id).await
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        if let Err(e) = self.primary.store_pattern(pattern).await {
            return Err(e);
        }
        if let Err(e) = self.cache.store_pattern(pattern).await {
            return Err(e);
        }
        Ok(())
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        if let Ok(Some(pattern)) = self.cache.get_pattern(id).await {
            return Ok(Some(pattern));
        }
        self.primary.get_pattern(id).await
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        if let Err(e) = self.primary.store_heuristic(heuristic).await {
            return Err(e);
        }
        if let Err(e) = self.cache.store_heuristic(heuristic).await {
            return Err(e);
        }
        Ok(())
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        if let Ok(Some(heuristic)) = self.cache.get_heuristic(id).await {
            return Ok(Some(heuristic));
        }
        self.primary.get_heuristic(id).await
    }

    async fn query_episodes_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Episode>> {
        // Query both backends and combine results
        let cache_episodes = self.cache.query_episodes_since(since).await.unwrap_or_default();
        let primary_episodes = self.primary.query_episodes_since(since).await.unwrap_or_default();
        
        // Combine and deduplicate by episode_id
        let mut episodes = cache_episodes;
        for episode in primary_episodes {
            if !episodes.iter().any(|e| e.episode_id == episode.episode_id) {
                episodes.push(episode);
            }
        }
        
        Ok(episodes)
    }
}

#[async_trait::async_trait]
impl MonitoringStorageBackend for SimpleMonitoringStorage {
    async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()> {
        // Convert execution record to an episode for storage
        let episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::Analysis,
            task_description: format!("Agent execution: {}", record.agent_name),
            context: TaskContext::default(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            steps: vec![],
            outcome: Some(match record.success {
                true => TaskOutcome::Success {
                    verdict: "Execution completed".to_string(),
                    artifacts: vec![],
                },
                false => TaskOutcome::Failure {
                    reason: "Execution failed".to_string(),
                    error_details: record.error_message.clone(),
                },
            }),
            reward: Some(RewardScore {
                total: if record.success { 1.0 } else { 0.0 },
                base: if record.success { 1.0 } else { 0.0 },
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            }),
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            metadata: HashMap::new(),
        };

        self.store_episode(&episode).await?;
        Ok(())
    }

    async fn store_agent_metrics(&self, _metrics: &AgentMetrics) -> Result<()> {
        // For now, just log this - in a full implementation,
        // this would store metrics in a dedicated table
        tracing::debug!("Storing agent metrics (basic implementation)");
        Ok(())
    }

    async fn store_task_metrics(&self, _metrics: &TaskMetrics) -> Result<()> {
        tracing::debug!("Storing task metrics (basic implementation)");
        Ok(())
    }

    async fn load_agent_metrics(&self, _agent_name: &str) -> Result<Option<AgentMetrics>> {
        // Basic implementation - return None for now
        Ok(None)
    }

    async fn load_execution_records(
        &self,
        _agent_name: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        // Basic implementation - return empty for now
        Ok(vec![])
    }

    async fn load_task_metrics(&self, _task_type: &str) -> Result<Option<TaskMetrics>> {
        // Basic implementation - return None for now
        Ok(None)
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
    pub fn has_durable_storage(&self) -> bool {
        self.durable_storage.is_some()
    }

    /// Check if cache storage backend is configured
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
        use std::collections::HashMap;
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
