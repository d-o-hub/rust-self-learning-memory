//! Utility functions and types for local embedding models
//!
//! This module provides helper functions and configuration types
//! for managing local embedding models.

use super::config::ModelConfig;

/// List available local models
pub fn list_available_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig::local_sentence_transformer("sentence-transformers/all-MiniLM-L6-v2", 384),
        ModelConfig::local_sentence_transformer("sentence-transformers/all-mpnet-base-v2", 768),
        ModelConfig::local_sentence_transformer(
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            384,
        ),
    ]
}

/// Get recommended model configuration for different use cases
pub fn get_recommended_model(use_case: LocalModelUseCase) -> ModelConfig {
    match use_case {
        LocalModelUseCase::Fast => {
            ModelConfig::local_sentence_transformer("sentence-transformers/all-MiniLM-L6-v2", 384)
        }
        LocalModelUseCase::Quality => {
            ModelConfig::local_sentence_transformer("sentence-transformers/all-mpnet-base-v2", 768)
        }
        LocalModelUseCase::Multilingual => ModelConfig::local_sentence_transformer(
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            384,
        ),
    }
}

/// Use cases for local model selection
pub enum LocalModelUseCase {
    /// Fast inference with good quality (384 dimensions)
    Fast,
    /// Best quality (768 dimensions, slower)
    Quality,
    /// Multilingual support (384 dimensions)
    Multilingual,
}
