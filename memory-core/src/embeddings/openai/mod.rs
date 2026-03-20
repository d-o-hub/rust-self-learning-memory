//! OpenAI embedding provider module.
//!
//! Contains the `OpenAIEmbeddingProvider` struct and its implementation.

pub mod client;

#[cfg(feature = "openai")]
pub use client::OpenAIEmbeddingProvider;
