//! Audit logging module
//!
//! This module provides comprehensive audit logging capabilities for the MCP server,
//! enabling security incident investigation through structured, immutable audit logs.
//!
//! ## Features
//!
//! - Structured JSON logging format
//! - Configurable log destinations (file, stdout, or both)
//! - Log rotation support
//! - Sensitive data redaction
//! - All security-relevant operations tracked
//!
//! ## Security Operations Logged
//!
//! - Episode creation/modification/deletion
//! - Relationship changes
//! - Configuration changes
//! - Authentication events
//! - Rate limit violations
//!
//! ## Example
//!
//! ```rust
//! use memory_mcp::server::audit::{AuditLogger, AuditConfig, AuditDestination};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = AuditConfig::from_env();
//!     let logger = AuditLogger::new(config).await?;
//!
//!     logger.log_episode_creation(
//!         "client-123",
//!         "episode-uuid",
//!         "task description",
//!         true,
//!         None
//!     ).await;
//!
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod episode_ops;
pub mod pattern_ops;
pub mod query_ops;
pub mod relationship_ops;
pub mod security_ops;
pub mod tag_ops;
pub mod types;

pub use core::AuditLogger;
pub use types::{AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel};
