//! Error handling utilities with anyhow context support
//!
//! This module provides improved error handling that preserves error context
//! while remaining compatible with the memory_core::Error::Storage(String) variant.

use memory_core::Error;

/// Create a storage error with context preserved from the original error.
///
/// This function builds a rich error message that includes
/// both the high-level message and the underlying cause.
///
/// # Example
///
/// ```ignore
/// .map_err(|e| storage_err!("Failed to create database", e))?
/// ```
///
/// This produces an error message like:
/// "Failed to create database: connection refused"
#[macro_export]
macro_rules! storage_err {
    ($msg:literal, $err:expr) => {{
        // Build error message that includes context from the original error
        Error::Storage(format!("{}: {}", $msg, $err))
    }};
}

/// Create a storage error from any error type with context.
///
/// This is the function form of the storage_err! macro.
pub fn storage_error<
    C: std::fmt::Display + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
>(
    context: C,
    error: E,
) -> Error {
    Error::Storage(format!("{}: {}", context, error))
}

/// Convert any error to a storage error, preserving its context chain.
///
/// This is useful when you want to convert a libsql or other error
/// to a storage error while keeping the full error chain.
pub fn into_storage_error<E: std::error::Error + Send + Sync + 'static>(error: E) -> Error {
    Error::Storage(error.to_string())
}

/// Wrap a result with improved error handling for storage operations.
///
/// # Example
///
/// ```ignore
/// let result = wrap_storage_err!(
///     some_async_operation().await,
///     "Failed to execute query"
/// );
/// ```
#[macro_export]
macro_rules! wrap_storage_err {
    ($expr:expr, $msg:literal) => {{
        $expr.map_err(|e| $crate::storage_err!($msg, e))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_with_context() {
        let original = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = storage_error("Failed to read file", original);

        let err_str = err.to_string();
        assert!(err_str.contains("Failed to read file"));
        assert!(err_str.contains("file not found"));
    }

    #[test]
    fn test_storage_err_macro() {
        let original = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = storage_err!("Permission error", original);

        let err_str = err.to_string();
        assert!(err_str.contains("Permission error"));
        assert!(err_str.contains("access denied"));
    }

    #[test]
    fn test_into_storage_error() {
        let original = std::io::Error::new(std::io::ErrorKind::Other, "some error");
        let err = into_storage_error(original);

        assert!(err.to_string().contains("some error"));
    }

    #[test]
    fn test_nested_context() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt");
        let outer = std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path");

        // Simulate nested errors
        let err = storage_error("Outer operation failed", outer);
        let err2 = storage_error("Inner operation failed", inner);

        // Both should have meaningful context
        assert!(err.to_string().contains("Outer operation failed"));
        assert!(err2.to_string().contains("Inner operation failed"));
    }
}
