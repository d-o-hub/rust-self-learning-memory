//! Mistral-specific embedding configuration and types

pub use config::{MistralConfig, MistralModel, OutputDtype};
// Note: Request/Response types are used internally, not re-exported

mod config;
mod types;
