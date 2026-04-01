//! Snapshot tests for error message formatting
//!
//! These tests verify that error messages remain consistent across changes.
//! Part of ADR-033 Phase 6.

use do_memory_core::episode::RelationshipType;
use do_memory_core::error::{CacheError, Error, RelationshipError};
use insta::assert_snapshot;
use uuid::Uuid;

// =============================================================================
// Main Error Type Snapshots
// =============================================================================

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
fn test_error_learning_message() {
    let err = Error::Learning("pattern extraction failed: insufficient data".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_mcp_message() {
    let err = Error::MCP("tool execution failed: timeout".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_pattern_message() {
    let err = Error::Pattern("pattern validation failed: empty sequence".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_invalid_state_message() {
    let err = Error::InvalidState("episode already completed".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_validation_failed_message() {
    let err = Error::ValidationFailed("task description exceeds maximum length".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_rate_limit_exceeded_message() {
    let err = Error::RateLimitExceeded("100 requests per minute".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_configuration_message() {
    let err = Error::Configuration("missing required field: database_url".into());
    assert_snapshot!(err.to_string());
}

#[test]
fn test_error_execution_timeout_message() {
    let err = Error::ExecutionTimeout;
    assert_snapshot!(err.to_string());
}

// =============================================================================
// Relationship Error Snapshots
// =============================================================================

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
fn test_relationship_error_duplicate() {
    let err = RelationshipError::Duplicate {
        from: Uuid::from_u128(100),
        to: Uuid::from_u128(200),
        rel_type: RelationshipType::DependsOn,
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_relationship_error_not_found() {
    let err = RelationshipError::NotFound {
        relationship_id: Uuid::from_u128(999),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_relationship_error_invalid_type() {
    let err = RelationshipError::InvalidType {
        type_name: "invalid_relation_type".into(),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_relationship_error_validation_failed() {
    let err = RelationshipError::ValidationFailed {
        reason: "priority must be between 0 and 100".into(),
    };
    assert_snapshot!(err.to_string());
}

// =============================================================================
// Cache Error Snapshots
// =============================================================================

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

#[test]
fn test_cache_error_serialization_failed() {
    let err = CacheError::SerializationFailed {
        key: "episode_xyz".into(),
        error: "invalid UTF-8 sequence".into(),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_cache_error_deserialization_failed() {
    let err = CacheError::DeserializationFailed {
        key: "cached_result".into(),
        error: "unexpected EOF".into(),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_cache_error_eviction_failed() {
    let err = CacheError::EvictionFailed {
        reason: "no entries available for eviction".into(),
    };
    assert_snapshot!(err.to_string());
}

#[test]
fn test_cache_error_lock_poisoned() {
    let err = CacheError::LockPoisoned {
        context: "LRU cache write lock".into(),
    };
    assert_snapshot!(err.to_string());
}
