//! Security tests for the memory-cli crate.
//!
//! These tests verify that the CLI properly handles malicious input,
//! prevents injection attacks, and sanitizes user data.
//!
//! Organized into focused submodules to keep each file under 500 LOC:
//! - injection_tests: command/SQL/null-byte/format injection, sanitization
//! - path_traversal_tests: path traversal prevention and detection
//! - config_tests: configuration file security and isolation
//! - bounds_tests: input size limits, clamping, large input handling
//! - validation_tests: argument validation, unicode handling

// Clippy allows are scoped to each submodule rather than crate-wide
// (coderabbitai: address lints individually or scope to specific items)

#[allow(clippy::unwrap_used, clippy::expect_used)]
mod bounds_tests;

#[allow(clippy::unwrap_used, clippy::expect_used, clippy::uninlined_format_args)]
mod config_tests;

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::single_match_else,
    clippy::uninlined_format_args
)]
mod injection_tests;

#[allow(clippy::unwrap_used, clippy::expect_used)]
mod path_traversal_tests;

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::excessive_nesting,
    clippy::single_match_else
)]
mod validation_tests;
