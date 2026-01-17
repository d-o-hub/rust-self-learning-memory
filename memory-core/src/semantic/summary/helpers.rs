//! Helper functions for semantic summarization.

#![allow(clippy::must_use_candidate)]

use crate::pre_storage::SalientFeatures;

/// Add salient features to summary parts.
pub fn add_salient_features_summary(features: &SalientFeatures, parts: &mut Vec<String>) {
    if !features.critical_decisions.is_empty() {
        let decision = &features.critical_decisions[0];
        parts.push(format!("Key decision: {decision}."));
    }

    if !features.error_recovery_patterns.is_empty() {
        let recovery = &features.error_recovery_patterns[0];
        parts.push(format!("Recovery pattern: {recovery}."));
    }

    if !features.key_insights.is_empty() {
        let insight = &features.key_insights[0];
        parts.push(format!("Insight: {insight}."));
    }
}

/// Extract step number from text like "Step 5: ..." or "step 3".
pub fn extract_step_number(text: &str) -> Option<usize> {
    let text_lower = text.to_lowercase();
    if let Some(pos) = text_lower.find("step") {
        let after_step = &text_lower[pos + 4..];
        for word in after_step.split_whitespace() {
            let num_str: String = word.chars().filter(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<usize>() {
                return Some(num);
            }
        }
    }
    None
}

/// Check if a word is a common stopword.
pub fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "for"
            | "that"
            | "this"
            | "with"
            | "from"
            | "have"
            | "has"
            | "had"
            | "was"
            | "were"
            | "been"
            | "will"
            | "are"
            | "not"
            | "but"
            | "can"
            | "all"
            | "would"
            | "there"
            | "their"
    )
}
