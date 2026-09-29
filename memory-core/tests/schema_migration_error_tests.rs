//! Issue #1069: behavior of the typed `SchemaMigrationRequired` error.
//!
//! Covers Display/Debug rendering of the payload, the
//! `is_schema_migration_required()` predicate and the non-recoverable
//! classification.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use do_memory_core::error::Error;

fn sample_error() -> Error {
    Error::SchemaMigrationRequired {
        path: "/var/lib/memory/memory.redb".to_string(),
        stored_version: Some(2),
        current_version: 4,
        detail: "database preserved".to_string(),
    }
}

#[test]
fn display_includes_path_versions_and_detail() {
    let message = sample_error().to_string();
    assert!(message.contains("/var/lib/memory/memory.redb"));
    assert!(message.contains("Some(2)"));
    assert!(message.contains('4'));
    assert!(message.contains("database preserved"));
}

#[test]
fn display_renders_a_missing_stored_version() {
    let err = Error::SchemaMigrationRequired {
        path: "memory.redb".to_string(),
        stored_version: None,
        current_version: 4,
        detail: "no version recorded".to_string(),
    };
    let message = err.to_string();
    assert!(message.contains("None"));
    assert!(message.contains("no version recorded"));
}

#[test]
fn debug_names_the_variant() {
    let debug = format!("{:?}", sample_error());
    assert!(debug.contains("SchemaMigrationRequired"));
}

#[test]
fn predicate_recognizes_only_this_variant() {
    assert!(sample_error().is_schema_migration_required());
    assert!(!Error::Storage("boom".to_string()).is_schema_migration_required());
    assert!(!Error::RetryQueueTimeout.is_schema_migration_required());
    assert!(!Error::InvalidState("bad".to_string()).is_schema_migration_required());
}

#[test]
fn migration_required_is_not_recoverable() {
    assert!(!sample_error().is_recoverable());
    assert!(Error::Storage("boom".to_string()).is_recoverable());
}
