//! Basic tool definitions for the MCP server
//!
//! This module contains the `create_default_tools()` function that defines
//! the core MCP tools for the memory system (querying, patterns, monitoring, embeddings).

use crate::types::Tool;
use serde_json::json;

/// Create the default set of basic tool definitions for the MCP server.
///
/// This function defines core tools including memory queries, pattern analysis,
/// health checks, metrics, embeddings, and pattern search/recommendation.
pub fn create_default_tools() -> Vec<Tool> {
    let mut tools = vec![Tool::new(
        "query_memory".to_string(),
        "Query episodic memory for relevant past experiences and learned patterns".to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query describing the task or context"
                },
                "domain": {
                    "type": "string",
                    "description": "Task domain (e.g., 'web-api', 'data-processing')"
                },
                "task_type": {
                    "type": "string",
                    "enum": [
                        "code_generation",
                        "debugging",
                        "refactoring",
                        "testing",
                        "analysis",
                        "documentation"
                    ],
                    "description": "Type of task being performed"
                },
                "limit": {
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum number of episodes to retrieve"
                },
                "sort": {
                    "type": "string",
                    "enum": ["relevance", "newest", "oldest", "duration", "success"],
                    "default": "relevance",
                    "description": "Sort order for results"
                }
            },
            "required": ["query", "domain"]
        }),
    )];

    // Check if WASM sandbox is available before adding execute_agent_code tool
    if crate::server::sandbox::is_wasm_sandbox_available() {
        tools.push(Tool::new(
            "execute_agent_code".to_string(),
            "Execute TypeScript/JavaScript code in a secure sandbox environment".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "TypeScript/JavaScript code to execute"
                    },
                    "context": {
                        "type": "object",
                        "properties": {
                            "task": {
                                "type": "string",
                                "description": "Task description"
                            },
                            "input": {
                                "type": "object",
                                "description": "Input data as JSON"
                            }
                        },
                        "required": ["task", "input"]
                    }
                },
                "required": ["code", "context"]
            }),
        ));
    } else {
        tracing::warn!("WASM sandbox not available - execute_agent_code tool disabled");
    }

    tools.push(Tool::new(
        "analyze_patterns".to_string(),
        "Analyze patterns from past episodes to identify successful strategies".to_string(),
        json!({
            "type": "object",
            "properties": {
                "task_type": {
                    "type": "string",
                    "description": "Type of task to analyze patterns for"
                },
                "min_success_rate": {
                    "type": "number",
                    "default": 0.7,
                    "description": "Minimum success rate for patterns (0.0-1.0)"
                },
                "limit": {
                    "type": "integer",
                    "default": 20,
                    "description": "Maximum number of patterns to return"
                }
            },
            "required": ["task_type"]
        }),
    ));

    tools.push(Tool::new(
        "health_check".to_string(),
        "Check the health status of the MCP server and its components".to_string(),
        json!({"type": "object", "properties": {}}),
    ));

    tools.push(Tool::new(
        "get_metrics".to_string(),
        "Get comprehensive monitoring metrics and statistics".to_string(),
        json!({
            "type": "object",
            "properties": {
                "metric_type": {
                    "type": "string",
                    "enum": ["all", "performance", "episodes", "system"],
                    "default": "all",
                    "description": "Type of metrics to retrieve"
                }
            }
        }),
    ));

    // Advanced pattern analysis tool
    tools.push(
        crate::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisTool::tool_definition(
        ),
    );

    // Quality metrics tool
    tools.push(crate::mcp::tools::quality_metrics::QualityMetricsTool::tool_definition());

    // Embedding configuration and query tools
    tools.push(crate::mcp::tools::embeddings::configure_embeddings_tool());
    tools.push(crate::mcp::tools::embeddings::query_semantic_memory_tool());
    tools.push(crate::mcp::tools::embeddings::test_embeddings_tool());

    // Pattern search tool
    tools.push(Tool::new(
        "search_patterns".to_string(),
        "Search for patterns semantically similar to a query using multi-signal ranking"
            .to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language query describing what pattern to search for"
                },
                "domain": {
                    "type": "string",
                    "description": "Domain to search in (e.g., 'web-api', 'cli', 'data-processing')"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional tags for filtering",
                    "default": []
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 5)",
                    "default": 5
                },
                "min_relevance": {
                    "type": "number",
                    "description": "Minimum relevance score 0.0-1.0 (default: 0.3)",
                    "default": 0.3
                },
                "filter_by_domain": {
                    "type": "boolean",
                    "description": "Whether to filter by domain (default: false)",
                    "default": false
                }
            },
            "required": ["query", "domain"]
        }),
    ));

    // Pattern recommendation tool
    tools.push(Tool::new(
        "recommend_patterns".to_string(),
        "Get pattern recommendations for a specific task with high-quality filtering".to_string(),
        json!({
            "type": "object",
            "properties": {
                "task_description": {
                    "type": "string",
                    "description": "Description of the task you're working on"
                },
                "domain": {
                    "type": "string",
                    "description": "Domain of the task (e.g., 'web-api', 'cli')"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional context tags",
                    "default": []
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of recommendations (default: 3)",
                    "default": 3
                }
            },
            "required": ["task_description", "domain"]
        }),
    ));

    tools
}
