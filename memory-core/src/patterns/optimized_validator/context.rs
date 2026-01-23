//! Context analysis module

use super::tool::Tool;
use crate::types::TaskContext;

/// Check if two domains are related
pub fn is_related_domain(domain1: &str, domain2: &str) -> bool {
    let related_pairs = [
        ("api_development", "web_development"),
        ("data_processing", "data_science"),
        ("testing", "debugging"),
        ("refactoring", "code_generation"),
    ];

    related_pairs
        .iter()
        .any(|(a, b)| (domain1 == *a && domain2 == *b) || (domain1 == *b && domain2 == *a))
}

/// Calculate historical success rate for tool in similar contexts
pub fn calculate_historical_success_rate(tool: &Tool, context: &TaskContext) -> f32 {
    if tool.success_history.is_empty() {
        return 0.5; // Neutral score for tools with no history
    }

    let mut total_rate = 0.0;
    let mut weight_sum = 0.0;

    // Weight by domain similarity
    for (domain, rate) in &tool.success_history {
        let weight = if domain == &context.domain {
            1.0 // Exact domain match
        } else if is_related_domain(domain, &context.domain) {
            0.7 // Related domain
        } else {
            0.3 // Different domain
        };

        total_rate += rate * weight;
        weight_sum += weight;
    }

    if weight_sum > 0.0 {
        total_rate / weight_sum
    } else {
        0.5
    }
}

/// Calculate success rate for specific context
pub fn calculate_context_success_rate(
    tool: &Tool,
    context: &TaskContext,
    context_similarity_fn: impl Fn(&TaskContext, &TaskContext) -> f32,
) -> f32 {
    // Find similar contexts in tool's typical contexts
    let mut similar_contexts = 0;
    let mut successful_contexts = 0;
    let mut total_similarity = 0.0;

    for typical_context in &tool.typical_contexts {
        let similarity = context_similarity_fn(typical_context, context);

        if similarity > 0.5 {
            // Lower threshold for considering contexts similar
            similar_contexts += 1;
            total_similarity += similarity;

            // Check if this context was successful
            let domain_key = &typical_context.domain;
            if let Some(&success_rate) = tool.success_history.get(domain_key) {
                #[allow(clippy::excessive_nesting)]
                if success_rate > 0.6 {
                    // Lowered threshold to 60%
                    successful_contexts += 1;
                }
            }
        }
    }

    if similar_contexts > 0 {
        // Weight by similarity scores
        let avg_similarity = total_similarity / similar_contexts as f32;
        let base_success_rate = successful_contexts as f32 / similar_contexts as f32;

        // Combine success rate with similarity weighting
        base_success_rate * avg_similarity + (1.0 - avg_similarity) * 0.7
    } else {
        // If no similar contexts, check domain success history directly
        if let Some(&domain_rate) = tool.success_history.get(&context.domain) {
            domain_rate * 0.8 // Slight penalty for no direct context match
        } else {
            0.5 // Neutral score when no context information available
        }
    }
}

/// Analyze context compatibility between tool and task
pub fn analyze_context_compatibility(tool: &Tool, context: &TaskContext) -> f32 {
    let mut compatibility: f32 = 0.0;
    let mut factors_considered = 0;

    // Domain compatibility (40% weight) - Check exact matches first
    for typical_context in &tool.typical_contexts {
        if typical_context.domain == context.domain {
            compatibility += 0.4;
            factors_considered += 1;
            break;
        }
    }

    // If no exact domain match, check for related domains
    if factors_considered == 0 {
        for typical_context in &tool.typical_contexts {
            if is_related_domain(&typical_context.domain, &context.domain) {
                compatibility += 0.25; // Partial credit for related domains
                factors_considered += 1;
                break;
            }
        }
    }

    // Language compatibility (25% weight)
    if let (Some(context_lang), Some(tool_lang_context)) = (
        &context.language,
        tool.typical_contexts
            .iter()
            .find_map(|tc| tc.language.as_ref()),
    ) {
        if context_lang == tool_lang_context {
            compatibility += 0.25;
            factors_considered += 1;
        }
    }

    // Framework compatibility (20% weight)
    if let (Some(context_framework), Some(tool_framework_context)) = (
        &context.framework,
        tool.typical_contexts
            .iter()
            .find_map(|tc| tc.framework.as_ref()),
    ) {
        if context_framework == tool_framework_context {
            compatibility += 0.20;
            factors_considered += 1;
        }
    }

    // Complexity compatibility (15% weight)
    let has_matching_complexity = tool
        .typical_contexts
        .iter()
        .any(|tc| tc.complexity == context.complexity);

    if has_matching_complexity {
        compatibility += 0.15;
        factors_considered += 1;
    }

    // Bonus for tools that have been used in exactly this domain before
    if tool.success_history.contains_key(&context.domain) {
        compatibility += 0.1; // Small bonus for known domain usage
        factors_considered += 1;
    }

    // If no factors were considered, return a score based on domain popularity
    if factors_considered == 0 {
        match context.domain.as_str() {
            "api_development" | "web_development" => 0.6, // Common domains get base score
            _ => 0.4,
        }
    } else {
        compatibility.min(1.0)
    }
}
