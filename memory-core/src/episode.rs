use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{
    ExecutionResult, Reflection, RewardScore, TaskContext, TaskOutcome, TaskType,
};

/// Unique identifier for patterns
pub type PatternId = Uuid;

/// A single execution step within an episode
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
    /// Create a new execution step
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

    /// Check if this step was successful
    pub fn is_success(&self) -> bool {
        self.result
            .as_ref()
            .map(|r| r.is_success())
            .unwrap_or(false)
    }
}

/// Complete episode from start to finish
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
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Episode {
    /// Create a new episode
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
            metadata: HashMap::new(),
        }
    }

    /// Check if episode is complete
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some() && self.outcome.is_some()
    }

    /// Get duration of episode (if complete)
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Add a new execution step
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    /// Complete the episode with an outcome
    pub fn complete(&mut self, outcome: TaskOutcome) {
        self.end_time = Some(Utc::now());
        self.outcome = Some(outcome);
    }

    /// Get count of successful steps
    pub fn successful_steps_count(&self) -> usize {
        self.steps.iter().filter(|s| s.is_success()).count()
    }

    /// Get count of failed steps
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
        let mut episode = Episode::new(
            "Test task".to_string(),
            context,
            TaskType::Testing,
        );

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
        let mut step = ExecutionStep::new(1, "read_file".to_string(), "Read source file".to_string());

        assert!(!step.is_success());

        step.result = Some(ExecutionResult::Success {
            output: "File contents".to_string(),
        });

        assert!(step.is_success());
    }

    #[test]
    fn test_add_steps() {
        let context = TaskContext::default();
        let mut episode = Episode::new(
            "Test task".to_string(),
            context,
            TaskType::Analysis,
        );

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
