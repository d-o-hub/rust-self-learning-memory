//! Mistral embedding provider implementation

#[cfg(feature = "mistral")]
pub use client::MistralEmbeddingProvider;

#[cfg(feature = "mistral")]
mod client;

#[cfg(feature = "mistral")]
mod types;
