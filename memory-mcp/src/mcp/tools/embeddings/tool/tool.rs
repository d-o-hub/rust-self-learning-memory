//! Embedding tools implementation.

use crate::mcp::tools::embeddings::types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
use crate::types::Tool;
use memory_core::SelfLearningMemory;
use serde_json::json;
use std::sync::Arc;

/// Embedding tools implementation
pub struct EmbeddingTools {
    memory: Arc<SelfLearningMemory>,
}

impl EmbeddingTools {
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }
}

/// Get the tool definition for configure_embeddings
pub fn configure_embeddings_tool() -> Tool {
    Tool::new(
        "configure_embeddings".to_string(),
        "Configure semantic embedding provider for enhanced memory retrieval.",
        json!({
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string",
                    "enum": ["openai", "local", "mistral", "azure", "cohere"],
                    "description": "Embedding provider to use"
                },
                "model": {"type": "string", "description": "Model name"},
                "api_key_env": {"type": "string", "description": "API key env var"},
                "similarity_threshold": {
                    "type": "number", "minimum": 0.0, "maximum": 1.0, "default": 0.7,
                    "description": "Min similarity score"
                },
                "batch_size": {
                    "type": "integer", "minimum": 1, "maximum": 2048, "default": 32,
                    "description": "Batch size"
                },
                "base_url": {"type": "string", "description": "Custom base URL"},
                "api_version": {"type": "string", "description": "API version"},
                "resource_name": {"type": "string", "description": "Azure resource"},
                "deployment_name": {"type": "string", "description": "Azure deployment"}
            },
            "required": ["provider"]
        }),
    )
}

/// Get the tool definition for query_semantic_memory
pub fn query_semantic_memory_tool() -> Tool {
    Tool::new(
        "query_semantic_memory".to_string(),
        "Search episodic memory using semantic similarity with embeddings.",
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "limit": {
                    "type": "integer", "minimum": 1, "maximum": 100, "default": 10,
                    "description": "Max results"
                },
                "similarity_threshold": {
                    "type": "number", "minimum": 0.0, "maximum": 1.0, "default": 0.7,
                    "description": "Min similarity"
                },
                "domain": {"type": "string", "description": "Filter by domain"},
                "task_type": {"type": "string", "description": "Filter by task type"}
            },
            "required": ["query"]
        }),
    )
}

/// Get the tool definition for test_embeddings
pub fn test_embeddings_tool() -> Tool {
    Tool::new(
        "test_embeddings".to_string(),
        "Test embedding provider connectivity.",
        json!({"type": "object", "properties": {}, "additionalProperties": false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_embeddings_tool_definition() {
        let tool = configure_embeddings_tool();
        assert_eq!(tool.name, "configure_embeddings");
    }

    #[test]
    fn test_query_semantic_memory_tool_definition() {
        let tool = query_semantic_memory_tool();
        assert_eq!(tool.name, "query_semantic_memory");
    }

    #[test]
    fn test_test_embeddings_tool_definition() {
        let tool = test_embeddings_tool();
        assert_eq!(tool.name, "test_embeddings");
    }
}
