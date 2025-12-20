use super::types::{AgentMetrics, ExecutionRecord, TaskMetrics};
use crate::storage::StorageBackend;
use crate::types::{RewardScore, TaskContext, TaskOutcome, TaskType};
use crate::{Episode, Result};

use chrono::Utc;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Extended storage backend trait for monitoring data
#[async_trait::async_trait]
pub trait MonitoringStorageBackend: Send + Sync {
    async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()>;
    async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()>;
    async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()>;
    async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>>;
    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>>;
    async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>>;
}

/// Simple monitoring storage that provides basic storage-backed monitoring
pub struct SimpleMonitoringStorage {
    /// Storage backend for monitoring data
    storage: Arc<dyn StorageBackend>,
}

impl SimpleMonitoringStorage {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl MonitoringStorageBackend for SimpleMonitoringStorage {
    async fn store_execution_record(&self, record: &ExecutionRecord) -> Result<()> {
        // Store execution record as a specialized episode for monitoring
        let mut metadata = HashMap::new();
        metadata.insert("monitoring_type".to_string(), "execution_record".to_string());
        metadata.insert("agent_name".to_string(), record.agent_name.clone());
        metadata.insert("agent_type".to_string(), record.agent_type.to_string());
        metadata.insert("success".to_string(), record.success.to_string());
        metadata.insert("duration_ms".to_string(), record.duration.as_millis().to_string());
        metadata.insert("started_at".to_string(), record.started_at.to_rfc3339());
        
        if let Some(ref task_desc) = record.task_description {
            metadata.insert("task_description".to_string(), task_desc.clone());
        }
        
        if let Some(ref error_msg) = record.error_message {
            metadata.insert("error_message".to_string(), error_msg.clone());
        }

        let episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::Analysis,
            task_description: format!("Monitoring: {} execution", record.agent_name),
            context: TaskContext::default(),
            start_time: record.started_at,
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
            metadata,
        };

        self.storage.store_episode(&episode).await?;
        Ok(())
    }

    async fn store_agent_metrics(&self, metrics: &AgentMetrics) -> Result<()> {
        tracing::debug!("Storing agent metrics for: {}", metrics.agent_name);
        
        // Store agent metrics as JSON in metadata of a specialized episode
        let mut metadata = HashMap::new();
        metadata.insert("monitoring_type".to_string(), "agent_metrics".to_string());
        metadata.insert("agent_name".to_string(), metrics.agent_name.clone());
        metadata.insert("agent_type".to_string(), metrics.agent_type.to_string());
        
        // Serialize metrics to JSON and store as metadata
        let metrics_json = serde_json::to_string(metrics)?;
        metadata.insert("metrics_data".to_string(), metrics_json);

        let episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::Analysis,
            task_description: format!("Agent Metrics: {}", metrics.agent_name),
            context: TaskContext::default(),
            start_time: metrics.last_execution.unwrap_or_else(|| Utc::now()),
            end_time: Some(Utc::now()),
            steps: vec![],
            outcome: Some(TaskOutcome::Success {
                verdict: "Metrics stored".to_string(),
                artifacts: vec![],
            }),
            reward: Some(RewardScore {
                total: metrics.success_rate() as f32,
                base: metrics.success_rate() as f32,
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            }),
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            metadata,
        };

        self.storage.store_episode(&episode).await?;
        Ok(())
    }

    async fn store_task_metrics(&self, metrics: &TaskMetrics) -> Result<()> {
        tracing::debug!("Storing task metrics for: {}", metrics.task_type);
        
        // Store task metrics as JSON in metadata of a specialized episode
        let mut metadata = HashMap::new();
        metadata.insert("monitoring_type".to_string(), "task_metrics".to_string());
        metadata.insert("task_type".to_string(), metrics.task_type.clone());
        
        // Serialize metrics to JSON and store as metadata
        let metrics_json = serde_json::to_string(metrics)?;
        metadata.insert("metrics_data".to_string(), metrics_json);

        let episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::Analysis,
            task_description: format!("Task Metrics: {}", metrics.task_type),
            context: TaskContext::default(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            steps: vec![],
            outcome: Some(TaskOutcome::Success {
                verdict: "Task metrics stored".to_string(),
                artifacts: vec![],
            }),
            reward: Some(RewardScore {
                total: metrics.success_rate() as f32,
                base: metrics.success_rate() as f32,
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            }),
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            metadata,
        };

        self.storage.store_episode(&episode).await?;
        Ok(())
    }

    async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>> {
        // Query episodes for agent metrics
        let episodes = self.storage.query_episodes_by_metadata("monitoring_type", "agent_metrics").await?;
        
        for episode in episodes {
            if let Some(stored_agent_name) = episode.metadata.get("agent_name") {
                if stored_agent_name == agent_name {
                    if let Some(metrics_json) = episode.metadata.get("metrics_data") {
                        let metrics: AgentMetrics = serde_json::from_str(metrics_json)?;
                        return Ok(Some(metrics));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        // Query episodes for execution records
        let mut episodes = self.storage.query_episodes_by_metadata("monitoring_type", "execution_record").await?;
        
        // Filter by agent name if specified
        if let Some(name) = agent_name {
            episodes.retain(|episode| {
                episode.metadata.get("agent_name")
                    .map(|stored_name| stored_name == name)
                    .unwrap_or(false)
            });
        }
        
        // Sort by timestamp (newest first) and limit
        episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        episodes.truncate(limit);
        
        let mut records = Vec::new();
        for episode in episodes {
            if let Some(_metrics_data) = episode.metadata.get("metrics_data") {
                // For execution records, we need to reconstruct from metadata since they're stored differently
                let agent_name = episode.metadata.get("agent_name")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                    
                let agent_type_str = episode.metadata.get("agent_type")
                    .and_then(|s| match s.as_str() {
                        "feature-implementer" => Some(crate::monitoring::types::AgentType::FeatureImplementer),
                        "code-reviewer" => Some(crate::monitoring::types::AgentType::CodeReviewer),
                        "test-runner" => Some(crate::monitoring::types::AgentType::TestRunner),
                        "architecture-validator" => Some(crate::monitoring::types::AgentType::ArchitectureValidator),
                        "debugger" => Some(crate::monitoring::types::AgentType::Debugger),
                        "analysis-swarm" => Some(crate::monitoring::types::AgentType::AnalysisSwarm),
                        "goap-agent" => Some(crate::monitoring::types::AgentType::GoapAgent),
                        _ => Some(crate::monitoring::types::AgentType::Other),
                    })
                    .unwrap_or(crate::monitoring::types::AgentType::Other);
                
                let success = episode.metadata.get("success")
                    .and_then(|s| s.parse::<bool>().ok())
                    .unwrap_or(false);
                
                let duration_ms = episode.metadata.get("duration_ms")
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                
                let started_at = episode.metadata.get("started_at")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|| chrono::Utc::now());
                
                let task_description = episode.metadata.get("task_description").cloned();
                let error_message = episode.metadata.get("error_message").cloned();
                
                let record = ExecutionRecord::new(
                    agent_name,
                    agent_type_str,
                    success,
                    std::time::Duration::from_millis(duration_ms),
                    task_description,
                    error_message,
                );
                
                // Override the started_at with the stored value
                let record = ExecutionRecord {
                    started_at,
                    ..record
                };
                
                records.push(record);
            }
        }
        
        Ok(records)
    }

    async fn load_task_metrics(&self, task_type: &str) -> Result<Option<TaskMetrics>> {
        // Query episodes for task metrics
        let episodes = self.storage.query_episodes_by_metadata("monitoring_type", "task_metrics").await?;
        
        for episode in episodes {
            if let Some(stored_task_type) = episode.metadata.get("task_type") {
                if stored_task_type == task_type {
                    if let Some(metrics_json) = episode.metadata.get("metrics_data") {
                        let metrics: TaskMetrics = serde_json::from_str(metrics_json)?;
                        return Ok(Some(metrics));
                    }
                }
            }
        }
        
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
