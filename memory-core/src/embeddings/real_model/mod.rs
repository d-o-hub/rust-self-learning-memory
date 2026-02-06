//! Real embedding model using ONNX runtime
//!
//! This module provides the actual ONNX-based embedding model implementation
//! that runs locally when the 'local-embeddings' feature is enabled.

#[cfg(feature = "local-embeddings")]
mod download;
#[cfg(feature = "local-embeddings")]
mod model;
#[cfg(feature = "local-embeddings")]
mod tests;

#[cfg(feature = "local-embeddings")]
pub use model::RealEmbeddingModel;
