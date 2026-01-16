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
        tools.push(crate::mcp::tools::embeddings::EmbeddingTools::configure_embeddings_tool());
        tools.push(crate::mcp::tools::embeddings::EmbeddingTools::query_semantic_memory_tool());
        tools.push(crate::mcp::tools::embeddings::EmbeddingTools::test_embeddings_tool());

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

        tools
    }
}
