use serde::{Deserialize, Serialize};

/// Task complexity levels for context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

/// Task context containing metadata about the task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskContext {
    /// Programming language (e.g., "rust", "python")
    pub language: Option<String>,
    /// Framework used (e.g., "tokio", "fastapi")
    pub framework: Option<String>,
    /// Task complexity level
    pub complexity: ComplexityLevel,
    /// Domain or category (e.g., "web-api", "data-processing")
    pub domain: String,
    /// Additional tags for categorization
    pub tags: Vec<String>,
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "general".to_string(),
            tags: Vec::new(),
        }
    }
}

/// Task type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    Debugging,
    Refactoring,
    Testing,
    Analysis,
    Documentation,
    Other,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::CodeGeneration => write!(f, "code_generation"),
            TaskType::Debugging => write!(f, "debugging"),
            TaskType::Refactoring => write!(f, "refactoring"),
            TaskType::Testing => write!(f, "testing"),
            TaskType::Analysis => write!(f, "analysis"),
            TaskType::Documentation => write!(f, "documentation"),
            TaskType::Other => write!(f, "other"),
        }
    }
}

/// Task outcome after completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskOutcome {
    Success {
        verdict: String,
        artifacts: Vec<String>,
    },
    PartialSuccess {
        verdict: String,
        completed: Vec<String>,
        failed: Vec<String>,
    },
    Failure {
        reason: String,
        error_details: Option<String>,
    },
}

/// Result of a single execution step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success { output: String },
    Error { message: String },
    Timeout,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }
}

/// Reward score for an episode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardScore {
    /// Total reward (0.0 to infinity, typically 0-2)
    pub total: f32,
    /// Base reward from outcome
    pub base: f32,
    /// Efficiency multiplier
    pub efficiency: f32,
    /// Complexity bonus multiplier
    pub complexity_bonus: f32,
    /// Quality multiplier (based on code quality, test coverage)
    pub quality_multiplier: f32,
    /// Learning bonus (for discovering patterns, improvements)
    pub learning_bonus: f32,
}

/// Reflection on episode execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reflection {
    /// What worked well
    pub successes: Vec<String>,
    /// What could be improved
    pub improvements: Vec<String>,
    /// Key insights from execution
    pub insights: Vec<String>,
    /// When reflection was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Statistics about pattern outcomes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeStats {
    pub success_count: usize,
    pub failure_count: usize,
    pub total_count: usize,
    pub avg_duration_secs: f32,
}

impl OutcomeStats {
    pub fn success_rate(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.total_count as f32
        }
    }
}

/// Evidence for a heuristic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub episode_ids: Vec<uuid::Uuid>,
    pub success_rate: f32,
    pub sample_size: usize,
}

/// Configuration for storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub max_episodes_cache: usize,
    pub sync_interval_secs: u64,
    pub enable_compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_episodes_cache: 1000,
            sync_interval_secs: 300, // 5 minutes
            enable_compression: false,
        }
    }
}

/// Configuration for memory system
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub storage: StorageConfig,
    pub enable_embeddings: bool,
    pub pattern_extraction_threshold: f32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            enable_embeddings: false,
            pattern_extraction_threshold: 0.7,
        }
    }
}
