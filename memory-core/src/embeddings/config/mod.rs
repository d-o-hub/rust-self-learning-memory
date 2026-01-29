//! Configuration for embedding providers

// Provider-specific configurations
pub use provider_config::{AzureOpenAIConfig, CustomConfig, LocalConfig, ProviderConfig};

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
mod embedding_config;
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
