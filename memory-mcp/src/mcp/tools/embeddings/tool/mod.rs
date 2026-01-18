//! Embedding tools module

mod execute;
mod tool;

pub use execute::EmbeddingToolsExecuteExt;
pub use tool::{
    configure_embeddings_tool, query_semantic_memory_tool, test_embeddings_tool, EmbeddingTools,
};
