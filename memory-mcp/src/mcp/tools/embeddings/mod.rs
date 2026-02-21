//! # Embedding Configuration and Semantic Query MCP Tools

mod tests;
mod tool;
mod types;

pub use tool::{
    EmbeddingTools, configure_embeddings_tool, query_semantic_memory_tool, test_embeddings_tool,
};
pub use types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
