//! Embedding tools module

mod definitions;
mod execute;

// Re-export everything from definitions
pub use definitions::*;
pub use execute::configured_embedding_storage;
