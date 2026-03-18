//! Step builder and reflection data for playbook generation.
//!
//! Extracted from `generator.rs` to maintain the ≤500 LOC invariant.

use uuid::Uuid;

use super::types::{PlaybookStep, PlaybookSynthesisSource};
use crate::pattern::Pattern;

// Helper functions to reduce nesting in synthesize_steps

pub(super) fn create_tool_step(order: usize, tool: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Use {} tool", tool))
        .with_tool_hint(tool.to_string())
        .with_expected_result(format!("Execute {} operation", tool.replace('_', " ")))
}

pub(super) fn create_decision_step(order: usize, condition: &str, action: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Evaluate: {}", condition))
        .with_expected_result(format!("Then: {}", action))
}

pub(super) fn create_error_step(order: usize, error_type: &str) -> PlaybookStep {
    PlaybookStep::new(order, format!("Handle {} error", error_type)).with_tool_hint("error_handler")
}

pub(super) fn create_context_step(order: usize, recommended_approach: &str) -> PlaybookStep {
    PlaybookStep::new(order, recommended_approach.to_string())
        .with_expected_result("Context-appropriate action")
}

/// Builder for creating playbook steps without excessive nesting.
pub(super) struct StepsBuilder {
    steps: Vec<PlaybookStep>,
    max_steps: usize,
    current_order: usize,
}

impl StepsBuilder {
    pub(super) fn new(max_steps: usize) -> Self {
        Self {
            steps: Vec::new(),
            max_steps,
            current_order: 1,
        }
    }

    pub(super) fn is_full(&self) -> bool {
        self.current_order > self.max_steps
    }

    fn add_step(&mut self, step: PlaybookStep) -> bool {
        if self.current_order > self.max_steps {
            return false;
        }
        self.steps.push(step);
        self.current_order += 1;
        true
    }

    pub(super) fn add_pattern_steps(
        &mut self,
        pattern: &Pattern,
        source: &mut PlaybookSynthesisSource,
    ) {
        match pattern {
            Pattern::ToolSequence { tools, .. } => {
                for tool in tools.iter().take(self.max_steps / 2) {
                    if !self.add_step(create_tool_step(self.current_order, tool)) {
                        break;
                    }
                }
            }
            Pattern::DecisionPoint {
                condition, action, ..
            } => {
                self.add_step(create_decision_step(self.current_order, condition, action));
            }
            Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                self.add_step(create_error_step(self.current_order, error_type));
                for recovery in recovery_steps.iter().take(2) {
                    if !self.add_step(PlaybookStep::new(self.current_order, recovery.clone())) {
                        break;
                    }
                }
            }
            Pattern::ContextPattern {
                recommended_approach,
                evidence,
                ..
            } => {
                self.add_step(create_context_step(
                    self.current_order,
                    recommended_approach,
                ));
                for ep_id in evidence {
                    source.add_episode(*ep_id);
                }
            }
        }
    }

    pub(super) fn build(self) -> Vec<PlaybookStep> {
        self.steps
    }
}

/// Reflection data used for playbook synthesis.
///
/// Extracted from episode reflections for synthesis.
#[derive(Debug, Clone)]
pub struct ReflectionData {
    /// Episode ID this reflection came from
    pub episode_id: Uuid,
    /// Successes from the reflection
    pub successes: Vec<String>,
    /// Improvements identified
    pub improvements: Vec<String>,
    /// Insights gained
    pub insights: Vec<String>,
    /// Steps that failed during execution
    pub failed_steps: Vec<String>,
}

impl ReflectionData {
    /// Create reflection data from a reflection struct.
    #[must_use]
    pub fn from_reflection(episode_id: Uuid, reflection: &crate::types::Reflection) -> Self {
        Self {
            episode_id,
            successes: reflection.successes.clone(),
            improvements: reflection.improvements.clone(),
            insights: reflection.insights.clone(),
            failed_steps: Vec::new(), // Would be extracted from steps
        }
    }
}
