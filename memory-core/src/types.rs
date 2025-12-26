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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
/// use memory_core::{MemoryConfig, StorageConfig, BatchConfig, ConcurrencyConfig, EvictionPolicy};
///
/// // Default configuration
/// let config = MemoryConfig::default();
///
/// // Custom configuration with embeddings and capacity management
/// let custom_config = MemoryConfig {
///     storage: StorageConfig::default(),
///     enable_embeddings: true,
///     pattern_extraction_threshold: 0.8,  // More selective pattern extraction
///     batch_config: Some(BatchConfig::default()),
///     concurrency: ConcurrencyConfig {
///         max_concurrent_cache_ops: 15,  // Allow more concurrent cache ops
///     },
///     max_episodes: Some(10000),  // Limit to 10k episodes
///     eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
///     enable_summarization: true,
///     summary_min_length: 100,
///     summary_max_length: 200,
///     quality_threshold: 0.7,
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct MemoryConfig {
    /// Storage configuration
    pub storage: StorageConfig,
    /// Whether to compute and use embeddings for semantic search
    pub enable_embeddings: bool,
    /// Minimum quality threshold for extracting patterns (0.0 to 1.0)
    pub pattern_extraction_threshold: f32,
    /// Minimum quality threshold for storing episodes (0.0 to 1.0) - `PREMem`
    pub quality_threshold: f32,
    /// Step batching configuration (None disables batching)
    pub batch_config: Option<BatchConfig>,
    /// Concurrency control configuration
    pub concurrency: ConcurrencyConfig,

    // Phase 2 (GENESIS) - Capacity management
    /// Maximum number of episodes to store (None = unlimited)
    pub max_episodes: Option<usize>,
    /// Eviction policy when capacity is reached (None = no eviction)
    pub eviction_policy: Option<crate::episodic::EvictionPolicy>,

    // Phase 2 (GENESIS) - Semantic summarization
    /// Whether to generate semantic summaries for episodes
    pub enable_summarization: bool,
    /// Minimum summary length in words
    pub summary_min_length: usize,
    /// Maximum summary length in words
    pub summary_max_length: usize,

    // Phase 3 (Spatiotemporal Memory Organization)
    /// Enable spatiotemporal hierarchical indexing (default: true)
    pub enable_spatiotemporal_indexing: bool,
    /// Enable diversity maximization via MMR (default: true)
    pub enable_diversity_maximization: bool,
    /// Lambda parameter for MMR diversity (0.0-1.0, default: 0.7)
    /// 1.0 = pure relevance, 0.0 = pure diversity
    pub diversity_lambda: f32,
    /// Temporal bias weight in retrieval scoring (default: 0.3)
    /// Higher values favor more recent episodes
    pub temporal_bias_weight: f32,
    /// Maximum temporal clusters to search (default: 5)
    pub max_clusters_to_search: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            enable_embeddings: false,
            pattern_extraction_threshold: 0.7,
            quality_threshold: 0.7,
            batch_config: Some(BatchConfig::default()),
            concurrency: ConcurrencyConfig::default(),

            // Phase 2 (GENESIS) - Capacity management defaults
            max_episodes: None, // Unlimited by default
            eviction_policy: Some(crate::episodic::EvictionPolicy::RelevanceWeighted),

            // Phase 2 (GENESIS) - Semantic summarization defaults
            enable_summarization: true,
            summary_min_length: 100,
            summary_max_length: 200,

            // Phase 3 (Spatiotemporal) - Defaults
            enable_spatiotemporal_indexing: true,
            enable_diversity_maximization: true,
            diversity_lambda: 0.7,
            temporal_bias_weight: 0.3,
            max_clusters_to_search: 5,
        }
    }
}

impl MemoryConfig {
    /// Create a `MemoryConfig` from environment variables.
    ///
    /// Reads configuration from environment variables, falling back to defaults
    /// for any missing values.
    ///
    /// # Environment Variables
    ///
    /// ## Phase 2 (GENESIS) - Capacity Management
    /// * `MEMORY_MAX_EPISODES` - Maximum number of episodes to store (default: `None`/unlimited)
    /// * `MEMORY_EVICTION_POLICY` - Eviction policy: `"LRU"` or `"RelevanceWeighted"` (default: `RelevanceWeighted`)
    ///
    /// ## Phase 2 (GENESIS) - Semantic Summarization
    /// * `MEMORY_ENABLE_SUMMARIZATION` - Enable summarization: `"true"` or `"false"` (default: `true`)
    ///
    /// ## Phase 3 (Spatiotemporal) - Hierarchical Retrieval
    /// * `MEMORY_ENABLE_SPATIOTEMPORAL` - Enable spatiotemporal indexing: `"true"` or `"false"` (default: `true`)
    /// * `MEMORY_ENABLE_DIVERSITY` - Enable diversity maximization: `"true"` or `"false"` (default: `true`)
    /// * `MEMORY_DIVERSITY_LAMBDA` - MMR lambda parameter (0.0-1.0, default: `0.7`)
    /// * `MEMORY_TEMPORAL_BIAS` - Temporal bias weight (0.0-1.0, default: `0.3`)
    /// * `MEMORY_MAX_CLUSTERS` - Maximum temporal clusters to search (default: `5`)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::MemoryConfig;
    ///
    /// // With environment variables set:
    /// // MEMORY_MAX_EPISODES=10000
    /// // MEMORY_EVICTION_POLICY=RelevanceWeighted
    /// // MEMORY_ENABLE_SUMMARIZATION=true
    ///
    /// let config = MemoryConfig::from_env();
    /// ```
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Phase 2 (GENESIS) - Capacity management
        if let Ok(max_episodes) = std::env::var("MEMORY_MAX_EPISODES") {
            config.max_episodes = max_episodes.parse().ok();
        }

        if let Ok(policy) = std::env::var("MEMORY_EVICTION_POLICY") {
            config.eviction_policy = match policy.to_lowercase().as_str() {
                "lru" => Some(crate::episodic::EvictionPolicy::LRU),
                "relevanceweighted" | "relevance_weighted" | "relevance-weighted" => {
                    Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
                }
                _ => {
                    tracing::warn!(
                        "Invalid MEMORY_EVICTION_POLICY '{}', using default RelevanceWeighted",
                        policy
                    );
                    Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
                }
            };
        }

        // Phase 2 (GENESIS) - Semantic summarization
        if let Ok(enable_summarization) = std::env::var("MEMORY_ENABLE_SUMMARIZATION") {
            config.enable_summarization = matches!(
                enable_summarization.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        // Phase 3 (Spatiotemporal Memory Organization)
        if let Ok(enable_spatiotemporal) = std::env::var("MEMORY_ENABLE_SPATIOTEMPORAL") {
            config.enable_spatiotemporal_indexing = matches!(
                enable_spatiotemporal.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        if let Ok(enable_diversity) = std::env::var("MEMORY_ENABLE_DIVERSITY") {
            config.enable_diversity_maximization = matches!(
                enable_diversity.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        if let Ok(lambda) = std::env::var("MEMORY_DIVERSITY_LAMBDA") {
            if let Ok(value) = lambda.parse::<f32>() {
                config.diversity_lambda = value.clamp(0.0, 1.0);
            }
        }

        if let Ok(bias) = std::env::var("MEMORY_TEMPORAL_BIAS") {
            if let Ok(value) = bias.parse::<f32>() {
                config.temporal_bias_weight = value.clamp(0.0, 1.0);
            }
        }

        if let Ok(clusters) = std::env::var("MEMORY_MAX_CLUSTERS") {
            if let Ok(value) = clusters.parse::<usize>() {
                config.max_clusters_to_search = value;
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();

        // Verify Phase 2 defaults
        assert_eq!(config.max_episodes, None); // Unlimited by default
        assert!(matches!(
            config.eviction_policy,
            Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
        ));
        assert!(config.enable_summarization);
        assert_eq!(config.summary_min_length, 100);
        assert_eq!(config.summary_max_length, 200);
    }

    #[test]
    #[ignore] // Ignored due to test isolation issues with parallel execution and env vars
    fn test_memory_config_from_env_defaults() {
        // Clear any environment variables that might be set
        std::env::remove_var("MEMORY_MAX_EPISODES");
        std::env::remove_var("MEMORY_EVICTION_POLICY");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");

        let config = MemoryConfig::from_env();

        // Should match defaults
        assert_eq!(config.max_episodes, None);
        assert!(matches!(
            config.eviction_policy,
            Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
        ));
        assert!(config.enable_summarization);
    }

    #[test]
    #[ignore] // Ignored due to test isolation issues with parallel execution and env vars
    fn test_memory_config_from_env_with_values() {
        // Set environment variables
        std::env::set_var("MEMORY_MAX_EPISODES", "10000");
        std::env::set_var("MEMORY_EVICTION_POLICY", "LRU");
        std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", "false");

        let config = MemoryConfig::from_env();

        // Verify values from environment
        assert_eq!(config.max_episodes, Some(10000));
        assert!(matches!(
            config.eviction_policy,
            Some(crate::episodic::EvictionPolicy::LRU)
        ));
        assert!(!config.enable_summarization);

        // Clean up
        std::env::remove_var("MEMORY_MAX_EPISODES");
        std::env::remove_var("MEMORY_EVICTION_POLICY");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
    }

    #[test]
    fn test_memory_config_eviction_policy_variants() {
        let test_cases = vec![
            ("lru", crate::episodic::EvictionPolicy::LRU),
            ("LRU", crate::episodic::EvictionPolicy::LRU),
            (
                "relevanceweighted",
                crate::episodic::EvictionPolicy::RelevanceWeighted,
            ),
            (
                "relevance_weighted",
                crate::episodic::EvictionPolicy::RelevanceWeighted,
            ),
            (
                "relevance-weighted",
                crate::episodic::EvictionPolicy::RelevanceWeighted,
            ),
            (
                "RelevanceWeighted",
                crate::episodic::EvictionPolicy::RelevanceWeighted,
            ),
        ];

        for (input, expected) in test_cases {
            std::env::set_var("MEMORY_EVICTION_POLICY", input);
            let config = MemoryConfig::from_env();
            assert!(
                matches!(config.eviction_policy, Some(policy) if policy == expected),
                "Failed for input: {input}"
            );
            std::env::remove_var("MEMORY_EVICTION_POLICY");
        }
    }

    #[test]
    fn test_memory_config_invalid_eviction_policy() {
        std::env::set_var("MEMORY_EVICTION_POLICY", "invalid_policy");
        let config = MemoryConfig::from_env();

        // Should fall back to default (RelevanceWeighted)
        assert!(matches!(
            config.eviction_policy,
            Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
        ));

        std::env::remove_var("MEMORY_EVICTION_POLICY");
    }

    #[test]
    fn test_memory_config_summarization_boolean_variants() {
        let true_cases = vec!["true", "TRUE", "1", "yes", "YES", "on", "ON"];
        let false_cases = vec!["false", "FALSE", "0", "no", "NO", "off", "OFF"];

        for input in true_cases {
            std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
            let config = MemoryConfig::from_env();
            assert!(config.enable_summarization, "Failed for input: {input}");
            std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
        }

        for input in false_cases {
            std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
            let config = MemoryConfig::from_env();
            assert!(!config.enable_summarization, "Failed for input: {input}");
            std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
        }
    }

    #[test]
    fn test_memory_config_max_episodes_parsing() {
        // Valid number
        std::env::set_var("MEMORY_MAX_EPISODES", "5000");
        let config = MemoryConfig::from_env();
        assert_eq!(config.max_episodes, Some(5000));
        std::env::remove_var("MEMORY_MAX_EPISODES");

        // Invalid number - should fall back to None
        std::env::set_var("MEMORY_MAX_EPISODES", "not_a_number");
        let config = MemoryConfig::from_env();
        assert_eq!(config.max_episodes, None);
        std::env::remove_var("MEMORY_MAX_EPISODES");
    }
}
