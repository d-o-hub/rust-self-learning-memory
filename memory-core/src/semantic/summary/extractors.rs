//! Key concept and step extraction for semantic summarization.

#![allow(clippy::must_use_candidate)]

use crate::episode::Episode;
use crate::semantic::summary::helpers::{extract_step_number, is_stopword};
use std::collections::HashSet;

/// Extract key concepts from an episode.
///
/// Identifies important concepts from:
/// - Task description and context
/// - Tools used in execution
/// - Salient features (if available)
///
/// Returns 10-20 normalized, deduplicated concepts.
///
/// # Arguments
///
/// * `episode` - The episode to extract concepts from
///
/// # Returns
///
/// Vector of key concepts (normalized, deduplicated)
pub fn extract_key_concepts(episode: &Episode) -> Vec<String> {
    let mut concepts = HashSet::new();

    // Extract from task description
    for word in episode.task_description.split_whitespace() {
        let normalized = word
            .to_lowercase()
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if normalized.len() > 3 && !is_stopword(&normalized) {
            concepts.insert(normalized);
        }
    }

    // Extract from context
    if let Some(ref lang) = episode.context.language {
        concepts.insert(lang.to_lowercase());
    }
    if let Some(ref framework) = episode.context.framework {
        concepts.insert(framework.to_lowercase());
    }
    concepts.insert(episode.context.domain.to_lowercase());
    for tag in &episode.context.tags {
        concepts.insert(tag.to_lowercase());
    }

    // Extract from task type
    concepts.insert(format!("{}", episode.task_type).to_lowercase());

    // Extract unique tools used
    for step in &episode.steps {
        concepts.insert(step.tool.to_lowercase());
    }

    // Extract from salient features if available
    if let Some(ref features) = episode.salient_features {
        for decision in &features.critical_decisions {
            for word in decision.split_whitespace() {
                let normalized = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();
                if normalized.len() > 3 && !is_stopword(&normalized) {
                    concepts.insert(normalized);
                }
            }
        }

        for insight in &features.key_insights {
            for word in insight.split_whitespace() {
                let normalized = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();
                if normalized.len() > 3 && !is_stopword(&normalized) {
                    concepts.insert(normalized);
                }
            }
        }
    }

    // Convert to sorted vector and limit
    let mut concept_vec: Vec<String> = concepts.into_iter().collect();
    concept_vec.sort();
    concept_vec.truncate(20);
    concept_vec
}

/// Extract critical steps from an episode.
///
/// Selects 3-5 most important steps based on:
/// - Steps with errors (error recovery learning)
/// - Steps using unique/critical tools
/// - Steps mentioned in salient features
/// - First and last steps (context)
///
/// # Arguments
///
/// * `episode` - The episode to extract steps from
/// * `max_key_steps` - Maximum number of key steps to extract
///
/// # Returns
///
/// Vector of formatted key steps (max 5)
pub fn extract_key_steps(episode: &Episode, max_key_steps: usize) -> Vec<String> {
    if episode.steps.is_empty() {
        return Vec::new();
    }

    let mut key_steps = Vec::new();
    let mut step_indices = Vec::new();

    // Always include first step (context)
    if !episode.steps.is_empty() {
        step_indices.push(0);
    }

    // Include steps with errors (error recovery)
    for (idx, step) in episode.steps.iter().enumerate() {
        if !step.is_success() {
            step_indices.push(idx);
        }
    }

    // Include steps mentioned in salient features
    if let Some(ref features) = episode.salient_features {
        for decision in &features.critical_decisions {
            if let Some(step_num) = extract_step_number(decision) {
                if step_num > 0 && step_num <= episode.steps.len() {
                    step_indices.push(step_num - 1);
                }
            }
        }
    }

    // Always include last step (outcome)
    if episode.steps.len() > 1 {
        step_indices.push(episode.steps.len() - 1);
    }

    // Deduplicate and sort
    step_indices.sort_unstable();
    step_indices.dedup();

    // If we have too many, prioritize errors and first/last
    if step_indices.len() > max_key_steps {
        let mut prioritized = Vec::new();

        // Keep first
        if !step_indices.is_empty() {
            prioritized.push(step_indices[0]);
        }

        // Keep errors
        for &idx in &step_indices {
            if !episode.steps[idx].is_success() && prioritized.len() < max_key_steps - 1 {
                prioritized.push(idx);
            }
        }

        // Keep last
        if let Some(&last) = step_indices.last() {
            if !prioritized.contains(&last) && prioritized.len() < max_key_steps {
                prioritized.push(last);
            }
        }

        // Fill remaining with middle steps
        for &idx in &step_indices {
            if !prioritized.contains(&idx) && prioritized.len() < max_key_steps {
                prioritized.push(idx);
            }
        }

        prioritized.sort_unstable();
        step_indices = prioritized;
    }

    // Format selected steps
    for idx in step_indices {
        let step = &episode.steps[idx];
        let status = if step.is_success() { "" } else { " [ERROR]" };
        key_steps.push(format!(
            "Step {}: {} - {}{}",
            step.step_number, step.tool, step.action, status
        ));
    }

    key_steps
}
