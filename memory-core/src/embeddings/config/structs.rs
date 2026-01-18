//! Configuration structs for embedding providers.

use serde::{Deserialize, Serialize};

/// Configuration for the embedding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider type
    pub provider: EmbeddingProvider,
    /// Model configuration
    pub model: ModelConfig,
    /// Similarity threshold for search (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Cache embeddings to avoid regeneration
    pub cache_embeddings: bool,
    /// Timeout for embedding requests (seconds)
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::Local,
            model: ModelConfig::default(),
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_embeddings: true,
            timeout_seconds: 30,
        }
    }
}

/// Supported embedding providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EmbeddingProvider {
    /// Local embedding using sentence transformers
    Local,
    /// `OpenAI`'s text embedding models
    OpenAI,
    /// Mistral AI's embedding models
    Mistral,
    /// Azure `OpenAI` Service
    AzureOpenAI,
    /// Custom provider implementation
    Custom(String),
}

/// Model configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name/identifier
    pub model_name: String,
    /// Expected embedding dimension
    pub embedding_dimension: usize,
    /// Base URL for the API endpoint (e.g., `https://api.openai.com/v1`)
    #[serde(default)]
    pub base_url: Option<String>,
    /// Custom API endpoint path (optional, for non-standard endpoints)
    #[serde(default)]
    pub api_endpoint: Option<String>,
    /// Provider-specific optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
    /// Model-specific parameters
    pub parameters: serde_json::Value,
}

/// Provider-specific optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Request timeout in seconds (None = use default)
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    /// Maximum number of retry attempts for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Base retry delay in milliseconds (exponential backoff)
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
    /// Maximum batch size for this provider (None = use provider default)
    #[serde(default)]
    pub max_batch_size: Option<usize>,
    /// Rate limit: requests per minute (None = no limit)
    #[serde(default)]
    pub rate_limit_rpm: Option<u32>,
    /// Rate limit: tokens per minute (None = no limit)
    #[serde(default)]
    pub rate_limit_tpm: Option<u64>,
    /// Enable request compression (gzip)
    #[serde(default = "default_compression")]
    pub compression_enabled: bool,
    /// Minimum request size (bytes) to trigger compression
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold_bytes: usize,
    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub connection_pool_size: usize,
    /// Enable circuit breaker
    #[serde(default)]
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker_config: Option<super::super::circuit_breaker::CircuitBreakerConfig>,
    /// Enable performance metrics collection
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    1000
}

fn default_compression() -> bool {
    false
}

fn default_pool_size() -> usize {
    10
}

fn default_compression_threshold() -> usize {
    1024
}

fn default_enable_metrics() -> bool {
    true
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: None,
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay_ms(),
            max_batch_size: None,
            rate_limit_rpm: None,
            rate_limit_tpm: None,
            compression_enabled: default_compression(),
            compression_threshold_bytes: default_compression_threshold(),
            connection_pool_size: default_pool_size(),
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(
                super::super::circuit_breaker::CircuitBreakerConfig::default(),
            ),
            enable_metrics: default_enable_metrics(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimension: 384,
            base_url: None,
            api_endpoint: None,
            optimization: OptimizationConfig::default(),
            parameters: serde_json::json!({}),
        }
    }
}

impl ModelConfig {
    /// Create configuration for a local sentence transformer model
    #[must_use]
    pub fn local_sentence_transformer(model_name: &str, dimension: usize) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimension: dimension,
            base_url: None,
            api_endpoint: None,
            optimization: OptimizationConfig::local(),
            parameters: serde_json::json!({
                "normalize": true,
                "pooling": "mean"
            }),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-ada-002
    #[must_use]
    pub fn openai_ada_002() -> Self {
        Self {
            model_name: "text-embedding-ada-002".to_string(),
            embedding_dimension: 1536,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-small
    #[must_use]
    pub fn openai_3_small() -> Self {
        Self {
            model_name: "text-embedding-3-small".to_string(),
            embedding_dimension: 1536,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-large
    #[must_use]
    pub fn openai_3_large() -> Self {
        Self {
            model_name: "text-embedding-3-large".to_string(),
            embedding_dimension: 3072,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for Mistral AI's mistral-embed model
    #[must_use]
    pub fn mistral_embed() -> Self {
        Self {
            model_name: "mistral-embed".to_string(),
            embedding_dimension: 1024,
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::mistral(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for Azure `OpenAI` Service
    #[must_use]
    pub fn azure_openai(
        deployment_name: &str,
        resource_name: &str,
        api_version: &str,
        dimension: usize,
    ) -> Self {
        Self {
            model_name: deployment_name.to_string(),
            embedding_dimension: dimension,
            base_url: Some(format!("https://{resource_name}.openai.azure.com")),
            api_endpoint: Some(format!(
                "/openai/deployments/{deployment_name}/embeddings?api-version={api_version}"
            )),
            optimization: OptimizationConfig::azure(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration with custom base URL and endpoint
    #[must_use]
    pub fn custom(
        model_name: &str,
        dimension: usize,
        base_url: &str,
        endpoint: Option<&str>,
    ) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimension: dimension,
            base_url: Some(base_url.to_string()),
            api_endpoint: endpoint.map(String::from),
            optimization: OptimizationConfig::local(),
            parameters: serde_json::json!({}),
        }
    }

    /// Get the full endpoint URL for embeddings
    #[must_use]
    pub fn get_embeddings_url(&self) -> String {
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = self.api_endpoint.as_deref().unwrap_or("/embeddings");

        if endpoint.starts_with('/') {
            format!("{base}{endpoint}")
        } else {
            format!("{base}/{endpoint}")
        }
    }
}
