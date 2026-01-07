//! Enumeration types for configuration

use serde::{Deserialize, Serialize};

/// Database type for simple configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatabaseType {
    /// Local SQLite database via Turso
    Local,
    /// Cloud database via Turso
    Cloud,
    /// In-memory only (temporary storage)
    Memory,
}

/// Performance level for simple configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PerformanceLevel {
    /// Minimal resources: < 100MB memory, < 100 episodes
    Minimal,
    /// Standard resources: < 1GB memory, < 1000 episodes
    Standard,
    /// High resources: < 4GB memory, < 10000 episodes
    High,
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("Simple mode error: {message}")]
    SimpleMode { message: String },
    #[error("Configuration validation error: {message}")]
    Validation { message: String },
    #[error("Environment detection error: {message}")]
    EnvironmentDetection { message: String },
    #[error("Storage initialization error: {message}")]
    StorageInit { message: String },
}
