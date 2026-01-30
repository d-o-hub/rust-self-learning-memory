//! Utility functions and types for local embedding models
//!
//! This module provides helper functions and configuration types
//! for managing local embedding models.

use super::config::LocalConfig;

/// List available local models
#[must_use]
pub fn list_available_models() -> Vec<LocalConfig> {
    vec![
        LocalConfig::new("sentence-transformers/all-MiniLM-L6-v2", 384),
        LocalConfig::new("sentence-transformers/all-mpnet-base-v2", 768),
        LocalConfig::new(
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            384,
        ),
    ]
}

/// Get recommended model configuration for different use cases
#[must_use]
pub fn get_recommended_model(use_case: LocalModelUseCase) -> LocalConfig {
    match use_case {
        LocalModelUseCase::Fast => LocalConfig::new("sentence-transformers/all-MiniLM-L6-v2", 384),
        LocalModelUseCase::Quality => {
            LocalConfig::new("sentence-transformers/all-mpnet-base-v2", 768)
        }
        LocalModelUseCase::Multilingual => LocalConfig::new(
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
