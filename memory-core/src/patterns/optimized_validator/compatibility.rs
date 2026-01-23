//! Tool compatibility assessment module

use super::tool::Tool;
use crate::types::TaskContext;

/// Compatibility assessment result
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub overall_score: f32,
    pub historical_success_rate: f32,
    pub context_compatibility: f32,
    pub capability_match: f32,
}

/// Analyze capability match between tool and task requirements
pub fn analyze_capability_match(tool: &Tool, context: &TaskContext) -> f32 {
    if tool.capabilities.is_empty() {
        return 0.5; // Neutral score for tools with no capability restrictions
    }

    // Get expected capabilities for the context
    let expected_capabilities = get_expected_capabilities_for_context(context);

    if expected_capabilities.is_empty() {
        return 1.0; // No specific capabilities required
    }

    let mut matched_capabilities = 0;

    // Check for exact matches
    for capability in &tool.capabilities {
        if expected_capabilities.contains(capability) {
            matched_capabilities += 1;
        }
    }

    // Check for partial matches (e.g., "rust_compiler" matches "compiler" requirement)
    for tool_cap in &tool.capabilities {
        for expected_cap in &expected_capabilities {
            if tool_cap.contains(expected_cap) || expected_cap.contains(tool_cap) {
                matched_capabilities += 1;
                break; // Don't double count
            }
        }
    }

    // Calculate match ratio
    let match_ratio = matched_capabilities as f32 / expected_capabilities.len() as f32;

    // Bonus for exact domain-specific tools
    let domain_bonus = match context.domain.as_str() {
        "api_development" => {
            if tool
                .capabilities
                .iter()
                .any(|c| c.contains("rust") || c.contains("compiler"))
            {
                0.2
            } else {
                0.0
            }
        }
        _ => 0.0,
    };

    (match_ratio + domain_bonus).min(1.0)
}

/// Get expected capabilities for a given context
pub fn get_expected_capabilities_for_context(context: &TaskContext) -> Vec<String> {
    let mut capabilities = Vec::new();

    // Domain-based capabilities
    match context.domain.as_str() {
        "api_development" => {
            capabilities.extend(vec!["http_client".to_string(), "json_parser".to_string()]);
        }
        "data_processing" => {
            capabilities.extend(vec!["file_reader".to_string(), "data_analyzer".to_string()]);
        }
        "debugging" => {
            capabilities.extend(vec!["error_analyzer".to_string(), "log_viewer".to_string()]);
        }
        "testing" => {
            capabilities.extend(vec![
                "test_runner".to_string(),
                "assertion_checker".to_string(),
            ]);
        }
        _ => {
            capabilities.push("general_processor".to_string());
        }
    }

    // Language-based capabilities
    if let Some(lang) = &context.language {
        capabilities.push(format!("{lang}_compiler"));
        capabilities.push(format!("{lang}_linter"));
    }

    // Framework-based capabilities
    if let Some(framework) = &context.framework {
        capabilities.push(format!("{framework}_integration"));
    }

    capabilities
}
