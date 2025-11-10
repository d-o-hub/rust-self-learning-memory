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

use crate::sandbox::CodeSandbox;
use crate::types::{ExecutionContext, ExecutionResult, ExecutionStats, SandboxConfig, Tool};
use anyhow::{Context as AnyhowContext, Result};
use memory_core::{Pattern, SelfLearningMemory, TaskContext};
use parking_lot::RwLock;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// MCP server for memory integration
pub struct MemoryMCPServer {
    /// Code execution sandbox
    sandbox: Arc<CodeSandbox>,
    /// Available tools
    tools: Arc<RwLock<Vec<Tool>>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
    /// Tool usage tracking for progressive disclosure
    tool_usage: Arc<RwLock<HashMap<String, usize>>>,
    /// Self-learning memory system
    memory: Arc<SelfLearningMemory>,
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
        let sandbox = Arc::new(CodeSandbox::new(config)?);
        let tools = Arc::new(RwLock::new(Self::create_default_tools()));

        info!("MCP server initialized with {} tools", tools.read().len());

        Ok(Self {
            sandbox,
            tools,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            tool_usage: Arc::new(RwLock::new(HashMap::new())),
            memory,
        })
    }

    /// Create default tool definitions
    fn create_default_tools() -> Vec<Tool> {
        vec![
            Tool::new(
                "query_memory".to_string(),
                "Query episodic memory for relevant past experiences and learned patterns"
                    .to_string(),
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
            ),
            Tool::new(
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
                            "description": "Execution context with task and input data",
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
            ),
            Tool::new(
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
            ),
        ]
    }

    /// List all available tools
    ///
    /// Returns tools based on progressive disclosure - commonly used tools
    /// are returned first, advanced tools are shown after usage patterns indicate need.
    pub async fn list_tools(&self) -> Vec<Tool> {
        let tools = self.tools.read();
        let usage = self.tool_usage.read();

        // Sort tools by usage frequency
        let mut sorted_tools: Vec<_> = tools.iter().cloned().collect();
        sorted_tools.sort_by(|a, b| {
            let usage_a = usage.get(&a.name).unwrap_or(&0);
            let usage_b = usage.get(&b.name).unwrap_or(&0);
            usage_b.cmp(usage_a)
        });

        debug!("Listed {} tools (sorted by usage)", sorted_tools.len());
        sorted_tools
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let tools = self.tools.read();
        tools.iter().find(|t| t.name == name).cloned()
    }

    /// Execute the query_memory tool
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `domain` - Task domain
    /// * `task_type` - Optional task type filter
    /// * `limit` - Maximum results to return
    ///
    /// # Returns
    ///
    /// Returns a JSON array of relevant episodes
    pub async fn query_memory(
        &self,
        query: String,
        domain: String,
        task_type: Option<String>,
        limit: usize,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("query_memory").await;

        debug!(
            "Querying memory: query='{}', domain='{}', limit={}",
            query, domain, limit
        );

        // Build task context from parameters
        let context = TaskContext {
            domain,
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: task_type
                .as_ref()
                .map(|t| vec![t.clone()])
                .unwrap_or_default(),
        };

        // Query actual memory for relevant episodes
        let episodes = self
            .memory
            .retrieve_relevant_context(query.clone(), context.clone(), limit)
            .await;

        // Also get relevant patterns
        let patterns = self
            .memory
            .retrieve_relevant_patterns(&context, limit)
            .await;

        // Calculate insights from retrieved data
        let success_count = episodes
            .iter()
            .filter(|e| e.reward.as_ref().is_some_and(|r| r.total > 0.7))
            .count();

        let avg_success_rate = if !episodes.is_empty() {
            success_count as f32 / episodes.len() as f32
        } else {
            0.0
        };

        Ok(json!({
            "episodes": episodes,
            "patterns": patterns,
            "insights": {
                "total_episodes": episodes.len(),
                "relevant_patterns": patterns.len(),
                "success_rate": avg_success_rate
            }
        }))
    }

    /// Execute the execute_agent_code tool
    ///
    /// # Arguments
    ///
    /// * `code` - TypeScript/JavaScript code to execute
    /// * `context` - Execution context
    ///
    /// # Returns
    ///
    /// Returns execution result from the sandbox
    ///
    /// # Security
    ///
    /// This method executes code in a secure sandbox with:
    /// - Timeout enforcement
    /// - Resource limits
    /// - No network access (by default)
    /// - No filesystem access (by default)
    /// - Malicious code detection
    pub async fn execute_agent_code(
        &self,
        code: String,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.track_tool_usage("execute_agent_code").await;

        info!(
            "Executing agent code: task='{}', code_length={}",
            context.task,
            code.len()
        );

        let start = std::time::Instant::now();

        // Execute in sandbox
        let result = self
            .sandbox
            .execute(&code, context)
            .await
            .context("Code execution failed")?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.record_execution(&result, duration_ms);
        }

        // Log result
        match &result {
            ExecutionResult::Success { .. } => {
                debug!("Code execution succeeded in {}ms", duration_ms);
            }
            ExecutionResult::Error { error_type, .. } => {
                warn!(
                    "Code execution failed: {:?} in {}ms",
                    error_type, duration_ms
                );
            }
            ExecutionResult::Timeout { elapsed_ms, .. } => {
                warn!("Code execution timed out after {}ms", elapsed_ms);
            }
            ExecutionResult::SecurityViolation { violation_type, .. } => {
                warn!("Security violation detected: {:?}", violation_type);
            }
        }

        Ok(result)
    }

    /// Execute the analyze_patterns tool
    ///
    /// # Arguments
    ///
    /// * `task_type` - Type of task to analyze
    /// * `min_success_rate` - Minimum success rate filter
    /// * `limit` - Maximum patterns to return
    ///
    /// # Returns
    ///
    /// Returns a JSON array of patterns with statistics
    pub async fn analyze_patterns(
        &self,
        task_type: String,
        min_success_rate: f32,
        limit: usize,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("analyze_patterns").await;

        debug!(
            "Analyzing patterns: task_type='{}', min_success_rate={}, limit={}",
            task_type, min_success_rate, limit
        );

        // Build context for pattern retrieval
        let context = TaskContext {
            domain: task_type.clone(),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: vec![task_type],
        };

        // Retrieve patterns from memory
        let all_patterns = self
            .memory
            .retrieve_relevant_patterns(&context, limit * 2)
            .await;

        // Filter by success rate and limit
        let filtered_patterns: Vec<_> = all_patterns
            .into_iter()
            .filter(|_p| {
                // Filter patterns based on success rate threshold
                // For now, we include all patterns as we don't have success_rate in Pattern yet
                // TODO: Add success_rate tracking to Pattern
                true
            })
            .take(limit)
            .collect();

        // Calculate statistics
        let total_patterns = filtered_patterns.len();
        let avg_success_rate = min_success_rate; // Placeholder until we track this in patterns

        // Extract most common tools from patterns
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        for pattern in &filtered_patterns {
            // Extract tools based on pattern type
            match pattern {
                Pattern::ToolSequence { tools, .. } => {
                    for tool in tools {
                        *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                    }
                }
                Pattern::DecisionPoint { action, .. } => {
                    // Count action as a tool usage
                    *tool_counts.entry(action.clone()).or_insert(0) += 1;
                }
                Pattern::ErrorRecovery { recovery_steps, .. } => {
                    for step in recovery_steps {
                        *tool_counts.entry(step.clone()).or_insert(0) += 1;
                    }
                }
                Pattern::ContextPattern {
                    recommended_approach,
                    ..
                } => {
                    *tool_counts.entry(recommended_approach.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut most_common_tools: Vec<_> = tool_counts.into_iter().collect();
        most_common_tools.sort_by(|a, b| b.1.cmp(&a.1));
        let most_common_tools: Vec<String> = most_common_tools
            .into_iter()
            .take(5)
            .map(|(tool, _)| tool)
            .collect();

        Ok(json!({
            "patterns": filtered_patterns,
            "statistics": {
                "total_patterns": total_patterns,
                "avg_success_rate": avg_success_rate,
                "most_common_tools": most_common_tools
            }
        }))
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.read().clone()
    }

    /// Get tool usage statistics
    pub async fn get_tool_usage(&self) -> HashMap<String, usize> {
        self.tool_usage.read().clone()
    }

    /// Track usage of a tool (for progressive disclosure)
    async fn track_tool_usage(&self, tool_name: &str) {
        let mut usage = self.tool_usage.write();
        *usage.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    /// Add a custom tool to the server
    pub async fn add_tool(&self, tool: Tool) -> Result<()> {
        let mut tools = self.tools.write();

        // Check for duplicate names
        if tools.iter().any(|t| t.name == tool.name) {
            anyhow::bail!("Tool with name '{}' already exists", tool.name);
        }

        info!("Adding custom tool: {}", tool.name);
        tools.push(tool);

        Ok(())
    }

    /// Remove a tool from the server
    pub async fn remove_tool(&self, tool_name: &str) -> Result<()> {
        let mut tools = self.tools.write();

        let initial_len = tools.len();
        tools.retain(|t| t.name != tool_name);

        if tools.len() == initial_len {
            anyhow::bail!("Tool '{}' not found", tool_name);
        }

        info!("Removed tool: {}", tool_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn create_test_server() -> MemoryMCPServer {
        let memory = Arc::new(SelfLearningMemory::new());
        MemoryMCPServer::new(SandboxConfig::default(), memory)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_test_server().await;
        let tools = server.list_tools().await;

        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "query_memory"));
        assert!(tools.iter().any(|t| t.name == "execute_agent_code"));
        assert!(tools.iter().any(|t| t.name == "analyze_patterns"));
    }

    #[tokio::test]
    async fn test_get_tool() {
        let server = create_test_server().await;

        let tool = server.get_tool("query_memory").await;
        assert!(tool.is_some());

        let tool = tool.unwrap();
        assert_eq!(tool.name, "query_memory");
        assert!(!tool.description.is_empty());
    }

    #[tokio::test]
    async fn test_execute_code() {
        let server = create_test_server().await;

        let code = "return 1 + 1;";
        let context = ExecutionContext::new("test".to_string(), json!({}));

        let result = server.execute_agent_code(code.to_string(), context).await;
        assert!(result.is_ok());

        // Check stats were updated
        let stats = server.get_stats().await;
        assert_eq!(stats.total_executions, 1);
    }

    #[tokio::test]
    async fn test_tool_usage_tracking() {
        let server = create_test_server().await;

        // Execute code multiple times
        for _ in 0..3 {
            let code = "return 1;";
            let context = ExecutionContext::new("test".to_string(), json!({}));
            let _ = server.execute_agent_code(code.to_string(), context).await;
        }

        // Check usage was tracked
        let usage = server.get_tool_usage().await;
        assert_eq!(usage.get("execute_agent_code"), Some(&3));
    }

    #[tokio::test]
    async fn test_progressive_tool_disclosure() {
        let server = create_test_server().await;

        // Use execute_agent_code multiple times
        for _ in 0..5 {
            let code = "return 1;";
            let context = ExecutionContext::new("test".to_string(), json!({}));
            let _ = server.execute_agent_code(code.to_string(), context).await;
        }

        // Use query_memory once
        let _ = server
            .query_memory("test".to_string(), "test".to_string(), None, 10)
            .await;

        // List tools - execute_agent_code should be first (most used)
        let tools = server.list_tools().await;
        assert_eq!(tools[0].name, "execute_agent_code");
    }

    #[tokio::test]
    async fn test_add_custom_tool() {
        let server = create_test_server().await;

        let custom_tool = Tool::new(
            "custom_tool".to_string(),
            "A custom tool".to_string(),
            json!({"type": "object"}),
        );

        let result = server.add_tool(custom_tool).await;
        assert!(result.is_ok());

        let tool = server.get_tool("custom_tool").await;
        assert!(tool.is_some());
    }

    #[tokio::test]
    async fn test_add_duplicate_tool_fails() {
        let server = create_test_server().await;

        let duplicate_tool = Tool::new(
            "query_memory".to_string(),
            "Duplicate".to_string(),
            json!({"type": "object"}),
        );

        let result = server.add_tool(duplicate_tool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_tool() {
        let server = create_test_server().await;

        // Add and then remove a tool
        let custom_tool = Tool::new(
            "temp_tool".to_string(),
            "Temporary".to_string(),
            json!({"type": "object"}),
        );

        server.add_tool(custom_tool).await.unwrap();
        assert!(server.get_tool("temp_tool").await.is_some());

        let result = server.remove_tool("temp_tool").await;
        assert!(result.is_ok());
        assert!(server.get_tool("temp_tool").await.is_none());
    }

    #[tokio::test]
    async fn test_query_memory() {
        let server = create_test_server().await;

        let result = server
            .query_memory(
                "test query".to_string(),
                "web-api".to_string(),
                Some("code_generation".to_string()),
                10,
            )
            .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("episodes").is_some());
        assert!(json.get("patterns").is_some());
    }

    #[tokio::test]
    async fn test_analyze_patterns() {
        let server = create_test_server().await;

        let result = server
            .analyze_patterns("code_generation".to_string(), 0.7, 20)
            .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("patterns").is_some());
        assert!(json.get("statistics").is_some());
    }

    #[tokio::test]
    async fn test_execution_stats() {
        let server = create_test_server().await;

        // Execute some code
        let code = "return 42;";
        let context = ExecutionContext::new("test".to_string(), json!({}));
        let _ = server.execute_agent_code(code.to_string(), context).await;

        let stats = server.get_stats().await;
        assert_eq!(stats.total_executions, 1);
        assert!(stats.avg_execution_time_ms > 0.0);
    }
}
