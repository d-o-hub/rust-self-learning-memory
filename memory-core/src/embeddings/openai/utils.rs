//! Utility functions for OpenAI provider.

use super::super::config::ModelConfig;
use anyhow::Result;

/// Validate OpenAI API key format
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
/// Based on OpenAI's pricing as of 2024. Prices may change.
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
///
/// This is a rough estimate. Actual token count may differ.
#[allow(dead_code)]
pub fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~1 token per 4 characters for English text
    (text.len() as f64 / 4.0).ceil() as usize
}

/// Use cases for OpenAI model selection
#[allow(dead_code)]
pub enum OpenAIModelUseCase {
    /// Balanced performance and cost (text-embedding-3-small)
    Balanced,
    /// Highest quality (text-embedding-3-large)
    Quality,
    /// Legacy compatibility (text-embedding-ada-002)
    Legacy,
}
