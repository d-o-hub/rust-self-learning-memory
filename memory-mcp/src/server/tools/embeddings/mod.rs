//! Embedding tool handlers for the MCP server
//!
//! This module contains the three embedding tool handlers:
//! - configure_embeddings: Configure embedding provider
//! - query_semantic_memory: Search episodes using semantic similarity
//! - test_embeddings: Test embedding provider connectivity

mod configure;
mod query;
mod test;

#[cfg(test)]
mod tests;
