// ============================================================================
// Security Module
// ============================================================================
//!
//! Security-related functionality for the memory system.
//!
//! This module provides security features including:
//! - **Audit logging**: Comprehensive tracking of all operations
//! - **Access control**: Future support for authorization
//! - **Encryption**: Future support for data encryption
//!
//! ## Audit Logging
//!
//! The audit logging system tracks all critical operations for security
//! compliance and incident investigation. See the [`audit`] module for
//! detailed documentation.
//!
//! ## Usage
//!
//! ```rust
//! use memory_core::security::audit::{AuditLogger, AuditConfig, AuditContext};
//!
//! // Create a logger
//! let config = AuditConfig::from_env();
//! let logger = AuditLogger::new(config);
//!
//! // Log operations
//! let context = AuditContext::system();
//! let entry = audit::episode_created(&context, episode_id, "Task", "code_generation");
//! logger.log(entry);
//! ```

pub mod audit;

// Re-export commonly used audit types
pub use audit::{
    access_denied, config_changed, episode_completed, episode_created, episode_deleted,
    relationship_added, relationship_removed, tags_modified, ActorType, AuditConfig, AuditContext,
    AuditEntry, AuditEventType, AuditLogLevel, AuditLogger, AuditOutput, AuditResult,
};
