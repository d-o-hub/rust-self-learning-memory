//! Success pattern analysis for episode reflections

use crate::episode::Episode;
use crate::types::TaskOutcome;

use super::helpers;

/// Identify what worked well in the episode
pub(super) fn identify_successes(episode: &Episode, max_items: usize) -> Vec<String> {
    let mut successes = Vec::new();

    // Check overall outcome
    match &episode.outcome {
        Some(TaskOutcome::Success { verdict, artifacts }) => {
            successes.push(format!("Successfully completed task: {}", verdict));

            if !artifacts.is_empty() {
                successes.push(format!("Generated {} artifact(s)", artifacts.len()));
            }
        }
        Some(TaskOutcome::PartialSuccess {
            verdict, completed, ..
        }) => {
            successes.push(format!("Partial success: {}", verdict));
            if !completed.is_empty() {
                successes.push(format!("Completed {} subtask(s)", completed.len()));
            }
        }
        _ => {}
    }

    // Identify successful execution patterns
    let successful_steps = episode.successful_steps_count();
    let total_steps = episode.steps.len();

    if successful_steps > 0 && total_steps > 0 {
        let success_rate = successful_steps as f32 / total_steps as f32;
        if success_rate >= 0.8 {
            successes.push(format!(
                "High execution success rate: {:.1}% ({}/{})",
                success_rate * 100.0,
                successful_steps,
                total_steps
            ));
        }
    }

    // Identify efficient tool usage
    if let Some(tool_pattern) = identify_effective_tool_sequence(episode) {
        successes.push(tool_pattern);
    }

    // Identify quick completion
    if let Some(duration) = episode.duration() {
        let duration_secs = duration.num_seconds();
        if duration_secs < 30 && total_steps > 0 {
            successes.push(format!(
                "Efficient execution: completed in {} seconds",
                duration_secs
            ));
        }
    }

    // Limit to max items
    successes.truncate(max_items);
    successes
}

/// Analyze success patterns with detailed context
pub(super) fn analyze_success_patterns(episode: &Episode) -> Vec<String> {
    let mut patterns = Vec::new();

    // Only analyze if episode was successful
    let is_success = matches!(
        episode.outcome,
        Some(TaskOutcome::Success { .. }) | Some(TaskOutcome::PartialSuccess { .. })
    );

    if !is_success || episode.steps.is_empty() {
        return patterns;
    }

    // Analyze tool combination effectiveness
    if let Some(combo) = analyze_tool_combination_strategy(episode) {
        patterns.push(combo);
    }

    // Analyze execution flow
    if let Some(flow) = analyze_execution_flow(episode) {
        patterns.push(flow);
    }

    // Analyze context-specific success factors
    if let Some(context_factor) = analyze_context_success_factors(episode) {
        patterns.push(context_factor);
    }

    // Analyze efficiency achievements
    if let Some(efficiency) = analyze_efficiency_achievements(episode) {
        patterns.push(efficiency);
    }

    patterns
}

// Helper methods

fn identify_effective_tool_sequence(episode: &Episode) -> Option<String> {
    if episode.steps.len() < 2 {
        return None;
    }

    let successful_tools: Vec<&str> = episode
        .steps
        .iter()
        .filter(|s| s.is_success())
        .map(|s| s.tool.as_str())
        .collect();

    if successful_tools.len() >= 3 {
        let sequence = successful_tools
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(" → ");
        Some(format!("Effective tool sequence: {}", sequence))
    } else {
        None
    }
}

fn analyze_tool_combination_strategy(episode: &Episode) -> Option<String> {
    let successful_tools: Vec<&str> = episode
        .steps
        .iter()
        .filter(|s| s.is_success())
        .map(|s| s.tool.as_str())
        .collect();

    if successful_tools.len() >= 3 {
        let unique_tools: std::collections::HashSet<_> = successful_tools.iter().collect();
        let strategy = if unique_tools.len() == successful_tools.len() {
            "diverse tool strategy"
        } else {
            "focused tool strategy with repetition"
        };

        Some(format!(
            "Effective {} with {} tools: {}",
            strategy,
            unique_tools.len(),
            unique_tools
                .iter()
                .take(3)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    } else {
        None
    }
}

fn analyze_execution_flow(episode: &Episode) -> Option<String> {
    let success_rate = episode.successful_steps_count() as f32 / episode.steps.len() as f32;

    if success_rate > 0.9 {
        Some(format!(
            "Smooth execution flow with {:.0}% success rate - minimal backtracking required",
            success_rate * 100.0
        ))
    } else if helpers::detect_iterative_refinement(episode) {
        Some(
            "Iterative refinement approach: successfully adapted strategy based on feedback"
                .to_string(),
        )
    } else {
        None
    }
}

fn analyze_context_success_factors(episode: &Episode) -> Option<String> {
    if let Some(language) = &episode.context.language {
        if episode.successful_steps_count() > 5 {
            return Some(format!(
                "Successfully leveraged {}-specific tools and patterns in {} domain",
                language, episode.context.domain
            ));
        }
    }

    if !episode.context.tags.is_empty() {
        return Some(format!(
            "Effectively utilized domain knowledge: {}",
            episode.context.tags.join(", ")
        ));
    }

    None
}

fn analyze_efficiency_achievements(episode: &Episode) -> Option<String> {
    if let Some(duration) = episode.duration() {
        let duration_secs = duration.num_seconds();
        let steps = episode.steps.len();

        if duration_secs < 60 && steps < 15 {
            Some(format!(
                "Highly efficient execution: {} steps in {}s - demonstrates expertise",
                steps, duration_secs
            ))
        } else if steps < 5 && duration_secs < 120 {
            Some(
                "Minimalist approach: achieved goal with minimal steps - shows clear strategy"
                    .to_string(),
            )
        } else {
            None
        }
    } else {
        None
    }
}
