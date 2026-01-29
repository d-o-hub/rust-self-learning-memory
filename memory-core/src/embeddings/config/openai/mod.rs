//! OpenAI-specific embedding configuration and types

pub use config::{EncodingFormat, OpenAIConfig, OpenAIModel};
// Note: Request/Response types are used internally, not re-exported

mod config;
mod types;
