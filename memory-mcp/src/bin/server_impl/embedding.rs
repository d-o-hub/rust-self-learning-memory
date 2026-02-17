//! Embedding configuration handler for MCP server
//!
//! This module provides embedding configuration support via environment variables
//! and a JSON-RPC handler for the embedding/config tool.
//!
//! All functions in this module are gated behind the `embeddings` feature flag
//! and reserved for future embedding configuration implementation.

#![cfg(feature = "embeddings")]

use super::types::EmbeddingEnvConfig;
use memory_mcp::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use tracing::info;

/// Load embedding configuration from environment variables
///
/// Environment variables:
/// - `EMBEDDING_PROVIDER`: Provider name (openai, local, mistral, azure, cohere)
/// - `OPENAI_API_KEY`: API key for cloud providers
/// - `OPENAI_API_KEY_ENV`: Environment variable name for API key (default: OPENAI_API_KEY)
/// - `EMBEDDING_MODEL`: Override default model for the provider
/// - `EMBEDDING_SIMILARITY_THRESHOLD`: Similarity threshold for matching (default: 0.7)
/// - `EMBEDDING_BATCH_SIZE`: Batch size for embedding operations (default: 32)
pub fn load_embedding_config() -> EmbeddingEnvConfig {
    let provider = std::env::var("EMBEDDING_PROVIDER")
        .unwrap_or_else(|_| "local".to_string())
        .to_lowercase();

    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let api_key_env =
        std::env::var("OPENAI_API_KEY_ENV").unwrap_or_else(|_| "OPENAI_API_KEY".to_string());

    let model = std::env::var("EMBEDDING_MODEL")
        .ok()
        .filter(|m| !m.is_empty());

    let similarity_threshold: f32 = std::env::var("EMBEDDING_SIMILARITY_THRESHOLD")
        .unwrap_or_else(|_| "0.7".to_string())
        .parse()
        .unwrap_or(0.7);

    let batch_size: usize = std::env::var("EMBEDDING_BATCH_SIZE")
        .unwrap_or_else(|_| "32".to_string())
        .parse()
        .unwrap_or(32);

    EmbeddingEnvConfig {
        provider,
        api_key,
        api_key_env,
        model,
        similarity_threshold,
        batch_size,
    }
}

/// Handle embedding/config - get or update embedding configuration
///
/// This handler returns the current embedding configuration from environment variables.
/// The configuration includes provider, model, dimensions, and other settings.
pub async fn handle_embedding_config(
    request: JsonRpcRequest,
    embedding_config: &EmbeddingEnvConfig,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling embedding/config");

    // Return current embedding configuration
    #[allow(clippy::wildcard_in_or_patterns)]
    let (model, dimension) = match embedding_config.provider.as_str() {
        "openai" => (
            embedding_config
                .model
                .clone()
                .unwrap_or_else(|| "text-embedding-3-small".to_string()),
            1536,
        ),
        "mistral" => (
            embedding_config
                .model
                .clone()
                .unwrap_or_else(|| "mistral-embed".to_string()),
            1024,
        ),
        "azure" => (
            embedding_config
                .model
                .clone()
                .unwrap_or_else(|| "text-embedding-ada-002".to_string()),
            1536,
        ),
        "cohere" => (
            embedding_config
                .model
                .clone()
                .unwrap_or_else(|| "embed-english-v3.0".to_string()),
            1024,
        ),
        "local" | _ => (
            embedding_config
                .model
                .clone()
                .unwrap_or_else(|| "all-MiniLM-L6-v2".to_string()),
            384,
        ),
    };

    let result = json!({
        "success": true,
        "provider": embedding_config.provider,
        "model": model,
        "dimension": dimension,
        "similarity_threshold": embedding_config.similarity_threshold,
        "batch_size": embedding_config.batch_size,
        "message": format!("Embedding provider: {} (model: {}, dim: {})",
            embedding_config.provider, model, dimension),
        "env_config": true
    });

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(result),
        error: None,
    })
}
