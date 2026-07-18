//! Audit logging module
//!
//! This module provides comprehensive audit logging capabilities for the MCP server,
//! enabling security incident investigation through structured, immutable audit logs.
//!
//! ## Features
//!
//! - Structured JSON logging format
//! - Configurable log destinations (file, stdout, or both)
//! - Log rotation with size tracking initialized from existing files
//! - Recursive sensitive-data redaction (nested objects/arrays, case-insensitive)
//! - Non-blocking bounded file writer with overflow drop metrics
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
//! use do_memory_mcp::server::audit::{AuditLogger, AuditConfig, AuditDestination};
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
pub mod redaction;
pub mod relationship_ops;
pub mod security_ops;
pub mod tag_ops;
pub mod types;
pub mod writer;

pub use core::AuditLogger;
pub use redaction::redact_sensitive_data;
pub use types::{AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel};
pub use writer::{AuditFileWriter, DEFAULT_AUDIT_WRITE_QUEUE_CAPACITY, WriterConfig};
