//! OpenAI embedding provider module.
//!
//! Contains the `OpenAIEmbeddingProvider` struct and its implementation.

pub mod client;
pub mod types;
pub mod utils;

#[cfg(feature = "openai")]
pub use client::OpenAIEmbeddingProvider;
