//! # Embedding Configuration and Semantic Query MCP Tools
//!
//! This module provides MCP tool integration for semantic embeddings, enabling:
//! - Embedding provider configuration (OpenAI, Local, Mistral, Azure, Cohere)
//! - Semantic memory queries using vector similarity
//! - Embedding provider testing and diagnostics
//!
//! ## Tools
//!
//! - `configure_embeddings`: Configure the semantic embedding provider
//! - `query_semantic_memory`: Search memory using semantic similarity
//! - `test_embeddings`: Test embedding provider connectivity and performance

mod tests;
mod tool;
mod types;

pub use tool::EmbeddingTools;
pub use types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
