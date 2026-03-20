//! Tool definitions for the lazy-loading registry
//!
//! This module defines the core and extended tools for the MCP server.

use crate::server::tool_definitions_extended;
use std::collections::HashMap;

#[path = "builder.rs"]
mod builder;

use builder::{create_additional_extended_tools, create_core_tools};

/// Create the default tool registry with core and extended tools
pub fn create_default_registry() -> super::ToolRegistry {
    // Core tools: essential for basic operations
    let core_tools = create_core_tools();

    // All extended tools from existing tool definitions
    let extended_defs = tool_definitions_extended::create_extended_tools();
    let mut extended_tools = HashMap::new();

    // Convert Vec<Tool> to HashMap
    for tool in extended_defs {
        // Skip core tools that are already defined above
        if !core_tools.iter().any(|t| t.name == tool.name) {
            extended_tools.insert(tool.name.clone(), tool);
        }
    }

    // Add additional extended tools from tool_definitions.rs
    // These include advanced pattern analysis, embeddings, and pattern search
    let additional_tools = create_additional_extended_tools();
    for tool in additional_tools {
        // Skip if already in core tools
        if !core_tools.iter().any(|t| t.name == tool.name) &&
           // Skip if already in extended tools
           !extended_tools.contains_key(&tool.name)
        {
            extended_tools.insert(tool.name.clone(), tool);
        }
    }

    super::ToolRegistry::new(core_tools, extended_tools)
}

#[cfg(test)]
mod tests {
    use super::create_default_registry;

    #[test]
    fn default_registry_has_expected_core_tools() {
        let registry = create_default_registry();
        let core_names: Vec<String> = registry
            .get_core_tools()
            .into_iter()
            .map(|tool| tool.name)
            .collect();

        assert!(core_names.iter().any(|name| name == "query_memory"));
        assert!(core_names.iter().any(|name| name == "health_check"));
        assert!(core_names.iter().any(|name| name == "get_metrics"));
        assert!(core_names.iter().any(|name| name == "analyze_patterns"));
        assert!(core_names.iter().any(|name| name == "create_episode"));
    }

    #[test]
    fn default_registry_includes_extended_tools() {
        let registry = create_default_registry();

        assert!(registry.tool_exists("search_patterns"));
        assert!(registry.tool_exists("recommend_patterns"));
        assert!(registry.total_tool_count() > registry.get_core_tools().len());
    }
}
