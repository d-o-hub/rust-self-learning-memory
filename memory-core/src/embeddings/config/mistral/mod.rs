//! Mistral-specific embedding configuration and types

pub use config::{MistralConfig, MistralModel, OutputDtype};
pub use types::{
    MistralEmbeddingData, MistralEmbeddingInput, MistralEmbeddingRequest, MistralEmbeddingResponse,
    MistralUsage,
};

mod config;
mod types;
