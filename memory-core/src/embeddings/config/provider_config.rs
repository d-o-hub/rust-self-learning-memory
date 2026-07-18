//! Unified provider configuration enum

use serde::{Deserialize, Serialize};

use super::{OptimizationConfig, mistral::MistralConfig, openai::OpenAIConfig};

#[allow(unused_imports)]
use super::mistral::OutputDtype;

/// Provider-specific configuration
///
/// This enum wraps all provider-specific configurations, allowing type-safe
/// access to provider-specific features while maintaining a unified interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    /// Local embedding configuration
    #[serde(rename = "local")]
    Local(LocalConfig),
    /// OpenAI embedding configuration
    #[serde(rename = "openai")]
    OpenAI(OpenAIConfig),
    /// Mistral AI embedding configuration
    #[serde(rename = "mistral")]
    Mistral(MistralConfig),
    /// Azure OpenAI configuration
    #[serde(rename = "azure_openai")]
    AzureOpenAI(AzureOpenAIConfig),
    /// Custom provider configuration
    #[serde(rename = "custom")]
    Custom(CustomConfig),
}

/// Configuration for local embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Model name/path
    pub model_name: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
    /// When true, allow mock embeddings for tests/dev (S1.5).
    ///
    /// Production defaults to `false`: if the real model cannot load, the
    /// provider reports degraded-mock / unavailable rather than silently
    /// advertising mock vectors as healthy (see `EmbeddingHealth`).
    #[serde(default)]
    pub allow_mock_fallback: bool,
    /// Optional model revision / pin label (S1.5b / F4.3).
    #[serde(default)]
    pub model_revision: Option<String>,
    /// Optional expected SHA-256 hex digest of the primary model artifact (S1.5b).
    #[serde(default)]
    pub expected_sha256: Option<String>,
    /// Optional maximum allowed artifact size in bytes (S1.5b).
    #[serde(default)]
    pub max_artifact_bytes: Option<u64>,
}

impl LocalConfig {
    /// Create a new local config.
    ///
    /// Mock fallback defaults to **enabled** so offline tests/dev keep working.
    /// Call [`Self::with_allow_mock_fallback`]`(false)` for production fail-closed
    /// loading (S1.5). Mock mode never reports production-ready via `is_available`.
    #[must_use]
    pub fn new(model_name: impl Into<String>, dimension: usize) -> Self {
        Self {
            model_name: model_name.into(),
            embedding_dimension: dimension,
            optimization: OptimizationConfig::local(),
            allow_mock_fallback: true,
            model_revision: None,
            expected_sha256: None,
            max_artifact_bytes: None,
        }
    }

    /// Enable mock embeddings for tests and local development (S1.5).
    #[must_use]
    pub fn with_allow_mock_fallback(mut self, allow: bool) -> Self {
        self.allow_mock_fallback = allow;
        self
    }

    /// Pin model revision string (S1.5b).
    #[must_use]
    pub fn with_model_revision(mut self, revision: impl Into<String>) -> Self {
        self.model_revision = Some(revision.into());
        self
    }

    /// Require SHA-256 hex digest of the model file (S1.5b).
    #[must_use]
    pub fn with_expected_sha256(mut self, digest_hex: impl Into<String>) -> Self {
        self.expected_sha256 = Some(digest_hex.into());
        self
    }

    /// Reject artifacts larger than `max_bytes` (S1.5b).
    #[must_use]
    pub fn with_max_artifact_bytes(mut self, max_bytes: u64) -> Self {
        self.max_artifact_bytes = Some(max_bytes);
        self
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

/// Verify a model artifact against optional digest and size limits (S1.5b / F4.3).
///
/// # Errors
/// Returns an error when the file is missing, oversized, or digest mismatches.
pub fn verify_model_artifact(
    path: &std::path::Path,
    expected_sha256: Option<&str>,
    max_bytes: Option<u64>,
) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::Read;

    let meta = fs::metadata(path).map_err(|e| {
        anyhow::anyhow!("model artifact metadata failed for {}: {e}", path.display())
    })?;
    let len = meta.len();
    if let Some(max) = max_bytes {
        if len > max {
            anyhow::bail!(
                "model artifact {} is {len} bytes, exceeds max_artifact_bytes={max}",
                path.display()
            );
        }
    }
    if let Some(expected) = expected_sha256 {
        let expected = expected.trim().to_ascii_lowercase();
        let mut file = fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("open model artifact {}: {e}", path.display()))?;
        let mut hasher = Sha256::new();
        let mut buf = [0_u8; 8192];
        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| anyhow::anyhow!("read model artifact {}: {e}", path.display()))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let actual = format!("{:x}", hasher.finalize());
        if actual != expected {
            anyhow::bail!(
                "model artifact {} digest mismatch: expected {expected}, got {actual}",
                path.display()
            );
        }
    }
    Ok(())
}

impl Default for LocalConfig {
    fn default() -> Self {
        // Offline/dev default: mock allowed. Production should set
        // `allow_mock_fallback: false` and treat only `EmbeddingHealth::Real`
        // as production-ready.
        Self::new("sentence-transformers/all-MiniLM-L6-v2", 384)
    }
}

/// Configuration for Azure OpenAI Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAIConfig {
    /// Deployment name
    pub deployment_name: String,
    /// Resource name
    pub resource_name: String,
    /// API version
    pub api_version: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl AzureOpenAIConfig {
    /// Create a new Azure OpenAI config
    #[must_use]
    pub fn new(
        deployment_name: impl Into<String>,
        resource_name: impl Into<String>,
        api_version: impl Into<String>,
        dimension: usize,
    ) -> Self {
        Self {
            deployment_name: deployment_name.into(),
            resource_name: resource_name.into(),
            api_version: api_version.into(),
            embedding_dimension: dimension,
            optimization: OptimizationConfig::azure(),
        }
    }

    /// Get the Azure OpenAI endpoint URL
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        format!(
            "https://{}.openai.azure.com/openai/deployments/{}/embeddings?api-version={}",
            self.resource_name, self.deployment_name, self.api_version
        )
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

/// Configuration for custom embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    /// Model identifier
    pub model_name: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Base URL for the API
    pub base_url: String,
    /// Custom endpoint path (optional)
    #[serde(default)]
    pub api_endpoint: Option<String>,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl CustomConfig {
    /// Create a new custom config
    #[must_use]
    pub fn new(
        model_name: impl Into<String>,
        dimension: usize,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            model_name: model_name.into(),
            embedding_dimension: dimension,
            base_url: base_url.into(),
            api_endpoint: None,
            optimization: OptimizationConfig::local(),
        }
    }

    /// Set custom API endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.api_endpoint = Some(endpoint.into());
        self
    }

    /// Get the full embeddings endpoint URL
    #[must_use]
    pub fn embeddings_url(&self) -> String {
        let endpoint = self.api_endpoint.as_deref().unwrap_or("/embeddings");
        let base = self.base_url.trim_end_matches('/');

        if endpoint.starts_with('/') {
            format!("{base}{endpoint}")
        } else {
            format!("{base}/{endpoint}")
        }
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

impl ProviderConfig {
    /// Get the effective embedding dimension for this provider
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        match self {
            Self::Local(config) => config.effective_dimension(),
            Self::OpenAI(config) => config.effective_dimension(),
            Self::Mistral(config) => config.effective_dimension(),
            Self::AzureOpenAI(config) => config.effective_dimension(),
            Self::Custom(config) => config.effective_dimension(),
        }
    }

    /// Get the optimization configuration
    #[must_use]
    pub fn optimization(&self) -> &OptimizationConfig {
        match self {
            Self::Local(config) => &config.optimization,
            Self::OpenAI(config) => &config.optimization,
            Self::Mistral(config) => &config.optimization,
            Self::AzureOpenAI(config) => &config.optimization,
            Self::Custom(config) => &config.optimization,
        }
    }

    /// Get the model name
    #[must_use]
    pub fn model_name(&self) -> String {
        match self {
            Self::Local(config) => config.model_name.clone(),
            Self::OpenAI(config) => config.model.model_name().to_string(),
            Self::Mistral(config) => config.model.model_name().to_string(),
            Self::AzureOpenAI(config) => config.deployment_name.clone(),
            Self::Custom(config) => config.model_name.clone(),
        }
    }

    /// Stable cache/provider identity string (`kind:model:dims`) for ADR-074.
    #[must_use]
    pub fn cache_identity(&self) -> String {
        let kind = match self {
            Self::Local(_) => "local",
            Self::OpenAI(_) => "openai",
            Self::Mistral(_) => "mistral",
            Self::AzureOpenAI(_) => "azure_openai",
            Self::Custom(_) => "custom",
        };
        format!(
            "{}:{}:{}",
            kind,
            self.model_name().trim(),
            self.effective_dimension()
        )
    }

    /// Validate the configuration
    ///
    /// # Errors
    /// Returns error if any provider-specific validation fails
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::OpenAI(config) => config.validate(),
            Self::Mistral(config) => config.validate(),
            _ => Ok(()),
        }
    }

    /// Create a default OpenAI configuration
    #[must_use]
    pub fn openai_default() -> Self {
        Self::OpenAI(OpenAIConfig::default())
    }

    /// Create a default Mistral configuration
    #[must_use]
    pub fn mistral_default() -> Self {
        Self::Mistral(MistralConfig::default())
    }

    /// Create a default local configuration
    #[must_use]
    pub fn local_default() -> Self {
        Self::Local(LocalConfig::default())
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::local_default()
    }
}

// Convenience constructors for common configurations
impl ProviderConfig {
    /// OpenAI text-embedding-3-small
    #[must_use]
    pub fn openai_3_small() -> Self {
        Self::OpenAI(OpenAIConfig::text_embedding_3_small())
    }

    /// OpenAI text-embedding-3-large
    #[must_use]
    pub fn openai_3_large() -> Self {
        Self::OpenAI(OpenAIConfig::text_embedding_3_large())
    }

    /// OpenAI text-embedding-ada-002
    #[must_use]
    pub fn openai_ada_002() -> Self {
        Self::OpenAI(OpenAIConfig::ada_002())
    }

    /// Mistral mistral-embed
    #[must_use]
    pub fn mistral_embed() -> Self {
        Self::Mistral(MistralConfig::mistral_embed())
    }

    /// Mistral codestral-embed
    #[must_use]
    pub fn codestral_embed() -> Self {
        Self::Mistral(MistralConfig::codestral_embed())
    }

    /// Mistral codestral-embed with binary output
    #[must_use]
    pub fn codestral_binary() -> Self {
        Self::Mistral(MistralConfig::codestral_binary())
    }

    /// Local sentence transformer
    #[must_use]
    pub fn local_sentence_transformer(model_name: impl Into<String>, dimension: usize) -> Self {
        Self::Local(LocalConfig::new(model_name, dimension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_dimensions() {
        let openai = ProviderConfig::openai_3_small();
        assert_eq!(openai.effective_dimension(), 1536);

        let mistral = ProviderConfig::mistral_embed();
        assert_eq!(mistral.effective_dimension(), 1024);

        let codestral = ProviderConfig::codestral_embed();
        assert_eq!(codestral.effective_dimension(), 1536);

        let local = ProviderConfig::local_default();
        assert_eq!(local.effective_dimension(), 384);
    }

    #[test]
    fn test_provider_config_model_names() {
        let openai = ProviderConfig::openai_3_small();
        assert_eq!(openai.model_name(), "text-embedding-3-small");

        let mistral = ProviderConfig::mistral_embed();
        assert_eq!(mistral.model_name(), "mistral-embed");

        let codestral = ProviderConfig::codestral_embed();
        assert_eq!(codestral.model_name(), "codestral-embed");
    }

    #[test]
    fn test_azure_openai_endpoint() {
        let config = AzureOpenAIConfig::new("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(
            config.endpoint_url(),
            "https://my-resource.openai.azure.com/openai/deployments/my-deployment/embeddings?api-version=2023-05-15"
        );
    }

    #[test]
    fn test_custom_config_url() {
        let config = CustomConfig::new("custom-model", 768, "https://api.example.com/v1")
            .with_endpoint("/custom-embeddings");
        assert_eq!(
            config.embeddings_url(),
            "https://api.example.com/v1/custom-embeddings"
        );
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig::openai_3_small();
        let json = serde_json::to_string(&config).unwrap();

        // Verify the tag is present - use either quoted or unquoted format
        assert!(
            json.contains("\"provider\":\"openai\"") || json.contains("\"openai\""),
            "Expected provider tag in JSON, got: {json}"
        );

        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.effective_dimension(),
            deserialized.effective_dimension()
        );
    }

    #[test]
    fn test_mistral_config_serialization() {
        let config = ProviderConfig::Mistral(
            MistralConfig::codestral_embed()
                .with_output_dimension(512)
                .with_output_dtype(OutputDtype::Int8),
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();

        match deserialized {
            ProviderConfig::Mistral(mistral) => {
                assert_eq!(mistral.output_dimension, Some(512));
                assert_eq!(mistral.output_dtype, OutputDtype::Int8);
            }
            _ => panic!("Expected Mistral config"),
        }
    }
}
