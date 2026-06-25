//! ACT-029: Comprehensive error handling tests for memory-core Error types.
//!
//! Tests cover:
//! - All Error variant construction
//! - Display/Debug formatting for each variant
//! - `is_recoverable()` classification for ALL variants
//! - From conversions (`serde_json::Error` -> Error, `io::Error` -> Error, etc.)
//! - `Result<T>` type alias
//! - `is_relationship_error()` / `is_cache_error()` checks
//! - `as_relationship_error()` / `as_cache_error()` accessors

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::panic)]

use do_memory_core::episode::RelationshipType;
use do_memory_core::error::{CacheError, Error, RelationshipError, Result};
use uuid::Uuid;

// ============================================================================
// Error Variant Construction
// ============================================================================

#[test]
fn error_storage_construction() {
    let err = Error::Storage("disk full".to_string());
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn error_learning_construction() {
    let err = Error::Learning("bad gradient".to_string());
    assert!(err.to_string().contains("bad gradient"));
}

#[test]
fn error_mcp_construction() {
    let err = Error::MCP("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn error_not_found_construction() {
    let id = Uuid::new_v4();
    let err = Error::NotFound(id);
    assert!(err.to_string().contains(&id.to_string()));
}

#[test]
fn error_pattern_construction() {
    let err = Error::Pattern("invalid regex".to_string());
    assert!(err.to_string().contains("invalid regex"));
}

#[test]
fn error_execution_timeout_construction() {
    let err = Error::ExecutionTimeout;
    assert!(err.to_string().contains("timeout"));
}

#[test]
fn error_circuit_breaker_open_construction() {
    let err = Error::CircuitBreakerOpen;
    assert!(err.to_string().contains("Circuit breaker open"));
}

#[test]
fn error_invalid_input_construction() {
    let err = Error::InvalidInput("negative count".to_string());
    assert!(err.to_string().contains("negative count"));
}

#[test]
fn error_invalid_state_construction() {
    let err = Error::InvalidState("already completed".to_string());
    assert!(err.to_string().contains("already completed"));
}

#[test]
fn error_security_construction() {
    let err = Error::Security("token expired".to_string());
    assert!(err.to_string().contains("token expired"));
}

#[test]
fn error_validation_failed_construction() {
    let err = Error::ValidationFailed("missing field".to_string());
    assert!(err.to_string().contains("missing field"));
}

#[test]
fn error_quota_exceeded_construction() {
    let err = Error::QuotaExceeded("100 episodes".to_string());
    assert!(err.to_string().contains("100 episodes"));
}

#[test]
fn error_rate_limit_exceeded_construction() {
    let err = Error::RateLimitExceeded("10 req/s".to_string());
    assert!(err.to_string().contains("10 req/s"));
}

#[test]
fn error_configuration_construction() {
    let err = Error::Configuration("missing TURSO_DATABASE_URL".to_string());
    assert!(err.to_string().contains("missing TURSO_DATABASE_URL"));
}

// ============================================================================
// Display Formatting
// ============================================================================

#[test]
fn error_display_storage() {
    let err = Error::Storage("db error".to_string());
    assert_eq!(err.to_string(), "Storage error: db error");
}

#[test]
fn error_display_learning() {
    let err = Error::Learning("convergence failed".to_string());
    assert_eq!(err.to_string(), "Learning error: convergence failed");
}

#[test]
fn error_display_mcp() {
    let err = Error::MCP("timeout".to_string());
    assert_eq!(err.to_string(), "MCP error: timeout");
}

#[test]
fn error_display_not_found() {
    let id = Uuid::nil();
    let err = Error::NotFound(id);
    assert_eq!(err.to_string(), format!("Episode not found: {id}"));
}

#[test]
fn error_display_pattern() {
    let err = Error::Pattern("bad".to_string());
    assert_eq!(err.to_string(), "Pattern error: bad");
}

#[test]
fn error_display_execution_timeout() {
    let err = Error::ExecutionTimeout;
    assert_eq!(err.to_string(), "Execution timeout");
}

#[test]
fn error_display_circuit_breaker_open() {
    let err = Error::CircuitBreakerOpen;
    assert_eq!(err.to_string(), "Circuit breaker open");
}

#[test]
fn error_display_invalid_input() {
    let err = Error::InvalidInput("x".to_string());
    assert_eq!(err.to_string(), "Invalid input: x");
}

#[test]
fn error_display_invalid_state() {
    let err = Error::InvalidState("y".to_string());
    assert_eq!(err.to_string(), "Invalid state: y");
}

#[test]
fn error_display_security() {
    let err = Error::Security("z".to_string());
    assert_eq!(err.to_string(), "Security validation failed: z");
}

#[test]
fn error_display_validation_failed() {
    let err = Error::ValidationFailed("bad".to_string());
    assert_eq!(err.to_string(), "Validation failed: bad");
}

#[test]
fn error_display_quota_exceeded() {
    let err = Error::QuotaExceeded("max".to_string());
    assert_eq!(err.to_string(), "Quota exceeded: max");
}

#[test]
fn error_display_rate_limit_exceeded() {
    let err = Error::RateLimitExceeded("slow down".to_string());
    assert_eq!(err.to_string(), "Rate limit exceeded: slow down");
}

#[test]
fn error_display_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err = Error::Io(io_err);
    assert!(err.to_string().starts_with("IO error:"));
    assert!(err.to_string().contains("file missing"));
}

#[test]
fn error_display_configuration() {
    let err = Error::Configuration("missing key".to_string());
    assert_eq!(err.to_string(), "Configuration error: missing key");
}

// ============================================================================
// Debug Formatting
// ============================================================================

#[test]
fn error_debug_contains_variant_name() {
    let err = Error::Storage("test".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Storage"));
}

#[test]
fn error_debug_not_found_contains_uuid() {
    let id = Uuid::new_v4();
    let err = Error::NotFound(id);
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("NotFound"));
    assert!(debug_str.contains(&id.to_string()));
}

#[test]
fn error_debug_relationship_variant() {
    let rel_err = RelationshipError::SelfReference {
        episode_id: Uuid::new_v4(),
    };
    let err = Error::Relationship(rel_err);
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Relationship"));
    assert!(debug_str.contains("SelfReference"));
}

#[test]
fn error_debug_cache_variant() {
    let cache_err = CacheError::EvictionFailed {
        reason: "test".to_string(),
    };
    let err = Error::Cache(cache_err);
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Cache"));
    assert!(debug_str.contains("EvictionFailed"));
}

// ============================================================================
// is_recoverable() Classification - ALL Variants
// ============================================================================

#[test]
fn error_handling_recoverable_storage() {
    assert!(Error::Storage("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_recoverable_mcp() {
    assert!(Error::MCP("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_recoverable_execution_timeout() {
    assert!(Error::ExecutionTimeout.is_recoverable());
}

#[test]
fn error_handling_recoverable_circuit_breaker_open() {
    assert!(Error::CircuitBreakerOpen.is_recoverable());
}

#[test]
fn error_handling_recoverable_rate_limit_exceeded() {
    assert!(Error::RateLimitExceeded("limit".to_string()).is_recoverable());
}

#[test]
fn error_handling_recoverable_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "tmp");
    assert!(Error::Io(io_err).is_recoverable());
}

#[test]
fn error_handling_recoverable_embedding() {
    let anyhow_err = anyhow::anyhow!("embedding service down");
    assert!(Error::Embedding(anyhow_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_learning() {
    assert!(!Error::Learning("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_not_found() {
    assert!(!Error::NotFound(Uuid::new_v4()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_pattern() {
    assert!(!Error::Pattern("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_serialization() {
    let json_err: std::result::Result<serde_json::Value, _> = serde_json::from_str("{{bad");
    let err = Error::Serialization(json_err.unwrap_err());
    assert!(!err.is_recoverable());
}

#[test]
fn error_handling_non_recoverable_invalid_input() {
    assert!(!Error::InvalidInput("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_invalid_state() {
    assert!(!Error::InvalidState("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_security() {
    assert!(!Error::Security("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_validation_failed() {
    assert!(!Error::ValidationFailed("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_quota_exceeded() {
    assert!(!Error::QuotaExceeded("err".to_string()).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_configuration() {
    assert!(!Error::Configuration("err".to_string()).is_recoverable());
}

// Relationship error recoverability
#[test]
fn error_handling_recoverable_relationship_validation_failed() {
    let rel_err = RelationshipError::ValidationFailed {
        reason: "retry".to_string(),
    };
    assert!(Error::Relationship(rel_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_relationship_self_reference() {
    let rel_err = RelationshipError::SelfReference {
        episode_id: Uuid::new_v4(),
    };
    assert!(!Error::Relationship(rel_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_relationship_duplicate() {
    let rel_err = RelationshipError::Duplicate {
        from: Uuid::new_v4(),
        to: Uuid::new_v4(),
        rel_type: RelationshipType::DependsOn,
    };
    assert!(!Error::Relationship(rel_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_relationship_cycle_detected() {
    let rel_err = RelationshipError::CycleDetected {
        path: vec![Uuid::new_v4(), Uuid::new_v4()],
    };
    assert!(!Error::Relationship(rel_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_relationship_not_found() {
    let rel_err = RelationshipError::NotFound {
        relationship_id: Uuid::new_v4(),
    };
    assert!(!Error::Relationship(rel_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_relationship_invalid_type() {
    let rel_err = RelationshipError::InvalidType {
        type_name: "bad".to_string(),
    };
    assert!(!Error::Relationship(rel_err).is_recoverable());
}

// Cache error recoverability
#[test]
fn error_handling_recoverable_cache_eviction_failed() {
    let cache_err = CacheError::EvictionFailed {
        reason: "retry".to_string(),
    };
    assert!(Error::Cache(cache_err).is_recoverable());
}

#[test]
fn error_handling_recoverable_cache_serialization_failed() {
    let cache_err = CacheError::SerializationFailed {
        key: "k".to_string(),
        error: "err".to_string(),
    };
    assert!(Error::Cache(cache_err).is_recoverable());
}

#[test]
fn error_handling_recoverable_cache_entry_too_large() {
    let cache_err = CacheError::EntryTooLarge {
        key: "k".to_string(),
        size: 100,
        max_size: 50,
    };
    assert!(Error::Cache(cache_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_cache_deserialization_failed() {
    let cache_err = CacheError::DeserializationFailed {
        key: "k".to_string(),
        error: "err".to_string(),
    };
    assert!(!Error::Cache(cache_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_cache_invalid_configuration() {
    let cache_err = CacheError::InvalidConfiguration {
        field: "max".to_string(),
        value: "0".to_string(),
    };
    assert!(!Error::Cache(cache_err).is_recoverable());
}

#[test]
fn error_handling_non_recoverable_cache_lock_poisoned() {
    let cache_err = CacheError::LockPoisoned {
        context: "test".to_string(),
    };
    assert!(!Error::Cache(cache_err).is_recoverable());
}

// ============================================================================
// From Conversions
// ============================================================================

#[test]
fn error_handling_from_serde_json_error() {
    let json_result: std::result::Result<serde_json::Value, _> = serde_json::from_str("not json");
    let serde_err = json_result.unwrap_err();
    let err: Error = serde_err.into();
    assert!(matches!(err, Error::Serialization(_)));
    assert!(err.to_string().contains("Serialization error"));
}

#[test]
fn error_handling_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
    assert!(err.to_string().contains("access denied"));
}

#[test]
fn error_handling_from_relationship_error() {
    let rel_err = RelationshipError::SelfReference {
        episode_id: Uuid::new_v4(),
    };
    let err: Error = rel_err.into();
    assert!(matches!(err, Error::Relationship(_)));
    assert!(err.is_relationship_error());
}

#[test]
fn error_handling_from_cache_error() {
    let cache_err = CacheError::EvictionFailed {
        reason: "full".to_string(),
    };
    let err: Error = cache_err.into();
    assert!(matches!(err, Error::Cache(_)));
    assert!(err.is_cache_error());
}

#[test]
fn error_handling_from_anyhow_error() {
    let anyhow_err = anyhow::anyhow!("embedding failure");
    let err: Error = anyhow_err.into();
    assert!(matches!(err, Error::Embedding(_)));
    assert!(err.to_string().contains("Embedding error"));
}

// ============================================================================
// is_relationship_error() / is_cache_error()
// ============================================================================

#[test]
fn error_handling_is_relationship_error_true() {
    let err = Error::Relationship(RelationshipError::NotFound {
        relationship_id: Uuid::new_v4(),
    });
    assert!(err.is_relationship_error());
    assert!(!err.is_cache_error());
}

#[test]
fn error_handling_is_cache_error_true() {
    let err = Error::Cache(CacheError::LockPoisoned {
        context: "test".to_string(),
    });
    assert!(err.is_cache_error());
    assert!(!err.is_relationship_error());
}

#[test]
fn error_handling_non_relationship_non_cache() {
    let err = Error::Storage("test".to_string());
    assert!(!err.is_relationship_error());
    assert!(!err.is_cache_error());
}

// ============================================================================
// as_relationship_error() / as_cache_error()
// ============================================================================

#[test]
fn error_handling_as_relationship_error_some() {
    let rel_err = RelationshipError::InvalidType {
        type_name: "bad".to_string(),
    };
    let err = Error::Relationship(rel_err.clone());
    let extracted = err.as_relationship_error();
    assert!(extracted.is_some());
    assert_eq!(extracted.unwrap(), &rel_err);
}

#[test]
fn error_handling_as_relationship_error_none() {
    let err = Error::Storage("test".to_string());
    assert!(err.as_relationship_error().is_none());
}

#[test]
fn error_handling_as_cache_error_some() {
    let cache_err = CacheError::LockPoisoned {
        context: "mutex".to_string(),
    };
    let err = Error::Cache(cache_err.clone());
    let extracted = err.as_cache_error();
    assert!(extracted.is_some());
    assert_eq!(extracted.unwrap(), &cache_err);
}

#[test]
fn error_handling_as_cache_error_none() {
    let err = Error::MCP("test".to_string());
    assert!(err.as_cache_error().is_none());
}

// ============================================================================
// Result<T> Type Alias
// ============================================================================

#[test]
fn error_handling_result_type_alias_ok() {
    let result: Result<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn error_handling_result_type_alias_err() {
    let result: Result<i32> = Err(Error::NotFound(Uuid::new_v4()));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
}

#[test]
fn error_handling_result_type_alias_in_function() {
    fn example_fn(succeed: bool) -> Result<String> {
        if succeed {
            Ok("done".to_string())
        } else {
            Err(Error::InvalidInput("bad".to_string()))
        }
    }

    assert!(example_fn(true).is_ok());
    assert!(example_fn(false).is_err());
}

// ============================================================================
// Error is Send + Sync (required for async)
// ============================================================================

#[test]
fn error_handling_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Error>();
}

#[test]
fn error_handling_error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Error>();
}

// ============================================================================
// std::error::Error Trait Implementation
// ============================================================================

#[test]
fn error_handling_error_impls_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<Error>();
}

#[test]
fn error_handling_relationship_error_impls_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<RelationshipError>();
}

#[test]
fn error_handling_cache_error_impls_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<CacheError>();
}

// ============================================================================
// Error Chain Propagation
// ============================================================================

#[test]
fn error_handling_propagation_with_question_operator() {
    fn inner_function() -> Result<String> {
        let json_result: std::result::Result<serde_json::Value, _> =
            serde_json::from_str("invalid");
        let _parsed = json_result?;
        Ok("unreachable".to_string())
    }

    fn middle_function() -> Result<String> {
        let result = inner_function()?;
        Ok(result)
    }

    fn outer_function() -> Result<String> {
        let result = middle_function()?;
        Ok(format!("processed: {result}"))
    }

    let result = outer_function();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Serialization(_)));
}

#[test]
fn error_handling_propagation_from_io_error() {
    fn inner_function() -> Result<String> {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        Err(io_err)?
    }

    fn outer_function() -> Result<String> {
        inner_function()
    }

    let result = outer_function();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn error_handling_propagation_from_relationship_error() {
    fn create_relationship() -> Result<()> {
        Err(RelationshipError::SelfReference {
            episode_id: Uuid::new_v4(),
        })?
    }

    fn save_episode() -> Result<()> {
        create_relationship()?;
        Ok(())
    }

    let result = save_episode();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_relationship_error());
}

#[test]
fn error_handling_propagation_from_cache_error() {
    fn cache_operation() -> Result<()> {
        Err(CacheError::EvictionFailed {
            reason: "memory pressure".to_string(),
        })?
    }

    fn process_with_cache() -> Result<()> {
        cache_operation()?;
        Ok(())
    }

    let result = process_with_cache();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_cache_error());
}

#[test]
fn error_handling_propagation_from_anyhow() {
    fn embedding_call() -> Result<()> {
        Err(anyhow::anyhow!("embedding service unavailable"))?
    }

    fn process_embeddings() -> Result<()> {
        embedding_call()?;
        Ok(())
    }

    let result = process_embeddings();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Embedding(_)));
    assert!(err.is_recoverable());
}

// ============================================================================
// Multi-Layer Error Propagation
// ============================================================================

#[test]
fn error_handling_multi_layer_propagation() {
    fn layer_three() -> Result<i32> {
        Err(Error::Storage("database connection failed".to_string()))
    }

    fn layer_two() -> Result<i32> {
        let value = layer_three()?;
        Ok(value * 2)
    }

    fn layer_one() -> Result<String> {
        let value = layer_two()?;
        Ok(format!("Result: {value}"))
    }

    let result = layer_one();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Storage(_)));
    assert!(err.to_string().contains("database connection failed"));
}

#[test]
fn error_handling_propagation_preserves_error_type() {
    fn create_error() -> Result<()> {
        Err(Error::InvalidInput("negative value".to_string()))
    }

    fn propagate_once() -> Result<()> {
        create_error()?;
        Ok(())
    }

    fn propagate_twice() -> Result<()> {
        propagate_once()?;
        Ok(())
    }

    let result = propagate_twice();
    assert!(result.is_err());
    let err = result.unwrap_err();

    // Verify the error type is preserved through propagation
    assert!(matches!(err, Error::InvalidInput(_)));
    assert!(!err.is_recoverable());
}

// ============================================================================
// Error Display Through Propagation
// ============================================================================

#[test]
fn error_handling_display_preserved_through_propagation() {
    fn failing_function() -> Result<String> {
        Err(Error::NotFound(Uuid::nil()))?
    }

    let result: Result<String> = failing_function();
    let err = result.unwrap_err();

    // Display should still work correctly after propagation
    let display = err.to_string();
    assert!(display.contains("not found"));
    assert!(display.contains("00000000-0000-0000-0000-000000000000"));
}

// ============================================================================
// Result<T> With Different Types
// ============================================================================

#[test]
fn error_handling_result_with_unit_type() {
    fn returns_result_unit() -> Result<()> {
        Err(Error::Configuration("missing env var".to_string()))
    }

    let result = returns_result_unit();
    assert!(result.is_err());
}

#[test]
fn error_handling_result_with_option() {
    fn returns_result_option() -> Result<Option<String>> {
        Ok(Some("value".to_string()))
    }

    let result = returns_result_option();
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn error_handling_result_with_vec() {
    fn returns_result_vec() -> Result<Vec<i32>> {
        Err(Error::QuotaExceeded("max 100 items".to_string()))
    }

    let result = returns_result_vec();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::QuotaExceeded(_)));
}

// ============================================================================
// Error Cloning (for types that support it)
// ============================================================================

#[test]
fn error_handling_relationship_error_can_be_cloned() {
    let original = RelationshipError::SelfReference {
        episode_id: Uuid::new_v4(),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn error_handling_cache_error_can_be_cloned() {
    let original = CacheError::EvictionFailed {
        reason: "test".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ============================================================================
// Error Matching And Extraction
// ============================================================================

#[test]
fn error_handling_match_all_variants() {
    let errors: Vec<Error> = vec![
        Error::Storage("test".to_string()),
        Error::Learning("test".to_string()),
        Error::MCP("test".to_string()),
        Error::NotFound(Uuid::nil()),
        Error::Pattern("test".to_string()),
        Error::ExecutionTimeout,
        Error::CircuitBreakerOpen,
        Error::InvalidInput("test".to_string()),
        Error::InvalidState("test".to_string()),
        Error::Security("test".to_string()),
        Error::ValidationFailed("test".to_string()),
        Error::QuotaExceeded("test".to_string()),
        Error::RateLimitExceeded("test".to_string()),
        Error::Configuration("test".to_string()),
    ];

    for err in errors {
        // Ensure each variant can be matched and displayed
        let _display = err.to_string();
        let _debug = format!("{err:?}");
    }
}

#[test]
fn error_handling_extract_embedded_data() {
    // Test extracting UUID from NotFound
    let id = Uuid::new_v4();
    let err = Error::NotFound(id);
    if let Error::NotFound(extracted_id) = err {
        assert_eq!(id, extracted_id);
    } else {
        panic!("Expected NotFound variant");
    }
}

#[test]
fn error_handling_extract_embedded_string() {
    // Test extracting string from InvalidInput
    let msg = "negative value not allowed".to_string();
    let err = Error::InvalidInput(msg.clone());
    if let Error::InvalidInput(extracted) = err {
        assert_eq!(msg, extracted);
    } else {
        panic!("Expected InvalidInput variant");
    }
}
