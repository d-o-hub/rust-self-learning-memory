//! External signal provider infrastructure
//!
//! This module provides traits and types for integrating external signal providers
//! (AgentFS, audit trails, etc.) into the reward system.
//!
//! ## Architecture
//!
//! ```text
//! ExternalSignalProvider (trait)
//!     │
//!     ├─ AgentFsProvider (AgentFS SDK integration)
//!     ├─ MockProvider (test fixtures)
//!     └─ FileProvider (local JSON/CSV)
//!     │
//!     ▼
//! ExternalSignalSet (normalized format)
//!     │
//!     ▼
//! SignalMerger (weighted combination)
//!     │
//!     ▼
//! MergedReward → RewardScore
//! ```
//!
//! ## Feature Flags
//!
//! - `agentfs`: Enables AgentFS SDK integration

mod merger;
mod provider;
mod registry;
mod types;

#[cfg(feature = "agentfs")]
mod agentfs;

pub use provider::{ExternalSignalProvider, ProviderHealth};

pub use merger::{ConflictResolution, MergedReward, SignalMerger};
#[cfg(test)]
pub use provider::mock::MockExternalSignalProvider;
pub use registry::ExternalSignalRegistry;
pub use types::{ExternalSignalConfig, ExternalSignalSet, ToolSignal};

#[cfg(feature = "agentfs")]
pub use agentfs::{AgentFsConfig, AgentFsProvider};

/// Result type for external signal operations
pub type Result<T> = std::result::Result<T, ExternalSignalError>;

/// Errors that can occur when working with external signal providers
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExternalSignalError {
    #[error("external provider error: {0}")]
    Provider(String),

    #[error("configuration missing: {0}")]
    ConfigMissing(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("circuit breaker open")]
    CircuitOpen,

    #[error("provider unhealthy: {0}")]
    Unhealthy(String),

    #[error("SDK not available for provider '{0}' - stub implementation returns no real data")]
    SdkUnavailable(String),
}
