use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
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

impl std::str::FromStr for TaskType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "code_generation" => Ok(TaskType::CodeGeneration),
            "debugging" => Ok(TaskType::Debugging),
            "refactoring" => Ok(TaskType::Refactoring),
            "testing" => Ok(TaskType::Testing),
            "analysis" => Ok(TaskType::Analysis),
            "documentation" => Ok(TaskType::Documentation),
            "other" => Ok(TaskType::Other),
            _ => Err(format!("Unknown TaskType: {s}")),
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

impl std::fmt::Display for TaskOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskOutcome::Success { verdict, .. } => write!(f, "success: {verdict}"),
            TaskOutcome::Failure { reason, .. } => write!(f, "failure: {reason}"),
            TaskOutcome::PartialSuccess { verdict, .. } => write!(f, "partial success: {verdict}"),
        }
    }
}
