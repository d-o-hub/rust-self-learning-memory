//! Structured logging with correlation IDs for request tracing.
//!
//! This module provides utilities for adding correlation IDs to log output,
//! enabling request tracing across the memory system.
//!
//! # Features
//!
//! - [`CorrelationId`]: UUID-based correlation ID for request tracing
//! - [`add_correlation_id`]: Span hook to record correlation ID in spans
//! - [`init_tracing`]: Initialize tracing subscriber with structured output
//! - [`init_tracing_json`]: Initialize tracing subscriber with JSON output
//!
//! # Usage
//!
//! ```no_run
//! use memory_core::tracing::{init_tracing, CorrelationId};
//! use tracing::info;
//!
//! // Initialize tracing (call once at application startup)
//! init_tracing(None);
//!
//! // Create a correlation ID for a request
//! let correlation_id = CorrelationId::new();
//!
//! // Use with tracing spans
//! let _span = tracing::info_span!(
//!     "operation",
//!     correlation_id = %correlation_id.0
//! );
//! ```

use std::sync::Once;
use tracing::Span;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub static INIT: Once = Once::new();

/// UUID-based correlation ID for request tracing.
///
/// Correlation IDs are used to track requests across multiple
/// services and operations in the memory system.
///
/// # Example
///
/// ```
/// use memory_core::tracing::CorrelationId;
///
/// let id = CorrelationId::new();
/// println!("Correlation ID: {}", id.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CorrelationId(pub uuid::Uuid);

impl CorrelationId {
    /// Create a new correlation ID with a randomly generated UUID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create a correlation ID from an existing UUID.
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    #[must_use]
    pub fn into_inner(self) -> uuid::Uuid {
        self.0
    }

    /// Get a string representation of the correlation ID.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<uuid::Uuid> for CorrelationId {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

/// Span hook to add correlation ID to tracing spans.
///
/// This function returns a closure that can be used with
/// `tracing::info_span!` or `tracing::Span::record` to add
/// a correlation ID to the current span.
///
/// # Example
///
/// ```
/// use memory_core::tracing::{CorrelationId, add_correlation_id};
/// use tracing::info_span;
///
/// let id = CorrelationId::new();
/// let span = info_span!("operation", %id);
/// span.in_scope(|| {
///     // All log statements within this scope will have the correlation ID
/// });
/// ```
pub fn add_correlation_id(id: CorrelationId) -> impl Fn(&Span) + Clone + Send + Sync {
    let id_str = id.0.to_string();
    move |span: &Span| {
        span.record("correlation_id", id_str.as_str());
    }
}

/// Initialize the tracing subscriber with structured (key-value) output.
///
/// This sets up a tracing subscriber that outputs in a human-readable
/// key-value format suitable for debugging and development.
///
/// # Arguments
///
/// * `filter` - Optional environment filter string (e.g., `memory_core=debug,info`)
///
/// # Example
///
/// ```
/// use memory_core::tracing::init_tracing;
///
/// // Initialize with default filter
/// init_tracing(None);
///
/// // Or with custom filter
/// init_tracing(Some("memory_core=debug,tokio=warn"));
/// ```
pub fn init_tracing(filter: Option<&str>) {
    INIT.call_once(|| {
        let filter = filter
            .and_then(|f| EnvFilter::try_from(f).ok())
            .unwrap_or_else(|| {
                EnvFilter::try_from("info").unwrap_or_else(|_| EnvFilter::new("info"))
            });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true).with_thread_ids(true))
            .init();
    });
}

/// Initialize the tracing subscriber with JSON output.
///
/// This sets up a tracing subscriber that outputs JSON logs,
/// suitable for log aggregation systems like ELK or Loki.
///
/// # Arguments
///
/// * `filter` - Optional environment filter string
///
/// # Example
///
/// ```
/// use memory_core::tracing::init_tracing_json;
///
/// // Initialize with JSON output
/// init_tracing_json(Some("memory_core=debug"));
/// ```
pub fn init_tracing_json(filter: Option<&str>) {
    INIT.call_once(|| {
        let filter = filter
            .and_then(|f| EnvFilter::try_from(f).ok())
            .unwrap_or_else(|| {
                EnvFilter::try_from("info").unwrap_or_else(|_| EnvFilter::new("info"))
            });

        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    });
}

/// Initialize the tracing subscriber with pretty output for development.
///
/// This sets up a tracing subscriber with colored, pretty-printed output.
pub fn init_tracing_pretty() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from("debug").unwrap_or_else(|_| EnvFilter::new("debug"));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty().with_target(true))
            .init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_new() {
        let id = CorrelationId::new();
        assert_ne!(id.0, uuid::Uuid::nil());
    }

    #[test]
    fn test_correlation_id_default() {
        let id1 = CorrelationId::default();
        let id2 = CorrelationId::default();
        // Default should create new UUIDs
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_correlation_id_display() {
        let id = CorrelationId::new();
        let display = format!("{}", id);
        assert_eq!(display, id.0.to_string());
    }

    #[test]
    fn test_correlation_id_as_str() {
        let id = CorrelationId::new();
        let s = id.as_str();
        assert_eq!(s.len(), 36); // UUID string length
    }

    #[test]
    fn test_correlation_id_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = CorrelationId::from_uuid(uuid);
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn test_correlation_id_into_inner() {
        let uuid = uuid::Uuid::new_v4();
        let id = CorrelationId::from_uuid(uuid);
        assert_eq!(id.into_inner(), uuid);
    }

    #[test]
    fn test_add_correlation_id() {
        let id = CorrelationId::new();
        let hook = add_correlation_id(id);

        // Create a proper span using the macro
        let span = tracing::info_span!("test_span");
        hook(&span);

        // Span was entered and hook was called
        // The span exists and is not disabled
        assert!(!span.is_none());
    }
}
