use serde::{Deserialize, Serialize};

// ============================================================================
// Structs
// ============================================================================

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

// Re-export ComplexityLevel for use in this module
pub use super::enums::ComplexityLevel;
