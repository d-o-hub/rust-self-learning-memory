//! Pattern search tool implementations for MCP server

use crate::mcp::tools::pattern_search::{RecommendPatternsInput, SearchPatternsInput};
use crate::server::MemoryMCPServer;
use serde_json::Value;

impl MemoryMCPServer {
    /// Execute search_patterns tool
    pub async fn execute_search_patterns(
        &self,
        input: SearchPatternsInput,
    ) -> anyhow::Result<Value> {
        crate::mcp::tools::pattern_search::execute(&self.memory, input).await
    }

    /// Execute recommend_patterns tool
    pub async fn execute_recommend_patterns(
        &self,
        input: RecommendPatternsInput,
    ) -> anyhow::Result<Value> {
        crate::mcp::tools::pattern_search::execute_recommend(&self.memory, input).await
    }
}
