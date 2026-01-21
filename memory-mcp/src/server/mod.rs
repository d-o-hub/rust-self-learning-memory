//! MCP server for memory integration
//!
//! This module provides the MCP (Model Context Protocol) server that integrates
//! the self-learning memory system with code execution capabilities.
//!
//! ## Features
//!
//! - Tool definitions for memory queries and code execution
//! - Progressive tool disclosure based on usage patterns
//! - Integration with SelfLearningMemory system
//! - Secure code execution via sandbox
//! - Execution statistics and monitoring
//!
//! ## Example
//!
//! ```no_run
//! use memory_mcp::server::MemoryMCPServer;
//! use memory_mcp::types::SandboxConfig;
//! use memory_core::SelfLearningMemory;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let memory = Arc::new(SelfLearningMemory::new());
//!     let server = MemoryMCPServer::new(SandboxConfig::default(), memory).await?;
//!
//!     // List available tools
//!     let tools = server.list_tools().await;
//!     println!("Available tools: {}", tools.len());
//!
//!     Ok(())
//! }
//! ```

// Submodules
pub mod cache_warming;
#[cfg(test)]
mod tests;
pub mod tools;

use crate::cache::QueryCache;
use crate::monitoring::{MonitoringConfig, MonitoringEndpoints, MonitoringSystem};
use crate::types::{ExecutionStats, SandboxConfig, Tool};
use crate::unified_sandbox::{SandboxBackend, UnifiedSandbox};
use anyhow::Result;
use memory_core::SelfLearningMemory;
use parking_lot::RwLock;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "javy-backend")]
use tracing::debug;
use tracing::{info, warn};

/// MCP server for memory integration
pub struct MemoryMCPServer {
    /// Code execution sandbox
    sandbox: Arc<UnifiedSandbox>,
    /// Available tools
    tools: Arc<RwLock<Vec<Tool>>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
    /// Tool usage tracking for progressive disclosure
    tool_usage: Arc<RwLock<HashMap<String, usize>>>,
    /// Self-learning memory system
    memory: Arc<SelfLearningMemory>,
    /// Monitoring system
    monitoring: Arc<MonitoringSystem>,
    /// Monitoring endpoints
    monitoring_endpoints: Arc<MonitoringEndpoints>,
    /// Query result cache
    #[allow(dead_code)]
    cache: Arc<QueryCache>,
}

impl MemoryMCPServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `config` - Sandbox configuration for code execution
    /// * `memory` - Self-learning memory system
    ///
    /// # Returns
    ///
    /// Returns a new `MemoryMCPServer` instance
    pub async fn new(config: SandboxConfig, memory: Arc<SelfLearningMemory>) -> Result<Self> {
        let sandbox_backend = Self::determine_sandbox_backend();
        let sandbox = Arc::new(UnifiedSandbox::new(config.clone(), sandbox_backend).await?);
        let tools = Arc::new(RwLock::new(Self::create_default_tools()));

        let monitoring = Self::initialize_monitoring();
        let monitoring_endpoints = Arc::new(MonitoringEndpoints::new(Arc::clone(&monitoring)));

        let server = Self {
            sandbox,
            tools,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            tool_usage: Arc::new(RwLock::new(HashMap::new())),
            memory,
            monitoring,
            monitoring_endpoints,
            cache: Arc::new(QueryCache::new()),
        };

        info!(
            "MCP server initialized with {} tools",
            server.tools.read().len()
        );
        info!(
            "Monitoring system initialized (enabled: {})",
            server.monitoring.config().enabled
        );

        // Perform cache warming if enabled
        if cache_warming::is_cache_warming_enabled() {
            info!("Starting cache warming process...");
            if let Err(e) = cache_warming::warm_cache(
                &server.memory,
                &cache_warming::CacheWarmingConfig::from_env(),
            )
            .await
            {
                warn!(
                    "Cache warming failed, but continuing with server startup: {}",
                    e
                );
            } else {
                info!("Cache warming completed successfully");
            }
        } else {
            info!("Cache warming disabled, skipping");
        }

        Ok(server)
    }

    fn determine_sandbox_backend() -> SandboxBackend {
        match std::env::var("MCP_USE_WASM")
            .unwrap_or_else(|_| "auto".to_string())
            .as_str()
        {
            "true" | "wasm" => SandboxBackend::Wasm,
            "false" | "node" => SandboxBackend::NodeJs,
            _ => {
                let ratio = std::env::var("MCP_WASM_RATIO")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.25);
                let intelligent = std::env::var("MCP_INTELLIGENT_ROUTING")
                    .map(|v| v.to_lowercase())
                    .ok()
                    .map(|v| v == "true" || v == "1" || v == "yes")
                    .unwrap_or(true);
                SandboxBackend::Hybrid {
                    wasm_ratio: ratio.clamp(0.0, 1.0),
                    intelligent_routing: intelligent,
                }
            }
        }
    }

    fn initialize_monitoring() -> Arc<MonitoringSystem> {
        let monitoring_config = MonitoringConfig::default();
        Arc::new(MonitoringSystem::new(monitoring_config))
    }

    /// Check if Javy plugin is valid (only when javy-backend feature is enabled)
    #[cfg(feature = "javy-backend")]
    fn is_javy_plugin_valid() -> bool {
        use std::path::Path;

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let plugin_path = Path::new(manifest_dir).join("javy-plugin.wasm");

        if let Ok(metadata) = std::fs::metadata(&plugin_path) {
            if metadata.len() > 100 {
                if let Ok(mut file) = std::fs::File::open(&plugin_path) {
                    let mut magic = [0u8; 4];
                    if std::io::Read::read_exact(&mut file, &mut magic).is_ok() {
                        if &magic == b"\0asm" {
                            debug!("Valid Javy plugin found ({} bytes)", metadata.len());
                            return true;
                        }
                    }
                }
            }
        }

        const EMBEDDED_PLUGIN: &[u8] = include_bytes!("../../javy-plugin.wasm");
        if EMBEDDED_PLUGIN.len() > 100 && EMBEDDED_PLUGIN.starts_with(b"\0asm") {
            debug!(
                "Valid embedded Javy plugin ({} bytes)",
                EMBEDDED_PLUGIN.len()
            );
            return true;
        }

        // Invalid plugin is expected when javy-backend feature is used without proper setup
        // Javy compiler will handle graceful degradation
        debug!(
            "Javy plugin not valid ({} bytes, expected >100 bytes with WASM magic bytes)",
            EMBEDDED_PLUGIN.len()
        );
        false
    }

    #[cfg(not(feature = "javy-backend"))]
    #[allow(dead_code)]
    fn is_javy_plugin_valid() -> bool {
        false
    }

    /// Check if WASM sandbox is available for code execution
    ///
    /// This function performs a lightweight check to avoid expensive initialization
    /// during server startup. The actual sandbox creation is deferred to first use.
    fn is_wasm_sandbox_available() -> bool {
        // Priority 1: Check environment variable for explicit override
        if let Ok(val) = std::env::var("MCP_USE_WASM") {
            match val.to_lowercase().as_str() {
                "true" | "wasm" => return true,
                "false" | "node" => return false,
                _ => {}
            }
        }

        // Priority 2: Lightweight check - only verify Javy plugin exists if needed
        // The actual sandbox initialization is deferred to first tool invocation
        #[cfg(feature = "javy-backend")]
        {
            if !Self::is_javy_plugin_valid() {
                debug!("WASM sandbox may be limited due to invalid Javy plugin");
                // Continue - sandbox might still work with pre-compiled WASM
            }
        }

        // Default to available - actual initialization happens lazily
        // This avoids spawning threads with nested async runtimes during startup
        true
    }

    /// Create default tool definitions
    fn create_default_tools() -> Vec<Tool> {
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

        if Self::is_wasm_sandbox_available() {
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
            warn!("WASM sandbox not available - execute_agent_code tool disabled");
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

        tools.push(crate::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisTool::tool_definition());
        tools.push(crate::mcp::tools::quality_metrics::QualityMetricsTool::tool_definition());
        tools.push(crate::mcp::tools::embeddings::configure_embeddings_tool());
        tools.push(crate::mcp::tools::embeddings::query_semantic_memory_tool());
        tools.push(crate::mcp::tools::embeddings::test_embeddings_tool());

        // Pattern search and recommendation tools
        tools.push(Tool::new(
            "search_patterns".to_string(),
            "Search for patterns semantically similar to a query using multi-signal ranking".to_string(),
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

        tools.push(Tool::new(
            "recommend_patterns".to_string(),
            "Get pattern recommendations for a specific task with high-quality filtering"
                .to_string(),
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

        // Bulk episode retrieval tool
        tools.push(Tool::new(
            "bulk_episodes".to_string(),
            "Retrieve multiple episodes by their IDs in a single efficient operation".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_ids": {
                        "type": "array",
                        "items": {"type": "string", "description": "Episode UUIDs"},
                        "description": "Array of episode IDs to retrieve"
                    }
                },
                "required": ["episode_ids"]
            }),
        ));

        // Episode lifecycle management tools
        tools.push(Tool::new(
            "create_episode".to_string(),
            "Create a new episode to track task execution programmatically".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "task_description": {
                        "type": "string",
                        "description": "Clear description of the task to be performed"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Task domain (e.g., 'web-api', 'cli', 'data-processing')"
                    },
                    "task_type": {
                        "type": "string",
                        "enum": ["code_generation", "debugging", "refactoring", "testing", "analysis", "documentation"],
                        "description": "Type of task being performed"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (optional)"
                    },
                    "framework": {
                        "type": "string",
                        "description": "Framework name (optional)"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional context tags"
                    },
                    "complexity": {
                        "type": "string",
                        "enum": ["simple", "moderate", "complex"],
                        "description": "Task complexity level (optional, default: moderate)"
                    }
                },
                "required": ["task_description", "domain", "task_type"]
            }),
        ));

        tools.push(Tool::new(
            "add_episode_step".to_string(),
            "Add an execution step to an ongoing episode to track progress".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode"
                    },
                    "step_number": {
                        "type": "integer",
                        "description": "Sequential step number"
                    },
                    "tool": {
                        "type": "string",
                        "description": "Name of the tool/component performing the action"
                    },
                    "action": {
                        "type": "string",
                        "description": "Description of the action taken"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Optional parameters used in this step"
                    },
                    "result": {
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["success", "error", "timeout"],
                                "description": "Result type"
                            },
                            "output": {
                                "type": "string",
                                "description": "Output message (for success)"
                            },
                            "message": {
                                "type": "string",
                                "description": "Error message (for error)"
                            }
                        },
                        "description": "Optional result of the step"
                    },
                    "latency_ms": {
                        "type": "integer",
                        "description": "Optional execution time in milliseconds"
                    }
                },
                "required": ["episode_id", "step_number", "tool", "action"]
            }),
        ));

        tools.push(Tool::new(
            "complete_episode".to_string(),
            "Complete an episode with an outcome and trigger the learning cycle (reward, reflection, patterns)".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode to complete"
                    },
                    "outcome_type": {
                        "type": "string",
                        "enum": ["success", "partial_success", "failure"],
                        "description": "Type of outcome"
                    },
                    "verdict": {
                        "type": "string",
                        "description": "Description of the outcome (required for success/partial_success)"
                    },
                    "artifacts": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of artifact names (optional, for success)"
                    },
                    "completed": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of completed items (required for partial_success)"
                    },
                    "failed": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of failed items (required for partial_success)"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Failure reason (required for failure)"
                    },
                    "error_details": {
                        "type": "string",
                        "description": "Detailed error information (optional, for failure)"
                    }
                },
                "required": ["episode_id", "outcome_type"]
            }),
        ));

        tools.push(Tool::new(
            "get_episode".to_string(),
            "Get complete details of an episode including steps, outcome, reflection, and patterns"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode to retrieve"
                    }
                },
                "required": ["episode_id"]
            }),
        ));

        tools.push(Tool::new(
            "delete_episode".to_string(),
            "Delete an episode permanently from all storage backends (requires confirmation)"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode to delete"
                    },
                    "confirm": {
                        "type": "boolean",
                        "description": "Must be set to true to confirm deletion"
                    }
                },
                "required": ["episode_id", "confirm"]
            }),
        ));

        tools.push(Tool::new(
            "get_episode_timeline".to_string(),
            "Get a chronological timeline view of all steps in an episode".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_id": {
                        "type": "string",
                        "description": "UUID of the episode"
                    }
                },
                "required": ["episode_id"]
            }),
        ));

        // Advanced batch operation tools
        tools.push(Tool::new(
            "batch_query_episodes".to_string(),
            "Efficiently query multiple episodes with filtering, aggregation, and optional inclusion of steps/patterns/reflections".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional specific episode UUIDs to retrieve"
                    },
                    "filter": {
                        "type": "object",
                        "properties": {
                            "domain": {"type": "string"},
                            "task_type": {"type": "string"},
                            "tags": {"type": "array", "items": {"type": "string"}},
                            "success_only": {"type": "boolean"}
                        },
                        "description": "Optional filter criteria"
                    },
                    "include_steps": {
                        "type": "boolean",
                        "description": "Include execution steps (default: true)"
                    },
                    "include_patterns": {
                        "type": "boolean",
                        "description": "Include extracted patterns (default: false)"
                    },
                    "include_reflections": {
                        "type": "boolean",
                        "description": "Include reflections (default: false)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum episodes to return (default: 100, max: 1000)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Pagination offset (default: 0)"
                    },
                    "aggregate_stats": {
                        "type": "boolean",
                        "description": "Compute aggregate statistics (default: true)"
                    }
                }
            }),
        ));

        tools.push(Tool::new(
            "batch_pattern_analysis".to_string(),
            "Analyze patterns across multiple episodes to identify successful approaches and anti-patterns".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Task domain to analyze (required)"
                    },
                    "task_type": {
                        "type": "string",
                        "description": "Optional specific task type"
                    },
                    "time_range": {
                        "type": "object",
                        "properties": {
                            "start": {"type": "string", "description": "ISO8601 start date"},
                            "end": {"type": "string", "description": "ISO8601 end date"}
                        },
                        "description": "Optional time range filter"
                    },
                    "min_episodes": {
                        "type": "integer",
                        "description": "Minimum episodes a pattern must appear in (default: 3)"
                    },
                    "min_success_rate": {
                        "type": "number",
                        "description": "Minimum success rate 0.0-1.0 (default: 0.6)"
                    },
                    "include_anti_patterns": {
                        "type": "boolean",
                        "description": "Include patterns with low success (default: true)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum patterns to return (default: 50)"
                    }
                },
                "required": ["domain"]
            }),
        ));

        tools.push(Tool::new(
            "batch_compare_episodes".to_string(),
            "Compare multiple episodes to identify differences in approach, performance, and outcomes".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "episode_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of episode UUIDs to compare (2-10 episodes)"
                    },
                    "compare_metrics": {
                        "type": "array",
                        "items": {"type": "string", "enum": ["duration", "reward_score", "step_count", "efficiency"]},
                        "description": "Metrics to compare (default: duration, reward_score, step_count)"
                    },
                    "compare_approaches": {
                        "type": "boolean",
                        "description": "Compare execution approaches (default: true)"
                    },
                    "generate_insights": {
                        "type": "boolean",
                        "description": "Generate AI insights (default: true)"
                    }
                },
                "required": ["episode_ids"]
            }),
        ));

        tools
    }
}
