//! # Completion Handlers (MCP 2025-11-25)
//!
//! Completion handler for the completion/complete request type.

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use tracing::{error, info};

use super::super::types::{CompletionParams, CompletionResult, CompletionValues};

/// Handle completion/complete request (MCP 2025-11-25)
pub async fn handle_completion_complete(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling completion/complete request");

    // Parse completion params
    let params: CompletionParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                })
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            })
        }
    };

    // Generate completions based on reference type and argument
    let completions = generate_completions(&params);

    let result = CompletionResult {
        completion: completions,
    };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize completion response: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                }),
            })
        }
    }
}

/// Generate completions based on reference type and argument value
fn generate_completions(params: &CompletionParams) -> CompletionValues {
    let argument_value = params.argument.value.clone();

    // Get context arguments if available
    let _context_args = params.context.as_ref().map(|c| &c.arguments);

    // Parse the reference Value to extract prompt name or resource URI
    // External tagging format: {"ref/prompt": {"name": "..."}} or {"ref/resource": {"uri": "..."}}
    let prompt_name = if let Some(prompt_ref) = params
        .reference
        .as_object()
        .and_then(|o| o.get("ref/prompt"))
    {
        prompt_ref
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Generate domain completions for query_memory tool
    if let Some(ref name) = prompt_name {
        // Handle prompt argument completions
        match name.as_str() {
            "query_memory" => {
                // Common domains for query_memory
                let domains = [
                    "web-api",
                    "data-processing",
                    "code-generation",
                    "debugging",
                    "refactoring",
                    "testing",
                    "analysis",
                    "documentation",
                    "infrastructure",
                    "security",
                ];
                let filtered: Vec<String> = domains
                    .iter()
                    .filter(|d| d.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            "analyze_patterns" => {
                // Task types for analyze_patterns
                let task_types = [
                    "code_generation",
                    "debugging",
                    "refactoring",
                    "testing",
                    "analysis",
                    "documentation",
                ];
                let filtered: Vec<String> = task_types
                    .iter()
                    .filter(|t| t.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            "advanced_pattern_analysis" => {
                // Analysis types
                let analysis_types = ["statistical", "predictive", "comprehensive"];
                let filtered: Vec<String> = analysis_types
                    .iter()
                    .filter(|a| a.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            _ => {
                // Generic completions based on argument name
                let arg_name = params.argument.name.as_str();
                let completions: Vec<&str> = match arg_name {
                    "domain" => vec!["web-api", "data-processing", "testing"],
                    "task_type" => vec!["code_generation", "debugging", "refactoring"],
                    "metric_type" => vec!["all", "performance", "episodes", "system"],
                    "analysis_type" => vec!["statistical", "predictive", "comprehensive"],
                    "time_range" => vec!["24h", "7d", "30d", "90d", "all"],
                    "provider" => vec!["openai", "local", "mistral", "azure", "cohere"],
                    _ => vec![],
                };
                let filtered: Vec<String> = completions
                    .iter()
                    .filter(|s| s.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
        }
    }

    // Handle resource completions
    let _is_resource_ref = params
        .reference
        .as_object()
        .and_then(|o| o.get("ref/resource"))
        .is_some();

    // For resource URI completions, return empty for now
    CompletionValues {
        values: vec![],
        total: Some(0),
        has_more: Some(false),
    }
}
