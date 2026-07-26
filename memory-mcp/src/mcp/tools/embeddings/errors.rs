//! Activation error types for embedding provider activation.
//!
//! These variants are used by the embedding activation pipeline (REA-2026-07-26)
//! to surface specific failure modes that callers can inspect and handle.

use thiserror::Error;

/// Errors that can occur during embedding provider activation.
#[derive(Debug, Error)]
pub enum ActivationError {
    /// The caller supplied an invalid configuration value.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// The requested provider is unavailable in this build or environment
    /// (e.g., feature flag not enabled, runtime library missing).
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    /// A required credential (API key, token, etc.) is absent.
    #[error("Missing credential: {0}")]
    MissingCredential(String),

    /// A live connectivity probe to the provider endpoint failed.
    #[error("Probe failed: {0}")]
    ProbeFailed(String),

    /// The embedding dimension of the new provider does not match what is
    /// already stored in the vector index.
    #[error("Dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch {
        /// Dimension already committed to storage.
        expected: usize,
        /// Dimension reported by the new provider.
        actual: usize,
    },

    /// Preparing or migrating the vector-storage layer failed.
    #[error("Storage setup failed: {0}")]
    StorageSetupFailed(String),
}
