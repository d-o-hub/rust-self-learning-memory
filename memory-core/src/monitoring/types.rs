use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Whether monitoring is enabled
    pub enabled: bool,
    /// Maximum number of execution records to keep in memory
    pub max_records: usize,
    /// Whether to store metrics in durable storage
    pub enable_persistence: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_records: 10000,
            enable_persistence: true,
        }
    }
}

/// Types of agents in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Feature implementation agent
    FeatureImplementer,
    /// Code review agent
    CodeReviewer,
    /// Test runner agent
    TestRunner,
    /// Architecture validator
    ArchitectureValidator,
    /// Debug agent
    Debugger,
    /// Analysis swarm agent
    AnalysisSwarm,
    /// GOAP planning agent
    GoapAgent,
    /// Other agent types
    Other,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::FeatureImplementer => write!(f, "feature-implementer"),
            AgentType::CodeReviewer => write!(f, "code-reviewer"),
            AgentType::TestRunner => write!(f, "test-runner"),
            AgentType::ArchitectureValidator => write!(f, "architecture-validator"),
            AgentType::Debugger => write!(f, "debugger"),
            AgentType::AnalysisSwarm => write!(f, "analysis-swarm"),
            AgentType::GoapAgent => write!(f, "goap-agent"),
            AgentType::Other => write!(f, "other"),
        }
    }
}

impl From<&str> for AgentType {
    fn from(s: &str) -> Self {
        match s {
            "feature-implementer" => AgentType::FeatureImplementer,
            "code-reviewer" => AgentType::CodeReviewer,
            "test-runner" => AgentType::TestRunner,
            "architecture-validator" => AgentType::ArchitectureValidator,
            "debugger" => AgentType::Debugger,
            "analysis-swarm" => AgentType::AnalysisSwarm,
            "goap-agent" => AgentType::GoapAgent,
            _ => AgentType::Other,
        }
    }
}

/// Record of a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Agent name or identifier
    pub agent_name: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Whether the execution was successful
    pub success: bool,
    /// Execution duration
    pub duration: Duration,
    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Task description (optional)
    pub task_description: Option<String>,
    /// Error message if execution failed
    pub error_message: Option<String>,
}

impl ExecutionRecord {
    /// Create a new execution record
    #[must_use]
    pub fn new(
        agent_name: String,
        agent_type: AgentType,
        success: bool,
        duration: Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            agent_name,
            agent_type,
            success,
            duration,
            started_at: chrono::Utc::now(),
            task_description,
            error_message,
        }
    }
}

/// Aggregated metrics for a specific agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent name
    pub agent_name: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Total execution time across all runs
    pub total_duration: Duration,
    /// Average execution time
    pub avg_duration: Duration,
    /// Minimum execution time
    pub min_duration: Duration,
    /// Maximum execution time
    pub max_duration: Duration,
    /// Last execution timestamp
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    /// Current success streak
    pub current_streak: u32,
    /// Longest success streak
    pub longest_streak: u32,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            agent_name: String::new(),
            agent_type: AgentType::Other,
            total_executions: 0,
            successful_executions: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            last_execution: None,
            current_streak: 0,
            longest_streak: 0,
        }
    }
}

impl AgentMetrics {
    /// Calculate success rate (0.0 to 1.0)
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }

    /// Get average duration in seconds
    #[must_use]
    pub fn avg_duration_secs(&self) -> f64 {
        self.avg_duration.as_secs_f64()
    }

    /// Update metrics with a new execution record
    pub fn update(&mut self, record: &ExecutionRecord) {
        self.total_executions += 1;
        self.total_duration += record.duration;
        self.last_execution = Some(record.started_at);

        // Update min/max duration
        if record.duration < self.min_duration {
            self.min_duration = record.duration;
        }
        if record.duration > self.max_duration {
            self.max_duration = record.duration;
        }

        // Update success metrics and streaks
        if record.success {
            self.successful_executions += 1;
            self.current_streak += 1;
            if self.current_streak > self.longest_streak {
                self.longest_streak = self.current_streak;
            }
        } else {
            self.current_streak = 0;
        }

        // Recalculate average
        self.avg_duration = self.total_duration / self.total_executions as u32;
    }
}

/// Aggregated metrics for task types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Task type or category
    pub task_type: String,
    /// Total tasks attempted
    pub total_tasks: u64,
    /// Tasks completed successfully
    pub completed_tasks: u64,
    /// Average completion time
    pub avg_completion_time: Duration,
    /// Success rate by agent type
    pub agent_success_rates: HashMap<AgentType, f64>,
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            task_type: String::new(),
            total_tasks: 0,
            completed_tasks: 0,
            avg_completion_time: Duration::ZERO,
            agent_success_rates: HashMap::new(),
        }
    }
}

impl TaskMetrics {
    /// Calculate overall success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed_tasks as f64 / self.total_tasks as f64
        }
    }

    /// Update metrics with a new execution record
    pub fn update(&mut self, record: &ExecutionRecord) {
        self.total_tasks += 1;

        if record.success {
            self.completed_tasks += 1;
        }

        // Update agent success rates
        let agent_rate = self
            .agent_success_rates
            .entry(record.agent_type)
            .or_insert(0.0);

        // Simple moving average update (could be made more sophisticated)
        let current_rate = if record.success { 1.0 } else { 0.0 };
        *agent_rate = (*agent_rate + current_rate) / 2.0;
    }
}
