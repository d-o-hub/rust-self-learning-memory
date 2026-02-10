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
pub mod audit;
pub mod cache_warming;
pub mod rate_limiter;
pub mod sandbox;
#[cfg(test)]
mod tests;
pub mod tool_definitions;
pub mod tool_definitions_extended;
pub mod tools;

use crate::cache::QueryCache;
use crate::monitoring::{MonitoringConfig, MonitoringEndpoints, MonitoringSystem};
use crate::server::audit::{AuditConfig, AuditLogger};
use crate::server::tools::registry::ToolRegistry;
use crate::types::{ExecutionStats, SandboxConfig};
use crate::unified_sandbox::UnifiedSandbox;
use anyhow::Result;
use memory_core::SelfLearningMemory;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// MCP server for memory integration
pub struct MemoryMCPServer {
    /// Code execution sandbox
    sandbox: Arc<UnifiedSandbox>,
    /// Tool registry for lazy-loading tools
    tool_registry: Arc<ToolRegistry>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
    /// Tool usage tracking for progressive disclosure (kept for compatibility)
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
    /// Audit logger for security events
    audit_logger: Arc<AuditLogger>,
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
        let sandbox_backend = sandbox::determine_sandbox_backend();
        let sandbox = Arc::new(UnifiedSandbox::new(config.clone(), sandbox_backend).await?);

        // Use tool registry for lazy loading
        let tool_registry = Arc::new(tools::registry::create_default_registry());

        let monitoring = Self::initialize_monitoring();
        let monitoring_endpoints = Arc::new(MonitoringEndpoints::new(Arc::clone(&monitoring)));

        // Initialize audit logger
        let audit_config = AuditConfig::from_env();
        let audit_logger = Arc::new(AuditLogger::new(audit_config).await?);

        let core_count = tool_registry.get_core_tools().len();
        let total_count = tool_registry.total_tool_count();

        let server = Self {
            sandbox,
            tool_registry,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            tool_usage: Arc::new(RwLock::new(HashMap::new())),
            memory,
            monitoring,
            monitoring_endpoints,
            cache: Arc::new(QueryCache::new()),
            audit_logger,
        };

        info!(
            "MCP server initialized with {} core tools ({} total tools available)",
            core_count, total_count
        );
        info!("Tools loaded on-demand to reduce token usage (lazy loading enabled)");
        info!(
            "Monitoring system initialized (enabled: {})",
            server.monitoring.config().enabled
        );
        info!("Audit logging system initialized");

        // Perform cache warming if enabled
        if cache_warming::is_cache_warming_enabled() {
            info!("Starting cache warming process...");
            if let Err(e) = cache_warming::warm_cache(
                &server.memory,
                &cache_warming::CacheWarmingConfig::from_env(),
            )
            .await
            {
                tracing::warn!(
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

    fn initialize_monitoring() -> Arc<MonitoringSystem> {
        let monitoring_config = MonitoringConfig::default();
        Arc::new(MonitoringSystem::new(monitoring_config))
    }

    /// Get a reference to the memory system
    ///
    /// # Returns
    ///
    /// Returns a clone of the `Arc<SelfLearningMemory>`
    pub fn memory(&self) -> Arc<SelfLearningMemory> {
        Arc::clone(&self.memory)
    }

    /// Get a reference to the audit logger
    ///
    /// # Returns
    ///
    /// Returns a clone of the `Arc<AuditLogger>`
    pub fn audit_logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit_logger)
    }
}
