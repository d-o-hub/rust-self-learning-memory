//! # Embedding Configuration and Semantic Query MCP Tools

mod tests;
mod tool;
mod types;

pub use tool::{
    configure_embeddings_tool, query_semantic_memory_tool, test_embeddings_tool, EmbeddingTools,
    EmbeddingToolsExecuteExt,
};
pub use types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
