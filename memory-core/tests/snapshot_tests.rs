//! Snapshot tests for error message formatting
//!
//! These tests verify that error messages remain consistent across changes.
//! Part of ADR-033 Phase 6.

use insta::assert_snapshot;
use memory_core::episode::RelationshipType;
use memory_core::error::{CacheError, Error, RelationshipError};
use uuid::Uuid;

#[test]
fn test_error_storage_message() {
    let err = Error::Storage("connection pool exhausted".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_not_found_message() {
    let err = Error::NotFound(Uuid::from_u128(1));
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_invalid_input_message() {
    let err = Error::InvalidInput("episode name must be non-empty".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_security_message() {
    let err = Error::Security("path traversal attempt detected: ../etc/passwd".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_circuit_breaker_message() {
    let err = Error::CircuitBreakerOpen;
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_quota_exceeded_message() {
    let err = Error::QuotaExceeded("max 1000 episodes per namespace".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_relationship_error_self_reference() {
    let err = RelationshipError::SelfReference {
        episode_id: Uuid::from_u128(42),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_relationship_error_cycle_detected() {
    let err = RelationshipError::CycleDetected {
        path: vec![
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
            Uuid::from_u128(1),
        ],
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_cache_error_entry_too_large() {
    let err = CacheError::EntryTooLarge {
        key: "episode_abc".into(),
        size: 10_485_760,
        max_size: 1_048_576,
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_cache_error_invalid_config() {
    let err = CacheError::InvalidConfiguration {
        field: "max_entries".into(),
        value: "-1".into(),
    };
    assert_snapshot!(err.to_string());
}
