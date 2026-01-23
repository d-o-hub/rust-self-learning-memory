//! Configuration for embedding providers

pub use self::{
    embedding_config::EmbeddingConfig, model_config::ModelConfig,
    optimization_config::OptimizationConfig, provider_enum::EmbeddingProvider,
};

mod embedding_config;
mod model_config;
mod optimization_config;
mod provider_enum;
