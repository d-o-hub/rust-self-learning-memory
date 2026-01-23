//! Real embedding model using ONNX runtime
//!
//! This module provides the actual ONNX-based embedding model implementation
//! that runs locally when the 'local-embeddings' feature is enabled.

mod download;
mod model;
mod tests;

pub use model::RealEmbeddingModel;
