//! Core tool registry implementation
//!
//! This module provides the core registry logic for lazy-loading MCP tools.

pub mod definitions;

pub use definitions::create_default_registry;

use crate::protocol::ToolStub;
use crate::types::Tool;
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Tool category for organizing tools by priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// Core tools - always loaded (query_memory, health_check, etc.)
    Core,
    /// Extended tools - loaded on-demand (advanced analysis, batch operations, etc.)
    Extended,
}

/// Tool registry for lazy-loading tools
pub struct ToolRegistry {
    /// Core tools loaded at initialization
    core_tools: Vec<Tool>,
    /// Extended tools that can be loaded on-demand
    extended_tools: HashMap<String, Tool>,
    /// Tools loaded in current session (tool name -> Tool)
    session_loaded: Arc<RwLock<HashMap<String, Tool>>>,
    /// Tool usage tracking for progressive disclosure
    usage_count: Arc<RwLock<HashMap<String, usize>>>,
}

impl ToolRegistry {
    /// Create a new tool registry with core and extended tools
    ///
    /// # Arguments
    ///
    /// * `core_tools` - Tools that should always be loaded
    /// * `extended_tools` - Tools that can be loaded on-demand
    pub fn new(core_tools: Vec<Tool>, extended_tools: HashMap<String, Tool>) -> Self {
        let core_count = core_tools.len();
        let extended_count = extended_tools.len();

        info!(
            "ToolRegistry initialized: {} core tools, {} extended tools",
            core_count, extended_count
        );

        // Validate that core tool names don't conflict with extended
        let core_names: std::collections::HashSet<_> =
            core_tools.iter().map(|t| t.name.clone()).collect();
        for extended_name in extended_tools.keys() {
            if core_names.contains(extended_name) {
                warn!(
                    "Tool name conflict: '{}' in both core and extended tools",
                    extended_name
                );
            }
        }

        Self {
            core_tools,
            extended_tools,
            session_loaded: Arc::new(RwLock::new(HashMap::new())),
            usage_count: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get core tools (always available)
    pub fn get_core_tools(&self) -> Vec<Tool> {
        self.core_tools.clone()
    }

    /// List all tool names only (lightweight - for token optimization)
    ///
    /// This returns only tool names without full schemas, providing significant
    /// token reduction for tool discovery. For full schemas, use `load_tool()`.
    ///
    /// # Token Savings
    ///
    /// - **Before**: Full tool list with schemas (~12,000 tokens)
    /// - **After**: Names only (~200 tokens)
    /// - **Reduction**: ~98%
    ///
    /// # Usage
    ///
    /// ```rust,no_run
    /// use do_memory_mcp::server::tools::registry::create_default_registry;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let registry = create_default_registry();
    ///
    /// // Get lightweight list of all tool names
    /// let names = registry.list_tool_names();
    /// // names = ["query_memory", "analyze_patterns", ...]
    ///
    /// // Load specific tool schema on-demand
    /// let tool = registry.load_tool("query_memory").await;
    /// # let _ = (names, tool);
    /// # }
    /// ```
    pub fn list_tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.core_tools.iter().map(|t| t.name.clone()).collect();
        let extended_names: Vec<String> = self.extended_tools.keys().cloned().collect();
        names.extend(extended_names);
        names
    }

    /// Get all currently loaded tools (core + session-loaded extended)
    pub fn get_loaded_tools(&self) -> Vec<Tool> {
        let loaded = self.session_loaded.read();
        let mut tools = self.core_tools.clone();
        tools.extend(loaded.values().cloned());
        tools.sort_by(|a, b| self.cmp_tools_by_usage(a, b));
        tools
    }

    /// Get every registered tool with its full schema (core + extended).
    ///
    /// Unlike [`Self::get_loaded_tools`], this is independent of the session
    /// loading state: it never mutates `session_loaded` or usage counters, so a
    /// freshly created registry returns the complete registry. Core tools come
    /// first (in definition order), followed by extended tools sorted by name
    /// for deterministic output.
    pub fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = self.core_tools.clone();
        tools.extend(self.sorted_extended_tools());
        tools
    }

    /// Get name/description stubs for every registered tool (core + extended).
    ///
    /// Data source for the lazy (`lazy=true`) `tools/list` response: it covers
    /// the whole registry without cloning full input schemas and without
    /// touching session loading state. Ordering matches [`Self::get_all_tools`].
    pub fn get_all_tool_stubs(&self) -> Vec<ToolStub> {
        let mut stubs: Vec<ToolStub> = self.core_tools.iter().map(tool_stub).collect();
        stubs.extend(self.sorted_extended_tools().iter().map(tool_stub));
        stubs
    }

    /// Extended tools in deterministic (name-sorted) order.
    fn sorted_extended_tools(&self) -> Vec<Tool> {
        let mut extended: Vec<Tool> = self.extended_tools.values().cloned().collect();
        extended.sort_by(|a, b| a.name.cmp(&b.name));
        extended
    }

    /// Load a specific tool by name (if not already loaded)
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name to load
    ///
    /// # Returns
    ///
    /// Returns `Some(Tool)` if found, `None` if tool doesn't exist
    #[expect(clippy::unused_async, clippy::unused_async_trait_impl)]
    pub async fn load_tool(&self, name: &str) -> Option<Tool> {
        // Check if already in core tools
        if let Some(tool) = self.core_tools.iter().find(|t| t.name == name) {
            self.track_usage(name);
            return Some(tool.clone());
        }

        // Check if already loaded in session
        {
            let loaded = self.session_loaded.read();
            if let Some(tool) = loaded.get(name) {
                self.track_usage(name);
                return Some(tool.clone());
            }
        }

        // Load from extended tools
        if let Some(tool) = self.extended_tools.get(name) {
            debug!("Loading extended tool: {}", name);
            let mut loaded = self.session_loaded.write();
            loaded.insert(name.to_string(), tool.clone());
            self.track_usage(name);
            return Some(tool.clone());
        }

        warn!("Tool not found: {}", name);
        None
    }

    /// Load multiple tools by name
    ///
    /// # Arguments
    ///
    /// * `names` - Tool names to load
    ///
    /// # Returns
    ///
    /// Returns vector of found tools (may be shorter than input if some not found)
    pub async fn load_tools(&self, names: &[String]) -> Vec<Tool> {
        let mut tools = Vec::new();
        for name in names {
            if let Some(tool) = self.load_tool(name).await {
                tools.push(tool);
            }
        }
        tools
    }

    /// Check if a tool exists (without loading it)
    pub fn tool_exists(&self, name: &str) -> bool {
        self.core_tools.iter().any(|t| t.name == name) || self.extended_tools.contains_key(name)
    }

    /// Get total tool count (core + extended)
    pub fn total_tool_count(&self) -> usize {
        self.core_tools.len() + self.extended_tools.len()
    }

    /// Get loaded tool count (core + session-loaded)
    pub fn loaded_tool_count(&self) -> usize {
        let loaded = self.session_loaded.read();
        self.core_tools.len() + loaded.len()
    }

    /// Get usage count for a specific tool
    pub fn get_tool_usage(&self, name: &str) -> usize {
        let usage = self.usage_count.read();
        *usage.get(name).unwrap_or(&0)
    }

    /// Get all tool usage statistics
    pub fn get_usage_stats(&self) -> HashMap<String, usize> {
        let usage = self.usage_count.read();
        usage.clone()
    }

    /// Clear session cache (for testing or session reset)
    pub fn clear_session_cache(&self) {
        let mut loaded = self.session_loaded.write();
        let cleared_count = loaded.len();
        loaded.clear();
        debug!("Cleared session cache ({} tools)", cleared_count);
    }

    /// Add a custom tool to the session
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to add
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the tool already exists
    pub fn add_tool(&self, tool: Tool) -> Result<()> {
        let name = tool.name.clone();

        if self.tool_exists(&name) {
            anyhow::bail!("Tool with name '{}' already exists", name);
        }

        let mut loaded = self.session_loaded.write();
        info!("Adding custom tool: {}", name);
        loaded.insert(name, tool);
        Ok(())
    }

    /// Remove a tool from the session (cannot remove core tools)
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the tool doesn't exist or is a core tool
    pub fn remove_tool(&self, name: &str) -> Result<()> {
        if self.core_tools.iter().any(|t| t.name == name) {
            anyhow::bail!("Cannot remove core tool '{}'", name);
        }

        let mut loaded = self.session_loaded.write();
        if loaded.remove(name).is_none() && !self.extended_tools.contains_key(name) {
            anyhow::bail!("Tool '{}' not found", name);
        }

        info!("Removed tool: {}", name);
        Ok(())
    }

    /// Track tool usage for progressive disclosure
    fn track_usage(&self, name: &str) {
        let mut usage = self.usage_count.write();
        *usage.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Compare tools by usage frequency (for sorting)
    fn cmp_tools_by_usage(&self, a: &Tool, b: &Tool) -> std::cmp::Ordering {
        let usage_a = self.get_tool_usage(&a.name);
        let usage_b = self.get_tool_usage(&b.name);
        usage_b.cmp(&usage_a) // Descending order (most used first)
    }
}

/// Convert a full tool into its lightweight name/description stub.
fn tool_stub(tool: &Tool) -> ToolStub {
    ToolStub {
        name: tool.name.clone(),
        title: None,
        description: tool.description.clone(),
    }
}

/// Create the default tool registry with core and extended tools
pub fn create_tool_registry() -> ToolRegistry {
    create_default_registry()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = create_tool_registry();
        assert!(registry.total_tool_count() > 8);
        assert_eq!(registry.get_core_tools().len(), 8);
    }

    #[test]
    fn test_core_tools_always_available() {
        let registry = create_tool_registry();
        let core = registry.get_core_tools();
        assert!(core.iter().any(|t| t.name == "query_memory"));
        assert!(core.iter().any(|t| t.name == "health_check"));
    }

    #[test]
    fn test_load_extended_tool() {
        let registry = create_tool_registry();

        // Find an extended tool name
        let extended_name = registry
            .extended_tools
            .keys()
            .next()
            .expect("Should have extended tools");

        // Initially not loaded
        assert_eq!(registry.loaded_tool_count(), 8);

        // Load the tool
        let rt = tokio::runtime::Runtime::new().unwrap();
        let tool = rt.block_on(registry.load_tool(extended_name));

        assert!(tool.is_some());
        assert_eq!(registry.loaded_tool_count(), 9);
    }

    #[test]
    fn test_tool_usage_tracking() {
        let registry = create_tool_registry();
        assert_eq!(registry.get_tool_usage("query_memory"), 0);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(registry.load_tool("query_memory"));

        assert_eq!(registry.get_tool_usage("query_memory"), 1);
    }

    #[test]
    fn test_clear_session_cache() {
        let registry = create_tool_registry();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let extended_name = registry
            .extended_tools
            .keys()
            .next()
            .expect("Should have extended tools");

        rt.block_on(registry.load_tool(extended_name));
        assert_eq!(registry.loaded_tool_count(), 9);

        registry.clear_session_cache();
        assert_eq!(registry.loaded_tool_count(), 8);
    }

    #[test]
    fn test_list_tool_names() {
        let registry = create_tool_registry();

        // Get lightweight list of all tool names
        let names = registry.list_tool_names();

        // Should return all tool names (core + extended)
        assert!(names.len() >= 8); // At least core tools
        assert!(names.contains(&"query_memory".to_string()));
        assert!(names.contains(&"health_check".to_string()));
    }

    #[test]
    fn test_list_tool_names_vs_full_schema() {
        let registry = create_tool_registry();

        // Lightweight: Names only (~200 tokens)
        let names = registry.list_tool_names();

        // Should be much smaller than full tool count
        let total = registry.total_tool_count();
        assert_eq!(names.len(), total);
    }

    #[test]
    fn test_get_all_tools_covers_full_registry() {
        let registry = create_tool_registry();

        let tools = registry.get_all_tools();
        assert_eq!(tools.len(), registry.total_tool_count());

        // Names are unique and match the lightweight name listing.
        let mut names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        let expected = registry.list_tool_names();
        names.sort();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();
        assert_eq!(names, expected_sorted);

        // Every tool carries a full input schema.
        for tool in &tools {
            assert!(
                tool.input_schema.is_object(),
                "tool '{}' must expose an object inputSchema",
                tool.name
            );
        }

        // Core tools come first, in definition order.
        let core_len = registry.get_core_tools().len();
        assert_eq!(
            tools[..core_len]
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>(),
            registry
                .get_core_tools()
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_get_all_tools_does_not_load_extended_tools() {
        let registry = create_tool_registry();
        let before = registry.get_loaded_tools();

        let all = registry.get_all_tools();
        assert!(all.len() > before.len());

        let after = registry.get_loaded_tools();
        assert_eq!(before.len(), after.len(), "listing must not load tools");
        assert_eq!(
            registry.loaded_tool_count(),
            before.len(),
            "listing must not change loaded count"
        );
    }

    #[test]
    fn test_get_all_tool_stubs_parity_with_full_tools() {
        let registry = create_tool_registry();

        let full = registry.get_all_tools();
        let stubs = registry.get_all_tool_stubs();

        assert_eq!(stubs.len(), full.len());
        for (tool, stub) in full.iter().zip(stubs.iter()) {
            assert_eq!(tool.name, stub.name);
            assert_eq!(tool.description, stub.description);
        }

        // Stubs never load execution state.
        assert_eq!(
            registry.loaded_tool_count(),
            registry.get_core_tools().len()
        );
    }
}
