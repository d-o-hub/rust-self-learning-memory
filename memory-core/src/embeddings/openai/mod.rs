//! OpenAI embedding provider module.
//!
//! Contains the `OpenAIEmbeddingProvider` struct and its implementation.

pub mod client;
pub mod types;
pub mod utils;

#[cfg(feature = "openai")]
pub use client::OpenAIEmbeddingProvider;

// Re-export config types for convenience
#[cfg(feature = "openai")]
pub use crate::embeddings::config::openai::{
    EncodingFormat, OpenAIConfig, OpenAIEmbeddingInput, OpenAIEmbeddingRequest, OpenAIModel,
};
