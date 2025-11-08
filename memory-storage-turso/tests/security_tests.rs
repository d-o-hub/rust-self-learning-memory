//! Security tests for Turso storage backend
//!
//! Tests TLS/HTTPS enforcement and certificate validation to ensure
//! only secure protocols are used for database connections.

use memory_storage_turso::TursoStorage;

#[tokio::test]
async fn test_rejects_insecure_http_protocol() {
    let result = TursoStorage::new("http://insecure.example.com", "token").await;
    assert!(result.is_err(), "Should reject HTTP protocol");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_insecure_https_protocol() {
    let result = TursoStorage::new("https://insecure.example.com", "token").await;
    assert!(
        result.is_err(),
        "Should reject HTTPS protocol (not libsql://)"
    );
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_empty_token_for_remote() {
    let result = TursoStorage::new("libsql://example.com", "").await;
    assert!(result.is_err(), "Should reject empty token");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("token") || err_msg.contains("Authentication"),
            "Error should mention token: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_whitespace_only_token() {
    let result = TursoStorage::new("libsql://example.com", "   ").await;
    assert!(result.is_err(), "Should reject whitespace-only token");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("token") || err_msg.contains("Authentication"),
            "Error should mention token: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_allows_file_protocol() {
    let result = TursoStorage::new("file:test.db", "").await;
    // File protocol should be allowed (may fail for other reasons but not security check)
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            !err_msg.contains("Insecure"),
            "Should not reject file: protocol for security reasons: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_allows_memory_database() {
    let result = TursoStorage::new(":memory:", "").await;
    // Memory database should be allowed
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            !err_msg.contains("Insecure"),
            "Should not reject :memory: for security reasons: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_allows_valid_libsql_with_token() {
    // This may fail due to network, but should pass security validation
    let result = TursoStorage::new("libsql://example.turso.io", "valid_token_here").await;
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            !err_msg.contains("Insecure") && !err_msg.contains("Security"),
            "Should not fail security validation: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_ftp_protocol() {
    let result = TursoStorage::new("ftp://insecure.example.com", "token").await;
    assert!(result.is_err(), "Should reject FTP protocol");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_ws_protocol() {
    let result = TursoStorage::new("ws://insecure.example.com", "token").await;
    assert!(result.is_err(), "Should reject WebSocket protocol");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_postgres_protocol() {
    let result = TursoStorage::new("postgresql://localhost/db", "token").await;
    assert!(result.is_err(), "Should reject PostgreSQL protocol");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_rejects_mysql_protocol() {
    let result = TursoStorage::new("mysql://localhost/db", "token").await;
    assert!(result.is_err(), "Should reject MySQL protocol");
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Insecure") || err_msg.contains("libsql://"),
            "Error should mention security: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_security_error_type() {
    let result = TursoStorage::new("http://bad.com", "token").await;
    assert!(result.is_err());
    if let Err(err) = result {
        // Verify it's a Security error variant
        assert!(
            err.to_string().contains("Security validation failed"),
            "Error should be Security variant: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_file_protocol_variations() {
    // Test various valid file: protocol formats
    let test_cases = vec!["file:test.db", "file:./test.db", "file:/tmp/test.db"];

    for url in test_cases {
        let result = TursoStorage::new(url, "").await;
        // Should not fail security validation (may fail for other reasons)
        if let Err(e) = result {
            let err_msg = e.to_string();
            assert!(
                !err_msg.contains("Insecure"),
                "file: protocol '{}' should not fail security check: {}",
                url,
                err_msg
            );
        }
    }
}

#[tokio::test]
async fn test_case_sensitive_protocol_check() {
    // Protocol checks should be case-sensitive (lowercase required)
    let result = TursoStorage::new("LIBSQL://example.turso.io", "token").await;
    assert!(result.is_err(), "Uppercase protocol should be rejected");
}
