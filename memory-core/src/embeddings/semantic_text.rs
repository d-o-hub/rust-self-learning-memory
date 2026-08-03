//! Text-formatting helpers for semantic embedding search.
//!
//! Pure string-building functions used by [`super::semantic_service`] to turn
//! episodes, patterns, queries, and task contexts into searchable text.
//! Extracted from `semantic_service.rs` to keep that file within the
//! 500-LOC source gate.

use crate::episode::Episode;
use crate::patterns::Pattern;
use crate::types::{TaskContext, TaskOutcome};

/// Convert episode to searchable text representation
pub(crate) fn episode_to_text(episode: &Episode) -> String {
    use std::collections::HashSet;
    use std::fmt::Write;

    // Build text directly using format! to avoid intermediate Vec clones
    let mut text = episode.task_description.clone();

    // Context information
    let _ = write!(text, ". domain: {}", episode.context.domain);
    if let Some(lang) = &episode.context.language {
        let _ = write!(text, ". language: {lang}");
    }
    if let Some(framework) = &episode.context.framework {
        let _ = write!(text, ". framework: {framework}");
    }
    if !episode.context.tags.is_empty() {
        let _ = write!(text, ". tags: {}", episode.context.tags.join(", "));
    }

    // Execution summary
    if !episode.steps.is_empty() {
        // Collect unique tools while preserving order
        let mut seen_tools = HashSet::new();
        let mut tools = Vec::new();
        for step in &episode.steps {
            if seen_tools.insert(step.tool.clone()) {
                tools.push(step.tool.clone());
            }
        }
        let _ = write!(text, ". tools used: {}", tools.join(", "));

        let actions: Vec<String> = episode
            .steps
            .iter()
            .take(3) // Take first few actions
            .map(|step| step.action.clone())
            .collect();
        let _ = write!(text, ". actions: {}", actions.join(", "));
    }

    // Outcome summary
    if let Some(outcome) = &episode.outcome {
        match outcome {
            TaskOutcome::Success { verdict, .. } => {
                let _ = write!(text, ". outcome: success - {verdict}");
            }
            TaskOutcome::PartialSuccess { verdict, .. } => {
                let _ = write!(text, ". outcome: partial success - {verdict}");
            }
            TaskOutcome::Failure { reason, .. } => {
                let _ = write!(text, ". outcome: failure - {reason}");
            }
            TaskOutcome::Abstained { reason, .. } => {
                let _ = write!(text, ". outcome: abstained - {reason}");
            }
        }
    }

    text
}

/// Convert pattern to searchable text representation
pub(crate) fn pattern_to_text(pattern: &Pattern) -> String {
    let mut parts = Vec::new();

    // Pattern description based on type
    let description = match pattern {
        Pattern::ToolSequence { tools, .. } => {
            format!("Tool sequence: {}", tools.join(" -> "))
        }
        Pattern::DecisionPoint {
            condition, action, ..
        } => {
            format!("Decision: if {condition} then {action}")
        }
        Pattern::ErrorRecovery {
            error_type,
            recovery_steps,
            ..
        } => {
            format!(
                "Error recovery: {} -> {}",
                error_type,
                recovery_steps.join(" -> ")
            )
        }
        Pattern::ContextPattern {
            context_features,
            recommended_approach,
            ..
        } => {
            format!(
                "Context pattern: {} suggests {}",
                context_features.join(", "),
                recommended_approach
            )
        }
    };
    parts.push(description);

    // Context information
    if let Some(pattern_context) = pattern.context() {
        parts.push(format!("domain: {}", pattern_context.domain));
        if let Some(lang) = &pattern_context.language {
            parts.push(format!("language: {lang}"));
        }
        if !pattern_context.tags.is_empty() {
            parts.push(format!("tags: {}", pattern_context.tags.join(", ")));
        }
    }

    parts.join(". ")
}

/// Create query text from description and context
pub(crate) fn create_query_text(query: &str, context: &TaskContext) -> String {
    let mut parts = vec![query.to_string()];

    parts.push(format!("domain: {}", context.domain));
    if let Some(lang) = &context.language {
        parts.push(format!("language: {lang}"));
    }
    if let Some(framework) = &context.framework {
        parts.push(format!("framework: {framework}"));
    }
    if !context.tags.is_empty() {
        parts.push(format!("tags: {}", context.tags.join(", ")));
    }

    parts.join(". ")
}

/// Convert context to searchable text
pub(crate) fn context_to_text(context: &TaskContext) -> String {
    let mut parts = Vec::new();

    parts.push(format!("domain: {}", context.domain));
    if let Some(lang) = &context.language {
        parts.push(format!("language: {lang}"));
    }
    if let Some(framework) = &context.framework {
        parts.push(format!("framework: {framework}"));
    }
    if !context.tags.is_empty() {
        parts.push(format!("tags: {}", context.tags.join(", ")));
    }
    parts.push(format!("complexity: {:?}", context.complexity));

    parts.join(". ")
}
