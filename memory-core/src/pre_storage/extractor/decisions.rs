//! Critical decision extraction logic.

use crate::episode::Episode;

/// Extract parameter decisions from a JSON object.
///
/// Helper function to extract strategic parameter choices.
pub(super) fn extract_parameter_decisions(
    obj: &serde_json::Map<String, serde_json::Value>,
    step_number: usize,
    decisions: &mut Vec<String>,
) {
    if obj.is_empty() {
        return;
    }

    // Look for strategy, approach, method parameters
    for key in obj.keys() {
        let key_lower = key.to_lowercase();
        if key_lower.contains("strategy")
            || key_lower.contains("approach")
            || key_lower.contains("method")
            || key_lower.contains("algorithm")
        {
            if let Some(value) = obj.get(key) {
                decisions.push(format!(
                    "Chose {key} = {} (step {step_number})",
                    value.to_string().trim_matches('"'),
                ));
            }
        }
    }
}

/// Extract critical decisions from episode steps and outcome.
///
/// Identifies important decision points such as:
/// - Choice of tools or approaches
/// - Branching logic (if different paths were taken)
/// - Significant parameter choices
/// - Final outcome decisions
pub(super) fn extract_critical_decisions(episode: &Episode) -> Vec<String> {
    let mut decisions = Vec::new();

    // Look for steps with significant parameter choices
    for step in &episode.steps {
        // Check for decision-indicating keywords in actions
        let action_lower = step.action.to_lowercase();
        if action_lower.contains("choose")
            || action_lower.contains("decide")
            || action_lower.contains("select")
            || action_lower.contains("opt for")
        {
            decisions.push(format!(
                "Step {}: {} using {}",
                step.step_number, step.action, step.tool
            ));
        }

        // Check for complex parameters that indicate choices
        if let Some(obj) = step.parameters.as_object() {
            extract_parameter_decisions(obj, step.step_number, &mut decisions);
        }
    }

    // Extract decision from outcome
    if let Some(ref outcome) = episode.outcome {
        match outcome {
            crate::types::TaskOutcome::Success { verdict, .. } => {
                if verdict.len() > 10 {
                    // Meaningful verdict
                    decisions.push(format!("Outcome: {verdict}"));
                }
            }
            crate::types::TaskOutcome::PartialSuccess { verdict, .. } => {
                decisions.push(format!("Partial success: {verdict}"));
            }
            crate::types::TaskOutcome::Failure { reason, .. } => {
                decisions.push(format!("Failure reason: {reason}"));
            }
        }
    }

    // Limit to most relevant decisions
    decisions.truncate(10);
    decisions
}
