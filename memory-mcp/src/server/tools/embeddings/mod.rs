//! Embedding tool handlers for the MCP server
//!
//! This module contains the embedding tool handlers:
//! - configure_embeddings: Configure embedding provider
//! - generate_embedding: Generate embedding vector for text
//! - query_semantic_memory: Search episodes using semantic similarity
//! - search_by_embedding: Search episodes by embedding vector
//! - embedding_provider_status: Get embedding provider status
//! - test_embeddings: Test embedding provider connectivity

mod configure;
mod generate;
mod query;
mod search;
mod status;
mod test;

#[cfg(test)]
mod tests;
