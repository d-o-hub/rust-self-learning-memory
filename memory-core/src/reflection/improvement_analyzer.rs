//! Improvement opportunity analysis for episode reflections

use crate::episode::Episode;
use crate::types::{ExecutionResult, TaskOutcome};

/// Identify areas for improvement
pub(super) fn identify_improvements(episode: &Episode, max_items: usize) -> Vec<String> {
    let mut improvements = Vec::new();

    // Check for failures
    match &episode.outcome {
        Some(TaskOutcome::Failure { reason, .. }) => {
            improvements.push(format!("Task failed: {}", reason));
        }
        Some(TaskOutcome::PartialSuccess { failed, .. }) if !failed.is_empty() => {
            improvements.push(format!("Failed {} subtask(s)", failed.len()));
        }
        _ => {}
    }

    // Check for failed steps
    let failed_steps = episode.failed_steps_count();
    if failed_steps > 0 {
        improvements.push(format!(
            "Reduce failed execution steps (current: {})",
            failed_steps
        ));

        // Identify specific tools that failed
        if let Some(problematic_tool) = identify_problematic_tool(episode) {
            improvements.push(problematic_tool);
        }
    }

    // Check for inefficiency
    if let Some(duration) = episode.duration() {
        let duration_secs = duration.num_seconds();
        if duration_secs > 300 {
            // > 5 minutes
            improvements.push(format!(
                "Optimize execution time (took {} seconds)",
                duration_secs
            ));
        }
    }

    // Check for excessive steps
    let step_count = episode.steps.len();
    if step_count > 50 {
        improvements.push(format!(
            "Reduce number of execution steps (current: {})",
            step_count
        ));
    }

    // Check for repeated errors
    if let Some(repeated_error) = identify_repeated_errors(episode) {
        improvements.push(repeated_error);
    }

    // If no improvements identified but task failed, add generic improvement
    if improvements.is_empty()
        && matches!(
            episode.outcome,
            Some(TaskOutcome::Failure { .. }) | Some(TaskOutcome::PartialSuccess { .. })
        )
    {
        improvements.push("Review and refine approach for better outcomes".to_string());
    }

    // Limit to max items
    improvements.truncate(max_items);
    improvements
}

/// Analyze improvement opportunities with actionable recommendations
pub(super) fn analyze_improvement_opportunities(episode: &Episode) -> Vec<String> {
    let mut opportunities = Vec::new();

    // Analyze step-level bottlenecks
    if let Some(bottleneck) = identify_bottlenecks(episode) {
        opportunities.push(bottleneck);
    }

    // Analyze redundancy and repetition
    if let Some(redundancy) = identify_redundancy(episode) {
        opportunities.push(redundancy);
    }

    // Analyze error patterns with root causes
    if let Some(error_pattern) = analyze_error_root_causes(episode) {
        opportunities.push(error_pattern);
    }

    // Analyze missed optimization opportunities
    if let Some(optimization) = identify_optimization_opportunities(episode) {
        opportunities.push(optimization);
    }

    // Analyze resource utilization
    if let Some(resource) = analyze_resource_utilization(episode) {
        opportunities.push(resource);
    }

    opportunities
}

// Helper methods

fn identify_problematic_tool(episode: &Episode) -> Option<String> {
    let mut tool_failures: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();

    for step in &episode.steps {
        if !step.is_success() {
            *tool_failures.entry(step.tool.as_str()).or_insert(0) += 1;
        }
    }

    tool_failures
        .iter()
        .max_by_key(|(_, &count)| count)
        .filter(|(_, &count)| count >= 2)
        .map(|(tool, count)| format!("Tool '{}' failed {} times - needs attention", tool, count))
}

fn identify_repeated_errors(episode: &Episode) -> Option<String> {
    let mut error_messages: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for step in &episode.steps {
        if let Some(ExecutionResult::Error { message }) = &step.result {
            *error_messages.entry(message.clone()).or_insert(0) += 1;
        }
    }

    error_messages
        .iter()
        .max_by_key(|(_, &count)| count)
        .filter(|(_, &count)| count >= 2)
        .map(|(msg, count)| format!("Repeated error ({} times): {}", count, msg))
}

fn identify_bottlenecks(episode: &Episode) -> Option<String> {
    if episode.steps.is_empty() {
        return None;
    }

    // Find slowest step
    let max_latency = episode.steps.iter().map(|s| s.latency_ms).max()?;
    let avg_latency: u64 =
        episode.steps.iter().map(|s| s.latency_ms).sum::<u64>() / episode.steps.len() as u64;

    if max_latency > avg_latency * 3 && max_latency > 1000 {
        let slow_step = episode.steps.iter().find(|s| s.latency_ms == max_latency)?;
        Some(format!(
            "Performance bottleneck: '{}' took {}ms (3x average) - consider optimization or caching",
            slow_step.tool, max_latency
        ))
    } else {
        None
    }
}

fn identify_redundancy(episode: &Episode) -> Option<String> {
    let mut tool_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for step in &episode.steps {
        *tool_counts.entry(step.tool.as_str()).or_insert(0) += 1;
    }

    // Find tools used many times
    let redundant = tool_counts
        .iter()
        .filter(|(_, &count)| count >= 5)
        .max_by_key(|(_, &count)| count);

    if let Some((tool, count)) = redundant {
        Some(format!(
            "High repetition of '{}' ({} times) - consider batching or alternative approach",
            tool, count
        ))
    } else {
        None
    }
}

fn analyze_error_root_causes(episode: &Episode) -> Option<String> {
    let error_steps: Vec<_> = episode.steps.iter().filter(|s| !s.is_success()).collect();

    if error_steps.len() >= 3 {
        // Check if errors follow a pattern
        let error_tools: Vec<_> = error_steps.iter().map(|s| s.tool.as_str()).collect();
        let unique_error_tools: std::collections::HashSet<_> = error_tools.iter().collect();

        if unique_error_tools.len() == 1 {
            Some(format!(
                "Systematic issue with '{}' - {} consecutive failures suggest incompatibility or misconfiguration",
                error_tools[0], error_steps.len()
            ))
        } else {
            Some(format!(
                "Multiple failure points ({} tools) - review overall approach and prerequisites",
                unique_error_tools.len()
            ))
        }
    } else {
        None
    }
}

fn identify_optimization_opportunities(episode: &Episode) -> Option<String> {
    // Check for sequential operations that could be parallelized
    if episode.steps.len() >= 4 {
        let consecutive_same_type: Vec<_> = episode
            .steps
            .windows(2)
            .filter(|w| w[0].tool == w[1].tool)
            .collect();

        if consecutive_same_type.len() >= 2 {
            Some(
                "Potential for parallelization: consecutive similar operations detected"
                    .to_string(),
            )
        } else {
            None
        }
    } else {
        None
    }
}

fn analyze_resource_utilization(episode: &Episode) -> Option<String> {
    let total_tokens: usize = episode.steps.iter().filter_map(|s| s.tokens_used).sum();

    if total_tokens > 10000 {
        Some(format!(
            "High token usage ({} tokens) - consider more focused prompts or caching",
            total_tokens
        ))
    } else if total_tokens > 0 && total_tokens < 1000 {
        Some(format!(
            "Efficient token usage ({} tokens) - demonstrates focused communication",
            total_tokens
        ))
    } else {
        None
    }
}
