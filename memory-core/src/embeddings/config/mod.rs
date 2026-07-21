//! Configuration for embedding providers

// Provider-specific configurations (split for ≤500 LOC)
pub use cloud_config::{AzureOpenAIConfig, CustomConfig};
pub use local_config::{LocalConfig, hex_encode_lower, verify_model_artifact};
pub use provider_config::ProviderConfig;

// Provider enums (unchanged)
pub use provider_enum::EmbeddingProvider;

// Optimization config (unchanged)
pub use optimization_config::OptimizationConfig;

// Top-level embedding config
pub use embedding_config::EmbeddingConfig;

// Provider-specific modules
pub mod mistral;
pub mod openai;

// Internal modules
mod cloud_config;
mod embedding_config;
mod local_config;
mod optimization_config;
mod provider_config;
mod provider_enum;

// Re-export commonly used types for convenience
// Note: These types are part of public API even if not used directly in this module
#[allow(unused_imports)]
pub use mistral::{MistralConfig, MistralModel, OutputDtype};
#[allow(unused_imports)]
pub use openai::{EncodingFormat, OpenAIConfig, OpenAIModel};
// Note: Request/Response types are used internally, not re-exported
