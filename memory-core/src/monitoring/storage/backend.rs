//! Monitoring storage backend trait and simple implementation
//!
//! Provides the trait definition and basic storage implementation
//! for monitoring data persistence.

use super::super::types::{AgentMetrics, ExecutionRecord, TaskMetrics};
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
        metadata.insert(
            "monitoring_type".to_string(),
            "execution_record".to_string(),
        );
        metadata.insert("agent_name".to_string(), record.agent_name.clone());
        metadata.insert("agent_type".to_string(), record.agent_type.to_string());
        metadata.insert("success".to_string(), record.success.to_string());
        metadata.insert(
            "duration_ms".to_string(),
            record.duration.as_millis().to_string(),
        );
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
            outcome: Some(if record.success {
                TaskOutcome::Success {
                    verdict: "Execution completed".to_string(),
                    artifacts: vec![],
                }
            } else {
                TaskOutcome::Failure {
                    reason: "Execution failed".to_string(),
                    error_details: record.error_message.clone(),
                }
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
            applied_patterns: vec![],
            salient_features: None,
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
            start_time: metrics.last_execution.unwrap_or_else(Utc::now),
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
            applied_patterns: vec![],
            salient_features: None,
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
            applied_patterns: vec![],
            salient_features: None,
            metadata,
        };

        self.storage.store_episode(&episode).await?;
        Ok(())
    }

    async fn load_agent_metrics(&self, agent_name: &str) -> Result<Option<AgentMetrics>> {
        // Query episodes for agent metrics
        let episodes = self
            .storage
            .query_episodes_by_metadata("monitoring_type", "agent_metrics")
            .await?;

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
        let mut episodes = self
            .storage
            .query_episodes_by_metadata("monitoring_type", "execution_record")
            .await?;

        // Filter by agent name if specified
        if let Some(name) = agent_name {
            episodes.retain(|episode| {
                episode
                    .metadata
                    .get("agent_name")
                    .is_some_and(|stored_name| stored_name == name)
            });
        }

        // Sort by timestamp (newest first) and limit
        episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        episodes.truncate(limit);

        let mut records = Vec::new();
        for episode in episodes {
            if let Some(_metrics_data) = episode.metadata.get("metrics_data") {
                // For execution records, we need to reconstruct from metadata since they're stored differently
                let agent_name = episode
                    .metadata
                    .get("agent_name")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                let agent_type_str = episode.metadata.get("agent_type").map_or(
                    crate::monitoring::types::AgentType::Other,
                    |s| match s.as_str() {
                        "feature-implementer" => {
                            crate::monitoring::types::AgentType::FeatureImplementer
                        }
                        "code-reviewer" => crate::monitoring::types::AgentType::CodeReviewer,
                        "test-runner" => crate::monitoring::types::AgentType::TestRunner,
                        "architecture-validator" => {
                            crate::monitoring::types::AgentType::ArchitectureValidator
                        }
                        "debugger" => crate::monitoring::types::AgentType::Debugger,
                        "analysis-swarm" => crate::monitoring::types::AgentType::AnalysisSwarm,
                        "goap-agent" => crate::monitoring::types::AgentType::GoapAgent,
                        _ => crate::monitoring::types::AgentType::Other,
                    },
                );

                let success = episode
                    .metadata
                    .get("success")
                    .and_then(|s| s.parse::<bool>().ok())
                    .unwrap_or(false);

                let duration_ms = episode
                    .metadata
                    .get("duration_ms")
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                let started_at = episode
                    .metadata
                    .get("started_at")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));

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
        let episodes = self
            .storage
            .query_episodes_by_metadata("monitoring_type", "task_metrics")
            .await?;

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
