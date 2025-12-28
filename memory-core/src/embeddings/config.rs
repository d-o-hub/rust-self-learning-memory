//! Configuration for semantic embedding providers

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
    pub circuit_breaker_config: Option<super::circuit_breaker::CircuitBreakerConfig>,
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
    1024 // 1KB
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
            enable_circuit_breaker: true,  // Enabled by default for production safety
            circuit_breaker_config: Some(super::circuit_breaker::CircuitBreakerConfig::default()),
            enable_metrics: default_enable_metrics(),
        }
    }
}

impl OptimizationConfig {
    /// Optimized configuration for `OpenAI`
    #[must_use]
    pub fn openai() -> Self {
        Self {
            timeout_seconds: Some(60),
            max_retries: 3,
            retry_delay_ms: 1000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(3000),
            rate_limit_tpm: Some(1_000_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 20,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 2,
                timeout_seconds: 30,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Mistral AI
    #[must_use]
    pub fn mistral() -> Self {
        Self {
            timeout_seconds: Some(30),
            max_retries: 3,
            retry_delay_ms: 500,
            max_batch_size: Some(128),
            rate_limit_rpm: Some(100),
            rate_limit_tpm: Some(100_000),
            compression_enabled: true,
            compression_threshold_bytes: 512,
            connection_pool_size: 10,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                timeout_seconds: 20,
                half_open_max_attempts: 2,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Azure `OpenAI`
    #[must_use]
    pub fn azure() -> Self {
        Self {
            timeout_seconds: Some(90),
            max_retries: 4,
            retry_delay_ms: 2000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(300), // Conservative default
            rate_limit_tpm: Some(300_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 15,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_seconds: 60,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for local/custom providers
    #[must_use]
    pub fn local() -> Self {
        Self {
            timeout_seconds: Some(10),
            max_retries: 2,
            retry_delay_ms: 100,
            max_batch_size: Some(32),
            rate_limit_rpm: None, // No rate limiting for local
            rate_limit_tpm: None,
            compression_enabled: false,
            compression_threshold_bytes: 2048,
            connection_pool_size: 5,
            enable_circuit_breaker: false, // Less useful for local providers
            circuit_breaker_config: None,
            enable_metrics: true,
        }
    }

    /// Get effective timeout (returns default if not specified)
    #[must_use]
    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds.unwrap_or(60)
    }

    /// Get effective batch size (returns default if not specified)
    #[must_use]
    pub fn get_max_batch_size(&self) -> usize {
        self.max_batch_size.unwrap_or(100)
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
    ///
    /// # Arguments
    /// * `deployment_name` - Your Azure deployment name
    /// * `resource_name` - Your Azure resource name
    /// * `api_version` - API version (e.g., \"2023-05-15\")
    /// * `dimension` - Embedding dimension for your model
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
    ///
    /// # Arguments
    /// * `model_name` - Model identifier
    /// * `dimension` - Embedding dimension
    /// * `base_url` - Base URL for the API (e.g., `<https://api.example.com/v1>`)
    /// * `endpoint` - Optional custom endpoint path (defaults to \"/embeddings\")
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
            optimization: OptimizationConfig::local(), // Default to local optimizations for custom
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_embedding_config() {
        let config = EmbeddingConfig::default();

        // Verify default provider is Local
        assert_eq!(config.provider, EmbeddingProvider::Local);

        // Verify default model configuration
        assert_eq!(
            config.model.model_name,
            "sentence-transformers/all-MiniLM-L6-v2"
        );
        assert_eq!(config.model.embedding_dimension, 384);

        // Verify default threshold and batch size
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.batch_size, 32);

        // Verify cache enabled and timeout
        assert!(config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_embedding_provider_variants() {
        // Test Local provider
        let local = EmbeddingProvider::Local;
        assert_eq!(local, EmbeddingProvider::Local);

        // Test OpenAI provider
        let openai = EmbeddingProvider::OpenAI;
        assert_eq!(openai, EmbeddingProvider::OpenAI);

        // Test Mistral provider
        let mistral = EmbeddingProvider::Mistral;
        assert_eq!(mistral, EmbeddingProvider::Mistral);

        // Test Azure OpenAI provider
        let azure = EmbeddingProvider::AzureOpenAI;
        assert_eq!(azure, EmbeddingProvider::AzureOpenAI);

        // Test Custom provider
        let custom = EmbeddingProvider::Custom("custom-provider".to_string());
        assert_eq!(
            custom,
            EmbeddingProvider::Custom("custom-provider".to_string())
        );

        // Test equality/inequality
        assert_ne!(local, openai);
        assert_ne!(openai, mistral);
    }

    #[test]
    fn test_model_config_constructors() {
        // Test local_sentence_transformer
        let local_config = ModelConfig::local_sentence_transformer("test-model", 512);
        assert_eq!(local_config.model_name, "test-model");
        assert_eq!(local_config.embedding_dimension, 512);
        assert!(local_config.base_url.is_none());
        assert!(local_config.api_endpoint.is_none());

        // Test openai_3_small
        let openai_small = ModelConfig::openai_3_small();
        assert_eq!(openai_small.model_name, "text-embedding-3-small");
        assert_eq!(openai_small.embedding_dimension, 1536);
        assert_eq!(
            openai_small.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );

        // Test openai_3_large
        let openai_large = ModelConfig::openai_3_large();
        assert_eq!(openai_large.model_name, "text-embedding-3-large");
        assert_eq!(openai_large.embedding_dimension, 3072);

        // Test openai_ada_002
        let openai_ada = ModelConfig::openai_ada_002();
        assert_eq!(openai_ada.model_name, "text-embedding-ada-002");
        assert_eq!(openai_ada.embedding_dimension, 1536);

        // Test mistral_embed
        let mistral = ModelConfig::mistral_embed();
        assert_eq!(mistral.model_name, "mistral-embed");
        assert_eq!(mistral.embedding_dimension, 1024);
        assert_eq!(
            mistral.base_url.as_deref(),
            Some("https://api.mistral.ai/v1")
        );

        // Test azure_openai
        let azure = ModelConfig::azure_openai("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(azure.model_name, "my-deployment");
        assert_eq!(azure.embedding_dimension, 1536);
        assert_eq!(
            azure.base_url.as_deref(),
            Some("https://my-resource.openai.azure.com")
        );
        let endpoint = azure.api_endpoint.as_ref().unwrap();
        assert!(endpoint.contains("my-deployment"));
        assert!(endpoint.contains("2023-05-15"));

        // Test custom
        let custom = ModelConfig::custom(
            "custom-model",
            256,
            "https://api.example.com/v1",
            Some("/custom-embeddings"),
        );
        assert_eq!(custom.model_name, "custom-model");
        assert_eq!(custom.embedding_dimension, 256);
        assert_eq!(
            custom.base_url.as_deref(),
            Some("https://api.example.com/v1")
        );
        assert_eq!(custom.api_endpoint.as_deref(), Some("/custom-embeddings"));
    }

    #[test]
    fn test_optimization_config_profiles() {
        // Test OpenAI profile
        let openai_profile = OptimizationConfig::openai();
        assert_eq!(openai_profile.timeout_seconds, Some(60));
        assert_eq!(openai_profile.max_retries, 3);
        assert_eq!(openai_profile.max_batch_size, Some(2048));
        assert_eq!(openai_profile.rate_limit_rpm, Some(3000));
        assert!(openai_profile.compression_enabled);
        assert_eq!(openai_profile.connection_pool_size, 20);
        assert!(openai_profile.enable_circuit_breaker);
        assert!(openai_profile.circuit_breaker_config.is_some());
        assert!(openai_profile.enable_metrics);

        // Test Mistral profile
        let mistral_profile = OptimizationConfig::mistral();
        assert_eq!(mistral_profile.timeout_seconds, Some(30));
        assert_eq!(mistral_profile.max_retries, 3);
        assert_eq!(mistral_profile.max_batch_size, Some(128));
        assert_eq!(mistral_profile.retry_delay_ms, 500);
        assert_eq!(mistral_profile.rate_limit_rpm, Some(100));

        // Test Azure profile
        let azure_profile = OptimizationConfig::azure();
        assert_eq!(azure_profile.timeout_seconds, Some(90));
        assert_eq!(azure_profile.max_retries, 4);
        assert_eq!(azure_profile.retry_delay_ms, 2000);
        assert_eq!(azure_profile.rate_limit_rpm, Some(300));

        // Test local profile
        let local_profile = OptimizationConfig::local();
        assert_eq!(local_profile.timeout_seconds, Some(10));
        assert_eq!(local_profile.max_retries, 2);
        assert_eq!(local_profile.max_batch_size, Some(32));
        assert!(!local_profile.compression_enabled);
        assert!(!local_profile.enable_circuit_breaker);
        assert!(local_profile.circuit_breaker_config.is_none());
        assert_eq!(local_profile.connection_pool_size, 5);
        assert!(local_profile.enable_metrics);
    }

    #[test]
    fn test_optimization_config_helper_methods() {
        // Test get_timeout_seconds with None (should return default)
        let mut config = OptimizationConfig::default();
        assert_eq!(config.timeout_seconds, None);
        assert_eq!(config.get_timeout_seconds(), 60); // Default is 60

        // Test get_timeout_seconds with Some value
        config.timeout_seconds = Some(120);
        assert_eq!(config.get_timeout_seconds(), 120);

        // Test get_max_batch_size with None (should return default)
        config.max_batch_size = None;
        assert_eq!(config.get_max_batch_size(), 100); // Default is 100

        // Test get_max_batch_size with Some value
        config.max_batch_size = Some(256);
        assert_eq!(config.get_max_batch_size(), 256);

        // Test OpenAI profile helpers
        let openai_config = OptimizationConfig::openai();
        assert_eq!(openai_config.get_timeout_seconds(), 60);
        assert_eq!(openai_config.get_max_batch_size(), 2048);

        // Test local profile helpers
        let local_config = OptimizationConfig::local();
        assert_eq!(local_config.get_timeout_seconds(), 10);
        assert_eq!(local_config.get_max_batch_size(), 32);
    }

    #[test]
    fn test_model_config_get_embeddings_url() {
        // Test default (no base_url, no api_endpoint)
        let config = ModelConfig::default();
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://api.openai.com/v1/embeddings");

        // Test with base_url only
        let config = ModelConfig {
            base_url: Some("https://custom.api.com/v1".to_string()),
            api_endpoint: None,
            ..Default::default()
        };
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://custom.api.com/v1/embeddings");

        // Test with both base_url and api_endpoint (with leading slash)
        let config = ModelConfig {
            base_url: Some("https://custom.api.com/v1".to_string()),
            api_endpoint: Some("/custom-path".to_string()),
            ..Default::default()
        };
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://custom.api.com/v1/custom-path");

        // Test with both base_url and api_endpoint (without leading slash)
        let config = ModelConfig {
            base_url: Some("https://custom.api.com/v1".to_string()),
            api_endpoint: Some("custom-path".to_string()),
            ..Default::default()
        };
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://custom.api.com/v1/custom-path");
    }

    #[test]
    fn test_optimization_config_serialization() {
        // Test that default config serializes/deserializes correctly
        let config = OptimizationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_retries, config.max_retries);
        assert_eq!(deserialized.retry_delay_ms, config.retry_delay_ms);
        assert_eq!(deserialized.compression_enabled, config.compression_enabled);

        // Test that openai profile serializes/deserializes correctly
        let openai_config = OptimizationConfig::openai();
        let json = serde_json::to_string(&openai_config).unwrap();
        let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timeout_seconds, openai_config.timeout_seconds);
        assert_eq!(deserialized.max_batch_size, openai_config.max_batch_size);
        assert_eq!(
            deserialized.enable_circuit_breaker,
            openai_config.enable_circuit_breaker
        );
    }
}
