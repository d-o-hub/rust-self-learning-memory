//! OpenAI-specific embedding configuration and types

pub use config::{EncodingFormat, OpenAIConfig, OpenAIModel};
pub use types::{
    OpenAIEmbeddingData, OpenAIEmbeddingInput, OpenAIEmbeddingRequest, OpenAIEmbeddingResponse,
};

mod config;
mod types;
