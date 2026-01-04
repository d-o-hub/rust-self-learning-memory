// Tool management handlers
//!
//! This module contains tool management functions: add_tool, remove_tool, get_stats, and tracking.

use crate::monitoring::MonitoringEndpoints;
use crate::monitoring::MonitoringSystem;
use crate::types::{ExecutionStats, Tool};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

impl crate::server::MemoryMCPServer {
    /// Get execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.read().clone()
    }

    /// Get tool usage statistics
    pub async fn get_tool_usage(&self) -> HashMap<String, usize> {
        self.tool_usage.read().clone()
    }

    /// Track usage of a tool (for progressive disclosure)
    pub(super) async fn track_tool_usage(&self, tool_name: &str) {
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

    /// Get monitoring endpoints
    pub fn monitoring_endpoints(&self) -> Arc<MonitoringEndpoints> {
        Arc::clone(&self.monitoring_endpoints)
    }

    /// Get monitoring system
    pub fn monitoring_system(&self) -> Arc<MonitoringSystem> {
        Arc::clone(&self.monitoring)
    }

    /// Update system metrics (memory, CPU)
    pub async fn update_system_metrics(&self) {
        if !self.monitoring.config().enabled {
            return;
        }

        // Get basic system metrics (simplified for now)
        let memory_mb = 50.0;
        let cpu_percent = 5.0;

        self.monitoring
            .update_system_metrics(memory_mb, cpu_percent);
    }
}
