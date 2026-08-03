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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExecutionStep;
    use crate::patterns::PatternEffectiveness;
    use crate::types::{ComplexityLevel, OutcomeStats, TaskType};
    use chrono::Duration;
    use uuid::Uuid;

    fn full_context() -> TaskContext {
        TaskContext {
            language: Some("rust".to_string()),
            framework: Some("axum".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["auth".to_string(), "async".to_string()],
        }
    }

    fn episode_with_outcome(outcome: TaskOutcome) -> Episode {
        let mut episode = Episode::new(
            "Implement auth".to_string(),
            full_context(),
            TaskType::CodeGeneration,
        );
        episode.steps = vec![
            ExecutionStep::new(1, "cargo".to_string(), "build".to_string()),
            ExecutionStep::new(2, "cargo".to_string(), "test".to_string()),
            ExecutionStep::new(3, "git".to_string(), "commit".to_string()),
            ExecutionStep::new(4, "git".to_string(), "push".to_string()),
        ];
        episode.outcome = Some(outcome);
        episode
    }

    #[test]
    fn episode_to_text_includes_context_steps_and_success() {
        let episode = episode_with_outcome(TaskOutcome::Success {
            verdict: "All tests pass".to_string(),
            artifacts: vec!["auth.rs".to_string()],
        });

        let text = episode_to_text(&episode);

        assert!(text.starts_with("Implement auth"));
        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("framework: axum"));
        assert!(text.contains("tags: auth, async"));
        // Tools are deduplicated while preserving first-seen order.
        assert!(text.contains("tools used: cargo, git"));
        // Actions take only the first three steps, even with a fourth present.
        assert!(text.contains("actions: build, test, commit"));
        assert!(!text.contains("push"));
        assert!(text.contains("outcome: success - All tests pass"));
    }

    #[test]
    fn episode_to_text_partial_success_outcome() {
        let episode = episode_with_outcome(TaskOutcome::PartialSuccess {
            verdict: "Core done".to_string(),
            completed: vec!["login".to_string()],
            failed: vec!["logout".to_string()],
        });

        let text = episode_to_text(&episode);

        assert!(text.contains("outcome: partial success - Core done"));
    }

    #[test]
    fn episode_to_text_failure_outcome() {
        let episode = episode_with_outcome(TaskOutcome::Failure {
            reason: "Compile error".to_string(),
            error_details: Some("line 42".to_string()),
        });

        let text = episode_to_text(&episode);

        assert!(text.contains("outcome: failure - Compile error"));
    }

    #[test]
    fn episode_to_text_abstained_outcome() {
        let episode = episode_with_outcome(TaskOutcome::Abstained {
            reason: "Infeasible".to_string(),
            stopped_at_step: 2,
            infeasibility_signals: vec!["timeout".to_string()],
        });

        let text = episode_to_text(&episode);

        assert!(text.contains("outcome: abstained - Infeasible"));
    }

    #[test]
    fn episode_to_text_minimal_omits_optional_sections() {
        let episode = Episode::new("Bare".to_string(), TaskContext::default(), TaskType::Other);

        let text = episode_to_text(&episode);

        assert_eq!(text, "Bare. domain: general");
        assert!(!text.contains("language:"));
        assert!(!text.contains("framework:"));
        assert!(!text.contains("tags:"));
        assert!(!text.contains("tools used:"));
        assert!(!text.contains("actions:"));
        assert!(!text.contains("outcome:"));
    }

    #[test]
    fn pattern_to_text_tool_sequence() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["cargo".to_string(), "rustc".to_string()],
            context: full_context(),
            success_rate: 1.0,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        };

        let text = pattern_to_text(&pattern);

        assert!(text.contains("Tool sequence: cargo -> rustc"));
        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("tags: auth, async"));
    }

    #[test]
    fn pattern_to_text_decision_point() {
        let pattern = Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "retry > 3".to_string(),
            action: "escalate".to_string(),
            outcome_stats: OutcomeStats {
                success_count: 8,
                failure_count: 2,
                total_count: 10,
                avg_duration_secs: 1.5,
            },
            context: full_context(),
            effectiveness: PatternEffectiveness::default(),
        };

        let text = pattern_to_text(&pattern);

        assert!(text.contains("Decision: if retry > 3 then escalate"));
        assert!(text.contains("domain: web-api"));
    }

    #[test]
    fn pattern_to_text_error_recovery() {
        let pattern = Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "io_error".to_string(),
            recovery_steps: vec!["retry".to_string(), "fallback".to_string()],
            success_rate: 0.8,
            context: full_context(),
            effectiveness: PatternEffectiveness::default(),
        };

        let text = pattern_to_text(&pattern);

        assert!(text.contains("Error recovery: io_error -> retry -> fallback"));
    }

    #[test]
    fn pattern_to_text_context_pattern_has_no_context_section() {
        let pattern = Pattern::ContextPattern {
            id: Uuid::new_v4(),
            context_features: vec!["high-latency".to_string(), "batch".to_string()],
            recommended_approach: "batch-processing".to_string(),
            evidence: vec![Uuid::new_v4()],
            success_rate: 0.9,
            effectiveness: PatternEffectiveness::default(),
        };

        let text = pattern_to_text(&pattern);

        assert!(text.contains("Context pattern: high-latency, batch suggests batch-processing"));
        // ContextPattern::context() returns None, so no domain section is appended.
        assert!(!text.contains("domain:"));
    }

    #[test]
    fn pattern_to_text_minimal_context_omits_optionals() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["cargo".to_string()],
            context: TaskContext::default(),
            success_rate: 1.0,
            avg_latency: Duration::milliseconds(50),
            occurrence_count: 1,
            effectiveness: PatternEffectiveness::default(),
        };

        let text = pattern_to_text(&pattern);

        assert!(text.contains("Tool sequence: cargo"));
        assert!(text.contains("domain: general"));
        assert!(!text.contains("language:"));
        assert!(!text.contains("tags:"));
    }

    #[test]
    fn create_query_text_includes_all_context_fields() {
        let text = create_query_text("fix auth", &full_context());

        assert!(text.starts_with("fix auth"));
        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("framework: axum"));
        assert!(text.contains("tags: auth, async"));
    }

    #[test]
    fn create_query_text_minimal_context() {
        let text = create_query_text("query", &TaskContext::default());

        assert_eq!(text, "query. domain: general");
        assert!(!text.contains("language:"));
        assert!(!text.contains("framework:"));
        assert!(!text.contains("tags:"));
    }

    #[test]
    fn context_to_text_includes_complexity() {
        let text = context_to_text(&full_context());

        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("framework: axum"));
        assert!(text.contains("tags: auth, async"));
        assert!(text.contains("complexity: Moderate"));
    }

    #[test]
    fn context_to_text_minimal_context() {
        let text = context_to_text(&TaskContext::default());

        assert_eq!(text, "domain: general. complexity: Moderate");
    }
}
