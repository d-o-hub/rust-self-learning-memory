//! Contextual insight generation for episode reflections

use crate::episode::Episode;
use crate::types::TaskOutcome;

use super::helpers;

/// Minimum step count to generate detailed reflections
const MIN_STEPS_FOR_REFLECTION: usize = 2;

/// Generate key insights from the episode
pub(super) fn generate_insights(episode: &Episode, max_items: usize) -> Vec<String> {
    let mut insights = Vec::new();

    // Only generate detailed insights for episodes with enough steps
    if episode.steps.len() < MIN_STEPS_FOR_REFLECTION {
        return insights;
    }

    // Insight from step patterns
    if let Some(pattern_insight) = analyze_step_patterns(episode) {
        insights.push(pattern_insight);
    }

    // Insight from error recovery
    if let Some(recovery_insight) = identify_error_recovery_pattern(episode) {
        insights.push(recovery_insight);
    }

    // Insight from context
    let context_insight = format!(
        "Task in {} domain with {:?} complexity",
        episode.context.domain, episode.context.complexity
    );
    insights.push(context_insight);

    // Insight from tool diversity
    let unique_tools = helpers::count_unique_tools(episode);
    if unique_tools > 5 {
        insights.push(format!(
            "Task required diverse toolset ({unique_tools} different tools)"
        ));
    } else if unique_tools == 1 && episode.steps.len() > 3 {
        insights.push("Task accomplished with single tool - potential for automation".to_string());
    }

    // Insight from execution time per step
    if let Some(avg_latency) = helpers::calculate_average_latency(episode) {
        if avg_latency > 5000 {
            // > 5 seconds
            insights.push(format!(
                "High average step latency: {avg_latency}ms - consider optimization"
            ));
        }
    }

    // Limit to max items
    insights.truncate(max_items);
    insights
}

/// Generate contextual insights with deep analysis
pub(super) fn generate_contextual_insights(episode: &Episode) -> Vec<String> {
    let mut insights = Vec::new();

    // Insight about task complexity vs execution
    if let Some(complexity_insight) = analyze_complexity_alignment(episode) {
        insights.push(complexity_insight);
    }

    // Insight about learning and adaptation
    if let Some(learning_insight) = analyze_learning_indicators(episode) {
        insights.push(learning_insight);
    }

    // Insight about strategy effectiveness
    if let Some(strategy_insight) = analyze_strategy_effectiveness(episode) {
        insights.push(strategy_insight);
    }

    // Recommendations for similar tasks
    if let Some(recommendation) = generate_recommendations_for_similar_tasks(episode) {
        insights.push(recommendation);
    }

    insights
}

// Helper methods

fn analyze_step_patterns(episode: &Episode) -> Option<String> {
    let total_steps = episode.steps.len();
    let successful_steps = episode.successful_steps_count();

    if total_steps == 0 {
        return None;
    }

    let success_rate = successful_steps as f32 / total_steps as f32;

    if success_rate == 1.0 {
        Some("All steps executed successfully - reliable execution pattern".to_string())
    } else if success_rate >= 0.8 {
        Some(format!(
            "High reliability pattern with {:.0}% step success rate",
            success_rate * 100.0
        ))
    } else if success_rate < 0.5 {
        Some(format!(
            "Low reliability pattern ({:.0}% success) - review approach",
            success_rate * 100.0
        ))
    } else {
        None
    }
}

fn identify_error_recovery_pattern(episode: &Episode) -> Option<String> {
    for i in 0..episode.steps.len().saturating_sub(1) {
        let current = &episode.steps[i];
        let next = &episode.steps[i + 1];

        // Check if error was followed by success
        if !current.is_success() && next.is_success() {
            return Some(format!(
                "Successfully recovered from error using '{}'",
                next.tool
            ));
        }
    }

    None
}

fn analyze_complexity_alignment(episode: &Episode) -> Option<String> {
    let step_count = episode.steps.len();
    let complexity = &episode.context.complexity;

    let expected_steps = match complexity {
        crate::types::ComplexityLevel::Simple => 5,
        crate::types::ComplexityLevel::Moderate => 15,
        crate::types::ComplexityLevel::Complex => 30,
    };

    if step_count < expected_steps / 2 {
        Some(format!(
            "Task complexity ({complexity:?}) handled more efficiently than expected ({step_count} vs ~{expected_steps} steps)"
        ))
    } else if step_count > expected_steps * 2 {
        Some(format!(
            "Task required more steps than typical for {complexity:?} complexity - may need approach refinement"
        ))
    } else {
        None
    }
}

fn analyze_learning_indicators(episode: &Episode) -> Option<String> {
    let pattern_count = episode.patterns.len();

    if pattern_count >= 3 {
        Some(format!(
            "Strong learning episode: discovered {pattern_count} reusable patterns for future tasks"
        ))
    } else if pattern_count > 0 && episode.successful_steps_count() > 0 {
        Some(format!(
            "Learning opportunity: {pattern_count} pattern(s) identified - build on this for similar tasks"
        ))
    } else if helpers::detect_error_recovery(episode) {
        Some(
            "Valuable learning from error recovery - demonstrates adaptability and problem-solving"
                .to_string(),
        )
    } else {
        None
    }
}

fn analyze_strategy_effectiveness(episode: &Episode) -> Option<String> {
    if episode.steps.is_empty() {
        return None;
    }

    let success_rate = episode.successful_steps_count() as f32 / episode.steps.len() as f32;
    let duration = episode.duration()?;

    if success_rate > 0.8 && duration.num_seconds() < 120 {
        Some(
            "Highly effective strategy: high success rate with quick execution - replicate for similar tasks"
                .to_string(),
        )
    } else if success_rate < 0.5 {
        Some(format!(
            "Strategy needs refinement: {:.0}% success rate indicates need for different approach",
            success_rate * 100.0
        ))
    } else {
        None
    }
}

fn generate_recommendations_for_similar_tasks(episode: &Episode) -> Option<String> {
    if let Some(TaskOutcome::Success { .. }) = &episode.outcome {
        let key_tools: Vec<_> = episode
            .steps
            .iter()
            .filter(|s| s.is_success())
            .map(|s| s.tool.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(3)
            .collect();

        if key_tools.is_empty() {
            None
        } else {
            Some(format!(
                "For similar {} tasks in {}, prioritize: {}",
                episode.context.domain,
                episode
                    .context
                    .language
                    .as_deref()
                    .unwrap_or("any language"),
                key_tools.join(", ")
            ))
        }
    } else {
        None
    }
}
