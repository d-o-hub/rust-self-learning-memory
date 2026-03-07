//! # Embedding Configuration and Semantic Query MCP Tools

mod tests;
mod tool;
mod types;

pub use tool::{
    EmbeddingTools, configure_embeddings_tool, embedding_provider_status_tool,
    generate_embedding_tool, query_semantic_memory_tool, search_by_embedding_tool,
    test_embeddings_tool,
};
pub use types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, EmbeddingProviderStatusInput,
    EmbeddingProviderStatusOutput, EmbeddingSearchResult, GenerateEmbeddingInput,
    GenerateEmbeddingOutput, ProviderTestResult, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SearchByEmbeddingInput, SearchByEmbeddingOutput, SemanticResult,
    TestEmbeddingsOutput,
};
