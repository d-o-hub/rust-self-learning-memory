use serde::{Deserialize, Serialize};

// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum length for task descriptions (10KB).
///
/// Prevents `DoS` attacks via unbounded input strings that could exhaust
/// memory during serialization or storage operations.
pub const MAX_DESCRIPTION_LEN: usize = 10_000;

/// Maximum number of execution steps per episode (1000).
///
/// Prevents resource exhaustion from episodes with excessive step logging.
pub const MAX_STEP_COUNT: usize = 1_000;

/// Maximum size for artifact data (1MB).
///
/// Limits the size of individual artifacts stored in episodes to prevent
/// storage bloat and memory exhaustion.
pub const MAX_ARTIFACT_SIZE: usize = 1_000_000;

/// Maximum length for step observations (10KB).
///
/// Prevents unbounded observation strings in execution steps.
pub const MAX_OBSERVATION_LEN: usize = 10_000;

/// Maximum size for serialized episode data (10MB).
///
/// Prevents `DoS` attacks via unbounded episode serialization that could
/// exhaust memory during bincode encoding/decoding operations.
pub const MAX_EPISODE_SIZE: usize = 10_000_000;

/// Maximum size for serialized pattern data (1MB).
///
/// Limits the size of individual patterns during serialization to prevent
/// memory exhaustion during bincode operations.
pub const MAX_PATTERN_SIZE: usize = 1_000_000;

/// Maximum size for serialized heuristic data (1MB).
///
/// Limits the size of individual heuristics during serialization to prevent
/// memory exhaustion during bincode operations.
pub const MAX_HEURISTIC_SIZE: usize = 1_000_000;

// ============================================================================
// Type Definitions
// ============================================================================

/// Task complexity level classification.
///
/// Used to categorize tasks by their inherent difficulty and scope.
/// This helps the system match similar tasks during retrieval and
/// adjust reward calculations appropriately.
///
/// # Examples
///
/// ```
/// use memory_core::ComplexityLevel;
///
/// let simple_task = ComplexityLevel::Simple;      // Single-step, straightforward
/// let moderate_task = ComplexityLevel::Moderate;  // Multi-step, some complexity
/// let complex_task = ComplexityLevel::Complex;    // Multi-phase, many considerations
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Simple, single-step tasks with minimal complexity.
    Simple,
    /// Moderate tasks requiring multiple steps or considerations.
    Moderate,
    /// Complex tasks with many dependencies and edge cases.
    Complex,
}

/// Metadata context for a task, used for similarity matching and retrieval.
///
/// Provides rich contextual information about a task to enable accurate
/// matching of relevant past episodes. The more fields populated, the
/// better the retrieval quality.
///
/// # Fields
///
/// * `language` - Programming language being used (optional)
/// * `framework` - Framework or library being used (optional)
/// * `complexity` - Difficulty level of the task
/// * `domain` - High-level domain or category
/// * `tags` - Additional free-form tags for categorization
///
/// # Examples
///
/// ```
/// use memory_core::{TaskContext, ComplexityLevel};
///
/// // Web API development context
/// let context = TaskContext {
///     language: Some("rust".to_string()),
///     framework: Some("axum".to_string()),
///     complexity: ComplexityLevel::Moderate,
///     domain: "web-api".to_string(),
///     tags: vec!["rest".to_string(), "async".to_string()],
/// };
///
/// // Data processing context
/// let data_context = TaskContext {
///     language: Some("python".to_string()),
///     framework: Some("pandas".to_string()),
///     complexity: ComplexityLevel::Complex,
///     domain: "data-science".to_string(),
///     tags: vec!["etl".to_string(), "analytics".to_string()],
/// };
/// ```
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

/// High-level classification of task types for episode categorization.
///
/// Categorizes the primary purpose or nature of a task. This classification
/// helps group similar tasks together for pattern extraction and retrieval.
///
/// # Examples
///
/// ```
/// use memory_core::TaskType;
///
/// let code_task = TaskType::CodeGeneration;  // Writing new code
/// let fix_task = TaskType::Debugging;         // Finding and fixing bugs
/// let test_task = TaskType::Testing;          // Writing or running tests
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Writing new code or implementing features
    CodeGeneration,
    /// Identifying and fixing bugs or errors
    Debugging,
    /// Improving existing code structure or quality
    Refactoring,
    /// Writing or executing tests
    Testing,
    /// Analyzing code, data, or system behavior
    Analysis,
    /// Writing or updating documentation
    Documentation,
    /// Tasks that don't fit other categories
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

/// Final outcome of a completed task.
///
/// Represents the result after task execution, including success status
/// and relevant details. This information is used to calculate rewards
/// and extract patterns.
///
/// # Variants
///
/// * [`Success`](TaskOutcome::Success) - Task completed successfully
/// * [`PartialSuccess`](TaskOutcome::PartialSuccess) - Task partially completed
/// * [`Failure`](TaskOutcome::Failure) - Task failed to complete
///
/// # Examples
///
/// ```
/// use memory_core::TaskOutcome;
///
/// // Complete success
/// let success = TaskOutcome::Success {
///     verdict: "All tests passing, feature complete".to_string(),
///     artifacts: vec!["auth.rs".to_string(), "auth_test.rs".to_string()],
/// };
///
/// // Partial success
/// let partial = TaskOutcome::PartialSuccess {
///     verdict: "Core functionality working".to_string(),
///     completed: vec!["login".to_string(), "logout".to_string()],
///     failed: vec!["password_reset".to_string()],
/// };
///
/// // Failure
/// let failure = TaskOutcome::Failure {
///     reason: "Compilation errors".to_string(),
///     error_details: Some("Type mismatch on line 42".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskOutcome {
    /// Task completed successfully with all objectives met
    Success {
        /// Summary of what was accomplished
        verdict: String,
        /// Files or outputs produced
        artifacts: Vec<String>,
    },
    /// Task partially completed with some objectives met
    PartialSuccess {
        /// Summary of partial completion
        verdict: String,
        /// Items successfully completed
        completed: Vec<String>,
        /// Items that failed or weren't completed
        failed: Vec<String>,
    },
    /// Task failed to complete
    Failure {
        /// High-level reason for failure
        reason: String,
        /// Detailed error information (optional)
        error_details: Option<String>,
    },
}

/// Result of executing a single step within an episode.
///
/// Records the outcome of an individual execution step, including
/// success status and relevant output or error information.
///
/// # Examples
///
/// ```
/// use memory_core::ExecutionResult;
///
/// // Successful execution
/// let success = ExecutionResult::Success {
///     output: "File processed successfully".to_string(),
/// };
///
/// assert!(success.is_success());
///
/// // Error during execution
/// let error = ExecutionResult::Error {
///     message: "File not found".to_string(),
/// };
///
/// assert!(!error.is_success());
///
/// // Timeout
/// let timeout = ExecutionResult::Timeout;
/// assert!(!timeout.is_success());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Step completed successfully
    Success {
        /// Output or result from the step
        output: String,
    },
    /// Step encountered an error
    Error {
        /// Error message
        message: String,
    },
    /// Step exceeded time limit
    Timeout,
}

impl ExecutionResult {
    /// Check if this result represents a successful execution.
    ///
    /// # Returns
    ///
    /// `true` if the result is [`ExecutionResult::Success`], `false` otherwise.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }
}

/// Calculated reward score for a completed episode.
///
/// Quantifies the quality and efficiency of task execution using multiple
/// scoring dimensions. Higher scores indicate better performance and are
/// used to rank episodes during retrieval.
///
/// The total score is computed from base outcome, efficiency, complexity,
/// quality, and learning factors.
///
/// # Score Ranges
///
/// * `total` - Combined score, typically 0.0 to 2.0
/// * `base` - Outcome-based score: 1.0 (success), 0.5 (partial), 0.0 (failure)
/// * `efficiency` - Multiplier based on execution speed: 0.5 to 1.5
/// * `complexity_bonus` - Bonus for handling complexity: 1.0 to 1.3
/// * `quality_multiplier` - Quality factor: 0.8 to 1.2
/// * `learning_bonus` - Bonus for novel patterns: 0.0 to 0.5
///
/// # Examples
///
/// ```
/// use memory_core::RewardScore;
///
/// let high_score = RewardScore {
///     total: 1.8,
///     base: 1.0,              // Full success
///     efficiency: 1.2,        // 20% faster than average
///     complexity_bonus: 1.2,  // Handled complex task well
///     quality_multiplier: 1.1, // High quality output
///     learning_bonus: 0.3,    // Discovered new pattern
/// };
/// ```
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

/// Generated reflection analyzing episode execution.
///
/// Provides structured analysis of what worked, what didn't, and key
/// learnings from the task. Used to improve future task execution.
///
/// # Fields
///
/// * `successes` - List of things that worked well
/// * `improvements` - List of areas for improvement
/// * `insights` - Key learnings or discoveries
/// * `generated_at` - Timestamp when reflection was created
///
/// # Examples
///
/// ```
/// use memory_core::Reflection;
/// use chrono::Utc;
///
/// let reflection = Reflection {
///     successes: vec![
///         "Efficient error handling pattern used".to_string(),
///         "Good test coverage achieved".to_string(),
///     ],
///     improvements: vec![
///         "Could reduce duplication in helper functions".to_string(),
///     ],
///     insights: vec![
///         "Builder pattern works well for this domain".to_string(),
///     ],
///     generated_at: Utc::now(),
/// };
/// ```
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

/// Aggregated statistics about pattern usage and outcomes.
///
/// Tracks success rates and performance metrics for patterns across
/// multiple episodes. Used to evaluate pattern effectiveness.
///
/// # Examples
///
/// ```
/// use memory_core::OutcomeStats;
///
/// let stats = OutcomeStats {
///     success_count: 8,
///     failure_count: 2,
///     total_count: 10,
///     avg_duration_secs: 45.5,
/// };
///
/// assert_eq!(stats.success_rate(), 0.8);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeStats {
    /// Number of successful outcomes
    pub success_count: usize,
    /// Number of failed outcomes
    pub failure_count: usize,
    /// Total number of outcomes tracked
    pub total_count: usize,
    /// Average execution duration in seconds
    pub avg_duration_secs: f32,
}

impl OutcomeStats {
    /// Calculate the success rate as a fraction.
    ///
    /// # Returns
    ///
    /// Success rate between 0.0 and 1.0, or 0.0 if no data available.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::OutcomeStats;
    ///
    /// let stats = OutcomeStats {
    ///     success_count: 7,
    ///     failure_count: 3,
    ///     total_count: 10,
    ///     avg_duration_secs: 30.0,
    /// };
    ///
    /// assert_eq!(stats.success_rate(), 0.7);
    /// ```
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.total_count as f32
        }
    }
}

/// Supporting evidence for a learned heuristic or pattern.
///
/// Tracks which episodes support a heuristic and the empirical
/// success rate, providing confidence in the learned rule.
///
/// # Examples
///
/// ```
/// use memory_core::Evidence;
/// use uuid::Uuid;
///
/// let evidence = Evidence {
///     episode_ids: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
///     success_rate: 0.85,
///     sample_size: 20,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Episode IDs that provide evidence for this heuristic
    pub episode_ids: Vec<uuid::Uuid>,
    /// Success rate when this heuristic was applied (0.0 to 1.0)
    pub success_rate: f32,
    /// Total number of episodes in the evidence set
    pub sample_size: usize,
}

/// Configuration for storage backend behavior.
///
/// Controls caching limits, synchronization timing, and optimization features
/// for the storage layer.
///
/// # Examples
///
/// ```
/// use memory_core::StorageConfig;
///
/// // Default configuration
/// let config = StorageConfig::default();
///
/// // Custom configuration
/// let custom_config = StorageConfig {
///     max_episodes_cache: 5000,      // Cache up to 5000 episodes
///     sync_interval_secs: 60,        // Sync every minute
///     enable_compression: true,      // Enable compression for storage
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum number of episodes to keep in cache
    pub max_episodes_cache: usize,
    /// Interval in seconds between cache-to-durable syncs
    pub sync_interval_secs: u64,
    /// Whether to compress data when storing
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

// BatchConfig is defined in memory::step_buffer and re-exported here
pub use crate::memory::step_buffer::BatchConfig;

/// Configuration for concurrency control to prevent cache contention.
///
/// Limits concurrent cache operations to prevent blocking the async runtime
/// when many operations occur simultaneously (e.g., from MCP server).
///
/// # Examples
///
/// ```
/// use memory_core::ConcurrencyConfig;
///
/// // Default configuration (moderate concurrency)
/// let config = ConcurrencyConfig::default();
///
/// // High concurrency for busy systems
/// let high_concurrency = ConcurrencyConfig {
///     max_concurrent_cache_ops: 20,
/// };
///
/// // Low concurrency for resource-constrained environments
/// let low_concurrency = ConcurrencyConfig {
///     max_concurrent_cache_ops: 5,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    /// Maximum number of concurrent cache operations allowed.
    ///
    /// Limits how many redb operations can run simultaneously to prevent
    /// overwhelming the async runtime with blocking tasks. Default is 10.
    pub max_concurrent_cache_ops: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_cache_ops: 10,
        }
    }
}

/// Main configuration for the self-learning memory system.
///
/// Controls all aspects of memory behavior including storage, pattern
/// extraction, concurrency control, and optional features like embeddings.
///
/// # Examples
///
/// ```
/// use memory_core::{MemoryConfig, StorageConfig, BatchConfig, ConcurrencyConfig};
///
/// // Default configuration
/// let config = MemoryConfig::default();
///
/// // Custom configuration with embeddings and concurrency control
/// let custom_config = MemoryConfig {
///     storage: StorageConfig::default(),
///     enable_embeddings: true,
///     pattern_extraction_threshold: 0.8,  // More selective pattern extraction
///     batch_config: Some(BatchConfig::default()),
///     concurrency: ConcurrencyConfig {
///         max_concurrent_cache_ops: 15,  // Allow more concurrent cache ops
///     },
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Storage configuration
    pub storage: StorageConfig,
    /// Whether to compute and use embeddings for semantic search
    pub enable_embeddings: bool,
    /// Minimum quality threshold for extracting patterns (0.0 to 1.0)
    pub pattern_extraction_threshold: f32,
    /// Step batching configuration (None disables batching)
    pub batch_config: Option<BatchConfig>,
    /// Concurrency control configuration
    pub concurrency: ConcurrencyConfig,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            enable_embeddings: false,
            pattern_extraction_threshold: 0.7,
            batch_config: Some(BatchConfig::default()),
            concurrency: ConcurrencyConfig::default(),
        }
    }
}
