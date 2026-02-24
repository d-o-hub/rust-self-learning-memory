use memory_core::{Heuristic, Pattern, Result, TaskContext};

pub(super) fn extract_pattern_data(
    pattern: &Pattern,
) -> Result<(String, TaskContext, Heuristic, f32, usize)> {
    match pattern {
        Pattern::ToolSequence {
            tools,
            context,
            success_rate,
            occurrence_count,
            ..
        } => {
            let desc = format!("Tool sequence: {}", tools.join(" -> "));
            let heur = Heuristic::new(
                format!("When need tools: {}", tools.join(", ")),
                format!("Use sequence: {}", tools.join(" -> ")),
                *success_rate,
            );
            Ok((
                desc,
                context.clone(),
                heur,
                *success_rate,
                *occurrence_count,
            ))
        }
        Pattern::DecisionPoint {
            condition,
            action,
            outcome_stats,
            context,
            ..
        } => {
            let desc = format!("Decision: {} -> {}", condition, action);
            let heur = Heuristic::new(
                condition.clone(),
                action.clone(),
                outcome_stats.success_rate(),
            );
            Ok((
                desc,
                context.clone(),
                heur,
                outcome_stats.success_rate(),
                outcome_stats.total_count,
            ))
        }
        Pattern::ErrorRecovery {
            error_type,
            recovery_steps,
            success_rate,
            context,
            ..
        } => {
            let desc = format!("Error recovery for: {}", error_type);
            let heur = Heuristic::new(
                format!("Error: {}", error_type),
                format!("Recovery: {}", recovery_steps.join(" -> ")),
                *success_rate,
            );
            Ok((
                desc,
                context.clone(),
                heur,
                *success_rate,
                recovery_steps.len(),
            ))
        }
        Pattern::ContextPattern {
            context_features,
            recommended_approach,
            success_rate,
            ..
        } => {
            let desc = format!("Context pattern: {}", recommended_approach);
            let heur = Heuristic::new(
                format!("Features: {}", context_features.join(", ")),
                recommended_approach.clone(),
                *success_rate,
            );
            Ok((
                desc,
                TaskContext::default(),
                heur,
                *success_rate,
                context_features.len(),
            ))
        }
    }
}
