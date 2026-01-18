//! OpenAI API models and types.
//!
//! Contains request/response structures and shared types.

use serde::{Deserialize, Serialize};

/// Input for embedding request (single text or batch)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Single(String),
    Batch(Vec<String>),
}

/// Utility functions for `OpenAI` provider
pub mod utils {
    use super::{ModelConfig, Result};

    /// Validate `OpenAI` API key format
    #[allow(dead_code)]
    pub fn validate_api_key(api_key: &str) -> Result<()> {
        if api_key.is_empty() {
            anyhow::bail!("OpenAI API key is empty");
        }

        if !api_key.starts_with("sk-") {
            anyhow::bail!("OpenAI API key should start with 'sk-'");
        }

        if api_key.len() < 20 {
            anyhow::bail!("OpenAI API key appears to be too short");
        }

        Ok(())
    }

    /// Get the appropriate model configuration for different use cases
    #[allow(dead_code)]
    pub fn get_recommended_model(use_case: OpenAIModelUseCase) -> ModelConfig {
        match use_case {
            OpenAIModelUseCase::Balanced => ModelConfig::openai_3_small(),
            OpenAIModelUseCase::Quality => ModelConfig::openai_3_large(),
            OpenAIModelUseCase::Legacy => ModelConfig::openai_ada_002(),
        }
    }

    /// Calculate approximate cost for embedding generation
    ///
    /// Based on `OpenAI`'s pricing as of 2024. Prices may change.
    #[allow(dead_code)]
    pub fn estimate_cost(num_tokens: usize, model: &str) -> f64 {
        let cost_per_million_tokens = match model {
            "text-embedding-ada-002" => 0.10,
            "text-embedding-3-small" => 0.02,
            "text-embedding-3-large" => 0.13,
            _ => 0.10,
        };

        (num_tokens as f64 / 1_000_000.0) * cost_per_million_tokens
    }

    /// Estimate token count for text (approximate)
    #[allow(dead_code)]
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f64 / 4.0).ceil() as usize
    }

    /// Use cases for `OpenAI` model selection
    #[allow(dead_code)]
    pub enum OpenAIModelUseCase {
        Balanced,
        Quality,
        Legacy,
    }
}
