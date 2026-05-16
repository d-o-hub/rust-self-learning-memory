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

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::single_match_else)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unreadable_literal)]

mod bounds_tests;
mod config_tests;
mod injection_tests;
mod path_traversal_tests;
mod validation_tests;
