use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{ExecutionResult, Reflection, RewardScore, TaskContext, TaskOutcome, TaskType};

/// Unique identifier for patterns extracted from episodes.
///
/// Each pattern gets a unique ID when created, allowing episodes to
/// reference patterns and track pattern usage over time.
pub type PatternId = Uuid;

/// A single execution step within an episode.
///
/// Represents one discrete action or operation performed during task execution.
/// Steps are logged sequentially and form the detailed execution trace of an episode.
///
/// # Fields
///
/// * `step_number` - Sequential position in the episode (1-indexed)
/// * `timestamp` - When the step was executed
/// * `tool` - Tool, function, or agent that performed the step
/// * `action` - Human-readable description of what was done
/// * `parameters` - Input parameters or configuration (as JSON)
/// * `result` - Outcome of the step execution
/// * `latency_ms` - How long the step took to execute
/// * `tokens_used` - LLM tokens consumed (if applicable)
/// * `metadata` - Additional key-value metadata
///
/// # Examples
///
/// ```
/// use memory_core::{ExecutionStep, ExecutionResult};
/// use std::collections::HashMap;
///
/// // Create a new step
/// let mut step = ExecutionStep::new(
///     1,
///     "file_reader".to_string(),
///     "Read configuration file".to_string()
/// );
///
/// // Add parameters and result
/// step.parameters = serde_json::json!({
///     "path": "/etc/config.toml"
/// });
/// step.result = Some(ExecutionResult::Success {
///     output: "Config loaded successfully".to_string(),
/// });
/// step.latency_ms = 15;
///
/// assert!(step.is_success());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number in sequence (1-indexed)
    pub step_number: usize,
    /// When this step was executed
    pub timestamp: DateTime<Utc>,
    /// Tool or function used
    pub tool: String,
    /// Description of action taken
    pub action: String,
    /// Input parameters (as JSON)
    pub parameters: serde_json::Value,
    /// Result of execution
    pub result: Option<ExecutionResult>,
    /// Execution time in milliseconds
    pub latency_ms: u64,
    /// Number of tokens used (if applicable)
    pub tokens_used: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExecutionStep {
    /// Create a new execution step with default values.
    ///
    /// Initializes a step with the current timestamp, empty parameters,
    /// no result, and zero latency. Fields like `result`, `latency_ms`,
    /// and `parameters` should be set after execution.
    ///
    /// # Arguments
    ///
    /// * `step_number` - Position in the execution sequence (1-indexed)
    /// * `tool` - Name of the tool or agent executing the step
    /// * `action` - Human-readable description of the action
    ///
    /// # Returns
    ///
    /// A new `ExecutionStep` initialized with current timestamp and defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::ExecutionStep;
    ///
    /// let step = ExecutionStep::new(
    ///     1,
    ///     "code_analyzer".to_string(),
    ///     "Analyze function complexity".to_string()
    /// );
    ///
    /// assert_eq!(step.step_number, 1);
    /// assert_eq!(step.tool, "code_analyzer");
    /// ```
    pub fn new(step_number: usize, tool: String, action: String) -> Self {
        Self {
            step_number,
            timestamp: Utc::now(),
            tool,
            action,
            parameters: serde_json::json!({}),
            result: None,
            latency_ms: 0,
            tokens_used: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if this step was successful.
    ///
    /// A step is considered successful if it has a result and that result
    /// is [`ExecutionResult::Success`].
    ///
    /// # Returns
    ///
    /// `true` if the step completed successfully, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{ExecutionStep, ExecutionResult};
    ///
    /// let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    ///
    /// // No result yet
    /// assert!(!step.is_success());
    ///
    /// // Add success result
    /// step.result = Some(ExecutionResult::Success {
    ///     output: "Done".to_string(),
    /// });
    /// assert!(step.is_success());
    ///
    /// // Add error result
    /// step.result = Some(ExecutionResult::Error {
    ///     message: "Failed".to_string(),
    /// });
    /// assert!(!step.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.result
            .as_ref()
            .map(|r| r.is_success())
            .unwrap_or(false)
    }
}

/// Complete record of a task execution from start to finish.
///
/// An episode captures the entire lifecycle of a task, including:
/// - Task description and context
/// - All execution steps
/// - Final outcome
/// - Learning outputs (reward, reflection, patterns)
///
/// Episodes are the fundamental unit of learning in the memory system.
/// They record what was done, how it was done, and whether it succeeded,
/// enabling the system to learn from experience.
///
/// # Lifecycle
///
/// 1. **Created** - Episode starts with [`Episode::new()`]
/// 2. **In Progress** - Steps are added with [`add_step()`](Episode::add_step)
/// 3. **Completed** - Episode finalized with [`complete()`](Episode::complete)
/// 4. **Analyzed** - Reward and reflection added by memory system
/// 5. **Learned** - Patterns extracted and linked
///
/// # Fields
///
/// * `episode_id` - Unique identifier
/// * `task_type` - Classification of task type
/// * `task_description` - Human-readable task description
/// * `context` - Contextual metadata for retrieval
/// * `start_time` - When episode began
/// * `end_time` - When episode completed (None if in progress)
/// * `steps` - Sequential execution steps
/// * `outcome` - Final result (None if in progress)
/// * `reward` - Calculated reward score (None until analyzed)
/// * `reflection` - Generated insights (None until analyzed)
/// * `patterns` - IDs of extracted patterns (empty until analyzed)
/// * `metadata` - Additional key-value metadata
///
/// # Examples
///
/// ```
/// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
/// use memory_core::ExecutionResult;
///
/// // Create a new episode
/// let mut episode = Episode::new(
///     "Implement user authentication".to_string(),
///     TaskContext::default(),
///     TaskType::CodeGeneration,
/// );
///
/// // Add execution steps
/// let mut step = ExecutionStep::new(1, "planner".to_string(), "Plan implementation".to_string());
/// step.result = Some(ExecutionResult::Success {
///     output: "Plan created".to_string(),
/// });
/// episode.add_step(step);
///
/// // Complete the episode
/// episode.complete(TaskOutcome::Success {
///     verdict: "Authentication implemented successfully".to_string(),
///     artifacts: vec!["auth.rs".to_string()],
/// });
///
/// assert!(episode.is_complete());
/// assert!(episode.duration().is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    /// Unique episode identifier
    pub episode_id: Uuid,
    /// Type of task
    pub task_type: TaskType,
    /// Description of the task
    pub task_description: String,
    /// Task context and metadata
    pub context: TaskContext,
    /// When episode started
    pub start_time: DateTime<Utc>,
    /// When episode completed (None if in progress)
    pub end_time: Option<DateTime<Utc>>,
    /// Execution steps
    pub steps: Vec<ExecutionStep>,
    /// Final outcome
    pub outcome: Option<TaskOutcome>,
    /// Reward score
    pub reward: Option<RewardScore>,
    /// Reflection on execution
    pub reflection: Option<Reflection>,
    /// Extracted pattern IDs
    pub patterns: Vec<PatternId>,
    /// Extracted heuristic IDs
    pub heuristics: Vec<Uuid>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Episode {
    /// Create a new episode for a task.
    ///
    /// Initializes an episode with a unique ID, current timestamp, and the
    /// provided task information. The episode starts in an incomplete state
    /// with no steps, outcome, or analysis.
    ///
    /// # Arguments
    ///
    /// * `task_description` - Human-readable description of what needs to be done
    /// * `context` - Contextual metadata for categorization and retrieval
    /// * `task_type` - Classification of the task type
    ///
    /// # Returns
    ///
    /// A new `Episode` ready to have steps logged to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, TaskContext, TaskType, ComplexityLevel};
    ///
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("tokio".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "async-io".to_string(),
    ///     tags: vec!["networking".to_string()],
    /// };
    ///
    /// let episode = Episode::new(
    ///     "Implement async HTTP client".to_string(),
    ///     context,
    ///     TaskType::CodeGeneration,
    /// );
    ///
    /// assert!(!episode.is_complete());
    /// assert_eq!(episode.steps.len(), 0);
    /// ```
    pub fn new(task_description: String, context: TaskContext, task_type: TaskType) -> Self {
        Self {
            episode_id: Uuid::new_v4(),
            task_type,
            task_description,
            context,
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            heuristics: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Check if the episode has been completed.
    ///
    /// An episode is considered complete when both `end_time` and `outcome`
    /// have been set via the [`complete()`](Episode::complete) method.
    ///
    /// # Returns
    ///
    /// `true` if the episode is complete, `false` if still in progress.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// assert!(!episode.is_complete());
    ///
    /// episode.complete(TaskOutcome::Success {
    ///     verdict: "Done".to_string(),
    ///     artifacts: vec![],
    /// });
    ///
    /// assert!(episode.is_complete());
    /// ```
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some() && self.outcome.is_some()
    }

    /// Get the total duration of the episode.
    ///
    /// Calculates the time elapsed from episode start to completion.
    /// Returns `None` if the episode hasn't been completed yet.
    ///
    /// # Returns
    ///
    /// Duration as a `chrono::Duration` if complete, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// assert!(episode.duration().is_none());
    ///
    /// episode.complete(TaskOutcome::Success {
    ///     verdict: "Done".to_string(),
    ///     artifacts: vec![],
    /// });
    ///
    /// let duration = episode.duration().unwrap();
    /// assert!(duration.num_milliseconds() >= 0);
    /// ```
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Add a new execution step to this episode.
    ///
    /// Steps should be added in the order they occur during task execution.
    /// Each step captures a discrete action or operation.
    ///
    /// # Arguments
    ///
    /// * `step` - The execution step to add
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, ExecutionStep, TaskContext, TaskType, ExecutionResult};
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let mut step = ExecutionStep::new(1, "tester".to_string(), "Run tests".to_string());
    /// step.result = Some(ExecutionResult::Success { output: "All passed".to_string() });
    ///
    /// episode.add_step(step);
    ///
    /// assert_eq!(episode.steps.len(), 1);
    /// assert_eq!(episode.successful_steps_count(), 1);
    /// ```
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    /// Mark the episode as complete with a final outcome.
    ///
    /// Sets the end time to now and records the task outcome. After calling
    /// this method, [`is_complete()`](Episode::is_complete) will return `true`.
    ///
    /// # Arguments
    ///
    /// * `outcome` - The final outcome of the task
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
    ///
    /// let mut episode = Episode::new(
    ///     "Build feature".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::CodeGeneration,
    /// );
    ///
    /// episode.complete(TaskOutcome::Success {
    ///     verdict: "Feature implemented and tested".to_string(),
    ///     artifacts: vec!["feature.rs".to_string(), "feature_test.rs".to_string()],
    /// });
    ///
    /// assert!(episode.is_complete());
    /// assert!(episode.end_time.is_some());
    /// assert!(episode.outcome.is_some());
    /// ```
    pub fn complete(&mut self, outcome: TaskOutcome) {
        self.end_time = Some(Utc::now());
        self.outcome = Some(outcome);
    }

    /// Count the number of successful execution steps.
    ///
    /// # Returns
    ///
    /// The number of steps with [`ExecutionResult::Success`].
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, ExecutionStep, TaskContext, TaskType, ExecutionResult};
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let mut success_step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    /// success_step.result = Some(ExecutionResult::Success { output: "OK".to_string() });
    ///
    /// let mut error_step = ExecutionStep::new(2, "tool".to_string(), "action".to_string());
    /// error_step.result = Some(ExecutionResult::Error { message: "Fail".to_string() });
    ///
    /// episode.add_step(success_step);
    /// episode.add_step(error_step);
    ///
    /// assert_eq!(episode.successful_steps_count(), 1);
    /// assert_eq!(episode.failed_steps_count(), 1);
    /// ```
    pub fn successful_steps_count(&self) -> usize {
        self.steps.iter().filter(|s| s.is_success()).count()
    }

    /// Count the number of failed execution steps.
    ///
    /// # Returns
    ///
    /// The number of steps that are not successful (error or no result).
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{Episode, ExecutionStep, TaskContext, TaskType, ExecutionResult};
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let mut error_step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    /// error_step.result = Some(ExecutionResult::Error { message: "Failed".to_string() });
    ///
    /// episode.add_step(error_step);
    ///
    /// assert_eq!(episode.failed_steps_count(), 1);
    /// ```
    pub fn failed_steps_count(&self) -> usize {
        self.steps.iter().filter(|s| !s.is_success()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;

    #[test]
    fn test_episode_creation() {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        let episode = Episode::new(
            "Test task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        );

        assert!(!episode.is_complete());
        assert_eq!(episode.task_description, "Test task");
        assert_eq!(episode.context.domain, "web-api");
        assert_eq!(episode.steps.len(), 0);
    }

    #[test]
    fn test_episode_completion() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        assert!(!episode.is_complete());

        let outcome = TaskOutcome::Success {
            verdict: "All tests passed".to_string(),
            artifacts: vec![],
        };

        episode.complete(outcome);

        assert!(episode.is_complete());
        assert!(episode.end_time.is_some());
        assert!(episode.duration().is_some());
    }

    #[test]
    fn test_execution_step() {
        let mut step =
            ExecutionStep::new(1, "read_file".to_string(), "Read source file".to_string());

        assert!(!step.is_success());

        step.result = Some(ExecutionResult::Success {
            output: "File contents".to_string(),
        });

        assert!(step.is_success());
    }

    #[test]
    fn test_add_steps() {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        assert_eq!(episode.steps.len(), 3);
        assert_eq!(episode.successful_steps_count(), 3);
        assert_eq!(episode.failed_steps_count(), 0);
    }
}
